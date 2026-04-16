mod common;
use common::prelude::*;

#[tokio::test]
async fn test_admin_search_users_by_email() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listUsersAdminV2(email: "alice@peer.com") {
                meta { ResponseCode }
                counter
                affectedRows { id email }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listUsersAdminV2"];
    assert_eq!(data["meta"]["ResponseCode"], "11009");
    assert!(data["counter"].as_i64().unwrap() >= 1);
    assert_eq!(data["affectedRows"][0]["email"], "alice@peer.com");
}

#[tokio::test]
async fn test_admin_search_users_by_ip() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listUsersAdminV2(ip: "192.168.1.1") {
                meta { ResponseCode }
                counter
                affectedRows { id ip }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listUsersAdminV2"];
    assert_eq!(data["meta"]["ResponseCode"], "11009");
    assert!(data["counter"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_admin_search_users_by_verified() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listUsersAdminV2(verified: 1) {
                meta { ResponseCode }
                counter
                affectedRows { id verified }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listUsersAdminV2"];
    assert_eq!(data["meta"]["ResponseCode"], "11009");
    assert!(data["counter"].as_i64().unwrap() >= 1);
    // All returned should be verified
    for user in data["affectedRows"].as_array().unwrap() {
        assert_eq!(user["verified"], 1);
    }
}

#[tokio::test]
async fn test_admin_search_userid_and_username_error() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{
                listUsersAdminV2(userid: "{}", username: "alice") {{
                    meta {{ ResponseCode }}
                }}
            }}"#,
            SEED_USER_ALICE
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["listUsersAdminV2"]["meta"]["ResponseCode"],
        "31012"
    );
}

#[tokio::test]
async fn test_admin_search_invalid_uuid() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listUsersAdminV2(userid: "not-a-uuid") {
                meta { ResponseCode }
            }
        }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["listUsersAdminV2"]["meta"]["ResponseCode"],
        "30201"
    );
}

#[tokio::test]
async fn test_admin_search_invalid_ip() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listUsersAdminV2(ip: "999.999.999.999") {
                meta { ResponseCode }
            }
        }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["listUsersAdminV2"]["meta"]["ResponseCode"],
        "30257"
    );
}

#[tokio::test]
async fn test_admin_search_extended_fields() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listUsersAdminV2(email: "admin@peerapp.de") {
                meta { ResponseCode }
                affectedRows { id email rolesMask liquidity verified }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["listUsersAdminV2"];
    assert_eq!(data["meta"]["ResponseCode"], "11009");
    let user = &data["affectedRows"][0];
    assert!(!user["email"].is_null());
    assert!(!user["rolesMask"].is_null());
    assert!(!user["liquidity"].is_null());
}

#[tokio::test]
async fn test_admin_search_as_regular_user() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listUsersAdminV2 {
                meta { ResponseCode }
            }
        }"#,
        &token,
    )
    .await;

    assert!(res["errors"].is_array());
}

#[tokio::test]
async fn test_admin_allfriends() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            allfriends {
                meta { ResponseCode }
                counter
                affectedRows { followerid followername followedid followedname }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["allfriends"];
    assert_eq!(data["meta"]["ResponseCode"], "11101");
    assert!(data["counter"].as_i64().unwrap() > 0);
    assert!(data["affectedRows"].is_array());
}

#[tokio::test]
async fn test_admin_postcomments() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let post_id = SEED_POST_1.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{
                postcomments(postid: "{post_id}") {{
                    meta {{ ResponseCode }}
                    counter
                    affectedRows {{
                        commentid
                        content
                        visibilityStatus
                        isHiddenForUsers
                        subcomments {{ commentid content }}
                    }}
                }}
            }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["postcomments"];
    assert_eq!(data["meta"]["ResponseCode"], "11101");
}

#[tokio::test]
async fn test_admin_generate_leaderboard() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            generateLeaderboard(leaderboardParams: {
                startDate: "2025-01-01"
                endDate: "2025-01-31"
                leaderboardUsersCount: 10
            }) {
                meta { ResponseCode }
                leaderboardResultLink
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["generateLeaderboard"];
    assert_eq!(data["meta"]["ResponseCode"], "12301");
    assert!(
        data["leaderboardResultLink"]
            .as_str()
            .unwrap()
            .contains(".csv")
    );
}

#[tokio::test]
async fn test_admin_generate_leaderboard_invalid_range() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            generateLeaderboard(leaderboardParams: {
                startDate: "2025-02-01"
                endDate: "2025-01-01"
                leaderboardUsersCount: 10
            }) {
                meta { ResponseCode }
            }
        }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["generateLeaderboard"]["meta"]["ResponseCode"],
        "33002"
    );
}
