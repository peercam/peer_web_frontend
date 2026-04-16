mod common;
use common::prelude::*;

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
    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/reset")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

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
