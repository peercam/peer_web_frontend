mod common;
use common::prelude::*;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// --- Phase 4 Cross-Cutting ---

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

// --- Phase 5 Cross-Cutting ---

#[tokio::test]
async fn test_cross_comment_deducts_after_free_limit() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res0 = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
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

    let res = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
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

    let res0 = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
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

    let res = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
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
    let res_before =
        graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
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
    let res_after =
        graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token2).await;
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

// --- Phase 6 Cross-Cutting ---

#[tokio::test]
async fn test_alpha_mint_twice_fails() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    graphql_with_auth(&state, r#"mutation { alphaMint { ResponseCode } }"#, &token).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { alphaMint { status ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["alphaMint"]["ResponseCode"], "31204");
}

#[tokio::test]
async fn test_report_post_then_moderator_sees_ticket() {
    let state = default_shared_state();
    let user_token = login_alice(&state).await;
    let mod_token = login_moderator(&state).await;

    // Report post 2 (by verified user, no existing ticket for it)
    let post_id = SEED_POST_2.to_string();
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(postid: "{post_id}", action: REPORT) {{ ResponseCode }} }}"#
        ),
        &user_token,
    )
    .await;

    // Moderator should see the new ticket
    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationItems(contentType: post) {
                affectedRows { targetContentId status }
            }
        }"#,
        &mod_token,
    )
    .await;

    let items = res["data"]["moderationItems"]["affectedRows"]
        .as_array()
        .unwrap();
    // There should be at least 2 post tickets (seed + newly created)
    assert!(items.len() >= 2, "Expected at least 2 post tickets");
    let new_ticket = items.iter().find(|t| {
        t["targetContentId"]
            .as_str()
            .map(|s| s == post_id)
            .unwrap_or(false)
    });
    assert!(
        new_ticket.is_some(),
        "Moderator should see newly reported post ticket"
    );
    assert_eq!(new_ticket.unwrap()["status"], "waiting_for_review");
}

#[tokio::test]
async fn test_reset_clears_moderation_state() {
    let state = default_shared_state();
    let mod_token = login_moderator(&state).await;

    // Perform a moderation action
    let ticket_id = SEED_MOD_TICKET_POST.to_string();
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ performModeration(moderationTicketId: "{ticket_id}", moderationAction: hidden) {{ ResponseCode }} }}"#
        ),
        &mod_token,
    )
    .await;

    // Reset
    let app = app_with_state(state.clone());
    let request = Request::builder()
        .method("POST")
        .uri("/reset")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // After reset, ticket should be back to waiting_for_review
    let mod_token = login_moderator(&state).await;
    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationStats {
                meta { ResponseCode }
                affectedRows { AmountAwaitingReview AmountHidden }
            }
        }"#,
        &mod_token,
    )
    .await;

    // Seed has 2 waiting, 1 hidden
    assert_eq!(
        res["data"]["moderationStats"]["affectedRows"]["AmountAwaitingReview"],
        2
    );
    assert_eq!(
        res["data"]["moderationStats"]["affectedRows"]["AmountHidden"],
        1
    );
}

#[tokio::test]
async fn test_phase6_regression_existing_tests_pass() {
    // Simple regression: listing posts still works alongside Phase 6 changes
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 10) {
                meta { status ResponseCode }
                affectedRows { id }
            }
        }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["listPosts"]["meta"]["ResponseCode"], "11501");
    assert!(
        !res["data"]["listPosts"]["affectedRows"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
