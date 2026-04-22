mod common;
use common::prelude::*;

#[tokio::test]
async fn test_like_post() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let postid = SEED_POST_4.to_string();

    let res = do_post_action(&state, &token, &postid, "LIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11503");

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

    let _postid = SEED_POST_3.to_string();
    let _res = graphql_with_auth(
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

    let res = do_post_action(&state, &token, &postid, "SAVE").await;
    assert_eq!(res["data"]["resolvePostAction"]["ResponseCode"], "11512");

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
