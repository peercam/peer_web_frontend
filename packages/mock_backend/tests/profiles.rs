mod common;
use common::prelude::*;

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
    assert_eq!(data["affectedRows"]["amountfriends"], 2);
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
