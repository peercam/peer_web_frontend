//! Server functions for the password reset flow.
//!
//! All three mutations use the guest schema (no auth required).
//!
//! ## Flow
//!
//! 1. `request_password_reset(email)` — Request a reset code
//! 2. `verify_reset_token(token)` — Verify the code is valid
//! 3. `reset_password(token, password)` — Set new password

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Types
// ============================================================================

/// Response payload for password reset mutations.
///
/// Mirrors the GraphQL response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordPayload {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: Option<String>,
}

impl ResetPasswordPayload {
    /// Check if the operation was successful.
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }

    /// Get the response code as a string slice.
    pub fn code(&self) -> Option<&str> {
        self.response_code.as_deref()
    }
}

// ============================================================================
// Server function: Request Password Reset (Step 1)
// ============================================================================

/// Request a password reset email.
///
/// Sends a reset code to the provided email address (if it exists).
/// Always returns success to prevent user enumeration.
#[server(RequestPasswordReset, "/api")]
pub async fn request_password_reset(email: String) -> Result<ResetPasswordPayload, ServerFnError> {
    use crate::api::graphql::mutate;

    // Basic server-side validation
    if email.is_empty() {
        return Err(ServerFnError::new("Email is required."));
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(ServerFnError::new("Invalid email format."));
    }

    const QUERY: &str = r#"
        mutation RequestPasswordReset($email: String!) {
            requestPasswordReset(email: $email) {
                status
                ResponseCode
            }
        }
    "#;

    #[derive(Serialize)]
    struct Vars {
        email: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        request_password_reset: ResetPasswordPayload,
    }

    let data: Data = mutate(QUERY, Vars { email }, None).await?;
    Ok(data.request_password_reset)
}

// ============================================================================
// Server function: Verify Reset Token (Step 2)
// ============================================================================

/// Verify a password reset token/code.
///
/// Checks if the provided token is valid and not expired.
#[server(VerifyResetToken, "/api")]
pub async fn verify_reset_token(token: String) -> Result<ResetPasswordPayload, ServerFnError> {
    use crate::api::graphql::mutate;

    if token.trim().is_empty() {
        return Err(ServerFnError::new("Code is required."));
    }

    const QUERY: &str = r#"
        mutation ResetPasswordTokenVerify($token: String!) {
            resetPasswordTokenVerify(token: $token) {
                status
                ResponseCode
            }
        }
    "#;

    #[derive(Serialize)]
    struct Vars {
        token: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        reset_password_token_verify: ResetPasswordPayload,
    }

    let data: Data = mutate(QUERY, Vars { token }, None).await?;
    Ok(data.reset_password_token_verify)
}

// ============================================================================
// Server function: Reset Password (Step 3)
// ============================================================================

/// Reset the user's password using a valid token.
///
/// Updates the password and invalidates all existing sessions.
#[server(ResetPassword, "/api")]
pub async fn reset_password(
    token: String,
    password: String,
) -> Result<ResetPasswordPayload, ServerFnError> {
    use crate::api::graphql::mutate;
    use crate::api::validation::validate_password;

    if token.trim().is_empty() {
        return Err(ServerFnError::new("Reset token is missing."));
    }

    // Server-side password validation
    validate_password(&password).map_err(|e| ServerFnError::new(e.to_string()))?;

    const QUERY: &str = r#"
        mutation ResetPassword($token: String!, $password: String!) {
            resetPassword(token: $token, password: $password) {
                status
                ResponseCode
            }
        }
    "#;

    #[derive(Serialize)]
    struct Vars {
        token: String,
        password: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        reset_password: ResetPasswordPayload,
    }

    let data: Data = mutate(QUERY, Vars { token, password }, None).await?;
    Ok(data.reset_password)
}
