//! Native-target integration tests for the pure, client-side chat
//! state logic that is exercised — but not isolated — by the Playwright
//! suite in `end2end/tests/chat.spec.ts`.
//!
//! Two scenarios in that E2E suite have no dedicated mock-API
//! counterpart in `chat_integration.rs` because their behaviour is
//! entirely in-browser:
//!
//!   * **T3 – search filter** narrows the chat list and shows a
//!     "No chats match" empty state. That is powered by
//!     `ChatContext::filtered_chats`, a pure method over the current
//!     `chats`, `filter_type` and `search_query` signals.
//!   * **T5 – connection-lost banner** appears after *two* consecutive
//!     polling failures and clears on the next success. That
//!     debouncing lives in `note_poll_failure` / `note_poll_success`.
//!
//! Both code paths compile on the native target (no DOM, no WASM) so we
//! cover them here rather than in the slower `tests-wasm` suite. The
//! browser-level "does the banner render?" check is added alongside,
//! in `tests-wasm/tests/chat_connection_banner.rs`, and the
//! "does the empty state render?" check in
//! `tests-wasm/tests/chat_search_filter.rs`.
//!
//! Run with: `cargo test --features ssr --test chat_client_state`

#![cfg(feature = "ssr")]

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use peer_web::models::chat::{Chat, ChatMessage, ChatParticipant, ChatType};
use peer_web::state::chat::{ChatContext, ConnectionState, note_poll_failure, note_poll_success};

// ── Fixture helpers ─────────────────────────────────────────────────────

fn participant(userid: &str, username: &str) -> ChatParticipant {
    ChatParticipant {
        userid: userid.to_string(),
        img: None,
        username: username.to_string(),
        slug: None,
        hasaccess: None,
    }
}

/// Build a private (1:1) chat between the viewer and a peer.
fn private_chat(id: &str, viewer_id: &str, peer_username: &str, preview: &str) -> Chat {
    Chat {
        id: id.to_string(),
        name: None,
        image: None,
        createdat: "2024-01-01T00:00:00Z".to_string(),
        updatedat: "2024-01-01T00:00:00Z".to_string(),
        chatmessages: vec![ChatMessage {
            id: format!("{id}-m1"),
            senderid: "peer-id".to_string(),
            chatid: id.to_string(),
            content: preview.to_string(),
            createdat: "2024-01-01T00:00:00Z".to_string(),
            ..Default::default()
        }],
        chatparticipants: vec![
            participant(viewer_id, "viewer"),
            participant("peer-id", peer_username),
        ],
        unread_count: 0,
        last_read_at: None,
    }
}

/// Build a group chat with a given name.
fn group_chat(id: &str, name: &str) -> Chat {
    Chat {
        id: id.to_string(),
        name: Some(name.to_string()),
        image: None,
        createdat: "2024-01-01T00:00:00Z".to_string(),
        updatedat: "2024-01-01T00:00:00Z".to_string(),
        chatmessages: vec![],
        chatparticipants: vec![],
        unread_count: 0,
        last_read_at: None,
    }
}

/// Build a fresh `ChatContext` with no chats loaded. Fields mirror
/// `provide_chat_context` but without the context-provider side effect —
/// tests construct and mutate the struct directly.
fn make_ctx() -> ChatContext {
    ChatContext {
        filter_type: RwSignal::new(ChatType::Private),
        chats: RwSignal::new(vec![]),
        active_chat: RwSignal::new(None),
        messages: RwSignal::new(vec![]),
        friends: RwSignal::new(vec![]),
        selected_users: RwSignal::new(vec![]),
        is_create_overlay_open: RwSignal::new(false),
        is_review_screen: RwSignal::new(false),
        is_loading_chats: RwSignal::new(false),
        is_loading_friends: RwSignal::new(false),
        is_sending: RwSignal::new(false),
        error: RwSignal::new(None),
        current_user_id: RwSignal::new(Some("viewer-id".to_string())),
        current_user_img: RwSignal::new(None),
        group_name: RwSignal::new(String::new()),
        group_image: RwSignal::new(None),
        unread_counts: RwSignal::new(HashMap::new()),
        last_read_at: RwSignal::new(HashMap::new()),
        search_query: RwSignal::new(String::new()),
        connection_state: RwSignal::new(ConnectionState::Connected),
        consecutive_poll_failures: RwSignal::new(0),
    }
}

/// Run a closure inside a fresh reactive `Owner`. Signals created or
/// read inside `body` are tied to a disposable scope.
fn with_owner<F: FnOnce() -> R, R>(body: F) -> R {
    let owner = Owner::new();
    let result = owner.with(body);
    drop(owner);
    result
}

// ── T3: Search filter (client-side) ─────────────────────────────────────

#[test]
fn search_filter_empty_query_returns_all_private_chats() {
    with_owner(|| {
        let ctx = make_ctx();
        ctx.chats.set(vec![
            private_chat("c1", "viewer-id", "alice", "hi alice"),
            private_chat("c2", "viewer-id", "bob", "hey bob"),
            group_chat("g1", "The Crew"),
        ]);

        let filtered = ctx.filtered_chats();
        assert_eq!(
            filtered.len(),
            2,
            "private filter + empty query should match both private chats"
        );
    });
}

#[test]
fn search_filter_matches_private_participant_username() {
    with_owner(|| {
        let ctx = make_ctx();
        ctx.chats.set(vec![
            private_chat("c1", "viewer-id", "alice", "hello"),
            private_chat("c2", "viewer-id", "bob", "morning"),
        ]);
        ctx.search_query.set("ali".to_string());

        let filtered = ctx.filtered_chats();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "c1");
    });
}

#[test]
fn search_filter_matches_group_display_name_case_insensitive() {
    with_owner(|| {
        let ctx = make_ctx();
        ctx.filter_type.set(ChatType::Group);
        ctx.chats.set(vec![
            group_chat("g1", "The Crew"),
            group_chat("g2", "Book Club"),
        ]);
        ctx.search_query.set("BOOK".to_string());

        let filtered = ctx.filtered_chats();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "g2");
    });
}

#[test]
fn search_filter_matches_last_message_preview() {
    with_owner(|| {
        let ctx = make_ctx();
        ctx.chats.set(vec![
            private_chat("c1", "viewer-id", "alice", "project status update"),
            private_chat("c2", "viewer-id", "bob", "lunch?"),
        ]);
        ctx.search_query.set("project".to_string());

        let filtered = ctx.filtered_chats();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "c1");
    });
}

/// Mirrors the E2E expectation: typing a nonsense query yields zero
/// matches — the consumer renders the "No chats match" empty state.
#[test]
fn search_filter_unmatched_query_returns_empty_vec() {
    with_owner(|| {
        let ctx = make_ctx();
        ctx.chats.set(vec![
            private_chat("c1", "viewer-id", "alice", "hi"),
            private_chat("c2", "viewer-id", "bob", "hey"),
        ]);
        ctx.search_query.set("zzz-no-such-chat-xyz".to_string());

        assert!(ctx.filtered_chats().is_empty());
    });
}

#[test]
fn search_filter_whitespace_only_query_is_treated_as_empty() {
    with_owner(|| {
        let ctx = make_ctx();
        ctx.chats.set(vec![
            private_chat("c1", "viewer-id", "alice", "hi"),
            private_chat("c2", "viewer-id", "bob", "hey"),
        ]);
        ctx.search_query.set("   ".to_string());

        // The filter trims before matching — both private chats should remain.
        assert_eq!(ctx.filtered_chats().len(), 2);
    });
}

#[test]
fn search_filter_respects_active_tab_when_matching_on_name() {
    with_owner(|| {
        let ctx = make_ctx();
        // "alice" appears as both a private peer username AND a group name.
        ctx.chats.set(vec![
            private_chat("c1", "viewer-id", "alice", "hi"),
            group_chat("g1", "Alice's Book Club"),
        ]);
        ctx.search_query.set("alice".to_string());

        ctx.filter_type.set(ChatType::Private);
        let priv_hits = ctx.filtered_chats();
        assert_eq!(priv_hits.len(), 1);
        assert_eq!(priv_hits[0].id, "c1");

        ctx.filter_type.set(ChatType::Group);
        let group_hits = ctx.filtered_chats();
        assert_eq!(group_hits.len(), 1);
        assert_eq!(group_hits[0].id, "g1");
    });
}

// ── T5: Connection-lost banner debouncing ────────────────────────────────

#[test]
fn one_poll_failure_does_not_flip_to_lost() {
    with_owner(|| {
        let ctx = make_ctx();
        note_poll_failure(ctx);

        assert_eq!(ctx.consecutive_poll_failures.get(), 1);
        assert_eq!(
            ctx.connection_state.get(),
            ConnectionState::Connected,
            "one-off blip must not surface the connection-lost banner"
        );
    });
}

#[test]
fn two_consecutive_poll_failures_flip_to_lost() {
    with_owner(|| {
        let ctx = make_ctx();
        note_poll_failure(ctx);
        note_poll_failure(ctx);

        assert_eq!(ctx.consecutive_poll_failures.get(), 2);
        assert_eq!(ctx.connection_state.get(), ConnectionState::Lost);
    });
}

#[test]
fn further_failures_keep_lost_state() {
    with_owner(|| {
        let ctx = make_ctx();
        for _ in 0..5 {
            note_poll_failure(ctx);
        }
        assert_eq!(ctx.consecutive_poll_failures.get(), 5);
        assert_eq!(ctx.connection_state.get(), ConnectionState::Lost);
    });
}

#[test]
fn poll_success_resets_streak_and_clears_lost_banner() {
    with_owner(|| {
        let ctx = make_ctx();
        note_poll_failure(ctx);
        note_poll_failure(ctx);
        assert_eq!(ctx.connection_state.get(), ConnectionState::Lost);

        note_poll_success(ctx);

        assert_eq!(ctx.consecutive_poll_failures.get(), 0);
        assert_eq!(
            ctx.connection_state.get(),
            ConnectionState::Connected,
            "a single successful poll must clear the banner"
        );
    });
}

#[test]
fn poll_success_is_a_noop_when_already_connected() {
    with_owner(|| {
        let ctx = make_ctx();
        note_poll_success(ctx);
        assert_eq!(ctx.consecutive_poll_failures.get(), 0);
        assert_eq!(ctx.connection_state.get(), ConnectionState::Connected);
    });
}
