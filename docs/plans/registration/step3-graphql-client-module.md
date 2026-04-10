# Step 3 — GraphQL Client Module

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Implement a reusable async GraphQL client in `src/api/graphql.rs` that sends queries/mutations to the backend and deserializes typed responses. This module abstracts all HTTP communication with the Peer GraphQL API and provides a type-safe foundation for server functions in Step 4.

---

## 3.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 2 complete | `cd peer-web && cargo check` | Compiles without errors |
| Mock backend running | `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` | Returns `{"data":{"_health":true}}` |
| Models defined | Check `src/models/user.rs` exists | Contains `RegistrationInput`, response types |
| Dependencies in Cargo.toml | Check `reqwest` with `json` feature | Present under `[dependencies]` with `optional = true` |

---

## 3.2 — Architecture Overview

The GraphQL client module follows a layered design:

```
┌─────────────────────────────────────────────────────────────┐
│                     Server Functions                         │
│               (Step 4 — calls graphql module)                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   src/api/graphql.rs                         │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  mutate<V, R>() — Generic mutation executor             ││
│  │  query<V, R>()  — Generic query executor (future use)   ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  GraphQLRequest<V>  — Request envelope                  ││
│  │  GraphQLResponse<T> — Response envelope with errors     ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  get_endpoint() — Read GRAPHQL_ENDPOINT from env        ││
│  │  build_client() — Configure reqwest client with timeout ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Mock Backend (port 4000)                 │
│                  or Production API (future)                  │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| SSR-only (`#[cfg(feature = "ssr")]`) | GraphQL calls run on the Axum server, not in WASM — keeps secrets server-side and avoids CORS |
| Generic functions | Reusable across all mutations (registration, login, etc.) |
| Typed responses | Deserialize directly into Rust structs for compile-time safety |
| Explicit error handling | Transform network/parsing errors into `ApiError` enum |
| Configurable endpoint | Read from environment for easy dev/staging/prod switching |

---

## 3.3 — File: `src/api/graphql.rs`

Create the complete GraphQL client module:

```rust
//! Generic GraphQL client for communicating with the Peer API.
//!
//! This module provides type-safe query and mutation helpers that:
//! - Serialize variables to JSON
//! - Send POST requests to the configured GraphQL endpoint
//! - Deserialize typed responses
//! - Handle the `{ data, errors }` envelope
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::api::graphql::mutate;
//! use crate::models::user::{ReferralVerifyResponse};
//!
//! let response: ReferralVerifyResponse = mutate(
//!     VERIFY_REFERRAL_MUTATION,
//!     serde_json::json!({ "referralString": "85d5f836-..." }),
//!     None, // No auth token for guest mutations
//! ).await?;
//! ```

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::models::common::{ApiError, GraphQLError};

/// GraphQL request envelope.
///
/// Matches the standard GraphQL-over-HTTP POST body format:
/// ```json
/// {
///   "query": "mutation { ... }",
///   "variables": { ... },
///   "operationName": "OptionalName"
/// }
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLRequest<V: Serialize> {
    pub query: &'static str,
    pub variables: V,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<&'static str>,
}

impl<V: Serialize> GraphQLRequest<V> {
    /// Create a new GraphQL request.
    pub fn new(query: &'static str, variables: V) -> Self {
        Self {
            query,
            variables,
            operation_name: None,
        }
    }

    /// Create a request with an explicit operation name.
    pub fn with_operation(query: &'static str, variables: V, operation_name: &'static str) -> Self {
        Self {
            query,
            variables,
            operation_name: Some(operation_name),
        }
    }
}

/// GraphQL response envelope.
///
/// All GraphQL responses follow this shape:
/// ```json
/// {
///   "data": { ... },      // Present on success (may be null)
///   "errors": [ ... ]     // Present on error (may be absent)
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphQLError>,
}

impl<T> GraphQLResponse<T> {
    /// Check if the response contains any GraphQL errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the first error message, if any.
    pub fn first_error_message(&self) -> Option<&str> {
        self.errors.first().map(|e| e.message.as_str())
    }

    /// Extract data, returning an error if data is missing or errors are present.
    pub fn into_result(self) -> Result<T, ApiError> {
        if let Some(err) = self.errors.first() {
            return Err(ApiError::GraphQL(err.message.clone()));
        }

        self.data.ok_or_else(|| {
            ApiError::Unexpected("GraphQL response contained neither data nor errors".to_string())
        })
    }
}

// ============================================================================
// Server-side implementation (SSR only)
// ============================================================================

#[cfg(feature = "ssr")]
mod ssr {
    use super::*;
    use std::env;
    use std::time::Duration;

    /// Default timeout for GraphQL requests (10 seconds).
    const DEFAULT_TIMEOUT_SECS: u64 = 10;

    /// Get the GraphQL endpoint URL from environment.
    ///
    /// Reads `GRAPHQL_ENDPOINT` env var, falling back to local mock backend.
    pub fn get_endpoint() -> String {
        env::var("GRAPHQL_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string())
    }

    /// Build a configured reqwest client.
    ///
    /// Configures:
    /// - 10-second timeout
    /// - JSON content-type
    /// - Accept header
    fn build_client() -> Result<reqwest::Client, ApiError> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(|e| ApiError::Network(format!("Failed to build HTTP client: {}", e)))
    }

    /// Execute a GraphQL mutation and deserialize the response.
    ///
    /// # Type Parameters
    ///
    /// - `V`: The variables type (must implement `Serialize`)
    /// - `R`: The expected response data type (must implement `DeserializeOwned`)
    ///
    /// # Arguments
    ///
    /// - `query`: The GraphQL mutation string
    /// - `variables`: Mutation variables (will be serialized to JSON)
    /// - `auth_token`: Optional Bearer token for authenticated mutations
    ///
    /// # Returns
    ///
    /// The deserialized response data on success, or an `ApiError` on failure.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Serialize)]
    /// struct VerifyVars { referral_string: String }
    ///
    /// #[derive(Deserialize)]
    /// struct VerifyData { verify_referral_string: ReferralVerifyResponse }
    ///
    /// let vars = VerifyVars { referral_string: "...".into() };
    /// let data: VerifyData = mutate(MUTATION, vars, None).await?;
    /// ```
    pub async fn mutate<V, R>(
        query: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        execute_request(query, variables, auth_token).await
    }

    /// Execute a GraphQL query and deserialize the response.
    ///
    /// Identical to `mutate` but semantically for queries.
    /// Provided for clarity in calling code.
    pub async fn query<V, R>(
        query_str: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        execute_request(query_str, variables, auth_token).await
    }

    /// Internal: Execute a GraphQL request (query or mutation).
    async fn execute_request<V, R>(
        query: &'static str,
        variables: V,
        auth_token: Option<&str>,
    ) -> Result<R, ApiError>
    where
        V: Serialize,
        R: DeserializeOwned,
    {
        let client = build_client()?;
        let endpoint = get_endpoint();

        let request_body = GraphQLRequest::new(query, variables);

        let mut request = client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // Add authorization header if token provided
        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        // Send the request
        let response = request
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Network("Request timed out".to_string())
                } else if e.is_connect() {
                    ApiError::Network(format!("Failed to connect to {}: {}", endpoint, e))
                } else {
                    ApiError::Network(format!("Request failed: {}", e))
                }
            })?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Network(format!(
                "HTTP {} from {}: {}",
                status, endpoint, body
            )));
        }

        // Parse response body
        let response_text = response.text().await.map_err(|e| {
            ApiError::Deserialization(format!("Failed to read response body: {}", e))
        })?;

        // Deserialize GraphQL envelope
        let graphql_response: GraphQLResponse<R> =
            serde_json::from_str(&response_text).map_err(|e| {
                ApiError::Deserialization(format!(
                    "Failed to parse GraphQL response: {}. Body: {}",
                    e,
                    truncate_for_error(&response_text, 200)
                ))
            })?;

        // Extract data or convert errors
        graphql_response.into_result()
    }

    /// Truncate a string for error messages.
    fn truncate_for_error(s: &str, max_len: usize) -> &str {
        if s.len() <= max_len {
            s
        } else {
            &s[..max_len]
        }
    }
}

// Re-export SSR functions at module level
#[cfg(feature = "ssr")]
pub use ssr::*;

// ============================================================================
// GraphQL query/mutation string constants
// ============================================================================

/// Mutation: Verify a referral code.
///
/// Corresponds to JS: `mutation VerifyReferralString ($referralString: String!) { ... }`
pub const VERIFY_REFERRAL_MUTATION: &str = r#"
mutation VerifyReferralString($referralString: String!) {
    verifyReferralString(referralString: $referralString) {
        status
        ResponseCode
        affectedRows {
            uid
            username
            slug
            img
        }
    }
}
"#;

/// Mutation: Register a new user.
///
/// Corresponds to JS: `mutation Register($input: RegistrationInput!) { ... }`
pub const REGISTER_MUTATION: &str = r#"
mutation Register($input: RegistrationInput!) {
    register(input: $input) {
        status
        ResponseCode
        userid
    }
}
"#;

/// Mutation: Verify a user account after registration.
///
/// Corresponds to JS: `mutation VerifiedAccount($userId: ID!) { ... }`
pub const VERIFY_ACCOUNT_MUTATION: &str = r#"
mutation VerifyAccount($userId: ID!) {
    verifyAccount(userid: $userId) {
        status
        ResponseCode
    }
}
"#;

// ============================================================================
// Response wrapper types for GraphQL data field
// ============================================================================

/// Wrapper for the `verifyReferralString` mutation response.
///
/// The GraphQL response has shape:
/// ```json
/// { "data": { "verifyReferralString": { ... } } }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReferralData {
    pub verify_referral_string: crate::models::user::ReferralVerifyResponse,
}

/// Wrapper for the `register` mutation response.
#[derive(Debug, Deserialize)]
pub struct RegisterData {
    pub register: crate::models::user::RegisterResponse,
}

/// Wrapper for the `verifyAccount` mutation response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyAccountData {
    pub verify_account: crate::models::user::VerifyAccountResponse,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_serialization() {
        #[derive(Serialize)]
        struct TestVars {
            name: String,
        }

        let request = GraphQLRequest::new(
            "query Test($name: String!) { hello(name: $name) }",
            TestVars {
                name: "World".into(),
            },
        );

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""query":"query Test"#));
        assert!(json.contains(r#""variables":{"name":"World"}"#));
        assert!(!json.contains("operationName")); // Should be skipped when None
    }

    #[test]
    fn test_graphql_request_with_operation_name() {
        #[derive(Serialize)]
        struct EmptyVars {}

        let request =
            GraphQLRequest::with_operation("mutation DoThing { thing }", EmptyVars {}, "DoThing");

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""operationName":"DoThing""#));
    }

    #[test]
    fn test_graphql_response_success() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TestData {
            value: i32,
        }

        let json = r#"{"data": {"value": 42}}"#;
        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        assert!(!response.has_errors());
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().value, 42);
    }

    #[test]
    fn test_graphql_response_with_errors() {
        #[derive(Debug, Deserialize)]
        struct TestData {
            value: i32,
        }

        let json = r#"{
            "data": null,
            "errors": [
                {"message": "Something went wrong", "locations": [], "path": []}
            ]
        }"#;

        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        assert!(response.has_errors());
        assert_eq!(response.first_error_message(), Some("Something went wrong"));

        let result = response.into_result();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GraphQL(msg) => assert_eq!(msg, "Something went wrong"),
            _ => panic!("Expected GraphQL error"),
        }
    }

    #[test]
    fn test_graphql_response_empty() {
        #[derive(Debug, Deserialize)]
        struct TestData {}

        let json = r#"{"data": null}"#;
        let response: GraphQLResponse<TestData> = serde_json::from_str(json).unwrap();

        let result = response.into_result();
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::Unexpected(msg) => {
                assert!(msg.contains("neither data nor errors"));
            }
            _ => panic!("Expected Unexpected error"),
        }
    }

    #[test]
    fn test_verify_referral_data_deserialization() {
        let json = r#"{
            "verifyReferralString": {
                "status": "success",
                "ResponseCode": "11011",
                "affectedRows": [{
                    "uid": "abc-123",
                    "username": "testuser",
                    "slug": "12345",
                    "img": null
                }]
            }
        }"#;

        let data: VerifyReferralData = serde_json::from_str(json).unwrap();
        assert_eq!(data.verify_referral_string.status, "success");
        assert_eq!(data.verify_referral_string.response_code, "11011");
        assert!(data.verify_referral_string.affected_rows.is_some());
    }
}
```

---

## 3.4 — Update `src/api/mod.rs`

Ensure the module is properly exported:

```rust
//! API layer for communicating with the Peer GraphQL backend.

pub mod graphql;

// Re-export commonly used items
pub use graphql::{
    mutate, query, GraphQLRequest, GraphQLResponse,
    VERIFY_REFERRAL_MUTATION, REGISTER_MUTATION, VERIFY_ACCOUNT_MUTATION,
    VerifyReferralData, RegisterData, VerifyAccountData,
};
```

> **Note:** The `mutate` and `query` functions are only available when compiled with the `ssr` feature.

---

## 3.5 — Update `src/models/user.rs`

Ensure the response types support the field names from the GraphQL API:

```rust
//! User and registration-related types.

use serde::{Deserialize, Serialize};

// ... (keep existing RegistrationInput) ...

/// User info returned in referral verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralUser {
    pub uid: String,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
}

/// Response from the `verifyReferralString` mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReferralVerifyResponse {
    pub status: String,
    pub response_code: String,
    #[serde(default)]
    pub affected_rows: Option<Vec<ReferralUser>>,
}

impl ReferralVerifyResponse {
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}

/// Response from the `register` mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegisterResponse {
    pub status: String,
    pub response_code: String,
    pub userid: Option<String>,
}

impl RegisterResponse {
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}

/// Response from the `verifyAccount` mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyAccountResponse {
    pub status: String,
    pub response_code: String,
}

impl VerifyAccountResponse {
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}
```

---

## 3.6 — Integration test file

Create `peer-web/tests/graphql_integration.rs`:

```rust
//! Integration tests for the GraphQL client module.
//!
//! These tests require the mock backend to be running on port 4000.
//! Run with: `cargo test --features ssr`

#![cfg(feature = "ssr")]

use peer_web::api::graphql::{
    mutate, RegisterData, VerifyAccountData, VerifyReferralData,
    REGISTER_MUTATION, VERIFY_ACCOUNT_MUTATION, VERIFY_REFERRAL_MUTATION,
};
use peer_web::models::user::RegistrationInput;
use serde::Serialize;
use std::env;

/// Set up the test environment.
fn setup() {
    // Ensure we're pointing at the mock backend
    env::set_var("GRAPHQL_ENDPOINT", "http://localhost:4000/graphql");
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
    setup();

    let vars = VerifyReferralVars {
        referral_string: "85d5f836-b1f5-4c4e-9381-1b058e13df93".to_string(),
    };

    let result: Result<VerifyReferralData, _> =
        mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

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
    setup();

    let vars = VerifyReferralVars {
        referral_string: "00000000-0000-0000-0000-000000000000".to_string(),
    };

    let result: Result<VerifyReferralData, _> =
        mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

    // The mutation itself succeeds, but returns status: "error"
    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(!data.verify_referral_string.is_success());
    assert_eq!(data.verify_referral_string.response_code, "31010");
}

#[tokio::test]
async fn test_verify_referral_malformed_string() {
    setup();

    let vars = VerifyReferralVars {
        referral_string: "not-a-valid-uuid".to_string(),
    };

    let result: Result<VerifyReferralData, _> =
        mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(!data.verify_referral_string.is_success());
    // Should return invalid referral error
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
    setup();

    // Use a unique email to avoid "already registered" errors
    let unique_email = format!("test_{}@example.com", uuid::Uuid::new_v4());

    let vars = RegisterVars {
        input: RegistrationInput::new(unique_email, "SecurePass123!", "testuser_new"),
    };

    let result: Result<RegisterData, _> = mutate(REGISTER_MUTATION, vars, None).await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());

    let data = result.unwrap();
    assert!(data.register.is_success());
    assert_eq!(data.register.response_code, "10601");
    assert!(data.register.userid.is_some());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    setup();

    // First registration
    let email = format!("duplicate_{}@example.com", uuid::Uuid::new_v4());

    let vars1 = RegisterVars {
        input: RegistrationInput::new(email.clone(), "SecurePass123!", "user1"),
    };

    let _ = mutate::<_, RegisterData>(REGISTER_MUTATION, vars1, None).await;

    // Second registration with same email
    let vars2 = RegisterVars {
        input: RegistrationInput::new(email, "SecurePass456!", "user2"),
    };

    let result: Result<RegisterData, _> = mutate(REGISTER_MUTATION, vars2, None).await;

    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(!data.register.is_success());
    assert_eq!(data.register.response_code, "30601"); // Email already registered
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
    setup();

    // First, register a user to get a userid
    let unique_email = format!("verify_{}@example.com", uuid::Uuid::new_v4());

    let register_vars = RegisterVars {
        input: RegistrationInput::new(unique_email, "SecurePass123!", "verify_user"),
    };

    let register_result: RegisterData =
        mutate(REGISTER_MUTATION, register_vars, None).await.unwrap();

    let userid = register_result.register.userid.unwrap();

    // Now verify the account
    let vars = VerifyAccountVars { user_id: userid };

    let result: Result<VerifyAccountData, _> =
        mutate(VERIFY_ACCOUNT_MUTATION, vars, None).await;

    assert!(result.is_ok(), "Expected success, got: {:?}", result.err());

    let data = result.unwrap();
    assert!(data.verify_account.is_success());
    assert_eq!(data.verify_account.response_code, "10701");
}

#[tokio::test]
async fn test_verify_account_already_verified() {
    setup();

    // Register and verify once
    let unique_email = format!("already_verified_{}@example.com", uuid::Uuid::new_v4());

    let register_vars = RegisterVars {
        input: RegistrationInput::new(unique_email, "SecurePass123!", "already_verified"),
    };

    let register_result: RegisterData =
        mutate(REGISTER_MUTATION, register_vars, None).await.unwrap();

    let userid = register_result.register.userid.unwrap();

    // First verification
    let vars1 = VerifyAccountVars {
        user_id: userid.clone(),
    };
    let _ = mutate::<_, VerifyAccountData>(VERIFY_ACCOUNT_MUTATION, vars1, None).await;

    // Second verification (should return "already verified")
    let vars2 = VerifyAccountVars { user_id: userid };

    let result: Result<VerifyAccountData, _> =
        mutate(VERIFY_ACCOUNT_MUTATION, vars2, None).await;

    assert!(result.is_ok());

    let data = result.unwrap();
    // 30701 means already verified (still considered success in the mock)
    assert!(["10701", "30701"].contains(&data.verify_account.response_code.as_str()));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_network_error_wrong_endpoint() {
    // Point to a non-existent server
    env::set_var("GRAPHQL_ENDPOINT", "http://localhost:59999/graphql");

    let vars = VerifyReferralVars {
        referral_string: "test".to_string(),
    };

    let result: Result<VerifyReferralData, _> =
        mutate(VERIFY_REFERRAL_MUTATION, vars, None).await;

    assert!(result.is_err());

    match result.unwrap_err() {
        peer_web::models::common::ApiError::Network(msg) => {
            assert!(msg.contains("connect") || msg.contains("Connection"));
        }
        other => panic!("Expected Network error, got: {:?}", other),
    }
}
```

---

## 3.7 — Add test dependencies to `Cargo.toml`

Add these to the `[dev-dependencies]` section:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
uuid = { version = "1", features = ["v4"] }
serde_json = "1"
```

---

## 3.8 — Commands to verify

### Step 3.8.1 — Compile check

```bash
cd peer-web
cargo check --features ssr
```

**Expected:** No compilation errors.

### Step 3.8.2 — Run unit tests

```bash
cargo test --lib
```

**Expected:** All unit tests in `graphql.rs` pass:
- `test_graphql_request_serialization`
- `test_graphql_request_with_operation_name`
- `test_graphql_response_success`
- `test_graphql_response_with_errors`
- `test_graphql_response_empty`
- `test_verify_referral_data_deserialization`

### Step 3.8.3 — Run integration tests (requires mock backend)

First, ensure the mock backend is running:

```bash
# In a separate terminal
cd tests/mock_backend
npm start
# Should show: "Mock Peer GraphQL server running at http://localhost:4000/graphql"
```

Then run integration tests:

```bash
cargo test --features ssr --test graphql_integration
```

**Expected:** All integration tests pass:
- `test_verify_referral_success`
- `test_verify_referral_invalid_code`
- `test_verify_referral_malformed_string`
- `test_register_success`
- `test_register_duplicate_email`
- `test_verify_account_success`
- `test_verify_account_already_verified`
- `test_network_error_wrong_endpoint`

---

## 3.9 — Common Issues & Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| `unresolved import 'crate::models::user::ReferralVerifyResponse'` | Types not defined in Step 2 | Ensure `src/models/user.rs` contains all response types |
| `reqwest::Client::builder() not found` | Missing `ssr` feature | Run with `--features ssr` |
| `Connection refused` in tests | Mock backend not running | Start with `npm start` in `tests/mock_backend/` |
| Deserialization errors | Field name mismatch | Check `#[serde(rename_all = "...")]` matches API |
| `PascalCase` vs `camelCase` confusion | Peer API uses PascalCase for some fields | Use `#[serde(rename_all = "PascalCase")]` for response types |

---

## 3.10 — Files changed summary

| File | Action | Purpose |
|------|--------|---------|
| `src/api/graphql.rs` | Create | GraphQL client implementation |
| `src/api/mod.rs` | Update | Re-export graphql module items |
| `src/models/user.rs` | Update | Add missing response types |
| `tests/graphql_integration.rs` | Create | Integration tests |
| `Cargo.toml` | Update | Add dev-dependencies |

---

## 3.11 — What's next

With the GraphQL client module in place, **Step 4** will:
1. Create Leptos server functions (`#[server]`) that wrap these GraphQL calls
2. Make them callable from client-side WASM via generated HTTP endpoints
3. Add proper error transformation from `ApiError` to `ServerFnError`

The server functions will look like:

```rust
#[server(VerifyReferral, "/api")]
pub async fn verify_referral(referral_string: String) -> Result<ReferralVerifyResponse, ServerFnError> {
    let data: VerifyReferralData = mutate(
        VERIFY_REFERRAL_MUTATION,
        serde_json::json!({ "referralString": referral_string }),
        None,
    ).await?;
    Ok(data.verify_referral_string)
}
```

This keeps the GraphQL complexity isolated in the `api/` layer while exposing clean Rust functions to UI components.
