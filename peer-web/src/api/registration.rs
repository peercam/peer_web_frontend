//! Leptos server functions for the registration flow.
//!
//! These functions are the bridge between client-side WASM components
//! and the backend GraphQL API. Each function:
//!
//! 1. Runs on the Axum server (never in WASM)
//! 2. Validates input (defense-in-depth)
//! 3. Calls the GraphQL API via `api::graphql::mutate`
//! 4. Unwraps the GraphQL data envelope
//! 5. Returns clean domain types to the caller
//!
//! From the client side, these are called like normal async functions:
//! ```rust,ignore
//! let result = verify_referral("85d5f836-...".into()).await;
//! ```
//!
//! Leptos auto-generates the corresponding `POST /api/<fn_name>` endpoint.

use leptos::prelude::*;

use crate::models::user::{
    ReferralVerifyResponse, RegisterResponse, VerifyAccountResponse,
};

// ============================================================================
// Server function: Verify Referral Code
// ============================================================================

/// Verify a referral code against the backend.
///
/// Validates the UUID format on the server side, then calls the
/// `verifyReferralString` GraphQL mutation.
#[server(VerifyReferral, "/api")]
pub async fn verify_referral(
    referral_string: String,
) -> Result<ReferralVerifyResponse, ServerFnError> {
    use crate::api::graphql::{mutate, VerifyReferralData, VERIFY_REFERRAL_MUTATION};
    use crate::api::validation::validate_uuid;

    // 1. Server-side validation
    validate_uuid(&referral_string)?;

    // 2. Build variables matching the GraphQL mutation signature
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Vars {
        referral_string: String,
    }

    let variables = Vars { referral_string };

    // 3. Call the GraphQL API
    let data: VerifyReferralData = mutate(VERIFY_REFERRAL_MUTATION, variables, None).await?;

    // 4. Unwrap the GraphQL envelope and return the domain type
    Ok(data.verify_referral_string)
}

// ============================================================================
// Server function: Register User
// ============================================================================

/// Register a new user account.
///
/// Validates input fields on the server side, then calls the `register`
/// GraphQL mutation. On success, the response includes the new user's ID
/// which is needed for the subsequent `verify_account` call.
#[server(RegisterUser, "/api")]
pub async fn register_user(
    email: String,
    password: String,
    username: String,
    referral_uuid: String,
) -> Result<RegisterResponse, ServerFnError> {
    use crate::api::graphql::{mutate, RegisterData, REGISTER_MUTATION};
    use crate::api::validation::{validate_registration_input, validate_uuid};
    use crate::models::user::RegistrationInput;

    // 1. Server-side validation
    validate_registration_input(&email, &username, &password)?;
    validate_uuid(&referral_uuid)?;

    // 2. Construct the RegistrationInput
    let input = RegistrationInput::new(&email, &password, &username).with_referral(referral_uuid);

    // 3. Build variables matching the GraphQL mutation signature
    #[derive(serde::Serialize)]
    struct Vars {
        input: RegistrationInput,
    }

    let variables = Vars { input };

    // 4. Call the GraphQL API
    let data: RegisterData = mutate(REGISTER_MUTATION, variables, None).await?;

    // 5. Return the domain type
    Ok(data.register)
}

// ============================================================================
// Server function: Verify Account
// ============================================================================

/// Verify a newly registered account.
///
/// Called immediately after a successful registration to activate the account.
/// In the current Peer flow, this is triggered automatically (not by the user
/// clicking an email link).
#[server(VerifyAccount, "/api")]
pub async fn verify_account(userid: String) -> Result<VerifyAccountResponse, ServerFnError> {
    use crate::api::graphql::{mutate, VerifyAccountData, VERIFY_ACCOUNT_MUTATION};

    // 1. Basic validation — userid should be non-empty
    if userid.trim().is_empty() {
        return Err(ServerFnError::new("User ID is required"));
    }

    // 2. Build variables matching the GraphQL mutation signature
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Vars {
        user_id: String,
    }

    let variables = Vars { user_id: userid };

    // 3. Call the GraphQL API
    let data: VerifyAccountData = mutate(VERIFY_ACCOUNT_MUTATION, variables, None).await?;

    // 4. Return the domain type
    Ok(data.verify_account)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    /// Test that verify_referral rejects non-UUID strings before hitting GraphQL.
    #[tokio::test]
    async fn test_verify_referral_rejects_bad_format() {
        let result = verify_referral("not-a-uuid".into()).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Invalid referral code") || err.contains("31010"),
            "Error should mention invalid format, got: {}",
            err
        );
    }

    /// Test that verify_referral rejects empty strings.
    #[tokio::test]
    async fn test_verify_referral_rejects_empty_string() {
        let result = verify_referral("".into()).await;
        assert!(result.is_err());
    }

    /// Test that register_user rejects invalid email.
    #[tokio::test]
    async fn test_register_user_rejects_invalid_email() {
        let result = register_user(
            "no-at-sign".into(),
            "SecurePass123!".into(),
            "testuser".into(),
            "85d5f836-b1f5-4c4e-9381-1b058e13df93".into(),
        )
        .await;
        assert!(result.is_err());
    }

    /// Test that register_user rejects short passwords.
    #[tokio::test]
    async fn test_register_user_rejects_short_password() {
        let result = register_user(
            "user@example.com".into(),
            "short".into(),
            "testuser".into(),
            "85d5f836-b1f5-4c4e-9381-1b058e13df93".into(),
        )
        .await;
        assert!(result.is_err());
    }

    /// Test that register_user rejects bad username.
    #[tokio::test]
    async fn test_register_user_rejects_bad_username() {
        let result = register_user(
            "user@example.com".into(),
            "SecurePass123!".into(),
            "ab".into(), // too short
            "85d5f836-b1f5-4c4e-9381-1b058e13df93".into(),
        )
        .await;
        assert!(result.is_err());
    }

    /// Test that register_user rejects invalid referral UUID.
    #[tokio::test]
    async fn test_register_user_rejects_bad_referral_uuid() {
        let result = register_user(
            "user@example.com".into(),
            "SecurePass123!".into(),
            "testuser".into(),
            "not-a-uuid".into(),
        )
        .await;
        assert!(result.is_err());
    }

    /// Test that verify_account rejects empty userid.
    #[tokio::test]
    async fn test_verify_account_rejects_empty_userid() {
        let result = verify_account("".into()).await;
        assert!(result.is_err());
    }

    /// Test that verify_account rejects whitespace-only userid.
    #[tokio::test]
    async fn test_verify_account_rejects_whitespace_userid() {
        let result = verify_account("   ".into()).await;
        assert!(result.is_err());
    }
}
