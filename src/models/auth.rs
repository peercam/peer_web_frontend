//! Authentication-related types.
//!
//! These structs mirror the GraphQL types used in the login/refresh/logout flow.

use serde::{Deserialize, Serialize};

/// JWT token pair returned by login/refresh mutations.
///
/// ```graphql
/// type AuthPayload {
///   status: String!
///   ResponseCode: String
///   accessToken: String
///   refreshToken: String
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthPayload {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl AuthPayload {
    /// Returns the response code as a string slice, if present.
    pub fn code(&self) -> Option<&str> {
        self.response_code.as_deref()
    }

    /// Check if the login/refresh was successful (code 10801 or 10901).
    pub fn is_success(&self) -> bool {
        matches!(self.code(), Some("10801") | Some("10901"))
    }
}

/// Logout response from the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutPayload {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: Option<String>,
}

/// Typed login response codes for compile-time safety.
#[derive(Debug, Clone, PartialEq)]
pub enum LoginResponseCode {
    /// 10801 — Login successful.
    Success,
    /// 30801 — Invalid credentials.
    InvalidCredentials,
    /// 60801 — Account not verified.
    NotVerified,
    /// 40801 — Internal server error.
    ServerError,
    /// Unknown code.
    Unknown(String),
}

impl From<&str> for LoginResponseCode {
    fn from(code: &str) -> Self {
        match code {
            "10801" => Self::Success,
            "30801" => Self::InvalidCredentials,
            "60801" => Self::NotVerified,
            "40801" => Self::ServerError,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl LoginResponseCode {
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Success => "Login successful.",
            Self::InvalidCredentials => "Invalid email or password. Please try again.",
            Self::NotVerified => "Your account is not verified. Please check your email.",
            Self::ServerError => "Something went wrong. Please try again later.",
            Self::Unknown(_) => "Login failed. Please try again.",
        }
    }
}

/// Typed refresh token response codes.
#[derive(Debug, Clone, PartialEq)]
pub enum RefreshResponseCode {
    /// 10901 — Token refreshed successfully.
    Success,
    /// 30101 — Missing refresh token.
    MissingToken,
    /// 30901 — Invalid/expired/revoked token.
    InvalidToken,
    /// 40901 — Internal server error.
    ServerError,
    /// Unknown code.
    Unknown(String),
}

impl From<&str> for RefreshResponseCode {
    fn from(code: &str) -> Self {
        match code {
            "10901" => Self::Success,
            "30101" => Self::MissingToken,
            "30901" => Self::InvalidToken,
            "40901" => Self::ServerError,
            other => Self::Unknown(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_response_code_from_str() {
        assert_eq!(LoginResponseCode::from("10801"), LoginResponseCode::Success);
        assert_eq!(
            LoginResponseCode::from("30801"),
            LoginResponseCode::InvalidCredentials
        );
        assert_eq!(
            LoginResponseCode::from("60801"),
            LoginResponseCode::NotVerified
        );
        assert_eq!(
            LoginResponseCode::from("99999"),
            LoginResponseCode::Unknown("99999".to_string())
        );
    }

    #[test]
    fn refresh_response_code_from_str() {
        assert_eq!(
            RefreshResponseCode::from("10901"),
            RefreshResponseCode::Success
        );
        assert_eq!(
            RefreshResponseCode::from("30901"),
            RefreshResponseCode::InvalidToken
        );
    }

    #[test]
    fn auth_payload_is_success() {
        let payload = AuthPayload {
            status: "success".to_string(),
            response_code: Some("10801".to_string()),
            access_token: Some("token".to_string()),
            refresh_token: Some("refresh".to_string()),
        };
        assert!(payload.is_success());

        let failed = AuthPayload {
            status: "error".to_string(),
            response_code: Some("30801".to_string()),
            access_token: None,
            refresh_token: None,
        };
        assert!(!failed.is_success());
    }
}
