mod common;
use common::prelude::*;

#[tokio::test]
async fn test_toggle_follow() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Default user (verified) already follows alice via seed; first toggle unfollows.
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

    // Re-follow alice
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
    assert!(!following.as_array().unwrap().is_empty());
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
