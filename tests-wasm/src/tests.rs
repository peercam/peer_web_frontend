//! Smoke tests that exercise the harness itself.
//!
//! Real component tests live in `tests/` files at the crate root so
//! `wasm-pack test` picks them up via `wasm-bindgen-test`.

use crate::harness;
use leptos::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn harness_can_mount_a_trivial_view_and_read_text() {
    let host = harness::mount(|| view! { <p id="greeting">"hello"</p> });
    let p = harness::query(&host, "#greeting");
    assert_eq!(harness::text(&p), "hello");
    harness::unmount(&host);
}

#[wasm_bindgen_test]
async fn harness_input_event_updates_signal() {
    let host = harness::mount(move || {
        let value = RwSignal::new(String::new());
        view! {
            <input
                id="probe"
                on:input=move |ev| value.set(event_target_value(&ev))
            />
            <span id="echo">{move || value.get()}</span>
        }
    });

    harness::fire_input(&harness::query(&host, "#probe"), "leptos");
    harness::tick().await;
    assert_eq!(harness::text(&harness::query(&host, "#echo")), "leptos");
    harness::unmount(&host);
}
