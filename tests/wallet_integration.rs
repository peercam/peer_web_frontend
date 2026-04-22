//! Mock-API coverage for the wallet feature.
//!
//! Exercises the same GraphQL surface that the wallet UI drives in the
//! browser — `balance`, `transactionHistory`, `resolveTransferV2`, and
//! `listFriends` (used by the transfer modal's recipient picker) — by
//! calling `peer_web::api::graphql::query`/`mutate` against the
//! in-process `mock_backend`.
//!
//! These tests assert the wire contract: status fields, response codes,
//! balance arithmetic after transfers, and the basic guard rails the
//! mock backend mirrors from the real GraphQL service (auth required,
//! self-transfer rejected, system-account transfer rejected, message
//! validation, insufficient-balance handling).
//!
//! Run with: `cargo test --features ssr --test wallet_integration`

#![cfg(feature = "ssr")]

mod common;

use peer_web::api::graphql::{
    BALANCE_QUERY, LIST_FRIENDS_QUERY, LOGIN_MUTATION, TRANSACTION_HISTORY_QUERY,
    TRANSFER_MUTATION, mutate, query,
};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{Mutex, MutexGuard, RwLockReadGuard};

// Seeded credentials — see `tests/mock_backend/src/seed.rs::credentials`.
const VERIFIED_EMAIL: &str = "test@peer.com";
const VERIFIED_PASSWORD: &str = "TestPass123";
const ALICE_EMAIL: &str = "alice@peer.com";
const ALICE_PASSWORD: &str = "AlicePass123";
const BOB_EMAIL: &str = "bob@peer.com";
const BOB_PASSWORD: &str = "BobPass123";

// Seeded user ids — see `tests/mock_backend/src/seed.rs::SEED_USER_*`.
const SEED_USER_VERIFIED: &str = "00000000-0000-4000-a000-000000000001";
const SEED_USER_ALICE: &str = "00000000-0000-4000-a000-000000000003";
const SEED_USER_BOB: &str = "00000000-0000-4000-a000-000000000004";

// System accounts the mock backend rejects transfers to.
const SYSTEM_BURN_ACCOUNT: &str = "eeeeeeee-eeee-4eee-aeee-000000000001";

// Default seeded balance for non-mint users (see `DEFAULT_USER_BALANCE`).
const DEFAULT_USER_BALANCE: &str = "1000.0";

/// Serializes wallet tests that mutate shared balances/transactions on
/// the in-process mock backend. Without this, two concurrent transfers
/// from the same seeded user can race the balance check.
static WALLET_LOCK: Mutex<()> = Mutex::const_new(());

async fn setup() -> RwLockReadGuard<'static, ()> {
    let _ = common::mock_graphql_endpoint().await;
    common::endpoint_read_guard().await
}

async fn setup_exclusive() -> (RwLockReadGuard<'static, ()>, MutexGuard<'static, ()>) {
    let _ = common::mock_graphql_endpoint().await;
    let env = common::endpoint_read_guard().await;
    let wallet = WALLET_LOCK.lock().await;
    (env, wallet)
}

#[derive(Serialize)]
struct LoginVars {
    email: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionHistoryVars {
    offset: i32,
    limit: i32,
}

#[derive(Serialize)]
struct TransferVars {
    recipient: String,
    numberoftokens: Decimal,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListFriendsVars {
    #[serde(skip_serializing_if = "Option::is_none")]
    userid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_by: Option<String>,
    offset: i32,
    limit: i32,
}

/// Helper: log in via the production GraphQL client and return the
/// access token. Mirrors the helper used in `chat_integration.rs`.
async fn login(email: &str, password: &str) -> String {
    let vars = LoginVars {
        email: email.into(),
        password: password.into(),
    };
    let data: Value = mutate(LOGIN_MUTATION, vars, None)
        .await
        .expect("login mutation should succeed");
    let login = &data["login"];
    assert_eq!(
        login["status"].as_str(),
        Some("success"),
        "expected login success, got status={:?} code={:?}",
        login["status"],
        login["ResponseCode"],
    );
    login["accessToken"]
        .as_str()
        .expect("login should return an access token")
        .to_string()
}

fn meta_status(node: &Value) -> Option<&str> {
    node["meta"]["status"].as_str()
}

fn meta_code(node: &Value) -> Option<&str> {
    node["meta"]["ResponseCode"].as_str()
}

/// Pull the current balance for the authenticated viewer as a `Decimal`.
async fn fetch_balance(token: &str) -> Decimal {
    let data: Value = query(BALANCE_QUERY, (), Some(token))
        .await
        .expect("balance query should succeed");
    let bal = &data["balance"];
    assert_eq!(meta_status(bal), Some("success"));
    bal["currentliquidity"]
        .as_str()
        .expect("currentliquidity is serialized as a Decimal string")
        .parse()
        .expect("currentliquidity parses as Decimal")
}

// ── balance ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn balance_returns_seeded_default_for_authenticated_user() {
    let _env = setup().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let bal = fetch_balance(&token).await;

    let expected: Decimal = DEFAULT_USER_BALANCE.parse().unwrap();
    assert!(
        bal >= expected,
        "expected at least the seeded default balance ({expected}), got {bal}"
    );
}

#[tokio::test]
async fn balance_without_auth_returns_error_meta() {
    let _env = setup().await;

    let data: Value = query(BALANCE_QUERY, (), None)
        .await
        .expect("balance query itself should not error at the transport layer");
    let bal = &data["balance"];
    assert_eq!(meta_status(bal), Some("error"));
    assert_eq!(meta_code(bal), Some("60501"));
    assert!(bal["currentliquidity"].is_null());
}

// ── transactionHistory ───────────────────────────────────────────────────

#[tokio::test]
async fn transaction_history_returns_success_envelope_for_authenticated_user() {
    let _env = setup().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = query(
        TRANSACTION_HISTORY_QUERY,
        TransactionHistoryVars {
            offset: 0,
            limit: 20,
        },
        Some(&token),
    )
    .await
    .expect("transactionHistory query should succeed");

    let history = &data["transactionHistory"];
    assert_eq!(meta_status(history), Some("success"));
    // affectedRows is either an array or null on the wire; both are
    // valid for an empty history per the schema.
    let rows = &history["affectedRows"];
    assert!(
        rows.is_array() || rows.is_null(),
        "affectedRows should be an array or null, got: {rows}"
    );
}

#[tokio::test]
async fn transaction_history_without_auth_returns_error_meta() {
    let _env = setup().await;

    let data: Value = query(
        TRANSACTION_HISTORY_QUERY,
        TransactionHistoryVars {
            offset: 0,
            limit: 20,
        },
        None,
    )
    .await
    .expect("query should not error at the transport layer");

    let history = &data["transactionHistory"];
    assert_eq!(meta_status(history), Some("error"));
    assert_eq!(meta_code(history), Some("60501"));
}

// ── resolveTransferV2 (happy path) ───────────────────────────────────────

#[tokio::test]
async fn transfer_succeeds_and_updates_both_balances_and_history() {
    let _guards = setup_exclusive().await;
    let sender_token = login(BOB_EMAIL, BOB_PASSWORD).await;
    let recipient_token = login(ALICE_EMAIL, ALICE_PASSWORD).await;

    let sender_before = fetch_balance(&sender_token).await;
    let recipient_before = fetch_balance(&recipient_token).await;

    let amount: Decimal = "1.5".parse().unwrap();

    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SEED_USER_ALICE.into(),
            numberoftokens: amount,
            message: "thanks!".into(),
        },
        Some(&sender_token),
    )
    .await
    .expect("transfer mutation should succeed");

    let resp = &data["resolveTransferV2"];
    assert_eq!(
        meta_status(resp),
        Some("success"),
        "expected transfer success, got: {resp}"
    );

    // Recipient credited the gross amount.
    let recipient_after = fetch_balance(&recipient_token).await;
    assert_eq!(
        recipient_after - recipient_before,
        amount,
        "recipient should be credited exactly the transfer amount"
    );

    // Sender debited amount + fees (total 4% per the mock backend's
    // BURN(1%) + PEER(2%) + INVITER(1%) split).
    let sender_after = fetch_balance(&sender_token).await;
    let total_with_fees: Decimal = "1.56".parse().unwrap(); // 1.5 * 1.04
    assert_eq!(
        sender_before - sender_after,
        total_with_fees,
        "sender should be debited amount + 4% fees"
    );

    // The new transaction shows up at the head of the recipient's
    // history (mock backend sorts newest first).
    let hist: Value = query(
        TRANSACTION_HISTORY_QUERY,
        TransactionHistoryVars {
            offset: 0,
            limit: 20,
        },
        Some(&recipient_token),
    )
    .await
    .expect("transactionHistory query should succeed");
    let rows = hist["transactionHistory"]["affectedRows"]
        .as_array()
        .expect("affectedRows array");
    assert!(
        rows.iter().any(|tx| {
            tx["transactionCategory"].as_str() == Some("P2P_TRANSFER")
                && tx["sender"]["userid"].as_str() == Some(SEED_USER_BOB)
                && tx["recipient"]["userid"].as_str() == Some(SEED_USER_ALICE)
                && tx["message"].as_str() == Some("thanks!")
        }),
        "expected the new transfer in the recipient's transaction history, got: {rows:#?}"
    );
}

// ── resolveTransferV2 (validation / guards) ──────────────────────────────

#[tokio::test]
async fn transfer_to_self_is_rejected() {
    let _guards = setup_exclusive().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SEED_USER_VERIFIED.into(),
            numberoftokens: "1".parse().unwrap(),
            message: String::new(),
        },
        Some(&token),
    )
    .await
    .expect("transfer mutation should round-trip");

    let resp = &data["resolveTransferV2"];
    assert_eq!(meta_status(resp), Some("error"));
    assert_eq!(meta_code(resp), Some("31202"));
    assert!(resp["affectedRows"].is_null());
}

#[tokio::test]
async fn transfer_to_system_account_is_rejected() {
    let _guards = setup_exclusive().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SYSTEM_BURN_ACCOUNT.into(),
            numberoftokens: "1".parse().unwrap(),
            message: String::new(),
        },
        Some(&token),
    )
    .await
    .expect("transfer mutation should round-trip");

    let resp = &data["resolveTransferV2"];
    assert_eq!(meta_status(resp), Some("error"));
    assert_eq!(meta_code(resp), Some("31203"));
}

#[tokio::test]
async fn transfer_with_url_in_message_is_rejected() {
    let _guards = setup_exclusive().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SEED_USER_ALICE.into(),
            numberoftokens: "1".parse().unwrap(),
            message: "check https://example.com out".into(),
        },
        Some(&token),
    )
    .await
    .expect("transfer mutation should round-trip");

    let resp = &data["resolveTransferV2"];
    assert_eq!(meta_status(resp), Some("error"));
    assert_eq!(meta_code(resp), Some("30271"));
}

#[tokio::test]
async fn transfer_with_oversize_message_is_rejected() {
    let _guards = setup_exclusive().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SEED_USER_ALICE.into(),
            numberoftokens: "1".parse().unwrap(),
            message: "x".repeat(501),
        },
        Some(&token),
    )
    .await
    .expect("transfer mutation should round-trip");

    let resp = &data["resolveTransferV2"];
    assert_eq!(meta_status(resp), Some("error"));
    assert_eq!(meta_code(resp), Some("30270"));
}

#[tokio::test]
async fn transfer_below_minimum_amount_is_rejected() {
    let _guards = setup_exclusive().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SEED_USER_ALICE.into(),
            numberoftokens: "0.0000001".parse().unwrap(), // < 0.000001
            message: String::new(),
        },
        Some(&token),
    )
    .await
    .expect("transfer mutation should round-trip");

    let resp = &data["resolveTransferV2"];
    assert_eq!(meta_status(resp), Some("error"));
    assert_eq!(meta_code(resp), Some("30264"));
}

#[tokio::test]
async fn transfer_with_insufficient_balance_is_rejected() {
    let _guards = setup_exclusive().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    // Seeded balance is 1000.0; this is far above the spendable amount
    // even after accounting for fees.
    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SEED_USER_ALICE.into(),
            numberoftokens: "999999999".parse().unwrap(),
            message: String::new(),
        },
        Some(&token),
    )
    .await
    .expect("transfer mutation should round-trip");

    let resp = &data["resolveTransferV2"];
    assert_eq!(meta_status(resp), Some("error"));
    assert_eq!(meta_code(resp), Some("51301"));
}

#[tokio::test]
async fn transfer_without_auth_is_rejected() {
    let _env = setup().await;

    let data: Value = mutate(
        TRANSFER_MUTATION,
        TransferVars {
            recipient: SEED_USER_ALICE.into(),
            numberoftokens: "1".parse().unwrap(),
            message: String::new(),
        },
        None,
    )
    .await
    .expect("transfer mutation should round-trip at the transport layer");

    let resp = &data["resolveTransferV2"];
    assert_eq!(meta_status(resp), Some("error"));
    assert_eq!(meta_code(resp), Some("60501"));
}

// ── listFriends (transfer-modal recipient picker) ────────────────────────

#[tokio::test]
async fn list_friends_returns_success_envelope_for_authenticated_user() {
    let _env = setup().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = query(
        LIST_FRIENDS_QUERY,
        ListFriendsVars {
            userid: None,
            content_filter_by: None,
            offset: 0,
            limit: 100,
        },
        Some(&token),
    )
    .await
    .expect("listFriends query should succeed");

    let friends = &data["listFriends"];
    assert_eq!(meta_status(friends), Some("success"));
    let rows = &friends["affectedRows"];
    assert!(
        rows.is_array() || rows.is_null(),
        "affectedRows should be an array or null, got: {rows}"
    );
}
