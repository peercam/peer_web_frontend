//! Tiny test harness that mounts Leptos components into a detached
//! `<div>` and exposes ergonomic event/query helpers.
//!
//! This is the closest thing the Leptos ecosystem has to React
//! Testing Library: keep the surface deliberately small (mount, query,
//! fire input, click, text) so component tests read as behavioural
//! statements rather than DOM plumbing.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, Element, HtmlElement, HtmlInputElement};

/// Get the document, panicking with a clear message in non-browser
/// environments. Tests must be configured with `run_in_browser`.
fn document() -> Document {
    web_sys::window()
        .expect("no `window` — did you forget `wasm_bindgen_test_configure!(run_in_browser)`?")
        .document()
        .expect("`window.document` missing")
}

/// Mount a component into a fresh `<div>` appended to `<body>` and
/// return that host element so callers can query inside it.
///
/// The host element is intentionally NOT cleaned up between tests:
/// each test should use its own scoped IDs/classes. If you need
/// isolation, call [`unmount`] in a guard.
pub fn mount<F, V>(view_fn: F) -> Element
where
    F: FnOnce() -> V + 'static,
    V: IntoView + 'static,
{
    let document = document();
    let host = document
        .create_element("div")
        .expect("create_element failed");
    document
        .body()
        .expect("body missing")
        .append_child(&host)
        .expect("append_child failed");

    leptos::mount::mount_to(host.clone().unchecked_into::<HtmlElement>(), view_fn).forget();

    host
}

/// Remove a previously-mounted host from the DOM.
pub fn unmount(host: &Element) {
    if let Some(parent) = host.parent_node() {
        let _ = parent.remove_child(host);
    }
}

/// Query a single element under `host`, panicking if the selector
/// matches nothing. Use this for assertions about presence; for
/// "may be present" checks use [`maybe_query`].
pub fn query(host: &Element, selector: &str) -> Element {
    host.query_selector(selector)
        .expect("query_selector threw")
        .unwrap_or_else(|| panic!("no element matched selector {selector:?}"))
}

/// Variant that returns `None` rather than panicking on miss.
pub fn maybe_query(host: &Element, selector: &str) -> Option<Element> {
    host.query_selector(selector).ok().flatten()
}

/// Set the `value` of an `<input>` and dispatch an `input` event so
/// Leptos `on:input` handlers fire. Mirrors `fireEvent.input` in RTL.
pub fn fire_input(input: &Element, value: &str) {
    let html_input = input
        .clone()
        .dyn_into::<HtmlInputElement>()
        .expect("element is not an <input>");
    html_input.set_value(value);

    let event = web_sys::Event::new("input").expect("new Event(input) failed");
    html_input
        .dispatch_event(&event)
        .expect("dispatch_event failed");
}

/// Toggle a checkbox to `checked` and dispatch a `change` event.
pub fn fire_change_checked(input: &Element, checked: bool) {
    let html_input = input
        .clone()
        .dyn_into::<HtmlInputElement>()
        .expect("element is not an <input>");
    html_input.set_checked(checked);

    let event = web_sys::Event::new("change").expect("new Event(change) failed");
    html_input
        .dispatch_event(&event)
        .expect("dispatch_event failed");
}

/// Click an element by dispatching a synthetic `MouseEvent`.
pub fn click(element: &Element) {
    let html_el = element
        .clone()
        .dyn_into::<HtmlElement>()
        .expect("element is not an HTMLElement");
    html_el.click();
}

/// Trim a node's text content and lowercase it for case-insensitive
/// substring assertions. Returns `""` if the node has no text.
pub fn text(element: &Element) -> String {
    element.text_content().unwrap_or_default().trim().to_string()
}

/// Yield to the event loop via `setTimeout(0)` so Leptos effects
/// scheduled as microtasks get a chance to flush before assertions.
///
/// Leptos 0.8 schedules reactive effect re-runs asynchronously, so
/// synchronous reads of the DOM immediately after `fire_input` or
/// similar event-firing helpers can observe stale state. Awaiting
/// `tick()` lets the scheduler drain and settles the view.
pub async fn tick() {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("window missing");
        let closure = Closure::once_into_js(move || {
            resolve.call0(&wasm_bindgen::JsValue::NULL).expect("resolve");
        });
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.unchecked_ref(),
                0,
            )
            .expect("setTimeout failed");
    });
    JsFuture::from(promise).await.expect("tick promise failed");
}
