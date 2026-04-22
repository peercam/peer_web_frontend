//! Settings-related data models.
//!
//! These structs mirror the GraphQL types used for settings mutations:
//! - Profile updates (bio, image, username)
//! - Account credential changes (password, email)
//! - User preferences (content filtering)
//! - Account lifecycle (deactivation)

use serde::{Deserialize, Serialize};

/// Response from simple update mutations (updateBio, updateProfileImage, etc.).
///
/// ```graphql
/// type UpdateResponse {
///   status: String!
///   ResponseCode: String!
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateResponse {
    #[serde(rename = "status")]
    pub status: String,
    pub response_code: String,
}

impl UpdateResponse {
    /// Check if the update was successful.
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }
}

/// Response from updateUserPreferences mutation.
///
/// ```graphql
/// type UserPreferencesResponse {
///   status: String!
///   ResponseCode: String!
///   affectedRows: UserPreferences
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesUpdateResponse {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    pub affected_rows: Option<UserPreferencesPayload>,
}

impl UserPreferencesUpdateResponse {
    /// Check if the preferences update was successful.
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }
}

/// Updated preferences returned after a successful update.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesPayload {
    pub content_filtering_severity_level: Option<String>,
}

/// Content filtering severity level.
///
/// Controls how flagged/reported content is displayed to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ContentFilterLevel {
    /// Stricter: hides all flagged content.
    #[serde(rename = "MYGRANDMALIKES")]
    #[default]
    Strict,
    /// Lenient: shows placeholders for flagged content.
    #[serde(rename = "MYGRANDMAHATES")]
    Lenient,
}

impl ContentFilterLevel {
    /// Convert to the string value expected by the API.
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentFilterLevel::Strict => "MYGRANDMALIKES",
            ContentFilterLevel::Lenient => "MYGRANDMAHATES",
        }
    }

    /// Parse from API string value.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "MYGRANDMALIKES" => Some(ContentFilterLevel::Strict),
            "MYGRANDMAHATES" => Some(ContentFilterLevel::Lenient),
            _ => None,
        }
    }

    /// Check if this level is lenient (shows reported content).
    pub fn is_lenient(&self) -> bool {
        matches!(self, ContentFilterLevel::Lenient)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_response_is_success() {
        let success = UpdateResponse {
            status: "success".to_string(),
            response_code: "11003".to_string(),
        };
        assert!(success.is_success());

        let error = UpdateResponse {
            status: "error".to_string(),
            response_code: "30101".to_string(),
        };
        assert!(!error.is_success());
    }

    #[test]
    fn test_content_filter_level() {
        assert_eq!(ContentFilterLevel::Strict.as_str(), "MYGRANDMALIKES");
        assert_eq!(ContentFilterLevel::Lenient.as_str(), "MYGRANDMAHATES");

        assert_eq!(
            ContentFilterLevel::from_str("MYGRANDMALIKES"),
            Some(ContentFilterLevel::Strict)
        );
        assert_eq!(
            ContentFilterLevel::from_str("MYGRANDMAHATES"),
            Some(ContentFilterLevel::Lenient)
        );
        assert_eq!(ContentFilterLevel::from_str("INVALID"), None);

        assert!(!ContentFilterLevel::Strict.is_lenient());
        assert!(ContentFilterLevel::Lenient.is_lenient());
    }
}
