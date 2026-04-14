//! Tests that verify JSON fixtures deserialize into our model types.

use peer_web::models::user::{ReferralVerifyResponse, RegisterResponse, VerifyAccountResponse};
use serde::Deserialize;
use std::fs;

/// Wrapper for GraphQL response envelope.
#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct ReferralData {
    #[serde(rename = "verifyReferralString")]
    verify_referral_string: ReferralVerifyResponse,
}

#[derive(Deserialize)]
struct RegisterData {
    register: RegisterResponse,
}

#[derive(Deserialize)]
struct VerifyData {
    #[serde(rename = "verifyAccount")]
    verify_account: VerifyAccountResponse,
}

#[test]
fn test_referral_success_fixture() {
    let json = fs::read_to_string("src/fixtures/referral_success.json")
        .expect("Failed to read fixture");
    
    let response: GraphQLResponse<ReferralData> =
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    let data = response.data.verify_referral_string;
    assert!(data.is_success());
    assert_eq!(data.response_code, "11011");
    
    let referrer = data.referrer().expect("Should have referrer");
    assert_eq!(referrer.username, "peer_referrer");
}

#[test]
fn test_referral_invalid_fixture() {
    let json = fs::read_to_string("src/fixtures/referral_invalid.json")
        .expect("Failed to read fixture");
    
    let response: GraphQLResponse<ReferralData> =
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    let data = response.data.verify_referral_string;
    assert!(!data.is_success());
    assert_eq!(data.response_code, "31010");
    assert!(data.referrer().is_none());
}

#[test]
fn test_register_success_fixture() {
    let json = fs::read_to_string("src/fixtures/register_success.json")
        .expect("Failed to read fixture");
    
    let response: GraphQLResponse<RegisterData> =
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    let data = response.data.register;
    assert!(data.is_success());
    assert_eq!(data.code(), Some("10601"));
    assert!(data.user_id.is_some());
}

#[test]
fn test_register_duplicate_fixture() {
    let json = fs::read_to_string("src/fixtures/register_duplicate.json")
        .expect("Failed to read fixture");
    
    let response: GraphQLResponse<RegisterData> =
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    let data = response.data.register;
    assert!(!data.is_success());
    assert_eq!(data.code(), Some("30601"));
    assert!(data.user_id.is_none());
}

#[test]
fn test_verify_success_fixture() {
    let json = fs::read_to_string("src/fixtures/verify_success.json")
        .expect("Failed to read fixture");
    
    let response: GraphQLResponse<VerifyData> =
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    let data = response.data.verify_account;
    assert!(data.is_success());
    assert_eq!(data.response_code, "10701");
}
