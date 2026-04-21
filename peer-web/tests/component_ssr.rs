//! Layer 3 component-level tests — SSR rendering.
//!
//! These tests render individual components to an HTML string using
//! `leptos::prelude::*` server-rendering primitives. They run on the
//! native target (no browser, no wasm) and assert the **shape of the
//! initial HTML** — the exact bytes a Playwright `request.get()` would
//! see before hydration.
//!
//! Why this layer exists alongside `cargo test --lib`:
//!   * Layer 1 (in `src/components/registration_gate.rs`) covers pure
//!     decision logic.
//!   * Layer 3 (this file) covers what the component *renders* given
//!     an initial signal state — e.g. "the strength meter shows
//!     'Excellent' when given an excellent password".
//!   * Layer 2 (`tests-wasm/`, scaffolded separately) covers DOM events
//!     and runtime reactivity in a real headless browser.
//!
//! Run with: `cargo test --features ssr --test component_ssr`

#![cfg(feature = "ssr")]

use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use leptos::tachys::view::RenderHtml;
use peer_web::components::password_strength::PasswordStrengthMeter;
use peer_web::components::validation::{PasswordValidation, validate_password};

/// Helper: render any component to a `String` for substring assertions.
///
/// Wraps the view in a fresh `Owner` so signals/memos created inside
/// `view_fn` are tied to a disposable reactive scope. We then call
/// `RenderHtml::to_html` (the trait method `view!` produces output for)
/// to materialise the HTML synchronously — no Tokio, no axum.
fn render<F, V>(view_fn: F) -> String
where
    F: FnOnce() -> V,
    V: RenderHtml + Send + 'static,
{
    let owner = Owner::new();
    let html = owner.with(|| view_fn().to_html());
    drop(owner);
    html
}

// ── PasswordStrengthMeter ──────────────────────────────────────────

#[test]
fn strength_meter_shows_excellent_label_for_strong_password() {
    let html = render(|| {
        let validation = Memo::new(|_| validate_password("S3cur3P@ssw0rd!"));
        let visible = Memo::new(|_| true);
        view! { <PasswordStrengthMeter validation visible /> }
    });

    assert!(
        html.contains("Excellent"),
        "expected 'Excellent' label in:\n{html}"
    );
    assert!(
        html.contains("strength-fill excellent"),
        "expected fill-bar 'excellent' class in:\n{html}"
    );
    assert!(
        html.contains(r#"aria-valuenow="5""#),
        "expected aria-valuenow=5 in:\n{html}"
    );
}

#[test]
fn strength_meter_shows_very_weak_label_for_short_password() {
    let html = render(|| {
        let validation = Memo::new(|_| validate_password("ab"));
        let visible = Memo::new(|_| true);
        view! { <PasswordStrengthMeter validation visible /> }
    });

    assert!(
        html.contains("Very weak"),
        "expected 'Very weak' label in:\n{html}"
    );
    assert!(
        html.contains("strength-fill weak"),
        "expected fill-bar 'weak' class in:\n{html}"
    );
}

#[test]
fn strength_meter_hides_when_visible_is_false() {
    let html = render(|| {
        let validation = Memo::new(|_| validate_password(""));
        let visible = Memo::new(|_| false);
        view! { <PasswordStrengthMeter validation visible /> }
    });

    // The wrapper carries the `none` modifier — production CSS hides
    // every descendant via `.password-strength.none { display: none }`.
    assert!(
        html.contains("password-strength none"),
        "expected `none` visibility class in:\n{html}"
    );
    // The aria-live region must NOT contain a strength announcement
    // when hidden — otherwise screen readers would speak it on hide.
    // The component renders an empty (whitespace-only) live region.
    let live_region = extract_between(
        &html,
        r#"aria-live="polite" aria-atomic="true" class="sr-only">"#,
        "</div>",
    )
    .unwrap_or_default();
    assert!(
        !live_region.contains("Password strength:"),
        "aria-live region must be empty when hidden, got: {live_region:?}"
    );
}

/// Tiny helper for snippet extraction inside an HTML blob.
fn extract_between<'a>(haystack: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let s = haystack.find(start)? + start.len();
    let e = haystack[s..].find(end)? + s;
    Some(&haystack[s..e])
}

#[test]
fn strength_meter_marks_role_meter_for_assistive_tech() {
    // Smoke-test that the ARIA contract the A11y E2E test depends on
    // is present in the SSR'd HTML, before any JS has run.
    let html = render(|| {
        let validation = Memo::new(|_| validate_password("Average1"));
        let visible = Memo::new(|_| true);
        view! { <PasswordStrengthMeter validation visible /> }
    });

    assert!(html.contains(r#"role="meter""#));
    assert!(html.contains(r#"aria-label="Password strength""#));
    assert!(html.contains(r#"aria-valuemin="1""#));
    assert!(html.contains(r#"aria-valuemax="5""#));
}

// ── PasswordValidation invariants surfaced through rendering ───────

#[test]
fn good_password_without_specials_renders_good_not_excellent() {
    // Boundary: 8+ chars, mixed case, digit, no special, < 12 chars.
    let html = render(|| {
        let validation = Memo::new(|_| validate_password("Abcdefg1"));
        let visible = Memo::new(|_| true);
        view! { <PasswordStrengthMeter validation visible /> }
    });

    let v: PasswordValidation = validate_password("Abcdefg1");
    assert!(v.requirements.is_sufficient());
    assert!(!v.requirements.special);

    assert!(html.contains(">Good<"), "expected 'Good' label in:\n{html}");
    assert!(html.contains("strength-fill strong"));
}
