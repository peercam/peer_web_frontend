//! Parity tests for every mock-backend API call issued by the Playwright
//! suite in `end2end/`. Each end2end helper or `*.spec.ts` that
//! reaches the mock backend must have a direct counterpart here so that
//! changes to the mock API surface are caught without booting a browser.
//!
//! End2end ↔ tests/ mapping (this file + siblings):
//!
//!   * `helpers/mock-server.ts::resetMockState`              → `test_reset_endpoint_*`
//!   * `helpers/mock-server.ts::preRegisterEmail`            → `test_pre_register_email_helper_mutation`
//!     (the exact GraphQL body the helper POSTs)
//!   * `chat.spec.ts` `login()` helper (UI login)            → `test_login_mutation_verified_user_success`
//!   * `chat.spec.ts` T4 direct `POST /graphql { login }`    → `test_login_mutation_alice_success`
//!     (Alice peer login used to inject a message)
//!   * Login failure path exercised by the UI error toast    → `test_login_mutation_invalid_credentials`
//!
//! Other end2end mock-API calls (verifyReferralString, register, sendChatMessage,
//! listChats, listChatMessages, markChatRead) are covered in
//! `graphql_integration.rs`, `server_functions_integration.rs`, and
//! `chat_integration.rs`.
//!
//! Run with: `cargo test --features ssr --test end2end_mock_api_parity`

#![cfg(feature = "ssr")]

mod common;

use peer_web::api::graphql::{LOGIN_MUTATION, mutate};
use serde::Serialize;
use serde_json::Value;

// Seed credentials — `tests/mock_backend/src/seed.rs::credentials*`.
const VERIFIED_EMAIL: &str = "test@peer.com";
const VERIFIED_PASSWORD: &str = "TestPass123";
const ALICE_EMAIL: &str = "alice@peer.com";
const ALICE_PASSWORD: &str = "AlicePass123";

#[derive(Serialize)]
struct LoginVars {
    email: String,
    password: String,
}

/// Return the mock backend's base URL (i.e. the `/graphql` URL with the
/// path stripped) so tests can hit non-GraphQL routes such as `/reset`.
fn base_url(graphql_endpoint: &str) -> String {
    graphql_endpoint
        .strip_suffix("/graphql")
        .expect("mock endpoint should end with /graphql")
        .to_string()
}

// ── helpers/mock-server.ts::resetMockState ───────────────────────────────
//
// The Playwright suite calls `POST /reset` in every `beforeEach` to wipe
// mutable state for test isolation. These tests verify the endpoint
// responds successfully and actually restores the seeded baseline.

#[tokio::test]
async fn test_reset_endpoint_returns_ok() {
    let endpoint = common::mock_graphql_endpoint().await;
    // Use the write guard: `/reset` mutates shared state and must not race
    // with concurrent read-guarded tests (graphql/chat integration tests).
    let _guard = common::endpoint_write_guard().await;

    let url = format!("{}/reset", base_url(&endpoint));
    let res = reqwest::Client::new()
        .post(&url)
        .send()
        .await
        .expect("POST /reset should reach the mock backend");

    assert!(
        res.status().is_success(),
        "POST /reset should return 2xx, got {}",
        res.status()
    );

    let body: Value = res.json().await.expect("/reset should return JSON");
    assert_eq!(body["status"].as_str(), Some("ok"));
}

#[tokio::test]
async fn test_reset_endpoint_restores_seeded_login() {
    // After a reset, the canonical seeded user must still be able to log
    // in — this is the implicit contract every end2end `beforeEach` relies
    // on: "reset and then drive the UI against a freshly-seeded backend".
    let endpoint = common::mock_graphql_endpoint().await;
    let _guard = common::endpoint_write_guard().await;

    let url = format!("{}/reset", base_url(&endpoint));
    let res = reqwest::Client::new()
        .post(&url)
        .send()
        .await
        .expect("POST /reset should succeed");
    assert!(res.status().is_success());

    let data: Value = mutate(
        LOGIN_MUTATION,
        LoginVars {
            email: VERIFIED_EMAIL.into(),
            password: VERIFIED_PASSWORD.into(),
        },
        None,
    )
    .await
    .expect("login after /reset should succeed");

    let login = &data["login"];
    assert_eq!(
        login["status"].as_str(),
        Some("success"),
        "seeded verified user should log in after /reset; got {:?}",
        login
    );
    assert!(
        login["accessToken"].as_str().is_some_and(|t| !t.is_empty()),
        "post-reset login should return a non-empty access token"
    );
}

// ── helpers/mock-server.ts::preRegisterEmail ─────────────────────────────
//
// The Playwright helper POSTs the exact GraphQL body below to seed an
// already-taken email for the duplicate-email test (T4). We issue the
// same mutation string here so any drift in argument shape is caught.

#[tokio::test]
async fn test_pre_register_email_helper_mutation() {
    let _env = common::mock_graphql_endpoint().await;
    let _guard = common::endpoint_read_guard().await;

    // Use a unique email per run so we don't collide with a previous
    // invocation — the Playwright helper relies on `resetMockState()`
    // between tests for the same effect.
    let email = format!("prereg_{}@example.com", uuid::Uuid::new_v4());
    let query = format!(
        r#"
        mutation {{
          register(input: {{
            email: "{email}",
            password: "TestPass123!",
            username: "preregistered_user",
            referralUuid: "85d5f836-b1f5-4c4e-9381-1b058e13df93"
          }}) {{
            status
            ResponseCode
          }}
        }}
        "#
    );

    let endpoint = std::env::var("GRAPHQL_ENDPOINT").expect("endpoint set by setup");
    let res = reqwest::Client::new()
        .post(&endpoint)
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .expect("preRegisterEmail mutation should reach the mock backend");

    assert!(res.status().is_success());
    let body: Value = res.json().await.expect("response JSON");
    let register = &body["data"]["register"];
    assert_eq!(
        register["status"].as_str(),
        Some("success"),
        "preRegisterEmail helper should succeed; got {:?}",
        body
    );
    assert_eq!(register["ResponseCode"].as_str(), Some("10601"));
}

// ── chat.spec.ts `login()` helper ────────────────────────────────────────
//
// The helper drives the `/login` page in the UI, which ultimately calls
// the `login` GraphQL mutation with the verified user's credentials.

#[tokio::test]
async fn test_login_mutation_verified_user_success() {
    let _env = common::mock_graphql_endpoint().await;
    let _guard = common::endpoint_read_guard().await;

    let data: Value = mutate(
        LOGIN_MUTATION,
        LoginVars {
            email: VERIFIED_EMAIL.into(),
            password: VERIFIED_PASSWORD.into(),
        },
        None,
    )
    .await
    .expect("verified user login should succeed");

    let login = &data["login"];
    assert_eq!(login["status"].as_str(), Some("success"));
    assert_eq!(login["ResponseCode"].as_str(), Some("10801"));
    assert!(
        login["accessToken"].as_str().is_some_and(|t| !t.is_empty()),
        "login should return a non-empty access token"
    );
    assert!(
        login["refreshToken"]
            .as_str()
            .is_some_and(|t| !t.is_empty()),
        "login should return a non-empty refresh token"
    );
}

// ── chat.spec.ts T4 direct `POST /graphql { login }` (Alice) ─────────────

#[tokio::test]
async fn test_login_mutation_alice_success() {
    let _env = common::mock_graphql_endpoint().await;
    let _guard = common::endpoint_read_guard().await;

    let data: Value = mutate(
        LOGIN_MUTATION,
        LoginVars {
            email: ALICE_EMAIL.into(),
            password: ALICE_PASSWORD.into(),
        },
        None,
    )
    .await
    .expect("alice login should succeed");

    let login = &data["login"];
    assert_eq!(login["status"].as_str(), Some("success"));
    assert!(
        login["accessToken"].as_str().is_some_and(|t| !t.is_empty()),
        "alice login should return a non-empty access token"
    );
}

// ── Login failure path (error toast in the UI) ───────────────────────────

#[tokio::test]
async fn test_login_mutation_invalid_credentials() {
    let _env = common::mock_graphql_endpoint().await;
    let _guard = common::endpoint_read_guard().await;

    let data: Value = mutate(
        LOGIN_MUTATION,
        LoginVars {
            email: VERIFIED_EMAIL.into(),
            password: "WrongPassword999".into(),
        },
        None,
    )
    .await
    .expect("mutation envelope should deliver the error");

    let login = &data["login"];
    assert_ne!(
        login["status"].as_str(),
        Some("success"),
        "bad password must not yield a success login; got {:?}",
        login
    );
    assert!(
        login["accessToken"].as_str().unwrap_or("").is_empty(),
        "failed login must not leak an access token; got {:?}",
        login["accessToken"]
    );
}
