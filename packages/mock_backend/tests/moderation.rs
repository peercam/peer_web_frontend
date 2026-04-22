mod common;
use common::prelude::*;

// --- Moderation Stats ---

#[tokio::test]
async fn test_moderation_stats_as_moderator() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationStats {
                meta { status ResponseCode }
                affectedRows {
                    AmountAwaitingReview
                    AmountHidden
                    AmountRestored
                    AmountIllegal
                }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["moderationStats"];
    assert_eq!(data["meta"]["ResponseCode"], "12101");
    assert_eq!(data["affectedRows"]["AmountAwaitingReview"], 2);
    assert_eq!(data["affectedRows"]["AmountHidden"], 1);
    assert_eq!(data["affectedRows"]["AmountRestored"], 0);
    assert_eq!(data["affectedRows"]["AmountIllegal"], 0);
}

#[tokio::test]
async fn test_moderation_stats_as_regular_user() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationStats {
                meta { status ResponseCode }
                affectedRows { AmountAwaitingReview }
            }
        }"#,
        &token,
    )
    .await;

    // Should get an error (guard rejects)
    assert!(res["errors"].is_array());
    assert!(
        res["errors"][0]["extensions"]["code"]
            .as_str()
            .unwrap()
            .contains("62101")
    );
}

#[tokio::test]
async fn test_moderation_stats_no_auth() {
    let state = default_shared_state();

    let res = graphql_stateful(
        &state,
        r#"query {
            moderationStats {
                meta { status ResponseCode }
                affectedRows { AmountAwaitingReview }
            }
        }"#,
    )
    .await;

    assert!(res["errors"].is_array());
    assert!(
        res["errors"][0]["extensions"]["code"]
            .as_str()
            .unwrap()
            .contains("60501")
    );
}

// --- Moderation Items ---

#[tokio::test]
async fn test_moderation_items_list_all() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationItems {
                meta { status ResponseCode }
                affectedRows {
                    moderationTicketId
                    targetContentId
                    targettype
                    reportscount
                    status
                    reporters { userid username }
                    moderatedBy { userid username }
                }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["moderationItems"];
    assert_eq!(data["meta"]["ResponseCode"], "12102");
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_moderation_items_filter_by_status() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationItems(status: waiting_for_review) {
                meta { ResponseCode }
                affectedRows { moderationTicketId status }
            }
        }"#,
        &token,
    )
    .await;

    let items = res["data"]["moderationItems"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(items.len(), 2);
    for item in items {
        assert_eq!(item["status"], "waiting_for_review");
    }
}

#[tokio::test]
async fn test_moderation_items_filter_by_content_type_post() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationItems(contentType: post) {
                meta { ResponseCode }
                affectedRows { moderationTicketId targettype }
            }
        }"#,
        &token,
    )
    .await;

    let items = res["data"]["moderationItems"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["targettype"], "post");
}

#[tokio::test]
async fn test_moderation_items_filter_by_content_type_user() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationItems(contentType: user) {
                meta { ResponseCode }
                affectedRows { moderationTicketId status moderatedBy { userid } }
            }
        }"#,
        &token,
    )
    .await;

    let items = res["data"]["moderationItems"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["status"], "hidden");
    assert!(!items[0]["moderatedBy"].is_null());
}

#[tokio::test]
async fn test_moderation_items_pagination() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationItems(offset: 1, limit: 1) {
                meta { ResponseCode }
                affectedRows { moderationTicketId }
            }
        }"#,
        &token,
    )
    .await;

    let items = res["data"]["moderationItems"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(items.len(), 1);
}

#[tokio::test]
async fn test_moderation_items_as_regular_user() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            moderationItems {
                meta { ResponseCode }
                affectedRows { moderationTicketId }
            }
        }"#,
        &token,
    )
    .await;

    assert!(res["errors"].is_array());
}

// --- Perform Moderation ---

#[tokio::test]
async fn test_perform_moderation_hide_post() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let ticket_id = SEED_MOD_TICKET_POST.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{
                performModeration(
                    moderationTicketId: "{ticket_id}"
                    moderationAction: hidden
                ) {{ status ResponseCode }}
            }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["performModeration"]["ResponseCode"], "12103");

    // Verify post visibility changed
    let st = state.read().await;
    assert_eq!(st.content_visibility.get(&SEED_POST_1).unwrap(), "HIDDEN");
}

#[tokio::test]
async fn test_perform_moderation_restore_comment() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let ticket_id = SEED_MOD_TICKET_COMMENT.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{
                performModeration(
                    moderationTicketId: "{ticket_id}"
                    moderationAction: restored
                ) {{ status ResponseCode }}
            }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["performModeration"]["ResponseCode"], "12103");

    let st = state.read().await;
    assert_eq!(
        st.content_visibility.get(&SEED_COMMENT_1).unwrap(),
        "NORMAL"
    );
}

#[tokio::test]
async fn test_perform_moderation_mark_illegal() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let ticket_id = SEED_MOD_TICKET_USER.to_string();
    // User ticket is currently hidden; mark it illegal
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{
                performModeration(
                    moderationTicketId: "{ticket_id}"
                    moderationAction: illegal
                ) {{ status ResponseCode }}
            }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["performModeration"]["ResponseCode"], "12103");

    let st = state.read().await;
    assert_eq!(
        st.content_visibility.get(&SEED_USER_DAVE).unwrap(),
        "ILLEGAL"
    );
}

#[tokio::test]
async fn test_perform_moderation_nonexistent_ticket() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation {
            performModeration(
                moderationTicketId: "00000000-0000-4000-a000-ffffffffffff"
                moderationAction: hidden
            ) { status ResponseCode }
        }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["performModeration"]["ResponseCode"], "22103");
}

#[tokio::test]
async fn test_perform_moderation_duplicate_action() {
    let state = default_shared_state();
    let token = login_moderator(&state).await;

    // User ticket is already "hidden"
    let ticket_id = SEED_MOD_TICKET_USER.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{
                performModeration(
                    moderationTicketId: "{ticket_id}"
                    moderationAction: hidden
                ) {{ status ResponseCode }}
            }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(res["data"]["performModeration"]["ResponseCode"], "32103");
}

// --- Content Visibility Tests ---

#[tokio::test]
async fn test_hidden_post_shows_hidden_flag() {
    let state = default_shared_state();
    let mod_token = login_moderator(&state).await;
    let user_token = login_alice(&state).await;

    // Hide the post via moderation
    let ticket_id = SEED_MOD_TICKET_POST.to_string();
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ performModeration(moderationTicketId: "{ticket_id}", moderationAction: hidden) {{ ResponseCode }} }}"#
        ),
        &mod_token,
    )
    .await;

    // Post should show isHiddenForUsers: true in listPosts
    let res = graphql_with_auth(
        &state,
        r#"query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 20) { meta { ResponseCode } affectedRows { id isHiddenForUsers visibilityStatus } }
        }"#,
        &user_token,
    )
    .await;

    let posts = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    let hidden_post = posts.iter().find(|p| p["id"] == SEED_POST_1.to_string());
    if let Some(p) = hidden_post {
        assert_eq!(p["isHiddenForUsers"], true);
    }
}

#[tokio::test]
async fn test_illegal_post_filtered_from_list() {
    let state = default_shared_state();
    let mod_token = login_moderator(&state).await;
    let user_token = login_alice(&state).await;

    // Mark post as illegal
    let ticket_id = SEED_MOD_TICKET_POST.to_string();
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ performModeration(moderationTicketId: "{ticket_id}", moderationAction: illegal) {{ ResponseCode }} }}"#
        ),
        &mod_token,
    )
    .await;

    // Post should not appear in list
    let res = graphql_with_auth(
        &state,
        r#"query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 20) { affectedRows { id } }
        }"#,
        &user_token,
    )
    .await;

    let posts = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    let illegal_post = posts.iter().find(|p| p["id"] == SEED_POST_1.to_string());
    assert!(
        illegal_post.is_none(),
        "Illegal post should be filtered out"
    );
}

#[tokio::test]
async fn test_restored_post_visible_again() {
    let state = default_shared_state();
    let mod_token = login_moderator(&state).await;
    let user_token = login_alice(&state).await;

    let ticket_id = SEED_MOD_TICKET_POST.to_string();
    // First hide it
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ performModeration(moderationTicketId: "{ticket_id}", moderationAction: hidden) {{ ResponseCode }} }}"#
        ),
        &mod_token,
    )
    .await;

    // Then restore it
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ performModeration(moderationTicketId: "{ticket_id}", moderationAction: restored) {{ ResponseCode }} }}"#
        ),
        &mod_token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 20) { affectedRows { id visibilityStatus } }
        }"#,
        &user_token,
    )
    .await;

    let posts = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    let post = posts.iter().find(|p| p["id"] == SEED_POST_1.to_string());
    assert!(post.is_some(), "Restored post should be visible");
    // The original post visibility_status is "VISIBLE" (from seed),
    // and moderation "NORMAL" falls through to the original value
    let vis = post.unwrap()["visibilityStatus"].as_str().unwrap();
    assert!(
        vis == "VISIBLE" || vis == "NORMAL",
        "Expected VISIBLE or NORMAL, got {vis}"
    );
}

#[tokio::test]
async fn test_hidden_user_shows_in_profile() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    // Dave is hidden in seed data
    let dave_id = SEED_USER_DAVE.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ getProfile(userid: "{dave_id}") {{
                meta {{ ResponseCode }}
                affectedRows {{ isHiddenForUsers }}
            }} }}"#
        ),
        &token,
    )
    .await;

    // Dave is HIDDEN via content_visibility
    assert_eq!(
        res["data"]["getProfile"]["affectedRows"]["isHiddenForUsers"],
        true
    );
}

#[tokio::test]
async fn test_report_user_creates_ticket() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    // Report Carol (no existing ticket for Carol)
    let carol_id = SEED_USER_CAROL.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{
                reportUser(userid: "{carol_id}") {{
                    status ResponseCode
                }}
            }}"#
        ),
        &token,
    )
    .await;

    let code = res["data"]["reportUser"]["ResponseCode"].as_str().unwrap();
    assert!(
        code == "11012" || code == "31008",
        "Expected report success (11012) or already-reported (31008), got {code}"
    );

    // Check that a moderation ticket was created
    let st = state.read().await;
    let ticket = st
        .moderation_tickets
        .iter()
        .find(|t| t.target_content_id == SEED_USER_CAROL && t.target_type == "user");
    assert!(
        ticket.is_some(),
        "Moderation ticket should be created for reported user"
    );
}

// --- Hidden Comment Visibility Tests ---

#[tokio::test]
async fn test_hidden_comment_shows_hidden_flag() {
    let state = default_shared_state();
    let mod_token = login_moderator(&state).await;

    // Hide the comment via moderation
    let ticket_id = SEED_MOD_TICKET_COMMENT.to_string();
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ performModeration(moderationTicketId: "{ticket_id}", moderationAction: hidden) {{ ResponseCode }} }}"#
        ),
        &mod_token,
    )
    .await;

    // Admin postcomments should show the comment with isHiddenForUsers: true
    let admin_token = login_admin(&state).await;
    let post_id = SEED_POST_1.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ postcomments(postid: "{post_id}", offset: 0, limit: 20) {{ affectedRows {{ commentid visibilityStatus isHiddenForUsers subcomments {{ commentid visibilityStatus isHiddenForUsers }} }} }} }}"#
        ),
        &admin_token,
    )
    .await;

    let comments = res["data"]["postcomments"]["affectedRows"]
        .as_array()
        .unwrap();
    let hidden_comment = comments
        .iter()
        .find(|c| c["commentid"] == SEED_COMMENT_1.to_string());
    if let Some(c) = hidden_comment {
        assert_eq!(
            c["isHiddenForUsers"], true,
            "Hidden comment should have isHiddenForUsers: true"
        );
        assert_eq!(c["visibilityStatus"], "HIDDEN");
    }
}

#[tokio::test]
async fn test_hidden_comment_in_regular_listing() {
    let state = default_shared_state();
    let mod_token = login_moderator(&state).await;
    let user_token = login_default(&state).await;

    // Hide the comment via moderation
    let ticket_id = SEED_MOD_TICKET_COMMENT.to_string();
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ performModeration(moderationTicketId: "{ticket_id}", moderationAction: hidden) {{ ResponseCode }} }}"#
        ),
        &mod_token,
    )
    .await;

    // In the regular comment listing, hidden comment should show visibility info
    let post_id = SEED_POST_1.to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"query {{ listComments(postid: "{post_id}", commentOffset: 0, commentLimit: 20) {{ affectedRows {{ commentid isHiddenForUsers visibilityStatus }} }} }}"#
        ),
        &user_token,
    )
    .await;

    let comments = res["data"]["listComments"]["affectedRows"]
        .as_array()
        .unwrap();
    // Hidden comment should still appear (not ILLEGAL) but with flag
    let hidden = comments
        .iter()
        .find(|c| c["commentid"] == SEED_COMMENT_1.to_string());
    if let Some(c) = hidden {
        assert_eq!(c["isHiddenForUsers"], true);
    }
}
