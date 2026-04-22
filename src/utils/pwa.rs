//! Progressive Web App integration.
//!
//! - Registers the service worker at `/sw.js` on hydrate.
//! - Captures `beforeinstallprompt` so the install banner can fire later.
//! - Polls `registration.update()` on visibilitychange + every 60s while the
//!   tab is visible so the "Update available" toast can surface promptly.
//! - Provides a `?nosw` escape hatch that unregisters all service workers.
//!
//! On SSR builds every function is a no-op stub so `App` can call them
//! unconditionally.

use leptos::prelude::*;

/// Build hash used to version the service worker cache.
///
/// Composed at compile time from `CARGO_PKG_VERSION` + optional `GIT_SHA`
/// (set by CI to `git rev-parse --short HEAD`). Local dev falls back to
/// the package version — acceptable because local devs use DevTools
/// "Update on reload".
pub const BUILD_HASH: &str = match option_env!("GIT_SHA") {
    Some(sha) => sha,
    None => env!("CARGO_PKG_VERSION"),
};

/// Key under which the captured `BeforeInstallPromptEvent` is stored.
///
/// Consumers obtain this via `use_context::<InstallPromptEvent>()`.
#[derive(Clone, Copy)]
pub struct InstallPromptEvent {
    /// Raw JS event with a callable `prompt()` method. `Some` once captured.
    #[cfg(feature = "hydrate")]
    pub event: RwSignal<Option<wasm_bindgen::JsValue>>,
    /// `true` after the user (or the platform) has installed the app.
    pub installed: RwSignal<bool>,
}

impl InstallPromptEvent {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "hydrate")]
            event: RwSignal::new(None),
            installed: RwSignal::new(false),
        }
    }
}

impl Default for InstallPromptEvent {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide an [`InstallPromptEvent`] in the Leptos context and, on hydrate,
/// wire up the `beforeinstallprompt` and `appinstalled` listeners.
pub fn provide_install_prompt_context() {
    let ctx = InstallPromptEvent::new();
    provide_context(ctx);

    #[cfg(feature = "hydrate")]
    hydrate_impl::wire_install_listeners(ctx);
}

/// Register the service worker. On hydrate only.
pub fn register_service_worker() {
    #[cfg(feature = "hydrate")]
    hydrate_impl::register();
}

/// Handle to listen for SW update events.
///
/// The component layer subscribes to `update_available` and dispatches a
/// toast when it flips to `true`.
#[derive(Clone, Copy)]
pub struct ServiceWorkerUpdate {
    pub update_available: RwSignal<bool>,
}

impl ServiceWorkerUpdate {
    pub fn new() -> Self {
        Self {
            update_available: RwSignal::new(false),
        }
    }
}

impl Default for ServiceWorkerUpdate {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide the SW-update context.
pub fn provide_sw_update_context() {
    provide_context(ServiceWorkerUpdate::new());
}

/// Called by the install banner when the user clicks "Install".
///
/// Invokes `event.prompt()` on the stashed `BeforeInstallPromptEvent` and
/// clears the stashed signal. No-op on SSR or when no event is captured.
pub fn trigger_install_prompt() {
    #[cfg(feature = "hydrate")]
    hydrate_impl::trigger_install_prompt();
}

/// Ask the waiting SW to activate and reload the page.
pub fn apply_service_worker_update() {
    #[cfg(feature = "hydrate")]
    hydrate_impl::apply_update();
}

#[cfg(feature = "hydrate")]
mod hydrate_impl {
    use super::{BUILD_HASH, InstallPromptEvent, ServiceWorkerUpdate};
    use leptos::prelude::*;
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::{Event, ServiceWorkerRegistration, ServiceWorkerState, window};

    pub(super) fn wire_install_listeners(ctx: InstallPromptEvent) {
        let Some(win) = window() else { return };

        // `beforeinstallprompt` — stash the event for later `prompt()`.
        let stash = ctx.event;
        let cb = Closure::<dyn FnMut(Event)>::new(move |e: Event| {
            e.prevent_default();
            stash.set(Some(JsValue::from(e)));
        });
        let _ = win
            .add_event_listener_with_callback("beforeinstallprompt", cb.as_ref().unchecked_ref());
        cb.forget();

        // `appinstalled` — mark installed and clear the stash.
        let stash = ctx.event;
        let installed = ctx.installed;
        let cb = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
            installed.set(true);
            stash.set(None);
        });
        let _ = win.add_event_listener_with_callback("appinstalled", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    pub(super) fn register() {
        let Some(win) = window() else { return };
        let location = win.location();
        let search = location.search().unwrap_or_default();

        let nav = win.navigator();
        let container = nav.service_worker();

        // `?nosw` — dev escape hatch: unregister everything and bail.
        if search.contains("nosw") {
            spawn_local(async move {
                let Ok(regs_val) = JsFuture::from(container.get_registrations()).await else {
                    return;
                };
                let regs = js_sys::Array::from(&regs_val);
                for reg in regs.iter() {
                    if let Ok(reg) = reg.dyn_into::<ServiceWorkerRegistration>() {
                        let _ = reg.unregister();
                    }
                }
            });
            return;
        }

        // Register `/sw.js?v=<HASH>` — cache name keys off the query string.
        let sw_url = format!("/sw.js?v={}", BUILD_HASH);
        let promise = container.register(&sw_url);

        spawn_local(async move {
            let Ok(reg_val) = JsFuture::from(promise).await else {
                web_sys::console::warn_1(&"[pwa] service worker registration failed".into());
                return;
            };
            let Ok(reg) = reg_val.dyn_into::<ServiceWorkerRegistration>() else {
                return;
            };

            // Wire update detection → ServiceWorkerUpdate context.
            if let Some(ctx) = use_context::<ServiceWorkerUpdate>() {
                wire_update_detection(&reg, ctx);
                start_update_polling(reg.clone());
            }
        });
    }

    fn wire_update_detection(reg: &ServiceWorkerRegistration, ctx: ServiceWorkerUpdate) {
        // If a controller already exists and the reg already has a waiting
        // worker (e.g. after a tab switch), surface the update immediately.
        if reg.waiting().is_some() && has_controller() {
            ctx.update_available.set(true);
        }

        let reg_clone = reg.clone();
        let cb = Closure::<dyn FnMut()>::new(move || {
            let Some(installing) = reg_clone.installing() else {
                return;
            };
            let ctx = ctx;
            let worker = installing.clone();
            let cb_state = Closure::<dyn FnMut()>::new(move || {
                // Fire exactly once per update, when the new worker has
                // finished installing AND an old controller exists (i.e.
                // this is an update, not a first-time install).
                if worker.state() == ServiceWorkerState::Installed && has_controller() {
                    ctx.update_available.set(true);
                }
            });
            installing.set_onstatechange(Some(cb_state.as_ref().unchecked_ref()));
            cb_state.forget();
        });
        reg.set_onupdatefound(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
    }

    fn has_controller() -> bool {
        window()
            .and_then(|w| w.navigator().service_worker().controller())
            .is_some()
    }

    fn start_update_polling(reg: ServiceWorkerRegistration) {
        let Some(win) = window() else { return };

        // visibilitychange — check for updates when the tab regains focus.
        let reg_vc = reg.clone();
        let cb = Closure::<dyn FnMut(Event)>::new(move |_e: Event| {
            if let Some(doc) = window().and_then(|w| w.document())
                && doc.visibility_state() == web_sys::VisibilityState::Visible
            {
                let _ = reg_vc.update();
            }
        });
        if let Some(doc) = win.document() {
            let _ = doc
                .add_event_listener_with_callback("visibilitychange", cb.as_ref().unchecked_ref());
        }
        cb.forget();

        // 60-second poll while visible.
        let reg_poll = reg.clone();
        let cb = Closure::<dyn FnMut()>::new(move || {
            if let Some(doc) = window().and_then(|w| w.document())
                && doc.visibility_state() == web_sys::VisibilityState::Visible
            {
                let _ = reg_poll.update();
            }
        });
        let _ = win.set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            60_000,
        );
        cb.forget();
    }

    pub(super) fn trigger_install_prompt() {
        let Some(ctx) = use_context::<InstallPromptEvent>() else {
            return;
        };
        let Some(evt) = ctx.event.get_untracked() else {
            return;
        };

        let prompt_fn = js_sys::Reflect::get(&evt, &JsValue::from_str("prompt"))
            .ok()
            .and_then(|v| v.dyn_into::<js_sys::Function>().ok());
        let Some(prompt_fn) = prompt_fn else { return };

        // Fire the prompt. We don't need to await the `userChoice` promise
        // for basic functionality — the `appinstalled` event already covers
        // the "accepted" side.
        let _ = prompt_fn.call0(&evt);

        // Clear — `prompt()` may only be called once per event.
        ctx.event.set(None);
    }

    pub(super) fn apply_update() {
        let Some(win) = window() else { return };
        let container = win.navigator().service_worker();
        let Ok(ready_promise) = container.ready() else {
            return;
        };

        spawn_local(async move {
            let Ok(reg_val) = JsFuture::from(ready_promise).await else {
                return;
            };
            let Ok(reg) = reg_val.dyn_into::<ServiceWorkerRegistration>() else {
                return;
            };

            if let Some(waiting) = reg.waiting() {
                let msg = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &msg,
                    &JsValue::from_str("type"),
                    &JsValue::from_str("SKIP_WAITING"),
                );
                let _ = waiting.post_message(&msg);
            }

            if let Some(w) = window() {
                let _ = w.location().reload();
            }
        });
    }
}
