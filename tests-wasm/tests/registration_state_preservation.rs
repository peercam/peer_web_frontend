//! Layer-2 component tests covering the registration flow's
//! state-preservation guarantees exercised by the Playwright suite:
//!
//!   * `end2end/tests/registration/happy-path.spec.ts` T8 — the
//!     `?ref=<uuid>` query parameter auto-fills the Step 1 referral
//!     input. At the component level this reduces to: "when the parent
//!     hands `ReferralStep` a pre-populated signal, the input shows
//!     that value on first paint."
//!   * `end2end/tests/registration/navigation.spec.ts` T9 — navigating
//!     back from Step 2 to Step 1 preserves the entered referral code.
//!     The full page-level routing is exercised by Playwright; the
//!     browser-only part of that contract is "typing into the input
//!     propagates to the parent signal", which is what we cover here.
//!
//! The routing + step-machine wiring is covered by the unit tests
//! inside `src/pages/register.rs` (`RegStep::previous`,
//! `show_back_button`, etc.).

#![cfg(target_arch = "wasm32")]

use std::cell::Cell;
use std::rc::Rc;

use leptos::prelude::*;
use peer_web::components::referral::ReferralStep;
use peer_web_tests_wasm::harness;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::HtmlInputElement;

wasm_bindgen_test_configure!(run_in_browser);

const VALID_CODE: &str = "85d5f836-b1f5-4c4e-9381-1b058e13df93";

/// Yield to the event loop so Leptos's render effects can flush. The
/// `prop:value` binding in `ReferralStep` schedules a microtask to set
/// the DOM property on first paint; without this helper a synchronous
/// `input.value()` read would race that microtask.
async fn tick() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window");
        let closure = Closure::once_into_js(move || {
            resolve
                .call0(&wasm_bindgen::JsValue::NULL)
                .expect("resolve call");
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(closure.unchecked_ref(), 0)
            .expect("setTimeout");
    });
    JsFuture::from(promise).await.expect("tick");
}

/// Read the current DOM `value` of an `<input>` element.
fn input_value(element: &web_sys::Element) -> String {
    element
        .clone()
        .dyn_into::<HtmlInputElement>()
        .expect("element is not an <input>")
        .value()
}

/// Mount `ReferralStep` with the referral code signal seeded from
/// `initial`. Signals are created *inside* the mount closure so they
/// live in the reactive Owner that owns the view — this is what makes
/// `prop:value` flush to the DOM on initial render. A handle to the
/// signal is exported through an `Rc<Cell<…>>` so the test body can
/// read/write it after mount (`RwSignal` is `Copy`).
fn mount_step(initial: &str) -> (web_sys::Element, RwSignal<String>) {
    let initial = initial.to_string();
    let handle: Rc<Cell<Option<RwSignal<String>>>> = Rc::new(Cell::new(None));
    let handle_for_view = Rc::clone(&handle);

    let host = harness::mount(move || {
        let code = RwSignal::new(initial.clone());
        handle_for_view.set(Some(code));

        let verify = Action::new(|_: &String| async move {});
        let pending = Signal::derive(|| false);
        let show_default = Callback::new(|_: ()| {});

        view! {
            <ReferralStep
                referral_code=code
                on_verify=verify
                pending=pending
                on_show_default=show_default
            />
        }
    });

    let code = handle
        .get()
        .expect("mount closure must have populated the handle");
    (host, code)
}

#[wasm_bindgen_test]
async fn t8_referral_input_is_prefilled_from_parent_signal() {
    // Seed the signal as if a `?ref=<uuid>` query-parameter effect had
    // already fired before the step mounted.
    let (host, _code) = mount_step(VALID_CODE);

    // Let Leptos flush the initial render effects that back `prop:value`.
    tick().await;

    let input = harness::query(&host, "#referralCode");
    assert_eq!(
        input_value(&input),
        VALID_CODE,
        "referral input must mirror the parent-owned signal on initial mount"
    );

    harness::unmount(&host);
}

#[wasm_bindgen_test]
fn t9_typed_value_propagates_to_parent_signal() {
    // Full T9 is: unmount Step 1, remount, and the previously-typed
    // value reappears. The page-level routing is exercised by
    // Playwright; the browser-only link in that chain is "typing into
    // the input updates the parent-owned signal". Once that holds, the
    // fact that re-mounting with a non-empty signal shows the value
    // (see the T8 test above) completes the contract.
    let (host, code) = mount_step("");

    let input = harness::query(&host, "#referralCode");
    harness::fire_input(&input, VALID_CODE);

    assert_eq!(
        code.get_untracked(),
        VALID_CODE,
        "typing into the input must update the parent signal"
    );
    assert_eq!(
        input_value(&input),
        VALID_CODE,
        "DOM value should also reflect the typed text",
    );

    harness::unmount(&host);
}

#[wasm_bindgen_test]
fn empty_mount_renders_empty_input() {
    // Complement to the tests above: an empty signal must render as
    // an empty input. Pins down that the input is driven purely by
    // the parent signal — no stale browser autofill leaks across
    // mounts.
    let (host, _code) = mount_step("");

    let input = harness::query(&host, "#referralCode");
    assert_eq!(input_value(&input), "");

    harness::unmount(&host);
}
