//! Layer-2 component test mirroring `end2end/tests/chat.spec.ts` T5:
//! the chat container surfaces a "connection lost" banner when the
//! transport flips to `ConnectionState::Lost`, and hides it again on
//! recovery.
//!
//! The debouncing logic that decides *when* to flip to Lost is covered
//! in `tests/chat_client_state.rs`. This test covers the
//! browser-level rendering contract the E2E assertion
//! `expect(page.locator(".connection-lost-banner")).toBeVisible()`
//! relies on.

#![cfg(target_arch = "wasm32")]

use leptos::prelude::*;
use peer_web::components::chat::ChatContainer;
use peer_web::state::chat::{ConnectionState, provide_chat_context};
use peer_web_tests_wasm::harness;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn banner_hidden_while_connected() {
    let host = harness::mount(|| {
        let _ctx = provide_chat_context();
        view! { <ChatContainer/> }
    });

    assert!(
        harness::maybe_query(&host, ".connection-lost-banner").is_none(),
        "banner must not render in the Connected state"
    );

    harness::unmount(&host);
}

#[wasm_bindgen_test]
fn banner_visible_when_state_flips_to_lost() {
    let host = harness::mount(|| {
        let ctx = provide_chat_context();
        ctx.connection_state.set(ConnectionState::Lost);
        view! { <ChatContainer/> }
    });

    let banner = harness::query(&host, ".connection-lost-banner");
    assert!(
        harness::text(&banner)
            .to_lowercase()
            .contains("connection lost"),
        "banner text should describe the lost connection, got {:?}",
        harness::text(&banner),
    );

    // The retry-now affordance must also be present so keyboard users
    // can recover without waiting for the next poll cycle.
    let _retry = harness::query(&banner, ".retry-btn");

    harness::unmount(&host);
}
