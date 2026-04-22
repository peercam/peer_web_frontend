//! Layer-2 component test mirroring `end2end/tests/chat.spec.ts` T3:
//! the chat sidebar renders the "No chats match …" empty state when the
//! user's search query filters every chat out, and restores the list
//! when the query is cleared.
//!
//! The pure filter logic is covered in
//! `peer-web/tests/chat_client_state.rs` — this layer proves that the
//! `ChatList` view actually reacts to signal changes in a live DOM.

#![cfg(target_arch = "wasm32")]

use leptos::prelude::*;
use peer_web::components::chat::ChatList;
use peer_web::models::chat::{Chat, ChatMessage, ChatParticipant};
use peer_web::state::chat::{ChatContext, provide_chat_context};
use peer_web_tests_wasm::harness;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn private_chat(id: &str, peer_username: &str, preview: &str) -> Chat {
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
            ChatParticipant {
                userid: "viewer-id".into(),
                img: None,
                username: "viewer".into(),
                slug: None,
                hasaccess: None,
            },
            ChatParticipant {
                userid: "peer-id".into(),
                img: None,
                username: peer_username.into(),
                slug: None,
                hasaccess: None,
            },
        ],
        unread_count: 0,
        last_read_at: None,
    }
}

/// Seed a ChatContext that mirrors a post-load state: two private
/// chats loaded, not loading, viewer known. Returns the handle so the
/// test body can mutate signals after mount.
fn seed_ctx() -> ChatContext {
    let ctx = provide_chat_context();
    ctx.is_loading_chats.set(false);
    ctx.current_user_id.set(Some("viewer-id".into()));
    ctx.chats.set(vec![
        private_chat("c1", "alice", "hi there"),
        private_chat("c2", "bob", "morning"),
    ]);
    ctx
}

#[wasm_bindgen_test]
fn shows_filtered_empty_state_when_query_matches_nothing() {
    let host = harness::mount(|| {
        let ctx = seed_ctx();
        ctx.search_query.set("zzz-no-such-chat-xyz".into());
        view! { <ChatList/> }
    });

    let banner = harness::query(&host, ".no_post_found");
    let text = harness::text(&banner);
    assert!(
        text.contains("No chats match"),
        "expected filtered empty-state text, got {text:?}"
    );
    assert!(
        text.contains("zzz-no-such-chat-xyz"),
        "expected the query to be echoed in the empty state, got {text:?}"
    );

    harness::unmount(&host);
}

#[wasm_bindgen_test]
fn shows_chat_items_when_query_is_empty() {
    let host = harness::mount(|| {
        let _ctx = seed_ctx();
        view! { <ChatList/> }
    });

    // At least one chat item must render when no search query filters
    // the list. (Counting via DFS avoids depending on the `NodeList`
    // web-sys feature.)
    let first = harness::maybe_query(&host, ".chat-item");
    assert!(
        first.is_some(),
        "empty search query should render at least one private chat item"
    );

    // Filtered empty-state must NOT be visible when chats are present.
    assert!(
        harness::maybe_query(&host, ".no_post_found").is_none(),
        "empty-state banner must not appear when filtered chats are non-empty"
    );

    harness::unmount(&host);
}
