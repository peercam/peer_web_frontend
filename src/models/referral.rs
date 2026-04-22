//! Referral board data models.
//!
//! These models map to the GraphQL responses for `getReferralInfo` and `referralList`.

use serde::{Deserialize, Deserializer, Serialize};

/// Response from the `getReferralInfo` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReferralInfoResponse {
    #[serde(rename = "status")]
    pub status: String,
    pub response_code: String,
    #[serde(default, rename = "referralUuid")]
    pub referral_uuid: Option<String>,
    #[serde(default, rename = "referralLink")]
    pub referral_link: Option<String>,
}

impl ReferralInfoResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }
}

/// A user in the referral list (lighter than ProfileUser — only id, username, slug, img).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralListUser {
    pub id: String,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub img: Option<String>,
}

impl ReferralListUser {
    /// Get the user's avatar URL, falling back to default if none.
    pub fn avatar_url(&self) -> &str {
        self.img.as_deref().unwrap_or("/svg/noname.svg")
    }

    /// Get the formatted slug with # prefix.
    pub fn display_slug(&self) -> String {
        format!("#{}", self.slug)
    }
}

/// The `affectedRows` envelope inside `referralList` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralUsers {
    /// The user who invited the current user (single or null).
    #[serde(default, deserialize_with = "deserialize_optional_single_or_vec")]
    pub invited_by: Vec<ReferralListUser>,
    /// Users the current user invited (array).
    #[serde(default, deserialize_with = "deserialize_optional_single_or_vec")]
    pub i_invited: Vec<ReferralListUser>,
}

/// Response from the `referralList` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReferralListResponse {
    #[serde(rename = "status")]
    pub status: String,
    #[serde(default)]
    pub counter: i32,
    pub response_code: String,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Option<ReferralUsers>,
}

impl ReferralListResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }

    /// Check if this is an "empty" response (no referral data).
    /// Response codes starting with '2' indicate empty data.
    pub fn is_empty_response(&self) -> bool {
        self.response_code.starts_with('2')
    }

    /// Get the user who invited the current user (if any).
    pub fn invited_by(&self) -> Vec<ReferralListUser> {
        if self.is_empty_response() {
            return vec![];
        }
        self.affected_rows
            .as_ref()
            .map(|r| r.invited_by.clone())
            .unwrap_or_default()
    }

    /// Get the users the current user has invited.
    pub fn i_invited(&self) -> Vec<ReferralListUser> {
        if self.is_empty_response() {
            return vec![];
        }
        self.affected_rows
            .as_ref()
            .map(|r| r.i_invited.clone())
            .unwrap_or_default()
    }
}

/// Deserialize a value that may be a single object or an array.
///
/// The backend sometimes returns a single object when there's one item,
/// and an array when there are multiple. This normalizes both to a Vec.
fn deserialize_optional_single_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};
    use std::fmt;
    use std::marker::PhantomData;

    struct SingleOrVec<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for SingleOrVec<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, a single object, or an array")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Vec::new())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(item) = seq.next_element()? {
                vec.push(item);
            }
            Ok(vec)
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            // Deserialize as a single object and wrap in a Vec
            let item = T::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(vec![item])
        }
    }

    deserializer.deserialize_any(SingleOrVec(PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referral_info_is_success() {
        let response = ReferralInfoResponse {
            status: "success".to_string(),
            response_code: "11011".to_string(),
            referral_uuid: Some("abc-123".to_string()),
            referral_link: Some("https://peer.network/invite?referralUuid=abc-123".to_string()),
        };
        assert!(response.is_success());
    }

    #[test]
    fn test_referral_list_user_avatar_url() {
        let user = ReferralListUser {
            id: "1".to_string(),
            username: "testuser".to_string(),
            slug: "test123".to_string(),
            img: Some("https://example.com/avatar.jpg".to_string()),
        };
        assert_eq!(user.avatar_url(), "https://example.com/avatar.jpg");

        let user_no_img = ReferralListUser {
            id: "2".to_string(),
            username: "testuser2".to_string(),
            slug: "test456".to_string(),
            img: None,
        };
        assert_eq!(user_no_img.avatar_url(), "/svg/noname.svg");
    }

    #[test]
    fn test_referral_list_user_display_slug() {
        let user = ReferralListUser {
            id: "1".to_string(),
            username: "testuser".to_string(),
            slug: "abc123".to_string(),
            img: None,
        };
        assert_eq!(user.display_slug(), "#abc123");
    }

    #[test]
    fn test_deserialize_single_object() {
        let json = r#"{
            "invitedBy": {"id": "1", "username": "user1", "slug": "u1"},
            "iInvited": [{"id": "2", "username": "user2", "slug": "u2"}]
        }"#;
        let users: ReferralUsers = serde_json::from_str(json).unwrap();
        assert_eq!(users.invited_by.len(), 1);
        assert_eq!(users.invited_by[0].username, "user1");
        assert_eq!(users.i_invited.len(), 1);
    }

    #[test]
    fn test_deserialize_empty_arrays() {
        let json = r#"{
            "invitedBy": [],
            "iInvited": []
        }"#;
        let users: ReferralUsers = serde_json::from_str(json).unwrap();
        assert!(users.invited_by.is_empty());
        assert!(users.i_invited.is_empty());
    }

    #[test]
    fn test_referral_list_empty_response() {
        let response = ReferralListResponse {
            status: "success".to_string(),
            counter: 0,
            response_code: "21003".to_string(),
            affected_rows: None,
        };
        assert!(response.is_empty_response());
        assert!(response.i_invited().is_empty());
        assert!(response.invited_by().is_empty());
    }
}
