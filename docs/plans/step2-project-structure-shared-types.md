# Step 2 — Project Structure & Shared Types

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Create the module skeleton for the Leptos application and define the Rust types that mirror the GraphQL registration schema. These types will be used throughout the application for type-safe API communication.

---

## 2.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 1 complete | `cd peer-web && cargo leptos build` | Compiles without errors |
| Mock backend running | `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` | Returns `{"data":{"_health":true}}` |
| Leptos dev server works | `cargo leptos watch` | Serves at `http://localhost:3000` |

---

## 2.2 — Directory structure to create

Starting from the `peer-web/src/` directory, create the following module structure:

```
src/
├── main.rs                 ← (exists) Axum server entry point
├── lib.rs                  ← (exists) Crate root, add module declarations
├── app.rs                  ← (exists) Root <App/> with router
├── error_template.rs       ← (exists) Error UI
│
├── api/
│   ├── mod.rs              ← Module declaration
│   └── graphql.rs          ← Generic GraphQL query/mutation helper
│
├── models/
│   ├── mod.rs              ← Module declaration, re-exports
│   ├── common.rs           ← Shared types (DefaultResponse, ApiError)
│   └── user.rs             ← Registration types (RegistrationInput, responses)
│
├── components/
│   └── mod.rs              ← Component module declaration (placeholder)
│
├── pages/
│   ├── mod.rs              ← Page module declaration
│   └── register.rs         ← Stub RegisterPage component
│
├── state/
│   └── mod.rs              ← Global state module (placeholder)
│
└── fixtures/               ← Test fixtures directory
    ├── referral_success.json
    ├── referral_invalid.json
    ├── register_success.json
    ├── register_duplicate.json
    └── verify_success.json
```

### Commands to create directories

```bash
cd peer-web/src
mkdir -p api models components pages state fixtures
```

---

## 2.3 — Module declarations

### 2.3.1 — Update `src/lib.rs`

Add module declarations to the crate root:

```rust
pub mod api;
pub mod components;
pub mod models;
pub mod pages;
pub mod state;

// Re-export commonly used types
pub use models::common::ApiError;
pub use models::user::{RegistrationInput, RegisterResponse, ReferralVerifyResponse, VerifyAccountResponse};

// Feature-gated re-exports
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

### 2.3.2 — Create `src/api/mod.rs`

```rust
//! API layer for communicating with the Peer GraphQL backend.

pub mod graphql;

pub use graphql::*;
```

### 2.3.3 — Create `src/models/mod.rs`

```rust
//! Data models mirroring the Peer GraphQL schema.

pub mod common;
pub mod user;

pub use common::*;
pub use user::*;
```

### 2.3.4 — Create `src/components/mod.rs`

```rust
//! Reusable UI components.

// Components will be added in later steps:
// pub mod toast;
// pub mod form_field;
// pub mod password_strength;
```

### 2.3.5 — Create `src/pages/mod.rs`

```rust
//! Page components (one per route).

pub mod register;

pub use register::RegisterPage;
```

### 2.3.6 — Create `src/state/mod.rs`

```rust
//! Global application state and context providers.

// State will be expanded in later steps:
// pub mod auth;
// pub mod user;
```

---

## 2.4 — Common types (`src/models/common.rs`)

These types are shared across multiple API responses.

```rust
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
        "31010" => "Hmm… that referral code doesn't seem to work. Ask your friend to send you a new link, or use a Peer code.",

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
```

---

## 2.5 — User/Registration types (`src/models/user.rs`)

These types mirror the GraphQL registration schema.

```rust
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
```

---

## 2.6 — GraphQL client stub (`src/api/graphql.rs`)

Create a placeholder for the GraphQL client (full implementation in Step 3):

```rust
//! GraphQL client for communicating with the Peer backend.
//!
//! This module provides a type-safe interface for sending GraphQL
//! queries and mutations to the backend.

use crate::models::ApiError;
use serde::{de::DeserializeOwned, Serialize};

/// GraphQL request body.
#[derive(Debug, Serialize)]
pub struct GraphQLRequest<V: Serialize> {
    pub query: &'static str,
    pub variables: V,
}

/// GraphQL response envelope.
#[derive(Debug, serde::Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<crate::models::GraphQLError>>,
}

/// Placeholder for the GraphQL endpoint URL.
/// In production, this will be read from environment variables.
pub fn get_graphql_endpoint() -> String {
    std::env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string())
}

/// Send a GraphQL mutation to the backend.
///
/// This is a placeholder signature — full implementation in Step 3.
///
/// # Type Parameters
/// - `V`: The variables type (must be serializable)
/// - `T`: The expected response data type (must be deserializable)
///
/// # Arguments
/// - `query`: The GraphQL mutation string
/// - `variables`: The mutation variables
/// - `auth_token`: Optional bearer token for authenticated requests
///
/// # Returns
/// The deserialized response data or an error.
#[cfg(feature = "ssr")]
pub async fn mutate<V, T>(
    _query: &'static str,
    _variables: V,
    _auth_token: Option<&str>,
) -> Result<T, ApiError>
where
    V: Serialize + Send,
    T: DeserializeOwned,
{
    // Placeholder — implementation in Step 3
    Err(ApiError::Unexpected(
        "GraphQL client not yet implemented".to_string(),
    ))
}

/// Marker module for GraphQL query/mutation strings.
pub mod queries {
    /// Referral verification mutation.
    pub const VERIFY_REFERRAL: &str = r#"
        mutation VerifyReferralString($referralString: String!) {
            verifyReferralString(referralString: $referralString) {
                ResponseCode
                affectedRows {
                    uid
                    username
                    slug
                    img
                }
                status
            }
        }
    "#;

    /// User registration mutation.
    pub const REGISTER: &str = r#"
        mutation Register($input: RegistrationInput!) {
            register(input: $input) {
                status
                ResponseCode
                userid
            }
        }
    "#;

    /// Account verification mutation.
    pub const VERIFY_ACCOUNT: &str = r#"
        mutation VerifyAccount($userId: ID!) {
            verifyAccount(userid: $userId) {
                status
                ResponseCode
            }
        }
    "#;
}
```

---

## 2.7 — Stub RegisterPage (`src/pages/register.rs`)

Create a minimal page component to verify the module structure works:

```rust
//! Registration page component.
//!
//! This is a multi-step registration flow:
//! 1. Referral code entry and verification
//! 2. Registration form (email, username, password)
//! 3. Success confirmation

use leptos::prelude::*;

/// The registration page component.
///
/// This is a placeholder that will be expanded in Steps 6-10.
#[component]
pub fn RegisterPage() -> impl IntoView {
    view! {
        <div class="container">
            <div class="form-box">
                <h1>"Create Account"</h1>
                <p>"Registration form coming in Step 6..."</p>
            </div>
        </div>
    }
}
```

---

## 2.8 — Test fixtures

Create JSON fixtures in `src/fixtures/` for unit tests. These match the mock backend responses.

### 2.8.1 — `fixtures/referral_success.json`

```json
{
  "data": {
    "verifyReferralString": {
      "status": "success",
      "ResponseCode": "11011",
      "affectedRows": [
        {
          "uid": "85d5f836-b1f5-4c4e-9381-1b058e13df93",
          "username": "peer_referrer",
          "slug": "abc12",
          "img": "https://cdn.peer.inc/avatars/default.png"
        }
      ]
    }
  }
}
```

### 2.8.2 — `fixtures/referral_invalid.json`

```json
{
  "data": {
    "verifyReferralString": {
      "status": "error",
      "ResponseCode": "31010",
      "affectedRows": null
    }
  }
}
```

### 2.8.3 — `fixtures/register_success.json`

```json
{
  "data": {
    "register": {
      "status": "success",
      "ResponseCode": "10601",
      "userid": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
    }
  }
}
```

### 2.8.4 — `fixtures/register_duplicate.json`

```json
{
  "data": {
    "register": {
      "status": "error",
      "ResponseCode": "30601",
      "userid": null
    }
  }
}
```

### 2.8.5 — `fixtures/verify_success.json`

```json
{
  "data": {
    "verifyAccount": {
      "status": "success",
      "ResponseCode": "10701"
    }
  }
}
```

---

## 2.9 — Fixture deserialization tests

Add integration tests that verify fixtures deserialize correctly. Create `tests/fixtures_test.rs`:

```rust
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
```

---

## 2.10 — Update `app.rs` to include the register page

Modify `src/app.rs` to add the `/register` route (preview for Step 5):

```rust
// In the Router configuration, add:
use crate::pages::RegisterPage;

// Inside the Routes:
<Route path="/register" view=RegisterPage />
```

> **Note:** The full router setup happens in Step 5. For now, just ensure the import compiles.

---

## 2.11 — Verification checklist

Run these commands to verify Step 2 is complete:

### 2.11.1 — Compilation check

```bash
cd peer-web
cargo check
```

**Expected:** No errors. All modules resolve correctly.

### 2.11.2 — Unit tests

```bash
cargo test
```

**Expected:** All tests in `models/common.rs` and `models/user.rs` pass:

```
running 8 tests
test models::common::tests::test_api_status_deserialize ... ok
test models::common::tests::test_api_status_is_success ... ok
test models::common::tests::test_user_friendly_message_known_codes ... ok
test models::common::tests::test_user_friendly_message_unknown_code ... ok
test models::user::tests::test_referral_response_is_success ... ok
test models::user::tests::test_register_response_is_success ... ok
test models::user::tests::test_registration_input_builder ... ok
test models::user::tests::test_registration_input_serialize ... ok
```

### 2.11.3 — Fixture deserialization tests

```bash
cargo test --test fixtures_test
```

**Expected:** All 5 fixture tests pass.

### 2.11.4 — Full build

```bash
cargo leptos build
```

**Expected:** Both SSR and CSR targets compile successfully.

---

## 2.12 — Summary of files created

| File | Purpose |
|------|---------|
| `src/api/mod.rs` | API module declaration |
| `src/api/graphql.rs` | GraphQL client stub + query strings |
| `src/models/mod.rs` | Models module declaration |
| `src/models/common.rs` | Shared types: `ApiError`, `DefaultResponse`, response codes |
| `src/models/user.rs` | Registration types: `RegistrationInput`, response types |
| `src/components/mod.rs` | Components module placeholder |
| `src/pages/mod.rs` | Pages module declaration |
| `src/pages/register.rs` | Stub `RegisterPage` component |
| `src/state/mod.rs` | State module placeholder |
| `src/fixtures/*.json` | 5 test fixture files |
| `tests/fixtures_test.rs` | Fixture deserialization tests |

---

## 2.13 — Type mapping reference

| GraphQL Type | Rust Struct | Location |
|--------------|-------------|----------|
| `RegistrationInput` | `RegistrationInput` | `models/user.rs` |
| `ReferralResponse` | `ReferralVerifyResponse` | `models/user.rs` |
| `ReferralUser` | `ReferralUser` | `models/user.rs` |
| `RegisterResponse` | `RegisterResponse` | `models/user.rs` |
| `VerifyAccountResponse` | `VerifyAccountResponse` | `models/user.rs` |
| `DefaultResponse` | `DefaultResponse` | `models/common.rs` |

---

## 2.14 — Next steps

With the module structure and types in place, proceed to:

- **Step 3:** Implement the full GraphQL client in `api/graphql.rs` with `reqwest`
- **Step 4:** Create server functions that wrap the GraphQL mutations
- **Step 5:** Set up the router and render the `RegisterPage` at `/register`

---

## Appendix: serde field naming

The Peer API uses `PascalCase` for response fields (`ResponseCode`, `affectedRows`) but `camelCase` for input fields. The `#[serde(rename_all = "...")]` attributes handle this:

```rust
// For responses (PascalCase from API):
#[serde(rename_all = "PascalCase")]
pub struct RegisterResponse { ... }

// For inputs (camelCase to API):
#[serde(rename_all = "camelCase")]
pub struct RegistrationInput { ... }
```

Individual field overrides use `#[serde(rename = "fieldName")]` when the pattern differs.
