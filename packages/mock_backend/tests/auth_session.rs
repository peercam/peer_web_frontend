mod common;
use common::prelude::*;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

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
    let access = res["data"]["login"]["accessToken"]
        .as_str()
        .unwrap()
        .to_string();
    let refresh = res["data"]["login"]["refreshToken"]
        .as_str()
        .unwrap()
        .to_string();

    // Logout — backend reads identity from JWT in Authorization header,
    // no `refreshToken` argument is accepted.
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            logout {
                status ResponseCode
            }
        }
    "#,
        &access,
    )
    .await;
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

    // Logout — JWT-only; no arguments.
    let res = graphql_with_auth(
        &state,
        r#"
        mutation {
            logout {
                status ResponseCode
            }
        }
    "#,
        &access,
    )
    .await;

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
