//! Integration tests for registration server functions.
//!
//! These tests call the server functions via their HTTP endpoints,
//! simulating what the WASM client does. They require:
//! 1. The mock backend running on port 4000
//! 2. The Leptos dev server running on port 3000
//!
//! Run with: `cargo test --features ssr --test server_functions_integration`

#![cfg(feature = "ssr")]

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

const SERVER_URL: &str = "http://localhost:3000";

/// Helper: POST to a server function endpoint and parse the response.
async fn call_server_fn<T: for<'de> Deserialize<'de>>(
    path: &str,
    body: serde_json::Value,
) -> Result<T, String> {
    let client = Client::new();
    let response = client
        .post(format!("{}{}", SERVER_URL, path))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?;

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }

    serde_json::from_str(&text).map_err(|e| format!("Deserialize error: {} (body: {})", e, text))
}

// ============================================================================
// Verify Referral Tests
// ============================================================================

#[derive(Debug, Deserialize)]
struct ReferralResponse {
    status: String,
    #[serde(alias = "ResponseCode")]
    response_code: String,
}

#[tokio::test]
async fn test_verify_referral_valid_code() {
    let result: Result<ReferralResponse, _> = call_server_fn(
        "/api/verify_referral",
        json!({ "referral_string": "85d5f836-b1f5-4c4e-9381-1b058e13df93" }),
    )
    .await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
    let data = result.unwrap();
    assert_eq!(data.status, "success");
    assert_eq!(data.response_code, "11011");
}

#[tokio::test]
async fn test_verify_referral_invalid_format() {
    let result: Result<ReferralResponse, _> = call_server_fn(
        "/api/verify_referral",
        json!({ "referral_string": "not-a-valid-uuid" }),
    )
    .await;

    // Should be rejected by server-side validation before hitting GraphQL
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_referral_unknown_code() {
    let result: Result<ReferralResponse, _> = call_server_fn(
        "/api/verify_referral",
        json!({ "referral_string": "00000000-0000-0000-0000-000000000000" }),
    )
    .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.status, "error");
    assert_eq!(data.response_code, "31010");
}

// ============================================================================
// Register User Tests
// ============================================================================

#[derive(Debug, Deserialize)]
struct RegResponse {
    status: String,
    #[serde(alias = "ResponseCode")]
    response_code: String,
    userid: Option<String>,
}

#[tokio::test]
async fn test_register_user_success() {
    let unique_email = format!("servfn_{}@example.com", uuid::Uuid::new_v4());

    let result: Result<RegResponse, _> = call_server_fn(
        "/api/register_user",
        json!({
            "email": unique_email,
            "password": "SecurePass123!",
            "username": "testuser_sf",
            "referral_uuid": "85d5f836-b1f5-4c4e-9381-1b058e13df93"
        }),
    )
    .await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
    let data = result.unwrap();
    assert_eq!(data.status, "success");
    assert_eq!(data.response_code, "10601");
    assert!(data.userid.is_some());
}

#[tokio::test]
async fn test_register_user_invalid_email() {
    let result: Result<RegResponse, _> = call_server_fn(
        "/api/register_user",
        json!({
            "email": "not-an-email",
            "password": "SecurePass123!",
            "username": "testuser",
            "referral_uuid": "85d5f836-b1f5-4c4e-9381-1b058e13df93"
        }),
    )
    .await;

    // Should be rejected by server-side validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_user_short_password() {
    let result: Result<RegResponse, _> = call_server_fn(
        "/api/register_user",
        json!({
            "email": "valid@example.com",
            "password": "short",
            "username": "testuser",
            "referral_uuid": "85d5f836-b1f5-4c4e-9381-1b058e13df93"
        }),
    )
    .await;

    // Should be rejected by server-side validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let email = format!("dup_sf_{}@example.com", uuid::Uuid::new_v4());

    // First registration
    let _ = call_server_fn::<RegResponse>(
        "/api/register_user",
        json!({
            "email": email,
            "password": "SecurePass123!",
            "username": "user1_sf",
            "referral_uuid": "85d5f836-b1f5-4c4e-9381-1b058e13df93"
        }),
    )
    .await;

    // Second registration with same email
    let result: Result<RegResponse, _> = call_server_fn(
        "/api/register_user",
        json!({
            "email": email,
            "password": "SecurePass456!",
            "username": "user2_sf",
            "referral_uuid": "85d5f836-b1f5-4c4e-9381-1b058e13df93"
        }),
    )
    .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.status, "error");
    assert_eq!(data.response_code, "30601");
}

// ============================================================================
// Verify Account Tests
// ============================================================================

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    status: String,
    #[serde(alias = "ResponseCode")]
    response_code: String,
}

#[tokio::test]
async fn test_verify_account_success() {
    // First register a user to get a userid
    let unique_email = format!("verify_sf_{}@example.com", uuid::Uuid::new_v4());

    let reg_result: RegResponse = call_server_fn(
        "/api/register_user",
        json!({
            "email": unique_email,
            "password": "SecurePass123!",
            "username": "verify_sf",
            "referral_uuid": "85d5f836-b1f5-4c4e-9381-1b058e13df93"
        }),
    )
    .await
    .expect("Registration should succeed");

    let userid = reg_result.userid.expect("Should have userid");

    // Now verify the account
    let result: Result<VerifyResponse, _> =
        call_server_fn("/api/verify_account", json!({ "userid": userid })).await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
    let data = result.unwrap();
    assert_eq!(data.status, "success");
    assert_eq!(data.response_code, "10701");
}

#[tokio::test]
async fn test_verify_account_empty_userid() {
    let result: Result<VerifyResponse, _> =
        call_server_fn("/api/verify_account", json!({ "userid": "" })).await;

    // Should be rejected by validation
    assert!(result.is_err());
}
