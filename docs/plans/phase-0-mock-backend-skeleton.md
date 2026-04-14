# Phase 0: Mock Backend Rust Skeleton & Parity

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Goal:** Replace Node.js mock with a Rust crate that passes identical tests

---

## Table of Contents

1. [Overview](#1-overview)
2. [Prerequisites](#2-prerequisites)
3. [Task Breakdown](#3-task-breakdown)
4. [Implementation Details](#4-implementation-details)
5. [Testing Strategy](#5-testing-strategy)
6. [Migration Checklist](#6-migration-checklist)
7. [Definition of Done](#7-definition-of-done)

---

## 1. Overview

### Current State (Node.js)

| File | Size | Purpose |
|------|------|---------|
| `server.js` | ~50 LOC | Express server, CORS, `/reset`, GraphQL handler |
| `schema.graphql` | ~50 LOC | SDL with 3 mutations, 1 query, 5 types |
| `resolvers.js` | ~110 LOC | Resolver implementations |
| `state.js` | ~15 LOC | Shared mutable state |
| `test.js` | ~175 LOC | 7 test scenarios |
| `fixtures/*.json` | 6 files | Reference fixtures (optional) |

### Target State (Rust)

```
tests/mock_backend/
├── Cargo.toml
├── src/
│   ├── main.rs              # Binary entrypoint
│   ├── lib.rs               # Library exports (app(), build_schema())
│   ├── state.rs             # SharedState type + MockState struct
│   ├── seed.rs              # Default seed data (known referrals, etc.)
│   └── schema/
│       ├── mod.rs           # Schema assembly
│       ├── query.rs         # QueryRoot (_health)
│       └── mutation/
│           ├── mod.rs       # MutationRoot assembly
│           └── registration.rs  # 3 registration mutations
│   └── types/
│       ├── mod.rs           # Type exports
│       └── registration.rs  # Response types for registration
└── tests/
    └── integration.rs       # 7 parity tests
```

---

## 2. Prerequisites

### Environment Requirements

- [ ] Rust toolchain ≥1.75 (check with `rustup show`)
- [ ] Cargo workspace awareness (optional — can be standalone crate)
- [ ] Port 4000 available for dev server testing

### Dependency Versions (Cargo.toml)

```toml
[package]
name = "mock_backend"
version = "0.1.0"
edition = "2024"

[dependencies]
async-graphql = "8"
async-graphql-axum = "8"
axum = "0.8"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"

[dev-dependencies]
reqwest = { version = "0.13", features = ["json"] }
tokio-test = "0.4"
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
```

---

## 3. Task Breakdown

### Phase 0.A — Project Setup (Est: 30 min)

| # | Task | Files | Notes |
|---|------|-------|-------|
| A1 | Create `tests/mock_backend/` directory structure | — | Use `cargo new` or manual |
| A2 | Write `Cargo.toml` with dependencies | `Cargo.toml` | Pin versions for reproducibility |
| A3 | Create empty module files with `mod` declarations | All `mod.rs` files | Establish module tree |
| A4 | Verify `cargo check` passes | — | Should compile with stubs |

### Phase 0.B — Types & State (Est: 1 hour)

| # | Task | Files | Notes |
|---|------|-------|-------|
| B1 | Define `DefaultResponse` struct | `types/registration.rs` | Match exact field names/casing |
| B2 | Define `ReferralUser` struct | `types/registration.rs` | Fields: `uid`, `username`, `slug`, `img` |
| B3 | Define `ReferralResponse` struct | `types/registration.rs` | Note: `ResponseCode` is PascalCase |
| B4 | Define `RegistrationInput` input type | `types/registration.rs` | GraphQL InputObject |
| B5 | Define `RegisterResponse` struct | `types/registration.rs` | `userid` is nullable |
| B6 | Define `VerifyAccountResponse` struct | `types/registration.rs` | Simpler response without userid |
| B7 | Define `MockState` struct | `state.rs` | Collections for referrals, emails, verified users |
| B8 | Define `SharedState` type alias | `state.rs` | `Arc<RwLock<MockState>>` |
| B9 | Implement `Default` for `MockState` with seed data | `seed.rs` | Two known referral UUIDs |

### Phase 0.C — GraphQL Schema (Est: 1.5 hours)

| # | Task | Files | Notes |
|---|------|-------|-------|
| C1 | Implement `QueryRoot` with `_health` | `schema/query.rs` | Returns `bool` |
| C2 | Create `RegistrationMutation` struct | `schema/mutation/registration.rs` | Will hold 3 resolvers |
| C3 | Implement `verify_referral_string` resolver | `schema/mutation/registration.rs` | UUID validation + set lookup |
| C4 | Implement `register` resolver | `schema/mutation/registration.rs` | fail@ check, duplicate check |
| C5 | Implement `verify_account` resolver | `schema/mutation/registration.rs` | Already-verified check |
| C6 | Assemble `MutationRoot` with `MergedObject` | `schema/mod.rs` | Single merged mutation type |
| C7 | Implement `build_schema()` function | `schema/mod.rs` | Returns full Schema with state |

### Phase 0.D — HTTP Layer (Est: 1 hour)

| # | Task | Files | Notes |
|---|------|-------|-------|
| D1 | Create `AppState` struct | `lib.rs` | Holds schema + shared mock state |
| D2 | Implement `graphql_handler` | `lib.rs` | Delegates to async-graphql-axum |
| D3 | Implement `reset_handler` | `lib.rs` | Clears & resets MockState |
| D4 | Build `app()` router function | `lib.rs` | Routes + CORS layer |
| D5 | Implement `main.rs` entrypoint | `main.rs` | TcpListener + axum::serve |

### Phase 0.E — Integration Tests (Est: 1.5 hours)

| # | Task | Files | Notes |
|---|------|-------|-------|
| E1 | Set up test harness with `tower::ServiceExt` | `tests/integration.rs` | In-process, no port binding |
| E2 | Test 1: Valid referral UUID | `tests/integration.rs` | Assert response shape |
| E3 | Test 2: Invalid referral string | `tests/integration.rs` | Assert error response |
| E4 | Test 3: Register success | `tests/integration.rs` | Assert userid is UUID |
| E5 | Test 4: Register duplicate email | `tests/integration.rs` | 30601 code |
| E6 | Test 5: Verify account success | `tests/integration.rs` | 10701 code |
| E7 | Test 6: Already verified | `tests/integration.rs` | 30701 code |
| E8 | Test 7: fail@ internal error | `tests/integration.rs` | 40601 code |
| E9 | Add reset endpoint test | `tests/integration.rs` | Verify state cleared |

### Phase 0.F — Cleanup & Migration (Est: 30 min)

| # | Task | Files | Notes |
|---|------|-------|-------|
| F1 | Run full test suite | — | `cargo test --all-targets` |
| F2 | Run clippy | — | `cargo clippy -- -D warnings` |
| F3 | Run fmt | — | `cargo fmt --check` |
| F4 | Test HTTP server manually | — | `cargo run`, curl test |
| F5 | Remove Node.js files | See list below | Keep fixtures as reference |
| F6 | Update README.md | `README.md` | Document new Rust usage |

---

## 4. Implementation Details

### 4.1 Type Definitions (`types/registration.rs`)

```rust
use async_graphql::{InputObject, SimpleObject, ID};
use serde::{Deserialize, Serialize};

/// Standard response envelope used by all mutations
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "PascalCase")]
pub struct DefaultResponse {
    pub status: String,
    #[graphql(name = "RequestId")]
    pub request_id: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    #[graphql(name = "ResponseMessage")]
    pub response_message: String,
}

impl DefaultResponse {
    pub fn success(code: &str, message: &str) -> Self {
        Self {
            status: "success".to_string(),
            request_id: format!("mock-req-{}", chrono::Utc::now().timestamp_millis()),
            response_code: code.to_string(),
            response_message: message.to_string(),
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            status: "error".to_string(),
            request_id: format!("mock-req-{}", chrono::Utc::now().timestamp_millis()),
            response_code: code.to_string(),
            response_message: message.to_string(),
        }
    }
}

/// User info returned in referral verification
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ReferralUser {
    pub uid: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
}

/// Response for verifyReferralString mutation
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "PascalCase")]
pub struct ReferralResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<ReferralUser>>,
    pub meta: DefaultResponse,
}

/// Input for register mutation
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct RegistrationInput {
    pub email: String,
    pub password: String,
    pub username: String,
    pub pkey: Option<String>,
    #[graphql(name = "referralUuid")]
    pub referral_uuid: Option<ID>,
}

/// Response for register mutation
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "PascalCase")]
pub struct RegisterResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    pub userid: Option<String>,
    pub meta: DefaultResponse,
}

/// Response for verifyAccount mutation
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "PascalCase")]
pub struct VerifyAccountResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    pub meta: DefaultResponse,
}
```

### 4.2 State Management (`state.rs`)

```rust
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Shared state wrapped in Arc<RwLock<>>
pub type SharedState = Arc<RwLock<MockState>>;

/// In-memory mock backend state
#[derive(Debug, Clone)]
pub struct MockState {
    /// Known valid referral UUIDs
    pub known_referrals: HashSet<Uuid>,
    /// Emails that have been used for registration
    pub registered_emails: HashSet<String>,
    /// User IDs that have completed verification
    pub verified_users: HashSet<Uuid>,
}

impl MockState {
    /// Reset state to defaults (for test isolation)
    pub fn reset(&mut self) {
        self.registered_emails.clear();
        self.verified_users.clear();
        // Note: known_referrals are NOT cleared — they're seed data
    }
}
```

### 4.3 Seed Data (`seed.rs`)

```rust
use std::collections::HashSet;
use uuid::{uuid, Uuid};

use crate::state::MockState;

/// Primary test referral — matches existing Node.js mock
pub const REFERRAL_PRIMARY: Uuid = uuid!("85d5f836-b1f5-4c4e-9381-1b058e13df93");
/// Secondary test referral
pub const REFERRAL_SECONDARY: Uuid = uuid!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");

impl Default for MockState {
    fn default() -> Self {
        Self {
            known_referrals: HashSet::from([REFERRAL_PRIMARY, REFERRAL_SECONDARY]),
            registered_emails: HashSet::new(),
            verified_users: HashSet::new(),
        }
    }
}

/// Mock referral user data returned on successful verification
pub mod mock_users {
    use crate::types::registration::ReferralUser;
    use async_graphql::ID;

    pub fn referral_user() -> ReferralUser {
        ReferralUser {
            uid: ID::from("usr_mock_001"),
            username: "peerTester".to_string(),
            slug: "peertester".to_string(),
            img: Some("https://via.placeholder.com/96".to_string()),
        }
    }
}
```

### 4.4 Registration Resolvers (`schema/mutation/registration.rs`)

```rust
use async_graphql::{Context, Object, ID};
use uuid::Uuid;

use crate::seed::mock_users;
use crate::state::SharedState;
use crate::types::registration::{
    DefaultResponse, ReferralResponse, RegisterResponse, RegistrationInput, VerifyAccountResponse,
};

/// UUID regex for validation
const UUID_REGEX: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

pub struct RegistrationMutation;

#[Object]
impl RegistrationMutation {
    /// Verify a referral code is valid
    ///
    /// Response codes:
    /// - 11011: Valid referral
    /// - 31010: Invalid referral string
    async fn verify_referral_string(
        &self,
        ctx: &Context<'_>,
        referral_string: String,
    ) -> ReferralResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        // Validate UUID format
        let regex = regex::Regex::new(UUID_REGEX).unwrap();
        if !regex.is_match(&referral_string) {
            return ReferralResponse {
                status: "error".to_string(),
                response_code: "31010".to_string(),
                affected_rows: None,
                meta: DefaultResponse::error("31010", "Invalid referral string"),
            };
        }

        // Parse and check against known referrals
        let uuid = match Uuid::parse_str(&referral_string) {
            Ok(u) => u,
            Err(_) => {
                return ReferralResponse {
                    status: "error".to_string(),
                    response_code: "31010".to_string(),
                    affected_rows: None,
                    meta: DefaultResponse::error("31010", "Invalid referral string"),
                }
            }
        };

        if state_read.known_referrals.contains(&uuid) {
            ReferralResponse {
                status: "success".to_string(),
                response_code: "11011".to_string(),
                affected_rows: Some(vec![mock_users::referral_user()]),
                meta: DefaultResponse::success("11011", "Referral info fetched successfully"),
            }
        } else {
            ReferralResponse {
                status: "error".to_string(),
                response_code: "31010".to_string(),
                affected_rows: None,
                meta: DefaultResponse::error("31010", "Invalid referral string"),
            }
        }
    }

    /// Register a new user account
    ///
    /// Response codes:
    /// - 10601: Registration successful
    /// - 30601: Email already registered
    /// - 40601: Internal server error (simulated with fail@ prefix)
    async fn register(&self, ctx: &Context<'_>, input: RegistrationInput) -> RegisterResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Simulate internal error for fail@ emails (testing hook)
        if input.email.starts_with("fail@") {
            return RegisterResponse {
                status: "error".to_string(),
                response_code: "40601".to_string(),
                userid: None,
                meta: DefaultResponse::error("40601", "Internal server error"),
            };
        }

        // Check for duplicate email
        if state_write.registered_emails.contains(&input.email) {
            return RegisterResponse {
                status: "error".to_string(),
                response_code: "30601".to_string(),
                userid: None,
                meta: DefaultResponse::error("30601", "Email is already registered"),
            };
        }

        // Success: generate user ID and record email
        let userid = Uuid::new_v4();
        state_write.registered_emails.insert(input.email.clone());

        RegisterResponse {
            status: "success".to_string(),
            response_code: "10601".to_string(),
            userid: Some(userid.to_string()),
            meta: DefaultResponse::success("10601", "User registered successfully"),
        }
    }

    /// Verify a newly registered account
    ///
    /// Response codes:
    /// - 10701: Account verified successfully
    /// - 30701: Account already verified
    async fn verify_account(&self, ctx: &Context<'_>, userid: ID) -> VerifyAccountResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Parse the user ID
        let uuid = match Uuid::parse_str(&userid) {
            Ok(u) => u,
            Err(_) => {
                // Invalid UUID still gets "verified" in the mock for compatibility
                return VerifyAccountResponse {
                    status: "success".to_string(),
                    response_code: "10701".to_string(),
                    meta: DefaultResponse::success("10701", "Account verified successfully"),
                };
            }
        };

        // Check if already verified
        if state_write.verified_users.contains(&uuid) {
            return VerifyAccountResponse {
                status: "success".to_string(),
                response_code: "30701".to_string(),
                meta: DefaultResponse::success("30701", "Account is already verified"),
            };
        }

        // Mark as verified
        state_write.verified_users.insert(uuid);

        VerifyAccountResponse {
            status: "success".to_string(),
            response_code: "10701".to_string(),
            meta: DefaultResponse::success("10701", "Account verified successfully"),
        }
    }
}
```

### 4.5 Schema Assembly (`schema/mod.rs`)

```rust
pub mod mutation;
pub mod query;

use async_graphql::{EmptySubscription, MergedObject, Schema};

use crate::state::SharedState;
use mutation::registration::RegistrationMutation;
use query::QueryRoot;

/// Combined mutation root (will grow as more phases are added)
#[derive(MergedObject, Default)]
pub struct MutationRoot(pub RegistrationMutation);

/// The full GraphQL schema
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Build the GraphQL schema with injected state
pub fn build_schema(state: SharedState) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
        .data(state)
        .finish()
}
```

### 4.6 Query Root (`schema/query.rs`)

```rust
use async_graphql::Object;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Health check endpoint (required by GraphQL spec to have at least one query)
    async fn _health(&self) -> bool {
        true
    }
}
```

### 4.7 HTTP Router (`lib.rs`)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

use async_graphql_axum::GraphQL;
use axum::{
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

pub mod schema;
pub mod seed;
pub mod state;
pub mod types;

use schema::{build_schema, AppSchema};
use state::{MockState, SharedState};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub schema: AppSchema,
    pub mock_state: SharedState,
}

/// Reset handler — clears mutable state for test isolation
async fn reset_handler(state: axum::extract::State<AppState>) -> Json<serde_json::Value> {
    let mut mock_state = state.mock_state.write().await;
    mock_state.reset();
    Json(serde_json::json!({ "status": "ok" }))
}

/// Build the application router
pub fn app() -> Router {
    let mock_state: SharedState = Arc::new(RwLock::new(MockState::default()));
    let schema = build_schema(mock_state.clone());

    let app_state = AppState {
        schema: schema.clone(),
        mock_state,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/graphql", get(GraphQL::new(schema.clone())).post(GraphQL::new(schema)))
        .route("/reset", post(reset_handler))
        .layer(cors)
        .with_state(app_state)
}

/// Build app with custom initial state (for testing)
pub fn app_with_state(mock_state: SharedState) -> Router {
    let schema = build_schema(mock_state.clone());

    let app_state = AppState {
        schema: schema.clone(),
        mock_state,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/graphql", get(GraphQL::new(schema.clone())).post(GraphQL::new(schema)))
        .route("/reset", post(reset_handler))
        .layer(cors)
        .with_state(app_state)
}
```

### 4.8 Main Entrypoint (`main.rs`)

```rust
use mock_backend::app;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    println!("🦀 Mock Peer backend running at http://localhost:{port}/graphql");
    println!("   POST /graphql  — GraphQL endpoint");
    println!("   POST /reset    — Reset state for testing");

    axum::serve(listener, app())
        .await
        .expect("Server error");
}
```

---

## 5. Testing Strategy

### 5.1 Test Harness Setup

```rust
// tests/integration.rs

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mock_backend::app;
use serde_json::{json, Value};
use tower::ServiceExt;

/// Send a GraphQL query/mutation and return parsed JSON response
async fn graphql(query: &str) -> Value {
    let app = app();

    let body = json!({ "query": query });

    let request = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body_bytes).unwrap()
}

/// Reset mock state via /reset endpoint
async fn reset_state(app: axum::Router) {
    let request = Request::builder()
        .method("POST")
        .uri("/reset")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

### 5.2 Test Cases

| # | Name | GraphQL | Assertions |
|---|------|---------|------------|
| 1 | `test_valid_referral` | `verifyReferralString(referralString: "85d5f836...")` | `status: "success"`, `ResponseCode: "11011"`, `affectedRows[0].uid == "usr_mock_001"` |
| 2 | `test_invalid_referral` | `verifyReferralString(referralString: "not-a-uuid")` | `status: "error"`, `ResponseCode: "31010"`, `affectedRows: null` |
| 3 | `test_register_success` | `register(input: {...})` | `status: "success"`, `ResponseCode: "10601"`, `userid` matches UUID regex |
| 4 | `test_register_duplicate_email` | Second register with same email | `status: "error"`, `ResponseCode: "30601"`, `userid: null` |
| 5 | `test_verify_account_success` | `verifyAccount(userid: "<uuid>")` | `status: "success"`, `ResponseCode: "10701"` |
| 6 | `test_already_verified` | Second `verifyAccount` with same userid | `status: "success"`, `ResponseCode: "30701"` |
| 7 | `test_fail_email_error` | `register(input: {email: "fail@test.com", ...})` | `status: "error"`, `ResponseCode: "40601"`, `userid: null` |
| 8 | `test_reset_endpoint` | POST /reset, then re-register | Second register succeeds (email cleared) |
| 9 | `test_health_query` | `{ _health }` | Returns `true` |

### 5.3 Full Test Implementation

```rust
#[tokio::test]
async fn test_valid_referral() {
    let res = graphql(r#"
        mutation {
            verifyReferralString(referralString: "85d5f836-b1f5-4c4e-9381-1b058e13df93") {
                status
                ResponseCode
                affectedRows { uid username slug img }
            }
        }
    "#).await;

    let data = &res["data"]["verifyReferralString"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "11011");
    assert!(data["affectedRows"].is_array());
    assert_eq!(data["affectedRows"][0]["uid"], "usr_mock_001");
    assert_eq!(data["affectedRows"][0]["username"], "peerTester");
}

#[tokio::test]
async fn test_invalid_referral() {
    let res = graphql(r#"
        mutation {
            verifyReferralString(referralString: "not-a-uuid") {
                status
                ResponseCode
                affectedRows { uid }
            }
        }
    "#).await;

    let data = &res["data"]["verifyReferralString"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "31010");
    assert!(data["affectedRows"].is_null());
}

#[tokio::test]
async fn test_register_success() {
    let res = graphql(r#"
        mutation {
            register(input: {
                email: "new@test.com"
                password: "Abcd1234"
                username: "newuser"
                pkey: null
                referralUuid: "85d5f836-b1f5-4c4e-9381-1b058e13df93"
            }) {
                status
                ResponseCode
                userid
            }
        }
    "#).await;

    let data = &res["data"]["register"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10601");

    // Verify userid is a valid UUID
    let userid = data["userid"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(userid).is_ok(), "userid should be valid UUID");
}

#[tokio::test]
async fn test_register_duplicate_email() {
    // First registration (uses fresh app instance per test)
    // Note: For this test we need a stateful app, so we use a custom approach
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use mock_backend::{app_with_state, state::MockState};

    let state = Arc::new(RwLock::new(MockState::default()));
    let app = app_with_state(state.clone());

    // Helper to make requests against the same app state
    let graphql_stateful = |app: axum::Router, query: &str| async move {
        let body = json!({ "query": query });
        let request = Request::builder()
            .method("POST")
            .uri("/graphql")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice::<Value>(&body_bytes).unwrap()
    };

    // First registration
    let app1 = app_with_state(state.clone());
    let _ = graphql_stateful(app1, r#"
        mutation {
            register(input: {
                email: "dupe@test.com"
                password: "Abcd1234"
                username: "user1"
            }) { status }
        }
    "#).await;

    // Second registration with same email
    let app2 = app_with_state(state.clone());
    let res = graphql_stateful(app2, r#"
        mutation {
            register(input: {
                email: "dupe@test.com"
                password: "Abcd1234"
                username: "user2"
            }) {
                status
                ResponseCode
                userid
            }
        }
    "#).await;

    let data = &res["data"]["register"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "30601");
    assert!(data["userid"].is_null());
}

// ... (additional tests follow same pattern)
```

---

## 6. Migration Checklist

### Files to Delete (after tests pass)

- [ ] `tests/mock_backend/server.js`
- [ ] `tests/mock_backend/resolvers.js`
- [ ] `tests/mock_backend/schema.graphql`
- [ ] `tests/mock_backend/state.js`
- [ ] `tests/mock_backend/test.js`
- [ ] `tests/mock_backend/package.json`
- [ ] `tests/mock_backend/package-lock.json`
- [ ] `tests/mock_backend/node_modules/` (entire directory)

### Files to Keep

- [ ] `tests/mock_backend/fixtures/*.json` — Useful as reference data
- [ ] `tests/mock_backend/README.md` — Update with Rust instructions

### README.md Updates

```markdown
# Mock Backend (Rust)

A lightweight mock backend for Peer Web frontend development and testing.

## Run

```bash
cd tests/mock_backend
cargo run
```

Server starts on `http://localhost:4000/graphql`

## Test

```bash
cargo test
```

## Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/graphql` | POST | GraphQL queries/mutations |
| `/reset` | POST | Reset state for test isolation |

## Supported Operations

### Mutations
- `verifyReferralString(referralString: String!): ReferralResponse!`
- `register(input: RegistrationInput!): RegisterResponse!`
- `verifyAccount(userid: ID!): VerifyAccountResponse!`

### Queries
- `_health: Boolean`
```

---

## 7. Definition of Done

### Build & Test Gates

- [ ] `cargo build` succeeds without warnings
- [ ] `cargo test --all-targets` passes all 9 tests
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes

### Functional Requirements

- [ ] `POST /graphql` with `verifyReferralString` returns correct response shapes
- [ ] `POST /graphql` with `register` returns correct response shapes
- [ ] `POST /graphql` with `verifyAccount` returns correct response shapes
- [ ] Response codes match exactly: `11011`, `31010`, `10601`, `30601`, `40601`, `10701`, `30701`
- [ ] Field names match casing: `ResponseCode` (PascalCase), `affectedRows` (camelCase), etc.
- [ ] `POST /reset` clears `registeredEmails` and `verifiedUsers` but preserves `knownReferrals`

### HTTP Requirements

- [ ] Server binds to `:4000` (or `$PORT`)
- [ ] CORS allows any origin (for Leptos dev server)
- [ ] GraphQL endpoint accepts both GET and POST

### Migration Requirements

- [ ] All Node.js files deleted (except fixtures/)
- [ ] README.md updated with Rust instructions
- [ ] No runtime dependency on Node.js/npm

### Integration Ready

- [ ] `mock_backend::app()` is exported as public API
- [ ] Can be imported as dev-dependency: `mock_backend = { path = "../../tests/mock_backend" }`
- [ ] `build_schema()` is exported for potential SDL snapshot tests

---

## Appendix: Response Code Reference

| Code | Status | Context | Message |
|------|--------|---------|---------|
| `11011` | success | `verifyReferralString` | Referral info fetched successfully |
| `31010` | error | `verifyReferralString` | Invalid referral string |
| `10601` | success | `register` | User registered successfully |
| `30601` | error | `register` | Email is already registered |
| `40601` | error | `register` | Internal server error |
| `10701` | success | `verifyAccount` | Account verified successfully |
| `30701` | success | `verifyAccount` | Account is already verified |
