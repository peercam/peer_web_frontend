mod common;
use common::prelude::*;

#[tokio::test]
async fn test_wallet_balance_seeded_user() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { balance { meta { ResponseCode } currentliquidity } }"#,
        &token,
    )
    .await;

    assert_eq!(res["data"]["balance"]["meta"]["ResponseCode"], "11204");
    let balance = decimal_val(&res["data"]["balance"]["currentliquidity"]);
    assert!((balance - 1000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_wallet_balance_no_auth() {
    let state = default_shared_state();
    let res = graphql_stateful(
        &state,
        r#"query { balance { meta { ResponseCode } currentliquidity } }"#,
    )
    .await;
    assert_eq!(res["data"]["balance"]["meta"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_wallet_transfer_tokens() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 50.0) {{ meta {{ ResponseCode }} affectedRows {{ tokenSendFormatted tokensSubstractedFromWalletFormatted }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "11211"
    );
    let rows = &res["data"]["resolveTransferV2"]["affectedRows"];
    assert!(rows["tokenSendFormatted"].as_str().is_some());
    assert!(
        rows["tokensSubstractedFromWalletFormatted"]
            .as_str()
            .is_some()
    );

    // Verify balances changed
    let res2 = graphql_with_auth(&state, r#"query { balance { currentliquidity } }"#, &token).await;
    let sender_bal = decimal_val(&res2["data"]["balance"]["currentliquidity"]);
    // 1000 - 50 - fees (4% of 50 = 2.0) = 948.0
    assert!(sender_bal < 950.0);
}

#[tokio::test]
async fn test_wallet_transfer_fee_breakdown() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 50.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    // Check transaction history for fee details
    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(type: TRANSACTION, limit: 5) { meta { ResponseCode } affectedRows { fees { burn peer inviter } } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    let rows = &res["data"]["transactionHistory"]["affectedRows"];
    assert!(rows.is_array());
    // Find the new transfer (first one due to sort by newest)
    let tx = &rows[0];
    let fees = &tx["fees"];
    assert!((decimal_val(&fees["burn"]) - 0.5).abs() < 0.01);
    assert!((decimal_val(&fees["peer"]) - 1.0).abs() < 0.01);
    assert!((decimal_val(&fees["inviter"]) - 0.5).abs() < 0.01);
}

#[tokio::test]
async fn test_wallet_transfer_formatted_values() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 50.0) {{ meta {{ ResponseCode }} affectedRows {{ tokenSendFormatted tokensSubstractedFromWalletFormatted }} }} }}"#
        ),
        &token,
    )
    .await;

    let rows = &res["data"]["resolveTransferV2"]["affectedRows"];
    let send = rows["tokenSendFormatted"].as_str().unwrap();
    let subtracted = rows["tokensSubstractedFromWalletFormatted"]
        .as_str()
        .unwrap();
    assert!(send.contains("50"));
    assert!(subtracted.contains("52"));
}

#[tokio::test]
async fn test_wallet_transfer_appears_in_history() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let _ = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 25.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(type: TRANSACTION, limit: 10) { meta { ResponseCode } affectedRows { tokenamount sender { userid } recipient { userid } } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    let rows = res["data"]["transactionHistory"]["affectedRows"]
        .as_array()
        .unwrap();
    assert!(!rows.is_empty());
}

#[tokio::test]
async fn test_wallet_transfer_to_self() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_VERIFIED}", numberoftokens: 10.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "31202"
    );
}

#[tokio::test]
async fn test_wallet_transfer_to_nonexistent_user() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"mutation { resolveTransferV2(recipient: "99999999-9999-4999-a999-999999999999", numberoftokens: 10.0) { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "31007"
    );
}

#[tokio::test]
async fn test_wallet_transfer_to_system_account() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // SYSTEM_PEER_ACCOUNT = "eeeeeeee-eeee-4eee-aeee-000000000002"
    let res = graphql_with_auth(
        &state,
        r#"mutation { resolveTransferV2(recipient: "eeeeeeee-eeee-4eee-aeee-000000000002", numberoftokens: 10.0) { meta { ResponseCode } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "31203"
    );
}

#[tokio::test]
async fn test_wallet_transfer_insufficient_balance() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 99999.0) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "51301"
    );
}

#[tokio::test]
async fn test_wallet_transfer_below_minimum() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 0.0000001) {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "30264"
    );
}

#[tokio::test]
async fn test_wallet_transfer_long_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;
    let long_msg = "x".repeat(501);

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 10.0, message: "{long_msg}") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "30270"
    );
}

#[tokio::test]
async fn test_wallet_transfer_url_in_message() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        &format!(
            r#"mutation {{ resolveTransferV2(recipient: "{SEED_USER_ALICE}", numberoftokens: 10.0, message: "Check https://evil.com") {{ meta {{ ResponseCode }} }} }}"#
        ),
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["resolveTransferV2"]["meta"]["ResponseCode"],
        "30271"
    );
}

#[tokio::test]
async fn test_wallet_transaction_history_pagination() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(limit: 1, offset: 0) { meta { ResponseCode } affectedRows { tokenamount } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    let rows = res["data"]["transactionHistory"]["affectedRows"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn test_wallet_transaction_history_no_transactions() {
    let state = default_shared_state();
    // Carol has no transactions in seed data
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory { meta { ResponseCode } affectedRows { tokenamount } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "21209"
    );
}

#[tokio::test]
async fn test_wallet_transaction_history_filter_by_type() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(
        &state,
        r#"query { transactionHistory(type: PAYMENT) { meta { ResponseCode } affectedRows { transactionCategory } } }"#,
        &token,
    )
    .await;

    assert_eq!(
        res["data"]["transactionHistory"]["meta"]["ResponseCode"],
        "11215"
    );
    // The seed has a Like payment for the verified user
    let rows = res["data"]["transactionHistory"]["affectedRows"]
        .as_array()
        .unwrap();
    assert!(!rows.is_empty());
}
