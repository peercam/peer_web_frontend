//! PWA install banner + iOS hint + update-available toast.
//!
//! Single-file module per convention (matches `toast.rs`, `auth_guard.rs`,
//! `referral.rs`). Mount `<InstallPrompt/>` once near the root of `App`.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::components::toast::{ToastType, use_toast};
use crate::utils::pwa::{
    InstallPromptEvent, ServiceWorkerUpdate, apply_service_worker_update, trigger_install_prompt,
};

/// LocalStorage key for "Not now" dismissal (14-day cooldown).
const DISMISS_KEY: &str = "peer.pwaDismissedAt";
/// LocalStorage key for "Never" dismissal (permanent).
const NEVER_KEY: &str = "peer.pwaNever";
/// LocalStorage key to suppress the iOS Add-to-Home-Screen hint after first show.
const IOS_HINT_KEY: &str = "peer.pwaIosHintShown";
/// Dismissal cooldown — 14 days, in milliseconds.
const DISMISS_COOLDOWN_MS: f64 = 14.0 * 24.0 * 60.0 * 60.0 * 1000.0;

/// Routes on which the install prompt is permitted to appear.
const ELIGIBLE_PATHS: &[&str] = &["/dashboard", "/profile"];

/// Mount once near the root. Renders the install banner, the iOS hint,
/// and the "update available" banner.
#[component]
pub fn InstallPrompt() -> impl IntoView {
    view! {
        <InstallBanner/>
        <IosHint/>
        <UpdateBanner/>
    }
}

#[component]
fn UpdateBanner() -> impl IntoView {
    let Some(update) = use_context::<ServiceWorkerUpdate>() else {
        return ().into_any();
    };
    let toast = use_toast();
    let dismissed = RwSignal::new(false);

    // Surface a toast the first time the update lands — users who ignore the
    // banner still see a passing notification.
    Effect::new(move |prev: Option<bool>| {
        let now_flag = update.update_available.get();
        if now_flag && prev != Some(true) {
            toast.show("A new version is available.", ToastType::Info);
        }
        now_flag
    });

    let visible = Memo::new(move |_| update.update_available.get() && !dismissed.get());

    let on_update = move |_| apply_service_worker_update();
    let on_dismiss = move |_| dismissed.set(true);

    view! {
        <Show when=move || visible.get() fallback=|| ()>
            <div class="pwa-update-banner" role="status" aria-live="polite">
                <div class="pwa-update-banner__inner">
                    <span class="pwa-update-banner__text">
                        "A new version of Peer is available."
                    </span>
                    <button
                        type="button"
                        class="pwa-update-banner__btn pwa-update-banner__btn--primary"
                        on:click=on_update
                    >
                        "Refresh"
                    </button>
                    <button
                        type="button"
                        class="pwa-update-banner__btn"
                        on:click=on_dismiss
                        aria-label="Dismiss"
                    >
                        "×"
                    </button>
                </div>
            </div>
        </Show>
    }
    .into_any()
}

#[component]
fn InstallBanner() -> impl IntoView {
    let ctx = use_context::<InstallPromptEvent>();
    let location = use_location();

    let dismissed_signal = RwSignal::new(false);

    // Visible when:
    //   - we captured a `beforeinstallprompt` event
    //   - the user hasn't dismissed recently / permanently
    //   - the current route is eligible
    //   - the app isn't already installed
    let is_visible = Memo::new(move |_| {
        let Some(ctx) = ctx else { return false };
        if dismissed_signal.get() {
            return false;
        }
        if ctx.installed.get() {
            return false;
        }

        #[cfg(feature = "hydrate")]
        let has_event = ctx.event.get().is_some();
        #[cfg(not(feature = "hydrate"))]
        let has_event = {
            // SSR: `ctx.event` doesn't exist; suppress unused-binding warning.
            let _ = ctx;
            false
        };
        if !has_event {
            return false;
        }

        let path = location.pathname.get();
        if !ELIGIBLE_PATHS.iter().any(|p| path == *p) {
            return false;
        }

        !is_dismissed()
    });

    let on_install = move |_| {
        trigger_install_prompt();
        dismissed_signal.set(true);
    };
    let on_later = move |_| {
        set_dismissed_now();
        dismissed_signal.set(true);
    };
    let on_never = move |_| {
        set_never();
        dismissed_signal.set(true);
    };

    view! {
        <Show when=move || is_visible.get() fallback=|| ()>
            <div
                class="pwa-install-banner"
                role="dialog"
                aria-labelledby="pwa-install-title"
                aria-describedby="pwa-install-body"
            >
                <div class="pwa-install-banner__inner">
                    <img
                        class="pwa-install-banner__icon"
                        src="/img/pwa/icon-192.png"
                        alt=""
                        width="48"
                        height="48"
                    />
                    <div class="pwa-install-banner__text">
                        <h2 id="pwa-install-title" class="pwa-install-banner__title">
                            "Install Peer"
                        </h2>
                        <p id="pwa-install-body" class="pwa-install-banner__body">
                            "Add Peer to your home screen for a faster, full-screen experience."
                        </p>
                    </div>
                    <div class="pwa-install-banner__actions">
                        <button
                            type="button"
                            class="pwa-install-banner__btn pwa-install-banner__btn--primary"
                            on:click=on_install
                        >
                            "Install"
                        </button>
                        <button
                            type="button"
                            class="pwa-install-banner__btn"
                            on:click=on_later
                        >
                            "Not now"
                        </button>
                        <button
                            type="button"
                            class="pwa-install-banner__btn pwa-install-banner__btn--subtle"
                            on:click=on_never
                            aria-label="Never show this prompt again"
                        >
                            "Never"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn IosHint() -> impl IntoView {
    let dismissed = RwSignal::new(false);

    let visible = Memo::new(move |_| {
        if dismissed.get() {
            return false;
        }
        is_ios_standalone_candidate() && !is_ios_hint_shown()
    });

    let on_dismiss = move |_| {
        mark_ios_hint_shown();
        dismissed.set(true);
    };

    view! {
        <Show when=move || visible.get() fallback=|| ()>
            <div class="pwa-ios-hint" role="status" aria-live="polite">
                <div class="pwa-ios-hint__inner">
                    <p class="pwa-ios-hint__body">
                        "Install Peer: tap "
                        <span aria-hidden="true" class="pwa-ios-hint__share">"⬆"</span>
                        " Share, then "
                        <strong>"Add to Home Screen"</strong>"."
                    </p>
                    <button
                        type="button"
                        class="pwa-ios-hint__close"
                        on:click=on_dismiss
                        aria-label="Dismiss"
                    >
                        "×"
                    </button>
                </div>
            </div>
        </Show>
    }
}

// ---- helpers -------------------------------------------------------------

fn is_dismissed() -> bool {
    if local_get(NEVER_KEY).is_some() {
        return true;
    }
    let Some(raw) = local_get(DISMISS_KEY) else {
        return false;
    };
    let Ok(stamp) = raw.parse::<f64>() else {
        return false;
    };
    let now = now_ms();
    now - stamp < DISMISS_COOLDOWN_MS
}

fn set_dismissed_now() {
    let now = now_ms();
    local_set(DISMISS_KEY, &format!("{}", now as u64));
}

fn set_never() {
    local_set(NEVER_KEY, "1");
}

fn is_ios_hint_shown() -> bool {
    local_get(IOS_HINT_KEY).is_some()
}

fn mark_ios_hint_shown() {
    local_set(IOS_HINT_KEY, "1");
}

fn is_ios_standalone_candidate() -> bool {
    #[cfg(feature = "hydrate")]
    {
        let Some(win) = leptos::web_sys::window() else {
            return false;
        };
        // Already running in standalone mode? Don't hint.
        if let Ok(mm) = win.match_media("(display-mode: standalone)")
            && let Some(mm) = mm
            && mm.matches()
        {
            return false;
        }
        // iOS also exposes the legacy `navigator.standalone` — treat truthy as standalone.
        let standalone = js_sys::Reflect::get(
            &win.navigator(),
            &wasm_bindgen::JsValue::from_str("standalone"),
        )
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
        if standalone {
            return false;
        }
        let ua = win.navigator().user_agent().unwrap_or_default();
        ua.contains("iPhone") || ua.contains("iPad") || ua.contains("iPod")
    }
    #[cfg(not(feature = "hydrate"))]
    {
        false
    }
}

fn now_ms() -> f64 {
    #[cfg(feature = "hydrate")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(feature = "hydrate"))]
    {
        0.0
    }
}

fn local_get(key: &str) -> Option<String> {
    #[cfg(feature = "hydrate")]
    {
        let win = leptos::web_sys::window()?;
        let storage = win.local_storage().ok().flatten()?;
        storage.get_item(key).ok().flatten()
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = key;
        None
    }
}

fn local_set(key: &str, value: &str) {
    #[cfg(feature = "hydrate")]
    {
        let Some(win) = leptos::web_sys::window() else {
            return;
        };
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (key, value);
    }
}
