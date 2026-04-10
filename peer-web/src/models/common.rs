//! Common types shared across API responses.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Standard GraphQL error envelope returned by the Peer API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(default)]
    pub locations: Vec<GraphQLErrorLocation>,
    #[serde(default)]
    pub path: Vec<String>,
    #[serde(default)]
    pub extensions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLErrorLocation {
    pub line: u32,
    pub column: u32,
}

/// Standard response envelope for Peer API mutations.
/// 
/// This matches the `DefaultResponse` type from the GraphQL schema.
/// Most mutations return this inside a `meta` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultResponse {
    pub status: String,
    pub request_id: String,
    pub response_code: String,
    pub response_message: String,
}

/// Status values returned by the Peer API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    Success,
    Error,
    #[serde(other)]
    Unknown,
}

impl Default for ApiStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl ApiStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

/// Application-level API errors.
#[derive(Debug, Clone, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("API error ({code}): {message}")]
    Api { code: String, message: String },

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl ApiError {
    /// Create an API error from a response code.
    pub fn from_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Api {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Response code constants matching `json/response-codes.json`.
pub mod response_codes {
    // Registration success
    pub const REGISTRATION_SUCCESS: &str = "10601";
    pub const ACCOUNT_VERIFIED: &str = "10701";
    pub const REFERRAL_INFO_FETCHED: &str = "11011";

    // Registration errors
    pub const EMAIL_ALREADY_REGISTERED: &str = "30601";
    pub const INVALID_REFERRAL_STRING: &str = "31010";
    pub const INVALID_REFERRAL_UUID: &str = "31007";
    pub const MISSING_REQUIRED_FIELDS: &str = "30301";
    pub const INVALID_USERNAME_FORMAT: &str = "30202";
    pub const INVALID_INPUT_FORMAT: &str = "30103";

    // Server errors
    pub const REGISTRATION_SERVER_ERROR: &str = "40601";
    pub const USERID_GENERATION_FAILED: &str = "40602";
    pub const VERIFICATION_SERVER_ERROR: &str = "40701";
    pub const REFERRAL_SERVER_ERROR: &str = "41013";
}

/// Map response codes to user-friendly messages.
/// 
/// This mirrors the logic in `js/global.js` `userfriendlymsg()` function
/// and the `json/response-codes.json` file.
pub fn user_friendly_message(code: &str) -> &'static str {
    match code {
        // Success codes
        "10601" => "Registration successful! Please check your email to verify your account.",
        "10701" => "Account verified! You can now log in.",
        "11011" => "Referral information loaded.",

        // Registration errors
        "30301" => "Please fill in all required fields.",
        "30601" => "This email is already registered.",
        "30202" => "Invalid username format. Use 3-23 characters with letters, numbers, underscores, or hyphens.",
        "30103" => "Invalid input format.",
        "31007" => "Invalid referral code. The referrer could not be found.",
        "31010" => "Hmm\u{2026} that referral code doesn't seem to work. Ask your friend to send you a new link, or use a Peer code.",

        // Already verified
        "30701" => "This account has already been verified.",

        // Server errors
        "40601" => "Registration failed due to a server error. Please try again later.",
        "40602" => "Failed to generate user ID. Please try again.",
        "40701" => "Account verification failed. Please try again.",
        "41013" => "Unable to verify referral code. Please try again.",

        // Unknown
        _ => "An unexpected error occurred. Please try again.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_status_is_success() {
        assert!(ApiStatus::Success.is_success());
        assert!(!ApiStatus::Error.is_success());
        assert!(!ApiStatus::Unknown.is_success());
    }

    #[test]
    fn test_user_friendly_message_known_codes() {
        assert_eq!(
            user_friendly_message("10601"),
            "Registration successful! Please check your email to verify your account."
        );
        assert_eq!(
            user_friendly_message("30601"),
            "This email is already registered."
        );
    }

    #[test]
    fn test_user_friendly_message_unknown_code() {
        assert_eq!(
            user_friendly_message("99999"),
            "An unexpected error occurred. Please try again."
        );
    }

    #[test]
    fn test_api_status_deserialize() {
        let success: ApiStatus = serde_json::from_str(r#""success""#).unwrap();
        assert_eq!(success, ApiStatus::Success);

        let error: ApiStatus = serde_json::from_str(r#""error""#).unwrap();
        assert_eq!(error, ApiStatus::Error);

        // Unknown values fallback
        let unknown: ApiStatus = serde_json::from_str(r#""something_else""#).unwrap();
        assert_eq!(unknown, ApiStatus::Unknown);
    }
}
