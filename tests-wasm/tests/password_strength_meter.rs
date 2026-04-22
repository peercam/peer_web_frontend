//! Layer-2 component test: the password-strength meter reacts to
//! real `input` events fired against the DOM.
//!
//! This complements `tests/component_ssr.rs` (which only verifies
//! initial render) and `src/components/registration_gate.rs` tests
//! (which only verify the pure submit predicate). The thing only this
//! layer can prove is that **typing into a field updates the meter**.

#![cfg(target_arch = "wasm32")]

use leptos::prelude::*;
use peer_web::components::password_strength::PasswordStrengthMeter;
use peer_web::components::validation::validate_password;
use peer_web_tests_wasm::harness;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn meter_updates_label_when_user_types_strong_password() {
    let host = harness::mount(|| {
        let password = RwSignal::new(String::new());
        let validation = Memo::new(move |_| validate_password(&password.get()));
        let visible = Memo::new(move |_| !password.get().is_empty());

        view! {
            <input
                id="pwd"
                type="text"
                on:input=move |ev| password.set(event_target_value(&ev))
            />
            <PasswordStrengthMeter validation visible />
        }
    });

    // Empty: wrapper carries the `none` modifier.
    let wrapper = harness::query(&host, "#passwordStrength");
    assert!(
        wrapper.class_name().contains("none"),
        "expected `none` initially, got {:?}",
        wrapper.class_name()
    );

    // Type a strong password — the meter must flip to `show` and
    // surface the "Excellent" label.
    harness::fire_input(&harness::query(&host, "#pwd"), "S3cur3P@ssw0rd!");
    harness::tick().await;
    assert!(
        wrapper.class_name().contains("show"),
        "expected `show` after typing, got {:?}",
        wrapper.class_name()
    );
    let label = harness::text(&harness::query(&host, ".strength-text"));
    assert!(
        label.contains("Excellent"),
        "expected Excellent label, got {label:?}"
    );

    harness::unmount(&host);
}

#[wasm_bindgen_test]
async fn meter_downgrades_label_when_user_clears_password() {
    let host = harness::mount(|| {
        let password = RwSignal::new(String::from("S3cur3!"));
        let validation = Memo::new(move |_| validate_password(&password.get()));
        let visible = Memo::new(move |_| !password.get().is_empty());

        view! {
            <input
                id="pwd"
                type="text"
                prop:value=move || password.get()
                on:input=move |ev| password.set(event_target_value(&ev))
            />
            <PasswordStrengthMeter validation visible />
        }
    });

    harness::tick().await;
    harness::fire_input(&harness::query(&host, "#pwd"), "ab");
    harness::tick().await;
    let label = harness::text(&harness::query(&host, ".strength-text"));
    assert!(
        label.to_lowercase().contains("weak"),
        "expected weak/very weak after downgrade, got {label:?}"
    );

    harness::unmount(&host);
}
