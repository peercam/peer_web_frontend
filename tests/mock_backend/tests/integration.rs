use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mock_backend::seed::{
    SEED_CHAT_GROUP, SEED_CHAT_PRIVATE, SEED_COMMENT_1, SEED_COMMENT_2, SEED_COMMENT_5,
    SEED_POST_1, SEED_POST_2, SEED_POST_3, SEED_POST_4, SEED_SHOP_TX_1, SEED_USER_ALICE,
    SEED_USER_BOB, SEED_USER_CAROL, SEED_USER_DAVE, SEED_USER_VERIFIED,
};
use mock_backend::{app, app_with_state, state::MockState};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

// ============================================================================
// Test helpers
// ============================================================================

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

/// Send a GraphQL query/mutation against a stateful app and return parsed JSON response
async fn graphql_stateful(state: &Arc<RwLock<MockState>>, query: &str) -> Value {
    let app = app_with_state(state.clone());

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

/// Send a GraphQL mutation with an Authorization: Bearer header
async fn graphql_with_auth(state: &Arc<RwLock<MockState>>, query: &str, token: &str) -> Value {
    let app = app_with_state(state.clone());

    let body = json!({ "query": query });

    let request = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body_bytes).unwrap()
}

fn default_shared_state() -> Arc<RwLock<MockState>> {
    Arc::new(RwLock::new(MockState::default()))
}

/// Extract a GraphQL Decimal value (serialized as JSON string) as f64.
fn decimal_val(v: &Value) -> f64 {
    v.as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| v.as_f64())
        .expect("Expected a numeric value (string or number)")
}

#[tokio::test]
async fn test_valid_referral() {
    let res = graphql(
        r#"
        mutation {
            verifyReferralString(referralString: "85d5f836-b1f5-4c4e-9381-1b058e13df93") {
                status
                ResponseCode
                affectedRows { uid username slug img }
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["verifyReferralString"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "11011");
    assert!(data["affectedRows"].is_array());
    assert_eq!(data["affectedRows"][0]["uid"], "usr_mock_001");
    assert_eq!(data["affectedRows"][0]["username"], "peerTester");
}

#[tokio::test]
async fn test_invalid_referral() {
    let res = graphql(
        r#"
        mutation {
            verifyReferralString(referralString: "not-a-uuid") {
                status
                ResponseCode
                affectedRows { uid }
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["verifyReferralString"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "31010");
    assert!(data["affectedRows"].is_null());
}

#[tokio::test]
async fn test_register_success() {
    let res = graphql(
        r#"
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
    "#,
    )
    .await;

    let data = &res["data"]["register"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10601");

    // Verify userid is a valid UUID
    let userid = data["userid"].as_str().unwrap();
    assert!(
        uuid::Uuid::parse_str(userid).is_ok(),
        "userid should be valid UUID"
    );
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let state = Arc::new(RwLock::new(MockState::default()));

    // First registration
    let _ = graphql_stateful(
        &state,
        r#"
        mutation {
            register(input: {
                email: "dupe@test.com"
                password: "Abcd1234"
                username: "user1"
            }) { status }
        }
    "#,
    )
    .await;

    // Second registration with same email
    let res = graphql_stateful(
        &state,
        r#"
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
    "#,
    )
    .await;

    let data = &res["data"]["register"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "30601");
    assert!(data["userid"].is_null());
}

#[tokio::test]
async fn test_verify_account_success() {
    let state = Arc::new(RwLock::new(MockState::default()));

    // Register first to get a userid
    let reg_res = graphql_stateful(
        &state,
        r#"
        mutation {
            register(input: {
                email: "verify@test.com"
                password: "Abcd1234"
                username: "verifyuser"
            }) {
                status
                userid
            }
        }
    "#,
    )
    .await;

    let userid = reg_res["data"]["register"]["userid"].as_str().unwrap();

    // Verify the account
    let query = format!(
        r#"
        mutation {{
            verifyAccount(userid: "{}") {{
                status
                ResponseCode
            }}
        }}
    "#,
        userid
    );

    let res = graphql_stateful(&state, &query).await;

    let data = &res["data"]["verifyAccount"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10701");
}

#[tokio::test]
async fn test_already_verified() {
    let state = Arc::new(RwLock::new(MockState::default()));

    // Register first
    let reg_res = graphql_stateful(
        &state,
        r#"
        mutation {
            register(input: {
                email: "alreadyverified@test.com"
                password: "Abcd1234"
                username: "alreadyverified"
            }) {
                status
                userid
            }
        }
    "#,
    )
    .await;

    let userid = reg_res["data"]["register"]["userid"].as_str().unwrap();

    let query = format!(
        r#"
        mutation {{
            verifyAccount(userid: "{}") {{
                status
                ResponseCode
            }}
        }}
    "#,
        userid
    );

    // First verification
    let _ = graphql_stateful(&state, &query).await;

    // Second verification — should be "already verified"
    let res = graphql_stateful(&state, &query).await;

    let data = &res["data"]["verifyAccount"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "30701");
}

#[tokio::test]
async fn test_fail_email_error() {
    let res = graphql(
        r#"
        mutation {
            register(input: {
                email: "fail@test.com"
                password: "Abcd1234"
                username: "failuser"
            }) {
                status
                ResponseCode
                userid
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["register"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "40601");
    assert!(data["userid"].is_null());
}

#[tokio::test]
async fn test_reset_endpoint() {
    let state = Arc::new(RwLock::new(MockState::default()));

    // Register an email
    let _ = graphql_stateful(
        &state,
        r#"
        mutation {
            register(input: {
                email: "reset@test.com"
                password: "Abcd1234"
                username: "resetuser"
            }) { status }
        }
    "#,
    )
    .await;

    // Reset state
    let app = app_with_state(state.clone());
    let request = Request::builder()
        .method("POST")
        .uri("/reset")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Re-register with the same email — should succeed now
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            register(input: {
                email: "reset@test.com"
                password: "Abcd1234"
                username: "resetuser2"
            }) {
                status
                ResponseCode
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["register"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10601");
}

#[tokio::test]
async fn test_health_query() {
    let res = graphql("{ _health }").await;
    assert_eq!(res["data"]["_health"], true);
}

// ============================================================================
// Phase 1: Auth & Session Tests
// ============================================================================

#[tokio::test]
async fn test_login_seeded_user() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                status
                ResponseCode
                accessToken
                refreshToken
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["login"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10801");
    assert!(data["accessToken"].is_string());
    assert!(data["refreshToken"].is_string());

    let access = data["accessToken"].as_str().unwrap();
    assert!(
        access.starts_with("mock-access-"),
        "Token should have mock prefix"
    );
}

#[tokio::test]
async fn test_register_verify_login_flow() {
    let state = default_shared_state();

    // Register
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            register(input: {
                email: "flow@test.com"
                password: "FlowPass123"
                username: "flowuser"
            }) { status ResponseCode userid }
        }
    "#,
    )
    .await;
    let userid = res["data"]["register"]["userid"].as_str().unwrap();
    assert_eq!(res["data"]["register"]["ResponseCode"], "10601");

    // Verify
    let query = format!(
        r#"
        mutation {{
            verifyAccount(userid: "{}") {{ status ResponseCode }}
        }}
    "#,
        userid
    );
    let res = graphql_stateful(&state, &query).await;
    assert_eq!(res["data"]["verifyAccount"]["ResponseCode"], "10701");

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "flow@test.com", password: "FlowPass123") {
                status ResponseCode accessToken refreshToken
            }
        }
    "#,
    )
    .await;
    let data = &res["data"]["login"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10801");
    assert!(data["accessToken"].is_string());
    assert!(data["refreshToken"].is_string());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "WrongPassword") {
                status ResponseCode accessToken refreshToken
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["login"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "30801");
    assert!(data["accessToken"].is_null());
    assert!(data["refreshToken"].is_null());
}

#[tokio::test]
async fn test_login_nonexistent_email() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "nobody@nowhere.com", password: "Whatever123") {
                status ResponseCode accessToken refreshToken
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["login"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "30801");
    assert!(data["accessToken"].is_null());
}

#[tokio::test]
async fn test_login_unverified_account() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "unverified@peer.com", password: "TestPass456") {
                status ResponseCode accessToken
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["login"];
    assert_eq!(data["ResponseCode"], "60801");
    assert!(data["accessToken"].is_null());
}

#[tokio::test]
async fn test_login_deleted_account() {
    let state = default_shared_state();

    // Login first
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken refreshToken
            }
        }
    "#,
    )
    .await;
    let access = res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete account
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            deleteAccount(password: "TestPass123") {
                status ResponseCode
            }
        }
    "#,
        &access,
    )
    .await;
    assert_eq!(res["data"]["deleteAccount"]["ResponseCode"], "11012");

    // Try to login again
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                ResponseCode
            }
        }
    "#,
    )
    .await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "30801");
}

#[tokio::test]
async fn test_refresh_token_success() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken refreshToken
            }
        }
    "#,
    )
    .await;
    let refresh = res["data"]["login"]["refreshToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Refresh
    let query = format!(
        r#"
        mutation {{
            refreshToken(refreshToken: "{}") {{
                status ResponseCode accessToken refreshToken
            }}
        }}
    "#,
        refresh
    );
    let res = graphql_stateful(&state, &query).await;

    let data = &res["data"]["refreshToken"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10901");
    assert!(data["accessToken"].is_string());
    assert!(data["refreshToken"].is_string());
}

#[tokio::test]
async fn test_refresh_invalid_token() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            refreshToken(refreshToken: "fake-token-12345") {
                status ResponseCode
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["refreshToken"]["ResponseCode"], "30901");
}

#[tokio::test]
async fn test_refresh_after_logout() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken refreshToken
            }
        }
    "#,
    )
    .await;
    let refresh = res["data"]["login"]["refreshToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Logout
    let query = format!(
        r#"
        mutation {{
            logout(refreshToken: "{}") {{
                status ResponseCode
            }}
        }}
    "#,
        refresh
    );
    let res = graphql_stateful(&state, &query).await;
    assert_eq!(res["data"]["logout"]["ResponseCode"], "11001");

    // Attempt refresh with old token
    let query = format!(
        r#"
        mutation {{
            refreshToken(refreshToken: "{}") {{
                ResponseCode
            }}
        }}
    "#,
        refresh
    );
    let res = graphql_stateful(&state, &query).await;
    assert_eq!(res["data"]["refreshToken"]["ResponseCode"], "30901");
}

#[tokio::test]
async fn test_logout_success() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                refreshToken
            }
        }
    "#,
    )
    .await;
    let refresh = res["data"]["login"]["refreshToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Logout
    let query = format!(
        r#"
        mutation {{
            logout(refreshToken: "{}") {{
                status ResponseCode
            }}
        }}
    "#,
        refresh
    );
    let res = graphql_stateful(&state, &query).await;

    assert_eq!(res["data"]["logout"]["status"], "success");
    assert_eq!(res["data"]["logout"]["ResponseCode"], "11001");
}

#[tokio::test]
async fn test_delete_account_success() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken
            }
        }
    "#,
    )
    .await;
    let access = res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete account
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            deleteAccount(password: "TestPass123") {
                status ResponseCode ResponseMessage
            }
        }
    "#,
        &access,
    )
    .await;

    let data = &res["data"]["deleteAccount"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "11012");

    // Subsequent login should fail
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") { ResponseCode }
        }
    "#,
    )
    .await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "30801");
}

#[tokio::test]
async fn test_delete_account_wrong_password() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken
            }
        }
    "#,
    )
    .await;
    let access = res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Delete with wrong password
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            deleteAccount(password: "WrongPass") {
                status ResponseCode
            }
        }
    "#,
        &access,
    )
    .await;

    assert_eq!(res["data"]["deleteAccount"]["ResponseCode"], "31001");
}

#[tokio::test]
async fn test_delete_account_unauthenticated() {
    let res = graphql(
        r#"
        mutation {
            deleteAccount(password: "anything") {
                status ResponseCode ResponseMessage
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["deleteAccount"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_password_reset_flow() {
    let state = default_shared_state();

    // 1. Request password reset
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            requestPasswordReset(email: "test@peer.com") {
                status ResponseCode nextAttemptAt
            }
        }
    "#,
    )
    .await;
    assert_eq!(res["data"]["requestPasswordReset"]["ResponseCode"], "11901");

    // 2. Extract the reset token from state
    let token = {
        let st = state.read().await;
        st.password_reset_tokens
            .keys()
            .next()
            .cloned()
            .expect("Reset token should exist")
    };

    // 3. Verify token
    let query = format!(
        r#"
        mutation {{
            resetPasswordTokenVerify(token: "{}") {{
                status ResponseCode
            }}
        }}
    "#,
        token
    );
    let res = graphql_stateful(&state, &query).await;
    assert_eq!(
        res["data"]["resetPasswordTokenVerify"]["ResponseCode"],
        "11902"
    );

    // 4. Reset password
    let query = format!(
        r#"
        mutation {{
            resetPassword(token: "{}", password: "NewPass789") {{
                status ResponseCode
            }}
        }}
    "#,
        token
    );
    let res = graphql_stateful(&state, &query).await;
    assert_eq!(res["data"]["resetPassword"]["ResponseCode"], "11005");

    // 5. Login with new password
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "NewPass789") {
                ResponseCode accessToken
            }
        }
    "#,
    )
    .await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "10801");
    assert!(res["data"]["login"]["accessToken"].is_string());

    // 6. Old password should no longer work
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                ResponseCode
            }
        }
    "#,
    )
    .await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "30801");
}

#[tokio::test]
async fn test_reset_token_verify_invalid() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            resetPasswordTokenVerify(token: "bad-token-xyz") {
                status ResponseCode
            }
        }
    "#,
    )
    .await;

    assert_eq!(
        res["data"]["resetPasswordTokenVerify"]["ResponseCode"],
        "31904"
    );
}

#[tokio::test]
async fn test_reset_password_invalid_token() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            resetPassword(token: "bad-token-xyz", password: "NewPass789") {
                status ResponseCode
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["resetPassword"]["ResponseCode"], "31904");
}

#[tokio::test]
async fn test_reset_password_invalidates_sessions() {
    let state = default_shared_state();

    // Login to get an access token
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken
            }
        }
    "#,
    )
    .await;
    let access = res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Request password reset
    graphql_stateful(
        &state,
        r#"
        mutation {
            requestPasswordReset(email: "test@peer.com") {
                status
            }
        }
    "#,
    )
    .await;

    // Get the reset token
    let token = {
        let st = state.read().await;
        st.password_reset_tokens
            .keys()
            .next()
            .cloned()
            .expect("Reset token should exist")
    };

    // Reset password (this should invalidate sessions)
    let query = format!(
        r#"
        mutation {{
            resetPassword(token: "{}", password: "ResetPass999") {{
                status ResponseCode
            }}
        }}
    "#,
        token
    );
    graphql_stateful(&state, &query).await;

    // Old access token should no longer work for protected mutations
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            deleteAccount(password: "ResetPass999") {
                ResponseCode
            }
        }
    "#,
        &access,
    )
    .await;
    assert_eq!(res["data"]["deleteAccount"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_update_password_success() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken
            }
        }
    "#,
    )
    .await;
    let access = res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Change password
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updatePassword(password: "NewPass456", expassword: "TestPass123") {
                status ResponseCode
            }
        }
    "#,
        &access,
    )
    .await;
    assert_eq!(res["data"]["updatePassword"]["ResponseCode"], "11001");

    // Login with new password
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "NewPass456") {
                ResponseCode accessToken
            }
        }
    "#,
    )
    .await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "10801");

    // Old password should fail
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                ResponseCode
            }
        }
    "#,
    )
    .await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "30801");
}

#[tokio::test]
async fn test_update_password_wrong_old() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken
            }
        }
    "#,
    )
    .await;
    let access = res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Change with wrong old password
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updatePassword(password: "NewPass456", expassword: "WrongOldPass") {
                status ResponseCode
            }
        }
    "#,
        &access,
    )
    .await;
    assert_eq!(res["data"]["updatePassword"]["ResponseCode"], "31001");
}

#[tokio::test]
async fn test_update_password_unauthenticated() {
    let res = graphql(
        r#"
        mutation {
            updatePassword(password: "NewPass456", expassword: "OldPass123") {
                status ResponseCode
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["updatePassword"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_contactus_success() {
    let res = graphql(
        r#"
        mutation {
            contactus(name: "Alice", email: "alice@test.com", message: "Hello!") {
                status
                ResponseCode
                affectedRows { msgid email name message }
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["contactus"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10401");
    assert_eq!(data["affectedRows"]["email"], "alice@test.com");
    assert_eq!(data["affectedRows"]["name"], "Alice");
    assert_eq!(data["affectedRows"]["message"], "Hello!");
}

#[tokio::test]
async fn test_request_password_reset_unknown_email() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            requestPasswordReset(email: "nobody@nowhere.com") {
                status ResponseCode
            }
        }
    "#,
    )
    .await;

    // Anti-enumeration: always returns success
    assert_eq!(res["data"]["requestPasswordReset"]["ResponseCode"], "11901");

    // But no token should have been generated
    let st = state.read().await;
    assert!(st.password_reset_tokens.is_empty());
}

#[tokio::test]
async fn test_refresh_then_old_token_invalid() {
    let state = default_shared_state();

    // Login
    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken refreshToken
            }
        }
    "#,
    )
    .await;
    let old_refresh = res["data"]["login"]["refreshToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Refresh
    let query = format!(
        r#"
        mutation {{
            refreshToken(refreshToken: "{}") {{
                status ResponseCode accessToken refreshToken
            }}
        }}
    "#,
        old_refresh
    );
    let res = graphql_stateful(&state, &query).await;
    assert_eq!(res["data"]["refreshToken"]["ResponseCode"], "10901");

    // Old refresh token should now be invalid
    let query = format!(
        r#"
        mutation {{
            refreshToken(refreshToken: "{}") {{
                ResponseCode
            }}
        }}
    "#,
        old_refresh
    );
    let res = graphql_stateful(&state, &query).await;
    assert_eq!(res["data"]["refreshToken"]["ResponseCode"], "30901");
}

#[tokio::test]
async fn test_protected_mutation_without_auth() {
    let res = graphql(
        r#"
        mutation {
            deleteAccount(password: "anything") {
                status ResponseCode ResponseMessage
            }
        }
    "#,
    )
    .await;

    let data = &res["data"]["deleteAccount"];
    assert_eq!(data["ResponseCode"], "60501");
}

// ============================================================================
// Phase 2: Users & Profiles Tests — Helpers
// ============================================================================

async fn login_as(state: &Arc<RwLock<MockState>>, email: &str, password: &str) -> String {
    let res = graphql_stateful(
        state,
        &format!(
            r#"
        mutation {{
            login(email: "{email}", password: "{password}") {{
                accessToken
            }}
        }}
    "#
        ),
    )
    .await;
    res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn login_default(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "test@peer.com", "TestPass123").await
}

async fn login_alice(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "alice@peer.com", "AlicePass123").await
}

// ============================================================================
// Phase 2: getProfile Tests
// ============================================================================

#[tokio::test]
async fn test_get_own_profile() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            getProfile {
                meta { status ResponseCode }
                affectedRows {
                    id username slug
                    iFollowThisUser thisUserFollowsMe
                    amountfollower amountfollowed amountfriends
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["getProfile"];
    assert_eq!(data["meta"]["ResponseCode"], "11008");
    assert_eq!(data["affectedRows"]["username"], "alice_peer");
    assert_eq!(data["affectedRows"]["amountfriends"], 1);
}

#[tokio::test]
async fn test_get_other_user_profile() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            getProfile(userid: "{}") {{
                meta {{ ResponseCode }}
                affectedRows {{
                    username
                    iFollowThisUser
                    thisUserFollowsMe
                }}
            }}
        }}
    "#,
            SEED_USER_BOB
        ),
        &token,
    )
    .await;

    let data = &res["data"]["getProfile"]["affectedRows"];
    assert_eq!(data["username"], "bob_peer");
    assert_eq!(data["iFollowThisUser"], true);
    assert_eq!(data["thisUserFollowsMe"], true);
}

#[tokio::test]
async fn test_get_profile_not_found() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            getProfile(userid: "00000000-0000-0000-0000-000000000099") {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["getProfile"]["meta"]["ResponseCode"], "21001");
    assert!(res["data"]["getProfile"]["affectedRows"].is_null());
}

#[tokio::test]
async fn test_get_profile_unauthenticated() {
    let res = graphql(
        r#"
        query {
            getProfile { meta { ResponseCode } }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["getProfile"]["meta"]["ResponseCode"], "60501");
}

// ============================================================================
// Phase 2: searchUser Tests
// ============================================================================

#[tokio::test]
async fn test_search_user_by_username() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            searchUser(username: "alice", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { id username slug img }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["searchUser"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert!(data["counter"].as_i64().unwrap() >= 1);
    assert_eq!(data["affectedRows"][0]["username"], "alice_peer");
}

#[tokio::test]
async fn test_search_user_no_results() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            searchUser(username: "nonexistent_user_xyz", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["searchUser"]["meta"]["ResponseCode"], "21001");
    assert_eq!(res["data"]["searchUser"]["counter"], 0);
}

#[tokio::test]
async fn test_search_user_pagination() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Search for "peer" which matches multiple users, limit to 1
    let res = graphql_with_auth(
        &state,
        r#"
        query {
            searchUser(username: "peer", offset: 0, limit: 1) {
                meta { ResponseCode }
                counter
                affectedRows { username }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["searchUser"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert!(data["counter"].as_i64().unwrap() > 1);
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Phase 2: listUsersV2 Tests
// ============================================================================

#[tokio::test]
async fn test_list_users_v2_by_username() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listUsersV2(username: "bob", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { id username slug }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listUsersV2"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert!(data["counter"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_list_users_v2_by_userid() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listUsersV2(userid: "{}") {{
                meta {{ ResponseCode }}
                counter
                affectedRows {{ id username }}
            }}
        }}
    "#,
            SEED_USER_BOB
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listUsersV2"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert_eq!(data["counter"], 1);
    assert_eq!(data["affectedRows"][0]["username"], "bob_peer");
}

#[tokio::test]
async fn test_list_users_v2_excludes_blocked() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listUsersV2(username: "dave", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { username }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listUsersV2"];
    if let Some(arr) = data["affectedRows"].as_array() {
        for user in arr {
            assert_ne!(user["username"], "dave_peer");
        }
    }
}

#[tokio::test]
async fn test_list_users_v2_empty_results() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listUsersV2(username: "zzz_no_such_user_zzz", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["listUsersV2"]["meta"]["ResponseCode"], "21001");
    assert_eq!(res["data"]["listUsersV2"]["counter"], 0);
}

// ============================================================================
// Phase 2: getUser Tests
// ============================================================================

#[tokio::test]
async fn test_get_user_by_id() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            getUser(id: "{}") {{
                meta {{ ResponseCode }}
                affectedRows {{
                    id username slug img biography
                    amountFollowers amountFollowing amountPeers
                    userPreferences {{ contentFilteringSeverityLevel }}
                }}
            }}
        }}
    "#,
            SEED_USER_ALICE
        ),
        &token,
    )
    .await;

    let data = &res["data"]["getUser"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert_eq!(data["affectedRows"]["username"], "alice_peer");
    assert!(data["affectedRows"]["userPreferences"]["contentFilteringSeverityLevel"].is_string());
}

#[tokio::test]
async fn test_get_user_not_found() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            getUser(id: "00000000-0000-0000-0000-000000000099") {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["getUser"]["meta"]["ResponseCode"], "21001");
}

// ============================================================================
// Phase 2: toggleUserFollowStatus Tests
// ============================================================================

#[tokio::test]
async fn test_toggle_follow() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Follow alice
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            toggleUserFollowStatus(userid: "{}") {{
                meta {{ ResponseCode }}
                isfollowing
            }}
        }}
    "#,
            SEED_USER_ALICE
        ),
        &token,
    )
    .await;

    let data = &res["data"]["toggleUserFollowStatus"];
    assert_eq!(data["meta"]["ResponseCode"], "11104");
    assert_eq!(data["isfollowing"], true);

    // Unfollow alice
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            toggleUserFollowStatus(userid: "{}") {{
                meta {{ ResponseCode }}
                isfollowing
            }}
        }}
    "#,
            SEED_USER_ALICE
        ),
        &token,
    )
    .await;

    let data = &res["data"]["toggleUserFollowStatus"];
    assert_eq!(data["meta"]["ResponseCode"], "11103");
    assert_eq!(data["isfollowing"], false);
}

#[tokio::test]
async fn test_toggle_follow_unauthenticated() {
    let res = graphql(&format!(
        r#"
        mutation {{
            toggleUserFollowStatus(userid: "{}") {{
                meta {{ ResponseCode }}
                isfollowing
            }}
        }}
    "#,
        SEED_USER_ALICE
    ))
    .await;

    assert_eq!(
        res["data"]["toggleUserFollowStatus"]["meta"]["ResponseCode"],
        "60501"
    );
}

// ============================================================================
// Phase 2: listFollowRelations Tests
// ============================================================================

#[tokio::test]
async fn test_list_follow_relations() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listFollowRelations(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows {
                    followers { userid username isfollowed isfollowing }
                    following { userid username isfollowed isfollowing }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listFollowRelations"];
    assert_eq!(data["meta"]["ResponseCode"], "11101");
    let followers = &data["affectedRows"]["followers"];
    let following = &data["affectedRows"]["following"];
    assert!(followers.as_array().unwrap().len() >= 2);
    assert!(following.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_follow_relations_other_user() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listFollowRelations(userid: "{}", offset: 0, limit: 20) {{
                meta {{ ResponseCode }}
                affectedRows {{
                    followers {{ userid isfollowed isfollowing }}
                    following {{ userid isfollowed isfollowing }}
                }}
            }}
        }}
    "#,
            SEED_USER_BOB
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listFollowRelations"];
    assert_eq!(data["meta"]["ResponseCode"], "11101");
}

// ============================================================================
// Phase 2: listFriends Tests
// ============================================================================

#[tokio::test]
async fn test_list_friends_mutual_only() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listFriends(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows { userid username }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listFriends"];
    assert_eq!(data["meta"]["ResponseCode"], "11102");
    let friends: Vec<&str> = data["affectedRows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["username"].as_str().unwrap())
        .collect();
    assert!(friends.contains(&"bob_peer"));
}

#[tokio::test]
async fn test_list_friends_no_friends() {
    let state = default_shared_state();
    let token = login_as(&state, "dave@peer.com", "DavePass123").await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listFriends(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows { userid }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["listFriends"]["meta"]["ResponseCode"], "21101");
    assert_eq!(res["data"]["listFriends"]["counter"], 0);
}

// ============================================================================
// Phase 2: toggleBlockUserStatus Tests
// ============================================================================

#[tokio::test]
async fn test_toggle_block_removes_follows() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            toggleBlockUserStatus(userid: "{}") {{
                status ResponseCode
            }}
        }}
    "#,
            SEED_USER_BOB
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["toggleBlockUserStatus"]["ResponseCode"],
        "11105"
    );

    let st = state.read().await;
    assert!(!st.is_following(&SEED_USER_ALICE, &SEED_USER_BOB));
    assert!(!st.is_following(&SEED_USER_BOB, &SEED_USER_ALICE));
}

#[tokio::test]
async fn test_toggle_block_unblock() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            toggleBlockUserStatus(userid: "{}") {{ ResponseCode }}
        }}
    "#,
            SEED_USER_ALICE
        ),
        &token,
    )
    .await;
    assert_eq!(
        res["data"]["toggleBlockUserStatus"]["ResponseCode"],
        "11105"
    );

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            toggleBlockUserStatus(userid: "{}") {{ ResponseCode }}
        }}
    "#,
            SEED_USER_ALICE
        ),
        &token,
    )
    .await;
    assert_eq!(
        res["data"]["toggleBlockUserStatus"]["ResponseCode"],
        "11106"
    );
}

#[tokio::test]
async fn test_toggle_block_unauthenticated() {
    let res = graphql(&format!(
        r#"
        mutation {{
            toggleBlockUserStatus(userid: "{}") {{ ResponseCode }}
        }}
    "#,
        SEED_USER_ALICE
    ))
    .await;

    assert_eq!(
        res["data"]["toggleBlockUserStatus"]["ResponseCode"],
        "60501"
    );
}

// ============================================================================
// Phase 2: listBlockedUsers Tests
// ============================================================================

#[tokio::test]
async fn test_list_blocked_users() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listBlockedUsers(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows {
                    iBlocked { userid username }
                    blockedBy { userid username }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listBlockedUsers"];
    assert_eq!(data["meta"]["ResponseCode"], "11107");
    let i_blocked: Vec<&str> = data["affectedRows"]["iBlocked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(i_blocked.contains(&"dave_peer"));
}

// ============================================================================
// Phase 2: reportUser Tests
// ============================================================================

#[tokio::test]
async fn test_report_user_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            reportUser(userid: "{}") {{
                status ResponseCode
            }}
        }}
    "#,
            SEED_USER_DAVE
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "11012");
}

#[tokio::test]
async fn test_report_user_self() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            reportUser(userid: "{}") {{
                ResponseCode
            }}
        }}
    "#,
            SEED_USER_VERIFIED
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "31009");
}

#[tokio::test]
async fn test_report_user_duplicate() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{ reportUser(userid: "{}") {{ ResponseCode }} }}
    "#,
            SEED_USER_DAVE
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{ reportUser(userid: "{}") {{ ResponseCode }} }}
    "#,
            SEED_USER_DAVE
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "31008");
}

#[tokio::test]
async fn test_report_user_not_found() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            reportUser(userid: "00000000-0000-0000-0000-000000000099") {
                ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "31007");
}

// ============================================================================
// Phase 2: updateProfileImage Tests
// ============================================================================

#[tokio::test]
async fn test_update_profile_image() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateProfileImage(img: "data:image/png;base64,iVBOR...") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateProfileImage"]["ResponseCode"], "11004");

    let res = graphql_with_auth(
        &state,
        r#"
        query { getProfile { affectedRows { img } } }
    "#,
        &token,
    )
    .await;
    assert_eq!(
        res["data"]["getProfile"]["affectedRows"]["img"],
        "data:image/png;base64,iVBOR..."
    );
}

// ============================================================================
// Phase 2: updateBio Tests
// ============================================================================

#[tokio::test]
async fn test_update_bio() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateBio(biography: "data:text/plain;base64,SGVsbG8gV29ybGQ=") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateBio"]["ResponseCode"], "11003");
}

// ============================================================================
// Phase 2: updateUsername Tests
// ============================================================================

#[tokio::test]
async fn test_update_username_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUsername(username: "newName123", password: "TestPass123") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "11007");
}

#[tokio::test]
async fn test_update_username_invalid_format() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUsername(username: "ab", password: "TestPass123") {
                ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "30202");
}

#[tokio::test]
async fn test_update_username_wrong_password() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUsername(username: "validName", password: "WrongPass") {
                ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "31001");
}

// ============================================================================
// Phase 2: updateEmail Tests
// ============================================================================

#[tokio::test]
async fn test_update_email_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateEmail(email: "newemail@peer.com", password: "TestPass123") {
                status ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateEmail"]["ResponseCode"], "11006");
}

#[tokio::test]
async fn test_update_email_wrong_password() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateEmail(email: "new@peer.com", password: "WrongPass") {
                ResponseCode
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["updateEmail"]["ResponseCode"], "31001");
}

// ============================================================================
// Phase 2: updateUserPreferences Tests
// ============================================================================

#[tokio::test]
async fn test_update_preferences() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            updateUserPreferences(userPreferences: {
                contentFilteringSeverityLevel: MYGRANDMAHATES
            }) {
                status ResponseCode
                affectedRows { contentFilteringSeverityLevel }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["updateUserPreferences"];
    assert_eq!(data["ResponseCode"], "11014");
    assert_eq!(
        data["affectedRows"]["contentFilteringSeverityLevel"],
        "MYGRANDMAHATES"
    );
}

// ============================================================================
// Phase 2: getUserInfo Tests
// ============================================================================

#[tokio::test]
async fn test_get_user_info() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            getUserInfo {
                meta { ResponseCode }
                affectedRows {
                    userid
                    amountfollower
                    amountfollowed
                    userPreferences { contentFilteringSeverityLevel }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["getUserInfo"]["meta"]["ResponseCode"], "11009");
    assert!(res["data"]["getUserInfo"]["affectedRows"]["userid"].is_string());
}

// ============================================================================
// Phase 2: getReferralInfo Tests
// ============================================================================

#[tokio::test]
async fn test_get_referral_info() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            getReferralInfo {
                meta { ResponseCode }
                referralUuid
                referralLink
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["getReferralInfo"]["meta"]["ResponseCode"],
        "11011"
    );
    assert!(res["data"]["getReferralInfo"]["referralUuid"].is_string());
}

// ============================================================================
// Phase 2: referralList Tests
// ============================================================================

#[tokio::test]
async fn test_referral_list() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            referralList(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows {
                    invitedBy { userid username }
                    iInvited { userid username }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["referralList"];
    assert_eq!(data["meta"]["ResponseCode"], "11011");
    assert!(data["affectedRows"]["invitedBy"]["userid"].is_string());
}

// ============================================================================
// Phase 2: Integration / Flow Tests
// ============================================================================

#[tokio::test]
async fn test_follow_then_block_removes_follow() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Follow dave
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{ toggleUserFollowStatus(userid: "{}") {{ isfollowing }} }}
    "#,
            SEED_USER_DAVE
        ),
        &token,
    )
    .await;
    assert_eq!(res["data"]["toggleUserFollowStatus"]["isfollowing"], true);

    // Block dave
    graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{ toggleBlockUserStatus(userid: "{}") {{ ResponseCode }} }}
    "#,
            SEED_USER_DAVE
        ),
        &token,
    )
    .await;

    let st = state.read().await;
    assert!(!st.is_following(&SEED_USER_VERIFIED, &SEED_USER_DAVE));
}

#[tokio::test]
async fn test_blocked_user_excluded_from_search() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            searchUser(username: "dave", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { username }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["searchUser"];
    if let Some(arr) = data["affectedRows"].as_array() {
        for user in arr {
            assert_ne!(user["username"], "dave_peer");
        }
    }
}

// ============================================================================
// Phase 3: Posts & Content Tests — Helpers
// ============================================================================

async fn login_bob(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "bob@peer.com", "BobPass123").await
}

async fn do_post_action(
    state: &Arc<RwLock<MockState>>,
    token: &str,
    postid: &str,
    action: &str,
) -> Value {
    graphql_with_auth(
        state,
        &format!(
            r#"
        mutation {{
            resolvePostAction(postid: "{postid}", action: {action}) {{
                status RequestId ResponseCode ResponseMessage
            }}
        }}
    "#
        ),
        token,
    )
    .await
}

// ============================================================================
// Phase 3: listPosts Tests
// ============================================================================

#[tokio::test]
async fn test_list_posts_with_seed_data() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 10) {
                meta { status ResponseCode }
                counter
                affectedRows {
                    id contenttype title amountlikes amountviews
                    isliked isviewed tags
                    user { id username slug }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    // 8 seed posts minus 1 ad post = 7 in normal feed
    assert!(data["counter"].as_i64().unwrap() >= 7);
    assert!(!data["affectedRows"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_posts_pagination() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 3) {
                meta { ResponseCode }
                counter
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 3);
    assert!(data["counter"].as_i64().unwrap() > 3);
}

#[tokio::test]
async fn test_list_posts_offset_beyond_range() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 100, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "21518");
    assert!(data["affectedRows"].as_array().unwrap().is_empty());
    assert!(data["counter"].as_i64().unwrap() >= 7);
}

#[tokio::test]
async fn test_list_posts_filter_by_image() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(filterBy: [IMAGE], sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { id contenttype }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(row["contenttype"], "image");
    }
}

#[tokio::test]
async fn test_list_posts_filter_by_text() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(filterBy: [TEXT], sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { id contenttype }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(row["contenttype"], "text");
    }
}

#[tokio::test]
async fn test_list_posts_sort_by_newest() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 20) {
                affectedRows { createdat }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    for i in 1..rows.len() {
        assert!(
            rows[i - 1]["createdat"].as_str().unwrap() >= rows[i]["createdat"].as_str().unwrap()
        );
    }
}

#[tokio::test]
async fn test_list_posts_sort_by_likes() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(sortBy: LIKES, offset: 0, limit: 20) {
                affectedRows { amountlikes }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    for i in 1..rows.len() {
        assert!(
            rows[i - 1]["amountlikes"].as_i64().unwrap()
                >= rows[i]["amountlikes"].as_i64().unwrap()
        );
    }
}

#[tokio::test]
async fn test_list_posts_search_by_title() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(title: "rust", sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { title }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert!(
            row["title"]
                .as_str()
                .unwrap()
                .to_lowercase()
                .contains("rust")
        );
    }
}

#[tokio::test]
async fn test_list_posts_filter_by_tag() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(tag: "tutorial", sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { tags }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        let tags: Vec<&str> = row["tags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap())
            .collect();
        assert!(tags.contains(&"tutorial"));
    }
}

// ============================================================================
// Phase 3: Single Post Tests
// ============================================================================

#[tokio::test]
async fn test_list_posts_single_by_postid() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_1.to_string();

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listPosts(postid: "{postid}", limit: 1) {{
                meta {{ ResponseCode }}
                counter
                affectedRows {{ id isliked isviewed }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert_eq!(data["counter"].as_i64().unwrap(), 1);
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_list_posts_invalid_postid() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(postid: "not-a-uuid", limit: 1) {
                meta { ResponseCode }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["listPosts"]["meta"]["ResponseCode"], "30209");
}

#[tokio::test]
async fn test_list_posts_nonexistent_postid() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(postid: "99999999-9999-4999-a999-999999999999", limit: 1) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["listPosts"]["meta"]["ResponseCode"], "21518");
}

#[tokio::test]
async fn test_guest_list_post() {
    let state = default_shared_state();
    let postid = SEED_POST_1.to_string();

    let res = graphql_stateful(
        &state,
        &format!(
            r#"
        query {{
            guestListPost(postid: "{postid}") {{
                meta {{ status ResponseCode }}
                counter
                affectedRows {{
                    id title contenttype
                    isliked isviewed isdisliked issaved
                    user {{ id username slug }}
                }}
            }}
        }}
    "#
        ),
    )
    .await;

    let data = &res["data"]["guestListPost"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert_eq!(data["counter"].as_i64().unwrap(), 1);
    let post = &data["affectedRows"][0];
    assert_eq!(post["isliked"], false);
    assert_eq!(post["isviewed"], false);
    assert_eq!(post["isdisliked"], false);
    assert_eq!(post["issaved"], false);
}

#[tokio::test]
async fn test_guest_list_post_not_found() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        query {
            guestListPost(postid: "99999999-9999-4999-a999-999999999999") {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(
        res["data"]["guestListPost"]["meta"]["ResponseCode"],
        "31510"
    );
}

#[tokio::test]
async fn test_guest_list_post_invalid_uuid() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        query {
            guestListPost(postid: "not-a-uuid") {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(
        res["data"]["guestListPost"]["meta"]["ResponseCode"],
        "30209"
    );
}

#[tokio::test]
async fn test_list_user_posts() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let userid = SEED_USER_VERIFIED.to_string();

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listPosts(userid: "{userid}", sortBy: NEWEST, offset: 0, limit: 20) {{
                meta {{ ResponseCode }}
                affectedRows {{ user {{ id }} }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    let rows = data["affectedRows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(row["user"]["id"], userid);
    }
}

#[tokio::test]
async fn test_list_posts_without_auth() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 10) {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["listPosts"]["meta"]["ResponseCode"], "60501");
}

// ============================================================================
// Phase 3: Create Post Tests
// ============================================================================

#[tokio::test]
async fn test_create_post_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "My new post",
                contenttype: text,
                mediadescription: "Test description"
            }) {
                meta { ResponseCode }
                affectedRows { id contenttype title }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["createPost"];
    assert_eq!(data["meta"]["ResponseCode"], "11508");
    assert!(data["affectedRows"]["id"].is_string());
    assert_eq!(data["affectedRows"]["contenttype"], "text");
    assert_eq!(data["affectedRows"]["title"], "My new post");
}

#[tokio::test]
async fn test_create_post_empty_title() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "",
                contenttype: text
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30210");
    assert!(res["data"]["createPost"]["affectedRows"].is_null());
}

#[tokio::test]
async fn test_create_post_title_too_long() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let long_title = "a".repeat(64);

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            createPost(action: POST, input: {{
                title: "{long_title}",
                contenttype: text
            }}) {{
                meta {{ ResponseCode }}
                affectedRows {{ id }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30210");
}

#[tokio::test]
async fn test_create_post_invalid_tag_format() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Tag test",
                contenttype: text,
                tags: ["a"]
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30262");
}

#[tokio::test]
async fn test_create_post_too_many_tags() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Tag test",
                contenttype: text,
                tags: ["tag1","tag2","tag3","tag4","tag5","tag6","tag7","tag8","tag9","tag10","tag11"]
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30262");
}

#[tokio::test]
async fn test_create_post_description_too_long() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let long_desc = "a".repeat(501);

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            createPost(action: POST, input: {{
                title: "Desc test",
                contenttype: text,
                mediadescription: "{long_desc}"
            }}) {{
                meta {{ ResponseCode }}
                affectedRows {{ id }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30263");
}

#[tokio::test]
async fn test_created_post_appears_in_list() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Create a post
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Unique findable post",
                contenttype: text,
                mediadescription: "For search test"
            }) {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;
    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "11508");

    // Search for it
    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(title: "Unique findable", sortBy: NEWEST, offset: 0, limit: 10) {
                affectedRows { title }
            }
        }
    "#,
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    assert!(rows.iter().any(|r| r["title"] == "Unique findable post"));
}

#[tokio::test]
async fn test_create_post_without_auth() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Unauth post",
                contenttype: text
            }) {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "60501");
}

// ============================================================================
// Phase 3: Post Action Tests
// ============================================================================

#[tokio::test]
async fn test_like_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    // SEED_POST_4 is by alice, not yet liked by verified user
    let postid = SEED_POST_4.to_string();

    let res = do_post_action(&state, &token, &postid, "LIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11503");

    // Verify isliked flag
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listPosts(postid: "{postid}", limit: 1) {{
                affectedRows {{ isliked amountlikes }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;

    let post = &res["data"]["listPosts"]["affectedRows"][0];
    assert_eq!(post["isliked"], true);
    assert!(post["amountlikes"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_like_own_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_1.to_string();

    let res = do_post_action(&state, &token, &postid, "LIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "31506");
}

#[tokio::test]
async fn test_like_post_twice() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_4.to_string();

    do_post_action(&state, &token, &postid, "LIKE").await;
    let res = do_post_action(&state, &token, &postid, "LIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "31501");
}

#[tokio::test]
async fn test_unlike_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_4.to_string();

    do_post_action(&state, &token, &postid, "LIKE").await;
    let res = do_post_action(&state, &token, &postid, "UNLIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11503");
}

#[tokio::test]
async fn test_dislike_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_4.to_string();

    let res = do_post_action(&state, &token, &postid, "DISLIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11504");
}

#[tokio::test]
async fn test_dislike_own_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_1.to_string();

    let res = do_post_action(&state, &token, &postid, "DISLIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "31507");
}

#[tokio::test]
async fn test_view_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    // SEED_POST_7 not viewed by verified user in seed
    let postid = SEED_POST_3.to_string(); // already viewed, use another
    // Use a different post not yet viewed
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "View test post",
                contenttype: text
            }) {
                affectedRows { id }
            }
        }
    "#,
        &token,
    )
    .await;
    // View alice's post 4 — but it's already viewed in seed
    // Let's view bob's (if any), or a post by alice not yet viewed
    // Simplify: just create post as alice and view as verified user
    let token_alice = login_alice(&state).await;
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            createPost(action: POST, input: {
                title: "Viewable post",
                contenttype: text
            }) {
                affectedRows { id }
            }
        }
    "#,
        &token_alice,
    )
    .await;
    let new_postid = res["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = do_post_action(&state, &token, &new_postid, "VIEW").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11506");

    // Verify isviewed
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listPosts(postid: "{new_postid}", limit: 1) {{
                affectedRows {{ isviewed }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;
    assert_eq!(
        res["data"]["listPosts"]["affectedRows"][0]["isviewed"],
        true
    );
}

#[tokio::test]
async fn test_save_post_toggle() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_3.to_string();

    // Save
    let res = do_post_action(&state, &token, &postid, "SAVE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11512");

    // Verify issaved
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"
        query {{
            listPosts(postid: "{postid}", limit: 1) {{
                affectedRows {{ issaved }}
            }}
        }}
    "#
        ),
        &token,
    )
    .await;
    assert_eq!(res["data"]["listPosts"]["affectedRows"][0]["issaved"], true);

    // Unsave (toggle)
    let res = do_post_action(&state, &token, &postid, "SAVE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11511");
}

#[tokio::test]
async fn test_report_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_3.to_string();

    let res = do_post_action(&state, &token, &postid, "REPORT").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11505");
}

#[tokio::test]
async fn test_report_own_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_1.to_string();

    let res = do_post_action(&state, &token, &postid, "REPORT").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "31508");
}

#[tokio::test]
async fn test_report_post_twice() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_3.to_string();

    do_post_action(&state, &token, &postid, "REPORT").await;
    let res = do_post_action(&state, &token, &postid, "REPORT").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "31503");
}

#[tokio::test]
async fn test_share_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_3.to_string();

    let res = do_post_action(&state, &token, &postid, "SHARE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11507");
}

#[tokio::test]
async fn test_post_action_nonexistent_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = do_post_action(
        &state,
        &token,
        "99999999-9999-4999-a999-999999999999",
        "VIEW",
    )
    .await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "31510");
}

#[tokio::test]
async fn test_post_action_without_auth() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        mutation {
            resolvePostAction(postid: "99999999-9999-4999-a999-999999999999", action: VIEW) {
                status ResponseCode
            }
        }
    "#,
    )
    .await;

    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "60501");
}

// ============================================================================
// Phase 3: Post Eligibility Tests
// ============================================================================

#[tokio::test]
async fn test_post_eligibility() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            postEligibility {
                meta { ResponseCode }
                eligibilityToken
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["postEligibility"];
    assert_eq!(data["meta"]["ResponseCode"], "10901");
    assert!(data["eligibilityToken"].is_string());
    assert!(
        data["eligibilityToken"]
            .as_str()
            .unwrap()
            .starts_with("mock-eligibility-")
    );
}

#[tokio::test]
async fn test_post_eligibility_without_auth() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"
        query {
            postEligibility {
                meta { ResponseCode }
            }
        }
    "#,
    )
    .await;

    assert_eq!(
        res["data"]["postEligibility"]["meta"]["ResponseCode"],
        "60501"
    );
}

// ============================================================================
// Phase 3: Search Tags Tests
// ============================================================================

#[tokio::test]
async fn test_search_tags_match() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            searchTags(tagName: "web") {
                meta { ResponseCode }
                counter
                affectedRows { name }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["searchTags"];
    assert_eq!(data["meta"]["ResponseCode"], "11701");
    let tags = data["affectedRows"].as_array().unwrap();
    assert!(tags.iter().any(|t| t["name"] == "webdev"));
}

#[tokio::test]
async fn test_search_tags_no_match() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            searchTags(tagName: "xyz_nonexistent") {
                meta { ResponseCode }
                counter
                affectedRows { name }
            }
        }
    "#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["searchTags"]["meta"]["ResponseCode"], "21701");
}

// ============================================================================
// Phase 3: Advertisement Posts Tests
// ============================================================================

#[tokio::test]
async fn test_list_advertisement_posts() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listAdvertisementPosts(offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows {
                    post { id title }
                    advertisement { advertisementid advertisementtype }
                }
            }
        }
    "#,
        &token,
    )
    .await;

    let data = &res["data"]["listAdvertisementPosts"];
    // Seed ad (end_date 2025-05-01) is in the past, so Phase 5 active-date filter
    // returns no results — 22002 means "no active advertisements found".
    let code = data["meta"]["ResponseCode"].as_str().unwrap();
    assert!(code == "22002" || code == "12002");
}

// ============================================================================
// Phase 3: Upload Post Tests
// ============================================================================

#[tokio::test]
async fn test_upload_without_auth() {
    let app = app();

    let boundary = "----test-boundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"eligibilityToken\"\r\n\r\nsome-token\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .method("POST")
        .uri("/upload-post")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_upload_invalid_token() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let app = app_with_state(state.clone());

    let boundary = "----test-boundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"eligibilityToken\"\r\n\r\ninvalid-token\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .method("POST")
        .uri("/upload-post")
        .header("Authorization", format!("Bearer {token}"))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["ResponseCode"], "40902");
}

#[tokio::test]
async fn test_upload_post_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Get eligibility token
    let res = graphql_with_auth(
        &state,
        r#"
        query {
            postEligibility {
                eligibilityToken
            }
        }
    "#,
        &token,
    )
    .await;
    let elig_token = res["data"]["postEligibility"]["eligibilityToken"]
        .as_str()
        .unwrap()
        .to_string();

    let app = app_with_state(state.clone());

    let boundary = "----test-boundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"eligibilityToken\"\r\n\r\n{elig_token}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake-image-data\r\n--{boundary}--\r\n"
    );

    let request = Request::builder()
        .method("POST")
        .uri("/upload-post")
        .header("Authorization", format!("Bearer {token}"))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["status"], "success");
    assert_eq!(json["ResponseCode"], "11515");
    assert!(json["affectedRows"]["uploadedFiles"].is_string());
}

// ============================================================================
// Phase 3: FOLLOWED Filter Test
// ============================================================================

#[tokio::test]
async fn test_list_posts_followed_filter() {
    let state = default_shared_state();
    // alice follows bob (mutual), so as alice, FOLLOWED filter should return bob's posts
    // But posts are authored by verified_user and alice
    // Let's have alice follow verified_user first
    let token_alice = login_alice(&state).await;

    // Alice follows verified_user
    graphql_with_auth(
        &state,
        &format!(
            r#"
        mutation {{
            toggleFollow(userid: "{}") {{
                meta {{ ResponseCode }}
            }}
        }}
    "#,
            SEED_USER_VERIFIED
        ),
        &token_alice,
    )
    .await;

    // Now list posts with FOLLOWED filter as alice
    let res = graphql_with_auth(
        &state,
        r#"
        query {
            listPosts(filterBy: [FOLLOWED], sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { user { id } }
            }
        }
    "#,
        &token_alice,
    )
    .await;

    let data = &res["data"]["listPosts"];
    // Should get posts from followed users
    let rows = data["affectedRows"].as_array().unwrap();
    if !rows.is_empty() {
        // All should be from users alice follows
        for row in rows {
            let author_id = row["user"]["id"].as_str().unwrap();
            // Alice follows bob and now verified_user
            assert!(
                author_id == SEED_USER_BOB.to_string()
                    || author_id == SEED_USER_VERIFIED.to_string()
            );
        }
    }
}

// ============================================================================
// Phase 4: Comments & Chat Tests
// ============================================================================

const COMMENT_FIELDS: &str = "commentid userid postid parentid content createdat amountlikes amountreplies isliked user { id username slug img isfollowed isfollowing }";

// --- Comment Query Tests ---

#[tokio::test]
async fn test_list_comments_with_seed_data() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{SEED_POST_1}") {{ meta {{ ResponseCode }} counter affectedRows {{ {COMMENT_FIELDS} }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11601");
    assert!(data["counter"].as_i64().unwrap() >= 2);
    let rows = data["affectedRows"].as_array().unwrap();
    assert!(rows.len() >= 2);
    // All should be top-level (no parentid)
    for row in rows {
        assert!(row["parentid"].is_null());
        assert!(!row["commentid"].as_str().unwrap().is_empty());
        assert!(!row["user"]["username"].as_str().unwrap().is_empty());
    }
}

#[tokio::test]
async fn test_list_comments_no_comments() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{SEED_POST_2}") {{ meta {{ ResponseCode }} counter affectedRows {{ {COMMENT_FIELDS} }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "21601");
    assert_eq!(data["counter"].as_i64().unwrap(), 0);
    assert!(data["affectedRows"].is_null());
}

#[tokio::test]
async fn test_list_comments_pagination() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{SEED_POST_1}", commentOffset: 0, commentLimit: 1) {{ meta {{ ResponseCode }} counter affectedRows {{ {COMMENT_FIELDS} }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11601");
    assert!(data["counter"].as_i64().unwrap() >= 2);
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_list_comments_invalid_post_uuid() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "not-a-uuid") {{ meta {{ ResponseCode }} counter }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30209");
}

#[tokio::test]
async fn test_list_child_comments_with_replies() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listChildComments(parent: "{SEED_COMMENT_1}") {{ meta {{ ResponseCode }} counter affectedRows {{ {COMMENT_FIELDS} }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listChildComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11607");
    assert!(data["counter"].as_i64().unwrap() >= 1);
    let rows = data["affectedRows"].as_array().unwrap();
    // Reply has parentid set
    assert_eq!(
        rows[0]["parentid"].as_str().unwrap(),
        SEED_COMMENT_1.to_string()
    );
}

#[tokio::test]
async fn test_list_child_comments_no_replies() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listChildComments(parent: "{SEED_COMMENT_2}") {{ meta {{ ResponseCode }} counter }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listChildComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "21606");
    assert_eq!(data["counter"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_list_child_comments_invalid_uuid() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { listChildComments(parent: "bad-uuid") { meta { ResponseCode } counter } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listChildComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30209");
}

// --- Comment Mutation Tests ---

#[tokio::test]
async fn test_create_top_level_comment() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "Test comment") {{ meta {{ ResponseCode }} counter affectedRows {{ {COMMENT_FIELDS} }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createComment"];
    let code = data["meta"]["ResponseCode"].as_str().unwrap();
    assert!(code == "11608" || code == "11605");
    assert_eq!(data["counter"].as_i64().unwrap(), 1);
    let rows = data["affectedRows"].as_array().unwrap();
    assert_eq!(rows[0]["content"].as_str().unwrap(), "Test comment");
    assert!(rows[0]["parentid"].is_null());
    assert_eq!(rows[0]["postid"].as_str().unwrap(), SEED_POST_1.to_string());
}

#[tokio::test]
async fn test_create_reply_comment() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "A reply", parentid: "{SEED_COMMENT_1}") {{ meta {{ ResponseCode }} counter affectedRows {{ {COMMENT_FIELDS} }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createComment"];
    let code = data["meta"]["ResponseCode"].as_str().unwrap();
    assert!(code == "11608" || code == "11605");
    let rows = data["affectedRows"].as_array().unwrap();
    assert_eq!(
        rows[0]["parentid"].as_str().unwrap(),
        SEED_COMMENT_1.to_string()
    );
}

#[tokio::test]
async fn test_create_comment_empty_content() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createComment"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30265");
}

#[tokio::test]
async fn test_create_comment_content_too_long() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let long_content = "x".repeat(201);
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "{long_content}") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createComment"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30265");
}

#[tokio::test]
async fn test_create_comment_nonexistent_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { createComment(action: COMMENT, postid: "ffffffff-ffff-4fff-afff-ffffffffffff", content: "test") { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["createComment"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "31602");
}

#[tokio::test]
async fn test_create_reply_nonexistent_parent() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "test", parentid: "ffffffff-ffff-4fff-afff-ffffffffffff") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createComment"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "31603");
}

#[tokio::test]
async fn test_create_reply_to_reply_fails() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // SEED_COMMENT_5 is a reply to SEED_COMMENT_1 (has parent_id set)
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "nested reply", parentid: "{SEED_COMMENT_5}") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createComment"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "41604");
}

#[tokio::test]
async fn test_create_comment_without_auth() {
    let res = graphql(
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "test") {{ meta {{ ResponseCode }} }} }}"#
        ),
    )
    .await;

    let data = &res["data"]["createComment"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "60501");
}

#[tokio::test]
async fn test_created_comment_appears_in_list() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Create a comment on post 2 (which has no comments)
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_2}", content: "New comment here") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Now list comments
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{SEED_POST_2}") {{ meta {{ ResponseCode }} counter affectedRows {{ content }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["listComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11601");
    assert_eq!(data["counter"].as_i64().unwrap(), 1);
    let rows = data["affectedRows"].as_array().unwrap();
    assert_eq!(rows[0]["content"].as_str().unwrap(), "New comment here");
}

// --- Comment Like/Unlike Tests ---

#[tokio::test]
async fn test_like_comment() {
    let state = default_shared_state();
    // Login as verified user (who did NOT author SEED_COMMENT_1, which was by alice)
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode ResponseMessage }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["likeComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "11603");

    // Verify isliked and amountlikes in listComments
    let res2 = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{SEED_POST_1}") {{ affectedRows {{ commentid isliked amountlikes }} }} }}"#
        ),
        &token,
    )
    .await;

    let rows = res2["data"]["listComments"]["affectedRows"]
        .as_array()
        .unwrap();
    let c1 = rows
        .iter()
        .find(|r| r["commentid"].as_str().unwrap() == SEED_COMMENT_1.to_string())
        .unwrap();
    assert_eq!(c1["isliked"].as_bool().unwrap(), true);
    assert!(c1["amountlikes"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_like_own_comment() {
    let state = default_shared_state();
    // SEED_COMMENT_2 is authored by SEED_USER_VERIFIED
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_2}") {{ ResponseCode }} }}"#),
        &token,
    )
    .await;

    let data = &res["data"]["likeComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "31606");
}

#[tokio::test]
async fn test_like_comment_duplicate() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Like once
    graphql_with_auth(
        &state,
        &format!(r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#),
        &token,
    )
    .await;

    // Like again
    let res = graphql_with_auth(
        &state,
        &format!(r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#),
        &token,
    )
    .await;

    let data = &res["data"]["likeComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "31604");
}

#[tokio::test]
async fn test_unlike_comment() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Like first
    graphql_with_auth(
        &state,
        &format!(r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#),
        &token,
    )
    .await;

    // Unlike
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ unlikeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["unlikeComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "11603");
}

#[tokio::test]
async fn test_like_nonexistent_comment() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { likeComment(commentid: "ffffffff-ffff-4fff-afff-ffffffffffff") { ResponseCode } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["likeComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "31601");
}

#[tokio::test]
async fn test_like_comment_without_auth() {
    let res = graphql(&format!(
        r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#
    ))
    .await;

    let data = &res["data"]["likeComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "60501");
}

// --- Comment Report Tests ---

#[tokio::test]
async fn test_report_comment() {
    let state = default_shared_state();
    // SEED_COMMENT_1 is by alice, login as verified user
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ reportComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["reportComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "11604");
}

#[tokio::test]
async fn test_report_own_comment() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ reportComment(commentid: "{SEED_COMMENT_2}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["reportComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "31607");
}

#[tokio::test]
async fn test_report_comment_duplicate() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Report once
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ reportComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    // Report again
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ reportComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["reportComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "31605");
}

// --- Daily Free Action Tests ---

#[tokio::test]
async fn test_daily_free_action_first_4_free() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    for i in 0..4 {
        let res = graphql_with_auth(
            &state,
            &format!(
                r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "comment {i}") {{ meta {{ ResponseCode }} }} }}"#
            ),
            &token,
        )
        .await;

        let code = res["data"]["createComment"]["meta"]["ResponseCode"]
            .as_str()
            .unwrap();
        assert_eq!(code, "11608", "Comment {i} should be free");
    }
}

#[tokio::test]
async fn test_daily_paid_action_5th_comment() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Create 4 free comments
    for i in 0..4 {
        graphql_with_auth(
            &state,
            &format!(
                r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "free {i}") {{ meta {{ ResponseCode }} }} }}"#
            ),
            &token,
        )
        .await;
    }

    // 5th should be paid
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "paid comment") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let code = res["data"]["createComment"]["meta"]["ResponseCode"]
        .as_str()
        .unwrap();
    assert_eq!(code, "11605");
}

// --- Chat Query Tests ---

#[tokio::test]
async fn test_list_chats_with_seed_data() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                meta { ResponseCode }
                affectedRows {
                    id name image createdat updatedat
                    chatparticipants { userid username slug img hasaccess }
                    chatmessages { id senderid chatid content createdat }
                }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listChats"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11801");
    let rows = data["affectedRows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    // Should be sorted by updatedat desc (group chat more recent)
    assert_eq!(rows[0]["id"].as_str().unwrap(), SEED_CHAT_GROUP.to_string());
    assert_eq!(
        rows[1]["id"].as_str().unwrap(),
        SEED_CHAT_PRIVATE.to_string()
    );
}

#[tokio::test]
async fn test_list_chats_no_chats() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        r#"query { listChats { meta { ResponseCode } affectedRows { id } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listChats"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "21801");
    assert!(data["affectedRows"].is_null());
}

#[tokio::test]
async fn test_list_chats_without_auth() {
    let res = graphql(r#"query { listChats { meta { ResponseCode } } }"#).await;

    let data = &res["data"]["listChats"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "60501");
}

#[tokio::test]
async fn test_chat_includes_correct_participants() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id chatparticipants { userid } }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let group = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_GROUP.to_string())
        .unwrap();
    let participants: Vec<&str> = group["chatparticipants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["userid"].as_str().unwrap())
        .collect();
    assert_eq!(participants.len(), 3);
    assert!(participants.contains(&SEED_USER_VERIFIED.to_string().as_str()));
    assert!(participants.contains(&SEED_USER_ALICE.to_string().as_str()));
    assert!(participants.contains(&SEED_USER_BOB.to_string().as_str()));
}

#[tokio::test]
async fn test_chat_messages_ordered_by_time() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id chatmessages { createdat } }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let private = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let msgs = private["chatmessages"].as_array().unwrap();
    assert!(msgs.len() >= 2);
    for pair in msgs.windows(2) {
        assert!(pair[0]["createdat"].as_str().unwrap() <= pair[1]["createdat"].as_str().unwrap());
    }
}

#[tokio::test]
async fn test_private_chat_no_name_image() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id name image }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let private = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    assert!(private["name"].is_null());
    assert!(private["image"].is_null());
}

#[tokio::test]
async fn test_group_chat_has_name() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id name }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let group = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_GROUP.to_string())
        .unwrap();
    assert_eq!(group["name"].as_str().unwrap(), "Rust Developers");
}

// --- Chat Mutation Tests ---

#[tokio::test]
async fn test_create_private_chat() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createChat(input: {{ name: "dm", recipients: ["{SEED_USER_DAVE}"] }}) {{ meta {{ ResponseCode }} affectedRows {{ chatid }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11802");
    assert!(!data["affectedRows"]["chatid"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_private_chat_already_exists() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Try to create a chat between verified_user and alice — already exists as SEED_CHAT_PRIVATE
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createChat(input: {{ name: "dm", recipients: ["{SEED_USER_ALICE}"] }}) {{ meta {{ ResponseCode }} affectedRows {{ chatid }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11803");
    assert_eq!(
        data["affectedRows"]["chatid"].as_str().unwrap(),
        SEED_CHAT_PRIVATE.to_string()
    );
}

#[tokio::test]
async fn test_create_group_chat() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createChat(input: {{ name: "New Group", recipients: ["{SEED_USER_ALICE}", "{SEED_USER_CAROL}"] }}) {{ meta {{ ResponseCode }} affectedRows {{ chatid }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11802");
}

#[tokio::test]
async fn test_create_chat_no_recipients() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { createChat(input: { name: "empty", recipients: [] }) { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30301");
}

#[tokio::test]
async fn test_create_chat_without_auth() {
    let res = graphql(
        &format!(
            r#"mutation {{ createChat(input: {{ name: "dm", recipients: ["{SEED_USER_ALICE}"] }}) {{ meta {{ ResponseCode }} }} }}"#
        ),
    )
    .await;

    let data = &res["data"]["createChat"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "60501");
}

#[tokio::test]
async fn test_send_chat_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "Hello!") {{ meta {{ ResponseCode }} affectedRows {{ id senderid chatid content createdat }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "11804");
    let msg = &data["affectedRows"];
    assert_eq!(msg["content"].as_str().unwrap(), "Hello!");
    assert_eq!(
        msg["senderid"].as_str().unwrap(),
        SEED_USER_VERIFIED.to_string()
    );
    assert_eq!(
        msg["chatid"].as_str().unwrap(),
        SEED_CHAT_PRIVATE.to_string()
    );
}

#[tokio::test]
async fn test_send_message_nonexistent_chat() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { sendChatMessage(chatid: "ffffffff-ffff-4fff-afff-ffffffffffff", content: "test") { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30304");
}

#[tokio::test]
async fn test_send_message_not_participant() {
    let state = default_shared_state();
    // Carol is not in SEED_CHAT_PRIVATE
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "sneaky") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30305");
}

#[tokio::test]
async fn test_send_empty_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30307");
}

#[tokio::test]
async fn test_send_message_too_long() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let long_msg = "x".repeat(501);
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "{long_msg}") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "30306");
}

#[tokio::test]
async fn test_send_message_without_auth() {
    let res = graphql(
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "test") {{ meta {{ ResponseCode }} }} }}"#
        ),
    )
    .await;

    let data = &res["data"]["sendChatMessage"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "60501");
}

#[tokio::test]
async fn test_send_message_updates_chat_updatedat() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Get original updatedat
    let res1 = graphql_with_auth(
        &state,
        r#"query { listChats { affectedRows { id updatedat } } }"#,
        &token,
    )
    .await;

    let rows1 = res1["data"]["listChats"]["affectedRows"]
        .as_array()
        .unwrap();
    let private1 = rows1
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let old_updated = private1["updatedat"].as_str().unwrap().to_string();

    // Send a message
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "new msg") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Check updated timestamp
    let res2 = graphql_with_auth(
        &state,
        r#"query { listChats { affectedRows { id updatedat } } }"#,
        &token,
    )
    .await;

    let rows2 = res2["data"]["listChats"]["affectedRows"]
        .as_array()
        .unwrap();
    let private2 = rows2
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let new_updated = private2["updatedat"].as_str().unwrap();
    assert!(new_updated > old_updated.as_str());
}

#[tokio::test]
async fn test_sent_message_appears_in_list() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ sendChatMessage(chatid: "{SEED_CHAT_PRIVATE}", content: "new visible msg") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listChats {
                affectedRows { id chatmessages { content } }
            }
        }"#,
        &token,
    )
    .await;

    let rows = res["data"]["listChats"]["affectedRows"].as_array().unwrap();
    let private = rows
        .iter()
        .find(|r| r["id"].as_str().unwrap() == SEED_CHAT_PRIVATE.to_string())
        .unwrap();
    let msgs = private["chatmessages"].as_array().unwrap();
    let has_new = msgs
        .iter()
        .any(|m| m["content"].as_str().unwrap() == "new visible msg");
    assert!(has_new);
}

// --- Cross-Cutting Tests ---

#[tokio::test]
async fn test_post_amountcomments_reflects_comment_count() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Post 1 has seed comments — check amountcomments
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listPosts(postid: "{SEED_POST_1}", filterBy: [], sortBy: NEWEST, offset: 0, limit: 1) {{ affectedRows {{ id amountcomments }} }} }}"#
        ),
        &token,
    )
    .await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    // Post 1 has SEED_COMMENT_1, SEED_COMMENT_2 (top-level) + SEED_COMMENT_5 (reply) = 3 visible comments
    let comments = rows[0]["amountcomments"].as_i64().unwrap();
    assert!(comments >= 3, "Expected >= 3 comments, got {comments}");

    // Create a new comment and verify count increases
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_1}", content: "bump count") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res2 = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listPosts(postid: "{SEED_POST_1}", filterBy: [], sortBy: NEWEST, offset: 0, limit: 1) {{ affectedRows {{ amountcomments }} }} }}"#
        ),
        &token,
    )
    .await;

    let new_comments = res2["data"]["listPosts"]["affectedRows"]
        .as_array()
        .unwrap()[0]["amountcomments"]
        .as_i64()
        .unwrap();
    assert_eq!(new_comments, comments + 1);
}

#[tokio::test]
async fn test_reset_clears_phase4_state() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Create a comment
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_2}", content: "will be cleared") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Reset
    {
        let mut s = state.write().await;
        s.reset();
    }

    // Re-login after reset
    let token2 = login_default(&state).await;

    // Post 2 should have no comments again (seed has none on post 2)
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{SEED_POST_2}") {{ meta {{ ResponseCode }} counter }} }}"#
        ),
        &token2,
    )
    .await;

    let data = &res["data"]["listComments"];
    assert_eq!(data["meta"]["ResponseCode"].as_str().unwrap(), "21601");
    assert_eq!(data["counter"].as_i64().unwrap(), 0);

    // But seed comments on post 1 should still be there
    let res2 = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{SEED_POST_1}") {{ meta {{ ResponseCode }} counter }} }}"#
        ),
        &token2,
    )
    .await;

    assert_eq!(
        res2["data"]["listComments"]["meta"]["ResponseCode"]
            .as_str()
            .unwrap(),
        "11601"
    );
    assert!(res2["data"]["listComments"]["counter"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Phase 5: Wallet Tests
// ============================================================================

#[tokio::test]
async fn test_wallet_balance_seeded_user() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { balance { meta { ResponseCode } currentliquidity } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["balance"]["meta"]["ResponseCode"], "11204");
    let balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);
    assert!((balance - 1000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_wallet_balance_no_auth() {
    let state = default_shared_state();
    let res = graphql_stateful(
        &state,
        r#"query { balance { meta { ResponseCode } currentliquidity } }"#,
    )
    .await;
    assert_eq!(res["data"]["balance"]["meta"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_wallet_transfer_tokens() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 50.0) {{ meta {{ ResponseCode }} affectedRows {{ tokenSendFormatted tokensSubstractedFromWalletFormatted }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "11211"
    );
    let rows = &res["data"]["resolveTransferV2"]["affectedRows"];
    assert!(rows["tokenSendFormatted"].as_str().is_some());
    assert!(rows["tokensSubstractedFromWalletFormatted"].as_str().is_some());

    // Verify balances changed
    let res2 = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let sender_bal = decimal_val(&res2["data"]["balance"]["currentliquidity"]);
    // 1000 - 50 - fees (4% of 50 = 2.0) = 948.0
    assert!(sender_bal < 950.0);
}

#[tokio::test]
async fn test_wallet_transfer_fee_breakdown() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 50.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Check transaction history for fee details
    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(type: TRANSACTION, limit: 5) { meta { ResponseCode } affectedRows { fees { burn peer inviter } } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    let rows = &res["data"]["transactionHistory"]["affectedRows"];
    assert!(rows.is_array());
    // Find the new transfer (first one due to sort by newest)
    let tx = &rows[0];
    let fees = &tx["fees"];
    assert!((decimal_val(&fees["burn"]) - 0.5).abs() < 0.01);
    assert!((decimal_val(&fees["peer"]) - 1.0).abs() < 0.01);
    assert!((decimal_val(&fees["inviter"]) - 0.5).abs() < 0.01);
}

#[tokio::test]
async fn test_wallet_transfer_formatted_values() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 50.0) {{ meta {{ ResponseCode }} affectedRows {{ tokenSendFormatted tokensSubstractedFromWalletFormatted }} }} }}"#
        ),
        &token,
    )
    .await;

    let rows = &res["data"]["resolveTransferV2"]["affectedRows"];
    let send = rows["tokenSendFormatted"].as_str().unwrap();
    let subtracted = rows["tokensSubstractedFromWalletFormatted"].as_str().unwrap();
    assert!(send.contains("50"));
    assert!(subtracted.contains("52"));
}

#[tokio::test]
async fn test_wallet_transfer_appears_in_history() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 25.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(type: TRANSACTION, limit: 10) { meta { ResponseCode } affectedRows { tokenamount sender { userid } recipient { userid } } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    let rows = res["data"]["transactionHistory"]["affectedRows"]
        .as_array()
        .unwrap();
    assert!(!rows.is_empty());
}

#[tokio::test]
async fn test_wallet_transfer_to_self() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_VERIFIED}", numberoftokens: 10.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "31202"
    );
}

#[tokio::test]
async fn test_wallet_transfer_to_nonexistent_user() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { resolveTransferV2(recipient: "99999999-9999-4999-a999-999999999999", numberoftokens: 10.0) { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "31007"
    );
}

#[tokio::test]
async fn test_wallet_transfer_to_system_account() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // SYSTEM_PEER_ACCOUNT = "eeeeeeee-eeee-4eee-aeee-000000000002"
    let res = graphql_with_auth(
        &state,
        r#"mutation { resolveTransferV2(recipient: "eeeeeeee-eeee-4eee-aeee-000000000002", numberoftokens: 10.0) { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "31203"
    );
}

#[tokio::test]
async fn test_wallet_transfer_insufficient_balance() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 99999.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "51301"
    );
}

#[tokio::test]
async fn test_wallet_transfer_below_minimum() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 0.0000001) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "30264"
    );
}

#[tokio::test]
async fn test_wallet_transfer_long_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let long_msg = "x".repeat(501);

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 10.0, message: "{long_msg}") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "30270"
    );
}

#[tokio::test]
async fn test_wallet_transfer_url_in_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 10.0, message: "Check https://evil.com") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "30271"
    );
}

#[tokio::test]
async fn test_wallet_transaction_history_pagination() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(limit: 1, offset: 0) { meta { ResponseCode } affectedRows { tokenamount } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    let rows = res["data"]["transactionHistory"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn test_wallet_transaction_history_no_transactions() {
    let state = default_shared_state();
    // Carol has no transactions in seed data
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory { meta { ResponseCode } affectedRows { tokenamount } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "21209"
    );
}

#[tokio::test]
async fn test_wallet_transaction_history_filter_by_type() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(type: PAYMENT) { meta { ResponseCode } affectedRows { transactionCategory } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    // The seed has a Like payment for the verified user
    let rows = res["data"]["transactionHistory"]["affectedRows"]
        .as_array()
        .unwrap();
    assert!(!rows.is_empty());
}

// ============================================================================
// Phase 5: Tokenomics Tests
// ============================================================================

#[tokio::test]
async fn test_tokenomics_action_prices() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { getActionPrices { meta { ResponseCode } affectedRows { postPrice likePrice dislikePrice commentPrice } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["getActionPrices"]["meta"]["ResponseCode"],
        "11304"
    );
    let prices = &res["data"]["getActionPrices"]["affectedRows"];
    assert!((prices["postPrice"].as_f64().unwrap() - 20.0).abs() < 0.01);
    assert!((prices["likePrice"].as_f64().unwrap() - 3.0).abs() < 0.01);
    assert!((prices["dislikePrice"].as_f64().unwrap() - 3.0).abs() < 0.01);
    assert!((prices["commentPrice"].as_f64().unwrap() - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_tokenomics_action_prices_no_auth() {
    let state = default_shared_state();
    let res = graphql_stateful(
        &state,
        r#"query { getActionPrices { meta { ResponseCode } } }"#,
    )
    .await;
    assert_eq!(
        res["data"]["getActionPrices"]["meta"]["ResponseCode"],
        "60501"
    );
}

#[tokio::test]
async fn test_tokenomics_get_tokenomics() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { getTokenomics { meta { ResponseCode } actionTokenPrices { postPrice likePrice } actionGemsReturns { viewGemsReturn likeGemsReturn } mintingData { tokensMintedYesterday } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["getTokenomics"]["meta"]["ResponseCode"],
        "11212"
    );
    assert!(
        res["data"]["getTokenomics"]["actionTokenPrices"]["postPrice"]
            .as_f64()
            .is_some()
    );
    assert!(
        res["data"]["getTokenomics"]["actionGemsReturns"]["viewGemsReturn"]
            .as_f64()
            .is_some()
    );
}

#[tokio::test]
async fn test_tokenomics_daily_free_status_fresh() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { getDailyFreeStatus { meta { ResponseCode } affectedRows { name used available } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["getDailyFreeStatus"]["meta"]["ResponseCode"],
        "11303"
    );
    let rows = res["data"]["getDailyFreeStatus"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 4);
    // All should have used == 0
    for row in rows {
        assert_eq!(row["used"].as_i64().unwrap(), 0);
    }
    // post=1, like=3, comment=4 are > 0; dislike=0
    let post_row = rows.iter().find(|r| r["name"] == "post").unwrap();
    assert_eq!(post_row["available"].as_i64().unwrap(), 1);
    let like_row = rows.iter().find(|r| r["name"] == "like").unwrap();
    assert_eq!(like_row["available"].as_i64().unwrap(), 3);
    let comment_row = rows.iter().find(|r| r["name"] == "comment").unwrap();
    assert_eq!(comment_row["available"].as_i64().unwrap(), 4);
    let dislike_row = rows.iter().find(|r| r["name"] == "dislike").unwrap();
    assert_eq!(dislike_row["available"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_tokenomics_daily_free_after_actions() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Perform 2 likes (SEED_POST_3 and SEED_POST_4 are alice's)
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{SEED_POST_3}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{SEED_POST_4}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { getDailyFreeStatus { meta { ResponseCode } affectedRows { name used available } } }"#,
        &token,
    )
    .await;

    let rows = res["data"]["getDailyFreeStatus"]["affectedRows"]
        .as_array()
        .unwrap();
    let like_row = rows.iter().find(|r| r["name"] == "like").unwrap();
    assert_eq!(like_row["used"].as_i64().unwrap(), 2);
    assert_eq!(like_row["available"].as_i64().unwrap(), 1); // FREE_LIKES = 3
}

#[tokio::test]
async fn test_tokenomics_daily_free_likes_no_cost() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Get initial balance
    let res0 = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let initial_balance = decimal_val(&res0["data"]["balance"]["currentliquidity"]);

    // Like 3 posts (all free: SEED_POST_3, 4 are alice's, we need a 3rd)
    for post in [SEED_POST_3, SEED_POST_4] {
        let _ = graphql_with_auth(
            &state,
            &format!(
                r#"mutation {{ resolvePostAction(action: LIKE, postid: "{post}") {{ ResponseCode }} }}"#
            ),
            &token,
        )
        .await;
    }

    let res = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let after_balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);

    // Balance should be unchanged (free likes)
    assert!((after_balance - initial_balance).abs() < 0.01);
}

#[tokio::test]
async fn test_tokenomics_paid_like_after_free_limit() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res0 = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let initial_balance = decimal_val(&res0["data"]["balance"]["currentliquidity"]);

    // Use all 3 free likes
    for post in [SEED_POST_3, SEED_POST_4] {
        let _ = graphql_with_auth(
            &state,
            &format!(
                r#"mutation {{ resolvePostAction(action: LIKE, postid: "{post}") {{ ResponseCode }} }}"#
            ),
            &token,
        )
        .await;
    }

    // Need a 3rd distinct post from another user. Create one via alice.
    let alice_token = login_alice(&state).await;
    let create_res = graphql_with_auth(
        &state,
        r#"mutation { createPost(action: POST, input: { title: "Test post for like", contenttype: image }) { meta { ResponseCode } affectedRows { id } } }"#,
        &alice_token,
    )
    .await;
    let new_post_id = create_res["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap();

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{new_post_id}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    // Now 4th like should cost tokens. Create another post by alice.
    let create_res2 = graphql_with_auth(
        &state,
        r#"mutation { createPost(action: POST, input: { title: "Test post for paid like", contenttype: image }) { meta { ResponseCode } affectedRows { id } } }"#,
        &alice_token,
    )
    .await;
    let new_post_id2 = create_res2["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap();

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{new_post_id2}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    // Check balance decreased by LIKE_PRICE (3.0)
    let res = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let after_balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);
    assert!((initial_balance - after_balance - 3.0).abs() < 0.5);
}

#[tokio::test]
async fn test_tokenomics_todays_interactions() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let alice_token = login_alice(&state).await;

    // Alice likes verified user's post (creates a gem for verified user)
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{SEED_POST_1}") {{ ResponseCode }} }}"#
        ),
        &alice_token,
    )
    .await;

    // Now check today's interactions for verified user
    let res = graphql_with_auth(
        &state,
        r#"query { listTodaysInteractions { meta { ResponseCode } affectedRows { totalScore } } }"#,
        &token,
    )
    .await;

    let code = res["data"]["listTodaysInteractions"]["meta"]["ResponseCode"]
        .as_str()
        .unwrap();
    assert!(code == "11204" || code == "21204");
}

// ============================================================================
// Phase 5: Advertisement Tests
// ============================================================================

#[tokio::test]
async fn test_ads_list_no_active_ads() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // SEED_AD_1 has end_date 2025-05-01 which is in the past
    let res = graphql_with_auth(
        &state,
        r#"query { listAdvertisementPosts { meta { ResponseCode } counter } }"#,
        &token,
    )
    .await;

    // The seed ad's dates are in the past, so it won't be active
    let code = res["data"]["listAdvertisementPosts"]["meta"]["ResponseCode"]
        .as_str()
        .unwrap();
    assert!(code == "22002" || code == "12002");
}

#[tokio::test]
async fn test_ads_list_no_auth() {
    let state = default_shared_state();
    let res = graphql_stateful(
        &state,
        r#"query { listAdvertisementPosts { meta { ResponseCode } } }"#,
    )
    .await;
    assert_eq!(
        res["data"]["listAdvertisementPosts"]["meta"]["ResponseCode"],
        "60501"
    );
}

#[tokio::test]
async fn test_ads_create_basic() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_1}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} affectedRows {{ id type timeframeStart timeframeEnd totalTokenCost }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["advertisePostBasic"]["meta"]["ResponseCode"],
        "12001"
    );
    let rows = &res["data"]["advertisePostBasic"]["affectedRows"];
    assert!(rows.is_array());
    let ad = &rows[0];
    // 3 days * 50.0/day = 150.0
    assert!((ad["totalTokenCost"].as_f64().unwrap() - 150.0).abs() < 0.01);
}

#[tokio::test]
async fn test_ads_basic_end_date() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_2}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} affectedRows {{ timeframeStart timeframeEnd }} }} }}"#
        ),
        &token,
    )
    .await;

    let ad = &res["data"]["advertisePostBasic"]["affectedRows"][0];
    let start = ad["timeframeStart"].as_str().unwrap();
    let end = ad["timeframeEnd"].as_str().unwrap();
    assert_eq!(start, today);
    // End should be start + 3 days
    let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap();
    let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap();
    assert_eq!((end_date - start_date).num_days(), 3);
}

#[tokio::test]
async fn test_ads_create_pinned() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostPinned(postid: "{SEED_POST_1}", advertisePlan: PINNED) {{ meta {{ ResponseCode }} affectedRows {{ totalTokenCost type }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["advertisePostPinned"]["meta"]["ResponseCode"],
        "12001"
    );
    let ad = &res["data"]["advertisePostPinned"]["affectedRows"][0];
    assert!((ad["totalTokenCost"].as_f64().unwrap() - 200.0).abs() < 0.01);
}

#[tokio::test]
async fn test_ads_create_for_non_owned_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    // SEED_POST_3 belongs to alice
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_3}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["advertisePostBasic"]["meta"]["ResponseCode"],
        "31510"
    );
}

#[tokio::test]
async fn test_ads_create_duplicate_active_ad() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Create first ad
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_1}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Try creating second ad on same post
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_1}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["advertisePostBasic"]["meta"]["ResponseCode"],
        "32006"
    );
}

#[tokio::test]
async fn test_ads_create_insufficient_balance() {
    let state = default_shared_state();
    // Carol has 1000.0 balance. Drain it first via transfer, then try create ad.
    let carol_token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    // Transfer most of Carol's balance away
    // Need to leave Carol with less than 150 tokens (ad cost).
    // Total deduction = transfer + 4% fees.
    // Balance 1000, transfer 850 → costs 850*1.04 = 884 → remaining 116
    let _transfer_res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 850.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &carol_token,
    )
    .await;

    // Carol needs a post to advertise - but Carol doesn't own one in seed data
    // Instead, create one for carol first
    let create_res = graphql_with_auth(
        &state,
        r#"mutation { createPost(action: POST, input: { title: "Carol test post", contenttype: text }) { affectedRows { id } } }"#,
        &carol_token,
    )
    .await;
    let post_id = create_res["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap();

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{post_id}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &carol_token,
    )
    .await;

    assert_eq!(
        res["data"]["advertisePostBasic"]["meta"]["ResponseCode"],
        "51301"
    );
}

#[tokio::test]
async fn test_ads_advertisement_history() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    // Create an ad first
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_1}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { advertisementHistory { meta { ResponseCode } affectedRows { stats { tokenSpent amountAds } advertisements { id type totalTokenCost } } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["advertisementHistory"]["meta"]["ResponseCode"],
        "12002"
    );
    let rows = &res["data"]["advertisementHistory"]["affectedRows"];
    assert!(rows["stats"]["amountAds"].as_i64().unwrap() >= 1);
    assert!(rows["stats"]["tokenSpent"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_ads_history_sort_by_cost() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    // Create two ads with different costs
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_1}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostPinned(postid: "{SEED_POST_2}", advertisePlan: PINNED) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { advertisementHistory(sort: BIGGEST_COST) { meta { ResponseCode } affectedRows { advertisements { totalTokenCost } } } }"#,
        &token,
    )
    .await;

    let ads = res["data"]["advertisementHistory"]["affectedRows"]["advertisements"]
        .as_array()
        .unwrap();
    if ads.len() >= 2 {
        let first_cost = ads[0]["totalTokenCost"].as_f64().unwrap();
        let second_cost = ads[1]["totalTokenCost"].as_f64().unwrap();
        assert!(first_cost >= second_cost);
    }
}

#[tokio::test]
async fn test_ads_advertised_posts_excluded_from_list_posts() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    // Create an active ad on SEED_POST_1
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ advertisePostBasic(postid: "{SEED_POST_1}", startday: "{today}", durationInDays: THREE_DAYS, advertisePlan: BASIC) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // list posts should not include SEED_POST_1
    let res = graphql_with_auth(
        &state,
        r#"query { listPosts(limit: 100) { affectedRows { id } } }"#,
        &token,
    )
    .await;

    let posts = res["data"]["listPosts"]["affectedRows"]
        .as_array()
        .unwrap();
    let has_advertised = posts
        .iter()
        .any(|p| p["id"].as_str().unwrap() == SEED_POST_1.to_string());
    assert!(!has_advertised, "Advertised post should be excluded from listPosts");
}

// ============================================================================
// Phase 5: Shop Tests
// ============================================================================

#[tokio::test]
async fn test_shop_purchase_item() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 50.0, shopItemId: "test-item-001", orderDetails: { name: "Test User", email: "test@example.com", addressline1: "Musterstraße 42", city: "Berlin", zipcode: "10115", country: GERMANY }) { status ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["performShopOrder"]["ResponseCode"],
        "12201"
    );

    // Balance should have decreased
    let res2 = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let balance = decimal_val(&res2["data"]["balance"]["currentliquidity"]);
    assert!((balance - 950.0).abs() < 0.1);
}

#[tokio::test]
async fn test_shop_order_details() {
    let state = default_shared_state();
    let alice_token = login_alice(&state).await;

    // SEED_SHOP_ORDER_1 belongs to alice with SEED_SHOP_TX_1
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ shopOrderDetails(transactionId: "{SEED_SHOP_TX_1}") {{ meta {{ ResponseCode }} affectedRows {{ shopOrderId shopItemId deliveryDetails {{ name email city zipcode }} }} }} }}"#
        ),
        &alice_token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "12202"
    );
    let order = &res["data"]["shopOrderDetails"]["affectedRows"][0];
    assert_eq!(order["shopItemId"], "peer-tshirt-001");
    assert_eq!(order["deliveryDetails"]["city"], "Berlin");
}

#[tokio::test]
async fn test_shop_order_not_found() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { shopOrderDetails(transactionId: "99999999-9999-4999-a999-999999999999") { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["shopOrderDetails"]["meta"]["ResponseCode"],
        "22101"
    );
}

#[tokio::test]
async fn test_shop_purchase_no_auth() {
    let state = default_shared_state();
    let res = graphql_stateful(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 50.0, shopItemId: "test-item", orderDetails: { name: "Test", email: "t@t.com", addressline1: "Street 123", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
    )
    .await;

    assert_eq!(
        res["data"]["performShopOrder"]["ResponseCode"],
        "60501"
    );
}

#[tokio::test]
async fn test_shop_purchase_insufficient_balance() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 99999.0, shopItemId: "expensive-item", orderDetails: { name: "Test User", email: "test@example.com", addressline1: "Street 123456", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["performShopOrder"]["ResponseCode"],
        "51301"
    );
}

#[tokio::test]
async fn test_shop_purchase_invalid_name() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 10.0, shopItemId: "item", orderDetails: { name: "X", email: "test@example.com", addressline1: "Street 123456", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "30101");
}

#[tokio::test]
async fn test_shop_purchase_invalid_zipcode() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 10.0, shopItemId: "item", orderDetails: { name: "Test User", email: "test@example.com", addressline1: "Street 123456", city: "Berlin", zipcode: "123", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "30101");
}

#[tokio::test]
async fn test_shop_purchase_invalid_email() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { performShopOrder(tokenAmount: 10.0, shopItemId: "item", orderDetails: { name: "Test User", email: "notanemail", addressline1: "Street 123456", city: "Berlin", zipcode: "10115", country: GERMANY }) { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performShopOrder"]["ResponseCode"], "30101");
}

// ============================================================================
// Phase 5: Cross-Cutting Tests
// ============================================================================

#[tokio::test]
async fn test_cross_comment_deducts_after_free_limit() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res0 = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let initial_balance = decimal_val(&res0["data"]["balance"]["currentliquidity"]);

    // Use 4 free comments (FREE_COMMENTS = 4)
    for i in 0..4 {
        let _ = graphql_with_auth(
            &state,
            &format!(
                r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_3}", content: "Free comment {i}") {{ meta {{ ResponseCode }} }} }}"#
            ),
            &token,
        )
        .await;
    }

    // 5th comment should cost tokens
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_3}", content: "Paid comment") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let after_balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);

    // Should have decreased by COMMENT_PRICE (1.0)
    assert!((initial_balance - after_balance - 1.0).abs() < 0.1);
}

#[tokio::test]
async fn test_cross_comment_insufficient_balance_after_free() {
    let state = default_shared_state();
    let carol_token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    // Drain Carol's balance. Transfer 961 costs 961*1.04 = 999.44, leaving ~0.56 tokens
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 961.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &carol_token,
    )
    .await;

    // Use 4 free comments
    for i in 0..4 {
        let _ = graphql_with_auth(
            &state,
            &format!(
                r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_3}", content: "Comment {i}") {{ meta {{ ResponseCode }} }} }}"#
            ),
            &carol_token,
        )
        .await;
    }

    // 5th should fail with insufficient balance (needs 1.0 tokens, has ~0.56)
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_3}", content: "Should fail") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &carol_token,
    )
    .await;

    assert_eq!(
        res["data"]["createComment"]["meta"]["ResponseCode"],
        "51301"
    );
}

#[tokio::test]
async fn test_cross_like_deducts_after_free_limit() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let alice_token = login_alice(&state).await;

    let res0 = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let initial_balance = decimal_val(&res0["data"]["balance"]["currentliquidity"]);

    // Use 3 free likes
    for post in [SEED_POST_3, SEED_POST_4] {
        let _ = graphql_with_auth(
            &state,
            &format!(
                r#"mutation {{ resolvePostAction(action: LIKE, postid: "{post}") {{ ResponseCode }} }}"#
            ),
            &token,
        )
        .await;
    }

    // Create a 3rd post by alice
    let create_res = graphql_with_auth(
        &state,
        r#"mutation { createPost(action: POST, input: { title: "Third like target", contenttype: text }) { affectedRows { id } } }"#,
        &alice_token,
    )
    .await;
    let post3 = create_res["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap();
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{post3}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    // 4th like should cost tokens
    let create_res2 = graphql_with_auth(
        &state,
        r#"mutation { createPost(action: POST, input: { title: "Fourth like target", contenttype: text }) { affectedRows { id } } }"#,
        &alice_token,
    )
    .await;
    let post4 = create_res2["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap();
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{post4}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let after_balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);

    // Should have decreased by LIKE_PRICE (3.0)
    assert!((initial_balance - after_balance - 3.0).abs() < 0.5);
}

#[tokio::test]
async fn test_cross_gem_accumulation_on_like() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let alice_token = login_alice(&state).await;

    // Alice likes verified user's post
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{SEED_POST_1}") {{ ResponseCode }} }}"#
        ),
        &alice_token,
    )
    .await;

    // Check win logs for verified user (gem should be recorded)
    let res = graphql_with_auth(
        &state,
        r#"query { listWinLogs(day: D0) { meta { ResponseCode } counter affectedRows { action } } }"#,
        &token,
    )
    .await;

    let code = res["data"]["listWinLogs"]["meta"]["ResponseCode"]
        .as_str()
        .unwrap();
    // Should have at least the seed gems + the new like gem
    assert!(code == "11203" || code == "21202");
    if code == "11203" {
        assert!(res["data"]["listWinLogs"]["counter"].as_i64().unwrap() >= 1);
    }
}

#[tokio::test]
async fn test_cross_reset_clears_economy_state() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Transfer some tokens to modify state
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 100.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Verify balance changed
    let res_before = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token,
    )
    .await;
    let bal_before = decimal_val(&res_before["data"]["balance"]["currentliquidity"]);
    assert!(bal_before < 1000.0);

    // Reset
    let app = app_with_state(state.clone());
    let request = Request::builder()
        .method("POST")
        .uri("/reset")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Login again (sessions cleared)
    let token2 = login_default(&state).await;

    // Balance should be back to seed default (1000.0)
    let res_after = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &token2,
    )
    .await;
    let bal_after = decimal_val(&res_after["data"]["balance"]["currentliquidity"]);
    assert!((bal_after - 1000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_cross_phase4_regression_comment_still_works() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ createComment(action: COMMENT, postid: "{SEED_POST_3}", content: "Regression test comment") {{ meta {{ ResponseCode status }} counter }} }}"#
        ),
        &token,
    )
    .await;

    let code = res["data"]["createComment"]["meta"]["ResponseCode"]
        .as_str()
        .unwrap();
    // Should succeed with either free (11608) or paid (11605) code
    assert!(code == "11608" || code == "11605");
    assert_eq!(res["data"]["createComment"]["counter"].as_i64().unwrap(), 1);
}
