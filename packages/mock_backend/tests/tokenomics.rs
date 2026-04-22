mod common;
use common::prelude::*;

#[tokio::test]
async fn test_tokenomics_action_prices() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { getActionPrices { meta { ResponseCode } affectedRows { postPrice likePrice dislikePrice commentPrice } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["getActionPrices"]["meta"]["ResponseCode"],
        "11304"
    );
    let prices = &res["data"]["getActionPrices"]["affectedRows"];
    assert!((prices["postPrice"].as_f64().unwrap() - 20.0).abs() < 0.01);
    assert!((prices["likePrice"].as_f64().unwrap() - 3.0).abs() < 0.01);
    assert!((prices["dislikePrice"].as_f64().unwrap() - 3.0).abs() < 0.01);
    assert!((prices["commentPrice"].as_f64().unwrap() - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_tokenomics_action_prices_no_auth() {
    let state = default_shared_state();
    let res = graphql_stateful(
        &state,
        r#"query { getActionPrices { meta { ResponseCode } } }"#,
    )
    .await;
    assert_eq!(
        res["data"]["getActionPrices"]["meta"]["ResponseCode"],
        "60501"
    );
}

#[tokio::test]
async fn test_tokenomics_get_tokenomics() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { getTokenomics { meta { ResponseCode } actionTokenPrices { postPrice likePrice } actionGemsReturns { viewGemsReturn likeGemsReturn } mintingData { tokensMintedYesterday } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["getTokenomics"]["meta"]["ResponseCode"],
        "11212"
    );
    assert!(
        res["data"]["getTokenomics"]["actionTokenPrices"]["postPrice"]
            .as_f64()
            .is_some()
    );
    assert!(
        res["data"]["getTokenomics"]["actionGemsReturns"]["viewGemsReturn"]
            .as_f64()
            .is_some()
    );
}

#[tokio::test]
async fn test_tokenomics_daily_free_status_fresh() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { getDailyFreeStatus { meta { ResponseCode } affectedRows { name used available } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["getDailyFreeStatus"]["meta"]["ResponseCode"],
        "11303"
    );
    let rows = res["data"]["getDailyFreeStatus"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 4);
    // All should have used == 0
    for row in rows {
        assert_eq!(row["used"].as_i64().unwrap(), 0);
    }
    // post=1, like=3, comment=4 are > 0; dislike=0
    let post_row = rows.iter().find(|r| r["name"] == "post").unwrap();
    assert_eq!(post_row["available"].as_i64().unwrap(), 1);
    let like_row = rows.iter().find(|r| r["name"] == "like").unwrap();
    assert_eq!(like_row["available"].as_i64().unwrap(), 3);
    let comment_row = rows.iter().find(|r| r["name"] == "comment").unwrap();
    assert_eq!(comment_row["available"].as_i64().unwrap(), 4);
    let dislike_row = rows.iter().find(|r| r["name"] == "dislike").unwrap();
    assert_eq!(dislike_row["available"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_tokenomics_daily_free_after_actions() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Perform 2 likes (SEED_POST_3 and SEED_POST_4 are alice's)
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{SEED_POST_3}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{SEED_POST_4}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { getDailyFreeStatus { meta { ResponseCode } affectedRows { name used available } } }"#,
        &token,
    )
    .await;

    let rows = res["data"]["getDailyFreeStatus"]["affectedRows"]
        .as_array()
        .unwrap();
    let like_row = rows.iter().find(|r| r["name"] == "like").unwrap();
    assert_eq!(like_row["used"].as_i64().unwrap(), 2);
    assert_eq!(like_row["available"].as_i64().unwrap(), 1); // FREE_LIKES = 3
}

#[tokio::test]
async fn test_tokenomics_daily_free_likes_no_cost() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Get initial balance
    let res0 = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
    let initial_balance = decimal_val(&res0["data"]["balance"]["currentliquidity"]);

    // Like 3 posts (all free: SEED_POST_3, 4 are alice's, we need a 3rd)
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

    let res = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
    let after_balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);

    // Balance should be unchanged (free likes)
    assert!((after_balance - initial_balance).abs() < 0.01);
}

#[tokio::test]
async fn test_tokenomics_paid_like_after_free_limit() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res0 = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
    let initial_balance = decimal_val(&res0["data"]["balance"]["currentliquidity"]);

    // Use all 3 free likes
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

    // Need a 3rd distinct post from another user. Create one via alice.
    let alice_token = login_alice(&state).await;
    let create_res = graphql_with_auth(
        &state,
        r#"mutation { createPost(action: POST, input: { title: "Test post for like", contenttype: image }) { meta { ResponseCode } affectedRows { id } } }"#,
        &alice_token,
    )
    .await;
    let new_post_id = create_res["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap();

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{new_post_id}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    // Now 4th like should cost tokens. Create another post by alice.
    let create_res2 = graphql_with_auth(
        &state,
        r#"mutation { createPost(action: POST, input: { title: "Test post for paid like", contenttype: image }) { meta { ResponseCode } affectedRows { id } } }"#,
        &alice_token,
    )
    .await;
    let new_post_id2 = create_res2["data"]["createPost"]["affectedRows"]["id"]
        .as_str()
        .unwrap();

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{new_post_id2}") {{ ResponseCode }} }}"#
        ),
        &token,
    )
    .await;

    // Check balance decreased by LIKE_PRICE (3.0)
    let res = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
    let after_balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);
    assert!((initial_balance - after_balance - 3.0).abs() < 0.5);
}

#[tokio::test]
async fn test_tokenomics_todays_interactions() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let alice_token = login_alice(&state).await;

    // Alice likes verified user's post (creates a gem for verified user)
    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolvePostAction(action: LIKE, postid: "{SEED_POST_1}") {{ ResponseCode }} }}"#
        ),
        &alice_token,
    )
    .await;

    // Now check today's interactions for verified user
    let res = graphql_with_auth(
        &state,
        r#"query { listTodaysInteractions { meta { ResponseCode } affectedRows { totalScore } } }"#,
        &token,
    )
    .await;

    let code = res["data"]["listTodaysInteractions"]["meta"]["ResponseCode"]
        .as_str()
        .unwrap();
    assert!(code == "11204" || code == "21204");
}
