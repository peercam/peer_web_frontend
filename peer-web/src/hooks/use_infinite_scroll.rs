//! Shared infinite scroll hook using IntersectionObserver.

use leptos::prelude::*;
use std::rc::Rc;

/// Return value from [`use_infinite_scroll`].
pub struct InfiniteScroll {
    /// Ref to attach to the sentinel/loader element.
    pub loader_ref: NodeRef<leptos::html::Div>,
    /// Whether a load is currently in progress.
    pub is_loading: ReadSignal<bool>,
    /// Whether more items are available.
    pub has_more: ReadSignal<bool>,
}

/// Sets up an IntersectionObserver that calls `load_fn` when the sentinel
/// element scrolls into view and `is_loading` is false / `has_more` is true.
///
/// The caller owns `is_loading` and `has_more` as `RwSignal`s and must update
/// them from inside their load callback. This hook only reads them to gate
/// observer triggers and exposes read-only handles for the view layer.
///
/// ```ignore
/// let is_loading = RwSignal::new(false);
/// let has_more = RwSignal::new(true);
/// let scroll = use_infinite_scroll(is_loading, has_more, move || { load_posts(); });
/// // In view:  <div node_ref=scroll.loader_ref />
/// ```
pub fn use_infinite_scroll(
    is_loading: RwSignal<bool>,
    has_more: RwSignal<bool>,
    load_fn: impl Fn() + 'static,
) -> InfiniteScroll {
    let loader_ref = NodeRef::<leptos::html::Div>::new();

    // Wrap in Rc so the closure can be shared between the observer callback
    // and the on_cleanup closure without requiring Copy.
    #[allow(unused_variables)]
    let load_fn = Rc::new(load_fn);

    #[cfg(feature = "hydrate")]
    {
        let load_fn = load_fn.clone();
        Effect::new(move |_| {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;

            let Some(el) = loader_ref.get() else {
                return;
            };

            let load_fn = load_fn.clone();
            let callback = Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
                for entry in entries.iter() {
                    let entry: web_sys::IntersectionObserverEntry = entry.unchecked_into();
                    if entry.is_intersecting() && !is_loading.get() && has_more.get() {
                        load_fn();
                    }
                }
            });

            let options = web_sys::IntersectionObserverInit::new();
            options.set_root_margin("0px 0px 200px 0px");
            options.set_threshold(&JsValue::from_f64(0.1));

            if let Ok(observer) = web_sys::IntersectionObserver::new_with_options(
                callback.as_ref().unchecked_ref(),
                &options,
            ) {
                observer.observe(&el);

                // Store both the observer and the closure in local (non-Send) storage
                // so they can be dropped on cleanup. Single-threaded wasm guarantees
                // safety here.
                let obs_and_cb = StoredValue::new_local(Some((observer, callback)));

                on_cleanup(move || {
                    if let Some((obs, _cb)) = obs_and_cb.try_update_value(|v| v.take()).flatten() {
                        obs.disconnect();
                        // _cb is dropped here, freeing the JS closure memory
                    }
                });
            }
        });
    }

    InfiniteScroll {
        loader_ref,
        is_loading: is_loading.read_only(),
        has_more: has_more.read_only(),
    }
}
