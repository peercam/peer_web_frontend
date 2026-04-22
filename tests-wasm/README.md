# peer-web component tests (Layer 2)

Headless-browser tests for individual Leptos components, using
`wasm-bindgen-test` against a real Chrome/Firefox via `wasm-pack`.

This is the **third leg** of the testing pyramid:

| Layer | Where | What it covers | Speed |
|------:|-------|----------------|------:|
| 1 | `src/**` `#[cfg(test)] mod tests` | Pure decision logic (e.g. `registration_gate::can_submit_registration`) | µs |
| 3 | `tests/component_ssr.rs` | Initial-render HTML snapshots | ms |
| **2** | **`tests-wasm/` (this crate)** | **Real DOM, events, reactivity** | seconds |
| 4 | `end2end/` (Playwright) | Routing, cookies, SW, axe | seconds–minutes |

Reach for Layer 2 **only when Layers 1+3 cannot answer the question**:
focus management, real `input`/`change`/`click` events going through
your handlers, `IntersectionObserver`, `localStorage`, `ServiceWorker`
registration, computed bounding rectangles, etc.

## Why a separate crate

`wasm-bindgen-test` requires `crate-type = ["cdylib", "rlib"]` and a
specific dev-dependency configuration that conflicts with the main
crate's SSR setup. Keeping it in its own workspace member (with a
relative `path = "../peer-web"` dependency) means:

* `cargo test` in the main crate stays fast and never touches wasm.
* CI can opt in to `wasm-pack test` independently.
* A new contributor doesn't need `wasm-pack` installed to run the
  unit and SSR tests.

## Running

```bash
# One-time setup
cargo install wasm-pack --locked

# Run the suite (Chrome — uses Playwright's bundled chromium if present)
cd tests-wasm
wasm-pack test --headless --chrome

# Or Firefox / Safari
wasm-pack test --headless --firefox
wasm-pack test --headless --safari
```

`wasm-pack` will download `chromedriver` automatically on first run.

## Writing a new test

Use the `harness` module to mount a component into a detached `<div>`
and to fire events. The pattern looks like this:

```rust
use wasm_bindgen_test::*;
use leptos::prelude::*;
use peer_web::components::password_strength::PasswordStrengthMeter;
use peer_web::components::validation::validate_password;

mod harness;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn meter_reacts_to_password_input() {
    let host = harness::mount(|| {
        let password = RwSignal::new(String::new());
        let validation = Memo::new(move |_| validate_password(&password.get()));
        let visible = Memo::new(move |_| !password.get().is_empty());

        view! {
            <input
                id="pwd"
                on:input=move |ev| password.set(event_target_value(&ev))
            />
            <PasswordStrengthMeter validation visible />
        }
    });

    harness::fire_input(&harness::query(&host, "#pwd"), "S3cur3P@ss!");
    let label = harness::query(&host, ".strength-text").text_content().unwrap_or_default();
    assert!(label.contains("Excellent"), "got: {label:?}");
}
```

## Mapping back to the failing E2E suite

Tests that genuinely need this layer (and would be flaky as Layer-1
unit tests):

* `registration/form-validation.spec.ts` T5 — strength meter updates
  on input event (Layer-1 covers the gating predicate; Layer-2 would
  cover the **DOM update** after typing).
* `chat.spec.ts` T3 — search-input filter narrows the rendered list.
* `pwa.spec.ts` — service-worker registration in `<body>`.

Everything else in `end2end/` is genuine browser-integration territory
(routing, cookies, axe) and should stay in Playwright.
