mod common;
use common::prelude::*;

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

    let posts = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    let has_advertised = posts
        .iter()
        .any(|p| p["id"].as_str().unwrap() == SEED_POST_1.to_string());
    assert!(
        !has_advertised,
        "Advertised post should be excluded from listPosts"
    );
}
