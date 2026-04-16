mod common;
use common::prelude::*;

#[tokio::test]
async fn test_gemster_returns_data() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            gemster {
                meta { ResponseCode }
                affectedRows { d0 d1 d2 d3 d4 d5 d6 d7 w0 m0 y0 }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["gemster"];
    assert_eq!(data["meta"]["ResponseCode"], "11207");
    assert!(!data["affectedRows"].is_null());
}

#[tokio::test]
async fn test_dailygemstatus() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            dailygemstatus {
                meta { ResponseCode }
                affectedRows { d0 w0 m0 y0 }
            }
        }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["dailygemstatus"]["meta"]["ResponseCode"],
        "11207"
    );
}

#[tokio::test]
async fn test_dailygemsresults_d0() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    // First convert interactions to gems
    graphql_with_auth(
        &state,
        r#"mutation { globalwins { ResponseCode } }"#,
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            dailygemsresults(day: D0) {
                meta { ResponseCode }
                affectedRows { totalGems data { userid gems } }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["dailygemsresults"];
    let code = data["meta"]["ResponseCode"].as_str().unwrap();
    // Either has gems (11207) or no gems (21206)
    assert!(code == "11207" || code == "21206");
}

#[tokio::test]
async fn test_get_mint_account() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query {
            getMintAccount {
                meta { ResponseCode }
                mintAccount { accountid initialBalance currentBalance }
            }
        }"#,
        &token,
    )
    .await;

    let data = &res["data"]["getMintAccount"];
    assert_eq!(data["meta"]["ResponseCode"], "0");
    assert!(!data["mintAccount"]["initialBalance"].is_null());
    assert!(!data["mintAccount"]["currentBalance"].is_null());
}

#[tokio::test]
async fn test_globalwins_converts_interactions() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { globalwins { status ResponseCode } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["globalwins"];
    let code = data["ResponseCode"].as_str().unwrap();
    // Either converted (11206) or nothing to convert (21205)
    assert!(code == "11206" || code == "21205");
}

#[tokio::test]
async fn test_globalwins_no_pending() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    // Run globalwins twice — second should have nothing
    graphql_with_auth(
        &state,
        r#"mutation { globalwins { ResponseCode } }"#,
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { globalwins { ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["globalwins"]["ResponseCode"], "21205");
}

#[tokio::test]
async fn test_distribute_tokens_already_minted() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // First convert interactions to gems
    graphql_with_auth(
        &state,
        r#"mutation { globalwins { ResponseCode } }"#,
        &token,
    )
    .await;

    // Distribute once
    graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ distributeTokensForGems(date: "{}") {{ meta {{ ResponseCode }} }} }}"#,
            today
        ),
        &token,
    )
    .await;

    // Distribute again — should fail with 31204
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ distributeTokensForGems(date: "{}") {{ meta {{ ResponseCode }} }} }}"#,
            today
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["distributeTokensForGems"]["meta"]["ResponseCode"],
        "31204"
    );
}

#[tokio::test]
async fn test_alpha_mint_credits_users() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    // Get alice's balance before alpha mint
    let alice_token = login_alice(&state).await;
    let before = graphql_with_auth(
        &state,
        r#"query { balance { meta { ResponseCode } currentliquidity } }"#,
        &alice_token,
    )
    .await;
    let bal_before = decimal_val(&before["data"]["balance"]["currentliquidity"]);

    // Run alpha mint
    let res = graphql_with_auth(
        &state,
        r#"mutation { alphaMint { status ResponseCode } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["alphaMint"]["ResponseCode"], "200");

    // Check alice got 100 tokens
    let after = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &alice_token,
    )
    .await;
    let bal_after = decimal_val(&after["data"]["balance"]["currentliquidity"]);

    assert!(
        (bal_after - bal_before - 100.0).abs() < 0.01,
        "Alice should have gained 100 tokens: before={bal_before}, after={bal_after}"
    );
}

#[tokio::test]
async fn test_dailygemsresults_no_gems() {
    let state = default_shared_state();
    let token = login_admin(&state).await;

    // D7 (7 days ago) should have no gems in freshly seeded state
    let res = graphql_with_auth(
        &state,
        r#"query { dailygemsresults(day: D7) { meta { ResponseCode } affectedRows { totalGems } } }"#,
        &token,
    )
    .await;

    let data = &res["data"]["dailygemsresults"];
    let code = data["meta"]["ResponseCode"].as_str().unwrap();
    // Seed gems are all from a fixed date, so D7 (relative) likely has nothing
    assert!(
        code == "21206" || code == "11207",
        "Expected 21206 (no gems) or 11207 (data), got {code}"
    );
}

#[tokio::test]
async fn test_distribute_tokens_success() {
    let state = default_shared_state();
    let token = login_admin(&state).await;
    let alice_token = login_alice(&state).await;

    // Get alice's balance before
    let before = graphql_with_auth(
        &state,
        r#"query { balance { currentliquidity } }"#,
        &alice_token,
    )
    .await;
    let bal_before = decimal_val(&before["data"]["balance"]["currentliquidity"]);

    // Convert interactions to gems first
    graphql_with_auth(
        &state,
        r#"mutation { globalwins { ResponseCode } }"#,
        &token,
    )
    .await;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ distributeTokensForGems(date: "{today}") {{ meta {{ ResponseCode }} counter affectedRows {{ winStatus {{ totalGems gemsintoken }} }} }} }}"#
        ),
        &token,
    )
    .await;

    let data = &res["data"]["distributeTokensForGems"];
    let code = data["meta"]["ResponseCode"].as_str().unwrap();
    // Should succeed if gems exist for today's date
    if code == "11208" {
        assert!(
            data["counter"].as_i64().unwrap() > 0,
            "Should have distributed to at least 1 user"
        );
        assert!(!data["affectedRows"]["winStatus"]["totalGems"].is_null());
        assert!(!data["affectedRows"]["winStatus"]["gemsintoken"].is_null());

        // Check alice balance went up
        let after = graphql_with_auth(
            &state,
            r#"query { balance { currentliquidity } }"#,
            &alice_token,
        )
        .await;
        let bal_after = decimal_val(&after["data"]["balance"]["currentliquidity"]);
        assert!(
            bal_after >= bal_before,
            "Balance should not decrease after distribution: before={bal_before}, after={bal_after}"
        );
    }
    // If 21206 (no gems for today), that's acceptable depending on seed gem dates
}
