mod common;
use common::prelude::*;

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
        r#"query { listComments(postid: "not-a-uuid") { meta { ResponseCode } counter } }"#,
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
    assert!(c1["isliked"].as_bool().unwrap());
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
async fn test_like_comment_toggles_off() {
    // Production peergamma's `likeComment` is a toggle — calling it twice
    // removes the like rather than returning an "already liked" error.
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Like once
    graphql_with_auth(
        &state,
        &format!(r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#),
        &token,
    )
    .await;

    // Like again — toggles off
    let res = graphql_with_auth(
        &state,
        &format!(r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode ResponseMessage }} }}"#),
        &token,
    )
    .await;

    let data = &res["data"]["likeComment"];
    assert_eq!(data["ResponseCode"].as_str().unwrap(), "11603");
    assert_eq!(data["ResponseMessage"].as_str().unwrap(), "Comment unliked");
}

#[tokio::test]
async fn test_unlike_comment() {
    // The frontend's `UNLIKE_COMMENT_MUTATION` aliases `likeComment` — there
    // is no `unlikeComment` resolver on production peergamma. Verify that
    // toggling via `likeComment` is sufficient to revoke a like.
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Like first
    graphql_with_auth(
        &state,
        &format!(r#"mutation {{ likeComment(commentid: "{SEED_COMMENT_1}") {{ ResponseCode }} }}"#),
        &token,
    )
    .await;

    // Unlike via the same toggle mutation
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
    assert_eq!(data["ResponseMessage"].as_str().unwrap(), "Comment unliked");
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
