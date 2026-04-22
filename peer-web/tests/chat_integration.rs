//! Mock-API coverage mirroring `end2end/tests/chat.spec.ts` (Track C).
//!
//! These tests exercise the same chat scenarios that the Playwright E2E
//! suite drives through the browser, but at the GraphQL transport layer —
//! calling `peer_web::api::graphql::mutate`/`query` against the in-process
//! `mock_backend`. Responses are inspected as raw `serde_json::Value` so
//! the tests validate the *wire contract* the frontend relies on
//! (GraphQL strings, variable shapes, auth headers, status fields) and
//! stay robust against pre-existing typed-model incompatibilities
//! between `models::common::DefaultResponse` (PascalCase) and the mock
//! backend's lowercase `status` field.
//!
//! E2E ↔ API mapping:
//!   * T1 chat list loads                    → `test_list_chats_returns_seeded_chats`
//!   * T2 unread badge renders/clears        → `test_unread_count_clears_after_mark_read`
//!   * T4 polled delivery                    → `test_poll_returns_new_peer_message`
//!   * T5 connection-lost / auth guard       → `test_list_chats_without_auth_fails`,
//!     `test_send_chat_message_without_auth_fails`
//!
//! Run with: `cargo test --features ssr --test chat_integration`

#![cfg(feature = "ssr")]

mod common;

use peer_web::api::graphql::{
    LIST_CHATS_QUERY, LIST_CHAT_MESSAGES_QUERY, LOGIN_MUTATION, MARK_CHAT_READ_MUTATION,
    SEND_CHAT_MESSAGE_MUTATION, mutate, query,
};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::RwLockReadGuard;

// Seed credentials — see `tests/mock_backend/src/seed.rs::credentials`.
// The verified user's private chat with Alice is pre-seeded with unread
// messages from Alice, mirroring the chat.spec.ts setup.
const VERIFIED_EMAIL: &str = "test@peer.com";
const VERIFIED_PASSWORD: &str = "TestPass123";
const ALICE_EMAIL: &str = "alice@peer.com";
const ALICE_PASSWORD: &str = "AlicePass123";
const SEED_CHAT_PRIVATE: &str = "30000000-0000-4000-a000-000000000001";

async fn setup() -> RwLockReadGuard<'static, ()> {
    let _ = common::mock_graphql_endpoint().await;
    common::endpoint_read_guard().await
}

/// Setup for tests that mutate shared chat state (seeded messages, read
/// cursors). Returns both the env guard and an exclusive chat-state
/// lock so concurrent test bodies can't race `sendChatMessage` against
/// `markChatRead` on the same seeded chat.
async fn setup_chat_exclusive() -> (
    RwLockReadGuard<'static, ()>,
    tokio::sync::MutexGuard<'static, ()>,
) {
    let _ = common::mock_graphql_endpoint().await;
    let env = common::endpoint_read_guard().await;
    let chat = common::chat_state_guard().await;
    (env, chat)
}

#[derive(Serialize)]
struct LoginVars {
    email: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ListChatsVars {
    limit: Option<i32>,
    offset: Option<i32>,
}

#[derive(Serialize)]
struct SendMessageVars {
    chatid: String,
    content: String,
}

#[derive(Serialize)]
struct ListChatMessagesVars {
    chatid: String,
    since: Option<String>,
}

#[derive(Serialize)]
struct MarkChatReadVars {
    chatid: String,
}

/// Helper: log in via the peer-web client and return the access token.
/// Mirrors the `login()` helper in `chat.spec.ts`.
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

// ── T1: Chat list loads and shows seeded chats ───────────────────────────
#[tokio::test]
async fn test_list_chats_returns_seeded_chats() {
    let _env = setup().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    let data: Value = query(
        LIST_CHATS_QUERY,
        ListChatsVars {
            limit: None,
            offset: None,
        },
        Some(&token),
    )
    .await
    .expect("listChats query should succeed");

    let list = &data["listChats"];
    assert_eq!(meta_status(list), Some("success"));
    let rows = list["affectedRows"].as_array().expect("affectedRows");
    assert!(
        !rows.is_empty(),
        "seed should expose at least one chat for the verified user"
    );
    assert!(
        rows.iter()
            .any(|c| c["id"].as_str() == Some(SEED_CHAT_PRIVATE)),
        "expected seeded private chat id in listChats response"
    );
}

// ── T2: Unread count renders and clears on mark-read ─────────────────────
#[tokio::test]
async fn test_unread_count_clears_after_mark_read() {
    let _guards = setup_chat_exclusive().await;
    let token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    // Before mark-read: the seeded private chat has unread peer messages.
    let before: Value = query(
        LIST_CHATS_QUERY,
        ListChatsVars {
            limit: None,
            offset: None,
        },
        Some(&token),
    )
    .await
    .expect("listChats #1");
    let total_unread_before: u64 = before["listChats"]["affectedRows"]
        .as_array()
        .expect("affectedRows #1")
        .iter()
        .map(|c| c["unreadCount"].as_u64().unwrap_or(0))
        .sum();
    assert!(
        total_unread_before > 0,
        "seed should produce at least one unread message for the verified user"
    );

    // Mark the private chat as read.
    let mark: Value = mutate(
        MARK_CHAT_READ_MUTATION,
        MarkChatReadVars {
            chatid: SEED_CHAT_PRIVATE.into(),
        },
        Some(&token),
    )
    .await
    .expect("markChatRead should succeed");
    let mcr = &mark["markChatRead"];
    assert_eq!(meta_status(mcr), Some("success"));
    assert!(mcr["lastReadAt"].is_string());

    // After mark-read: the private chat should have zero unread.
    let after: Value = query(
        LIST_CHATS_QUERY,
        ListChatsVars {
            limit: None,
            offset: None,
        },
        Some(&token),
    )
    .await
    .expect("listChats #2");
    let rows = after["listChats"]["affectedRows"]
        .as_array()
        .expect("affectedRows #2");
    let private_after = rows
        .iter()
        .find(|c| c["id"].as_str() == Some(SEED_CHAT_PRIVATE))
        .expect("private chat should still be listed");
    assert_eq!(
        private_after["unreadCount"].as_u64(),
        Some(0),
        "markChatRead should clear unread for the targeted chat"
    );
}

// ── T4: Polled delivery — new peer message shows up in listChatMessages ──
#[tokio::test]
async fn test_poll_returns_new_peer_message() {
    let _guards = setup_chat_exclusive().await;

    let viewer_token = login(VERIFIED_EMAIL, VERIFIED_PASSWORD).await;

    // Remember the latest message timestamp so we can use it as the
    // `since` cursor the polling transport uses in production.
    let first: Value = query(
        LIST_CHAT_MESSAGES_QUERY,
        ListChatMessagesVars {
            chatid: SEED_CHAT_PRIVATE.into(),
            since: None,
        },
        Some(&viewer_token),
    )
    .await
    .expect("listChatMessages initial");
    let list = &first["listChatMessages"];
    assert_eq!(meta_status(list), Some("success"));
    let last_createdat = list["affectedRows"]
        .as_array()
        .and_then(|rows| rows.last())
        .and_then(|m| m["createdat"].as_str())
        .expect("seed chat has messages")
        .to_string();

    // Alice (the peer) sends a new message into the same chat.
    let alice_token = login(ALICE_EMAIL, ALICE_PASSWORD).await;
    let sent: Value = mutate(
        SEND_CHAT_MESSAGE_MUTATION,
        SendMessageVars {
            chatid: SEED_CHAT_PRIVATE.into(),
            content: "hello from poll".into(),
        },
        Some(&alice_token),
    )
    .await
    .expect("sendChatMessage should succeed");
    let send = &sent["sendChatMessage"];
    assert_eq!(meta_status(send), Some("success"));
    assert_eq!(
        send["affectedRows"]["content"].as_str(),
        Some("hello from poll")
    );

    // The viewer polls with `since = last_createdat` and should see
    // the new message (no stale duplicates).
    let poll: Value = query(
        LIST_CHAT_MESSAGES_QUERY,
        ListChatMessagesVars {
            chatid: SEED_CHAT_PRIVATE.into(),
            since: Some(last_createdat),
        },
        Some(&viewer_token),
    )
    .await
    .expect("listChatMessages poll");
    let new_rows = poll["listChatMessages"]["affectedRows"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        new_rows
            .iter()
            .any(|m| m["content"].as_str() == Some("hello from poll")),
        "polled listChatMessages should include the newly sent peer message; got {:?}",
        new_rows
    );
}

// ── T5 (auth variant): listChats without a token must fail the auth guard ─
#[tokio::test]
async fn test_list_chats_without_auth_fails() {
    let _env = setup().await;

    let result: Result<Value, _> = query(
        LIST_CHATS_QUERY,
        ListChatsVars {
            limit: None,
            offset: None,
        },
        None,
    )
    .await;

    // Either the transport layer surfaces an error, or the response
    // returns meta.status != "success". Both are acceptable; neither
    // should yield a non-empty `affectedRows`.
    match result {
        Err(_) => {}
        Ok(data) => {
            let list = &data["listChats"];
            let status = meta_status(list);
            let rows_empty = list["affectedRows"]
                .as_array()
                .map(|r| r.is_empty())
                .unwrap_or(true);
            assert!(
                status != Some("success") || rows_empty,
                "expected auth failure when calling listChats without a token, got {:?}",
                data
            );
        }
    }
}

// ── sendChatMessage without auth must also be rejected ───────────────────
#[tokio::test]
async fn test_send_chat_message_without_auth_fails() {
    let _env = setup().await;

    let result: Result<Value, _> = mutate(
        SEND_CHAT_MESSAGE_MUTATION,
        SendMessageVars {
            chatid: SEED_CHAT_PRIVATE.into(),
            content: "noauth".into(),
        },
        None,
    )
    .await;

    match result {
        Err(_) => {}
        Ok(data) => {
            let send = &data["sendChatMessage"];
            assert_ne!(
                meta_status(send),
                Some("success"),
                "expected sendChatMessage without a token to be rejected, got {:?}",
                data
            );
        }
    }
}
