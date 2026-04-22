//! User and registration-related types.
//!
//! These structs mirror the GraphQL types used in the registration flow:
//! - `RegistrationInput` (input type)
//! - `ReferralVerifyResponse` (verifyReferralString mutation)
//! - `RegisterResponse` (register mutation)
//! - `VerifyAccountResponse` (verifyAccount mutation)

use serde::{Deserialize, Serialize};

use super::common::DefaultResponse;

/// Input for the `register` mutation.
///
/// Matches the GraphQL `RegistrationInput` type:
/// ```graphql
/// input RegistrationInput {
///   email: String!
///   password: String!
///   username: String!
///   pkey: String
///   referralUuid: ID
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationInput {
    pub email: String,
    pub password: String,
    pub username: String,
    /// Optional Solana public key (43-44 chars, base58).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkey: Option<String>,
    /// UUID of the referring user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_uuid: Option<String>,
}

impl RegistrationInput {
    /// Create a new registration input with required fields.
    pub fn new(
        email: impl Into<String>,
        password: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
            username: username.into(),
            pkey: None,
            referral_uuid: None,
        }
    }

    /// Set the referral UUID.
    pub fn with_referral(mut self, referral_uuid: impl Into<String>) -> Self {
        self.referral_uuid = Some(referral_uuid.into());
        self
    }

    /// Set the Solana public key.
    pub fn with_pkey(mut self, pkey: impl Into<String>) -> Self {
        self.pkey = Some(pkey.into());
        self
    }
}

/// A user returned in the referral verification response.
///
/// This is the referrer's basic profile information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralUser {
    pub uid: String,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub img: Option<String>,
}

/// Response from the `verifyReferralString` mutation.
///
/// ```graphql
/// type ReferralResponse {
///   status: String!
///   ResponseCode: String!
///   affectedRows: [ReferralUser]
///   meta: DefaultResponse!
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReferralVerifyResponse {
    /// "success" or "error"
    #[serde(rename = "status")]
    pub status: String,
    pub response_code: String,
    /// Contains the referrer's info on success; null/empty on error.
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Option<Vec<ReferralUser>>,
    /// Detailed response metadata.
    #[serde(default)]
    pub meta: Option<DefaultResponse>,
}

impl ReferralVerifyResponse {
    /// Check if the referral verification was successful.
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }

    /// Get the referrer's info if available.
    pub fn referrer(&self) -> Option<&ReferralUser> {
        self.affected_rows.as_ref()?.first()
    }
}

/// Response from the `register` mutation.
///
/// ```graphql
/// type RegisterResponse {
///   status: String!
///   ResponseCode: String
///   userid: String
///   meta: DefaultResponse!
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegisterResponse {
    /// "success" or "error"
    #[serde(rename = "status")]
    pub status: String,
    #[serde(default)]
    pub response_code: Option<String>,
    /// The newly created user's UUID (only on success).
    #[serde(default, rename = "userid")]
    pub user_id: Option<String>,
    /// Detailed response metadata.
    #[serde(default)]
    pub meta: Option<DefaultResponse>,
}

impl RegisterResponse {
    /// Check if the registration was successful.
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }

    /// Get the response code, preferring meta if available.
    pub fn code(&self) -> Option<&str> {
        self.meta
            .as_ref()
            .map(|m| m.response_code.as_str())
            .or(self.response_code.as_deref())
    }
}

/// Response from the `verifyAccount` mutation.
///
/// ```graphql
/// type VerifyAccountResponse {
///   status: String!
///   ResponseCode: String!
///   meta: DefaultResponse!
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyAccountResponse {
    /// "success" or "error"
    #[serde(rename = "status")]
    pub status: String,
    pub response_code: String,
    /// Detailed response metadata.
    #[serde(default)]
    pub meta: Option<DefaultResponse>,
}

impl VerifyAccountResponse {
    /// Check if the account verification was successful.
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }
}

/// User preferences from the API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    /// Content filtering severity level.
    #[serde(default)]
    pub content_filtering_severity_level: Option<String>,
}

/// Logged-in user profile info (from `getUser` query).
///
/// ```graphql
/// type User {
///   id: ID!
///   username: String!
///   slug: String!
///   img: String
///   biography: String
///   amountFollowers: Int!
///   amountFollowing: Int!
///   amountPeers: Int!
///   userPreferences: UserPreferences
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub img: Option<String>,
    #[serde(default)]
    pub biography: Option<String>,
    #[serde(default)]
    pub amount_followers: i32,
    #[serde(default)]
    pub amount_following: i32,
    #[serde(default)]
    pub amount_peers: i32,
    #[serde(default)]
    pub user_preferences: Option<UserPreferences>,
}

/// Response wrapper for `getUser` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResponse {
    pub meta: DefaultResponse,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<UserInfo>,
}

impl UserInfoResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_input_builder() {
        let input = RegistrationInput::new("user@example.com", "SecurePass123", "testuser")
            .with_referral("85d5f836-b1f5-4c4e-9381-1b058e13df93");

        assert_eq!(input.email, "user@example.com");
        assert_eq!(input.password, "SecurePass123");
        assert_eq!(input.username, "testuser");
        assert_eq!(
            input.referral_uuid,
            Some("85d5f836-b1f5-4c4e-9381-1b058e13df93".to_string())
        );
        assert!(input.pkey.is_none());
    }

    #[test]
    fn test_registration_input_serialize() {
        let input = RegistrationInput::new("user@example.com", "Pass123", "testuser")
            .with_referral("85d5f836-b1f5-4c4e-9381-1b058e13df93");

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains(r#""email":"user@example.com""#));
        assert!(json.contains(r#""referralUuid":"85d5f836"#));
        // pkey should be omitted when None
        assert!(!json.contains("pkey"));
    }

    #[test]
    fn test_referral_response_is_success() {
        let success = ReferralVerifyResponse {
            status: "success".to_string(),
            response_code: "11011".to_string(),
            affected_rows: Some(vec![ReferralUser {
                uid: "123".to_string(),
                username: "referrer".to_string(),
                slug: "abc12".to_string(),
                img: None,
            }]),
            meta: None,
        };
        assert!(success.is_success());
        assert!(success.referrer().is_some());

        let error = ReferralVerifyResponse {
            status: "error".to_string(),
            response_code: "31010".to_string(),
            affected_rows: None,
            meta: None,
        };
        assert!(!error.is_success());
        assert!(error.referrer().is_none());
    }

    #[test]
    fn test_register_response_is_success() {
        let success = RegisterResponse {
            status: "success".to_string(),
            response_code: Some("10601".to_string()),
            user_id: Some("new-user-uuid".to_string()),
            meta: None,
        };
        assert!(success.is_success());
        assert_eq!(success.code(), Some("10601"));
    }
}
