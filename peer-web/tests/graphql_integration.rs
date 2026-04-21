//! Integration tests for the GraphQL client module.
//!
//! Uses `mock_backend` as an **in-process library** — no separate service
//! on port 4000 is required. Run with: `cargo test --features ssr`

#![cfg(feature = "ssr")]

mod common;

use peer_web::api::graphql::{
    REGISTER_MUTATION, RegisterData, VERIFY_ACCOUNT_MUTATION, VERIFY_REFERRAL_MUTATION,
    VerifyAccountData, VerifyReferralData, mutate,
};
use peer_web::models::user::RegistrationInput;
use serde::Serialize;
use std::env;

/// Boot the in-process mock backend and return its `/graphql` URL plus a
/// read guard on the env var. Hold the guard for the whole test body so
/// that the `test_network_error_wrong_endpoint` test cannot swap the env
/// var out from under us.
async fn setup() -> (String, tokio::sync::RwLockReadGuard<'static, ()>) {
    let endpoint = common::mock_graphql_endpoint().await;
    let guard = common::endpoint_read_guard().await;
    (endpoint, guard)
}

// ============================================================================
// Verify Referral Tests
// ============================================================================

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerifyReferralVars {
    referral_string: String,
}

#[tokio::test]
async fn test_verify_referral_success() {
    let _env = setup().await;

    let vars = VerifyReferralVars {
        referral_string: "85d5f836-b1f5-4c4e-9381-1b058e13df93".to_string(),
    };

    let result: Result<VerifyReferralData, _> = mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());

    let data = result.unwrap();
    assert!(data.verify_referral_string.is_success());
    assert_eq!(data.verify_referral_string.response_code, "11011");
    assert!(data.verify_referral_string.affected_rows.is_some());

    let users = data.verify_referral_string.affected_rows.unwrap();
    assert_eq!(users.len(), 1);
    assert!(!users[0].username.is_empty());
}

#[tokio::test]
async fn test_verify_referral_invalid_code() {
    let _env = setup().await;

    let vars = VerifyReferralVars {
        referral_string: "00000000-0000-0000-0000-000000000000".to_string(),
    };

    let result: Result<VerifyReferralData, _> = mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

    // The mutation itself succeeds, but returns status: "error"
    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(!data.verify_referral_string.is_success());
    assert_eq!(data.verify_referral_string.response_code, "31010");
}

#[tokio::test]
async fn test_verify_referral_malformed_string() {
    let _env = setup().await;

    let vars = VerifyReferralVars {
        referral_string: "not-a-valid-uuid".to_string(),
    };

    let result: Result<VerifyReferralData, _> = mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(!data.verify_referral_string.is_success());
    assert!(["31010", "31007"].contains(&data.verify_referral_string.response_code.as_str()));
}

// ============================================================================
// Register Tests
// ============================================================================

#[derive(Serialize)]
struct RegisterVars {
    input: RegistrationInput,
}

#[tokio::test]
async fn test_register_success() {
    let _env = setup().await;

    let unique_email = format!("test_{}@example.com", uuid::Uuid::new_v4());

    let vars = RegisterVars {
        input: RegistrationInput::new(unique_email, "SecurePass123!", "testuser_new"),
    };

    let result: Result<RegisterData, _> = mutate(REGISTER_MUTATION, vars, None).await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());

    let data = result.unwrap();
    assert!(data.register.is_success());
    assert_eq!(data.register.response_code.as_deref(), Some("10601"));
    assert!(data.register.user_id.is_some());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let _env = setup().await;

    let email = format!("duplicate_{}@example.com", uuid::Uuid::new_v4());

    let vars1 = RegisterVars {
        input: RegistrationInput::new(email.clone(), "SecurePass123!", "user1"),
    };

    let _ = mutate::<_, RegisterData>(REGISTER_MUTATION, vars1, None).await;

    let vars2 = RegisterVars {
        input: RegistrationInput::new(email, "SecurePass456!", "user2"),
    };

    let result: Result<RegisterData, _> = mutate(REGISTER_MUTATION, vars2, None).await;

    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(!data.register.is_success());
    assert_eq!(data.register.response_code.as_deref(), Some("30601"));
}

// Mirrors end2end/tests/registration/server-errors.spec.ts T4b:
// a `fail@…` email triggers a simulated internal-server-error (40601).
#[tokio::test]
async fn test_register_fail_email_simulated_error() {
    let _env = setup().await;

    let vars = RegisterVars {
        input: RegistrationInput::new(
            "fail@example.com".to_string(),
            "SecurePass123!",
            "fail_user",
        ),
    };

    let result: Result<RegisterData, _> = mutate(REGISTER_MUTATION, vars, None).await;

    assert!(result.is_ok(), "mutation envelope should deliver the error");
    let data = result.unwrap();
    assert!(!data.register.is_success());
    assert_eq!(data.register.response_code.as_deref(), Some("40601"));
}

// ============================================================================
// Verify Account Tests
// ============================================================================

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerifyAccountVars {
    user_id: String,
}

#[tokio::test]
async fn test_verify_account_success() {
    let _env = setup().await;

    let unique_email = format!("verify_{}@example.com", uuid::Uuid::new_v4());

    let register_vars = RegisterVars {
        input: RegistrationInput::new(unique_email, "SecurePass123!", "verify_user"),
    };

    let register_result: RegisterData = mutate(REGISTER_MUTATION, register_vars, None)
        .await
        .unwrap();

    let userid = register_result.register.user_id.unwrap();

    let vars = VerifyAccountVars { user_id: userid };

    let result: Result<VerifyAccountData, _> = mutate(VERIFY_ACCOUNT_MUTATION, vars, None).await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());

    let data = result.unwrap();
    assert!(data.verify_account.is_success());
    assert_eq!(data.verify_account.response_code, "10701");
}

#[tokio::test]
async fn test_verify_account_already_verified() {
    let _env = setup().await;

    let unique_email = format!("already_verified_{}@example.com", uuid::Uuid::new_v4());

    let register_vars = RegisterVars {
        input: RegistrationInput::new(unique_email, "SecurePass123!", "already_verified"),
    };

    let register_result: RegisterData = mutate(REGISTER_MUTATION, register_vars, None)
        .await
        .unwrap();

    let userid = register_result.register.user_id.unwrap();

    // First verification
    let vars1 = VerifyAccountVars {
        user_id: userid.clone(),
    };
    let _ = mutate::<_, VerifyAccountData>(VERIFY_ACCOUNT_MUTATION, vars1, None).await;

    // Second verification
    let vars2 = VerifyAccountVars { user_id: userid };

    let result: Result<VerifyAccountData, _> = mutate(VERIFY_ACCOUNT_MUTATION, vars2, None).await;

    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(["10701", "30701"].contains(&data.verify_account.response_code.as_str()));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_network_error_wrong_endpoint() {
    // Ensure the mock backend is up, then take an *exclusive* lock on the
    // env var for the duration of this test so we don't fight with other
    // concurrent tests that expect the real endpoint.
    let real_endpoint = common::mock_graphql_endpoint().await;
    let _env = common::endpoint_write_guard().await;

    // Point to a port that nothing is listening on.
    // We run this test serially-safe by picking a high port unlikely to conflict.
    unsafe {
        env::set_var("GRAPHQL_ENDPOINT", "http://127.0.0.1:59999/graphql");
    }

    let vars = VerifyReferralVars {
        referral_string: "test".to_string(),
    };

    // Run the mutation against the bad endpoint
    let result: Result<VerifyReferralData, _> = mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

    // Restore the correct endpoint immediately so other tests aren't affected
    unsafe {
        env::set_var("GRAPHQL_ENDPOINT", &real_endpoint);
    }

    assert!(result.is_err(), "Expected connection error, got success");

    match result.unwrap_err() {
        peer_web::models::common::ApiError::Network(msg) => {
            assert!(
                msg.to_lowercase().contains("connect"),
                "Expected connection error message, got: {}",
                msg
            );
        }
        other => panic!("Expected Network error, got: {:?}", other),
    }
}
