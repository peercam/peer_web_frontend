//! Invite page — deep-link landing for referral links.
//!
//! Accessed via `/invite?referralUuid=...`. Attempts to open the native
//! Peer app, then falls back to the appropriate app store or registration page.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

/// App store URLs.
#[cfg(feature = "hydrate")]
const ANDROID_STORE: &str = "https://play.google.com/store/apps/details?id=eu.peernetwork.app";
#[cfg(feature = "hydrate")]
const IOS_STORE: &str = "https://apps.apple.com/app/peer-network/id6744612499";

/// Platform detected from user agent.
#[cfg(feature = "hydrate")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Android,
    Ios,
    Desktop,
}

/// Invite page — deep-link redirect landing.
///
/// Reads `?referralUuid=` from the URL, stores it in `localStorage`,
/// attempts a `peer://invite/{uuid}` deep link, and falls back to
/// app store or registration after 1.5 seconds.
#[component]
pub fn InvitePage() -> impl IntoView {
    let query = use_query_map();

    let referral_uuid = Memo::new(move |_| {
        query
            .get()
            .get("referralUuid")
            .unwrap_or_default()
    });

    // Client-side deep link logic (hydrate-only)
    #[cfg(feature = "hydrate")]
    {
        use gloo_timers::callback::Timeout;
        use std::cell::Cell;
        use std::rc::Rc;
        use wasm_bindgen::{closure::Closure, JsCast};

        Effect::new(move |_| {
            let uuid = referral_uuid.get();
            if uuid.is_empty() {
                return;
            }

            let Some(window) = web_sys::window() else {
                return;
            };

            // Store in localStorage for later registration pickup
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("referralUuid", &uuid);
            }

            let platform = detect_platform(&window);
            let deep_link = format!("peer://invite/{}", uuid);

            // Navigate to deep link (triggers native app if installed)
            let _ = window.location().set_href(&deep_link);

            // Set fallback timeout — only desktop auto-redirects (matching legacy
            // where Android/iOS auto-redirect is commented out)
            let uuid_for_fallback = uuid.clone();
            let timeout_handle: Rc<Cell<Option<Timeout>>> = Rc::new(Cell::new(None));
            let timeout_handle_clone = timeout_handle.clone();

            let fallback = Timeout::new(1_500, move || {
                let Some(w) = web_sys::window() else {
                    return;
                };
                match platform {
                    // Legacy has Android/iOS auto-redirect commented out
                    Platform::Android | Platform::Ios => {}
                    Platform::Desktop => {
                        let origin = w.location().origin().unwrap_or_default();
                        let url = format!(
                            "{}/register?referralUuid={}",
                            origin, uuid_for_fallback
                        );
                        let _ = w.location().set_href(&url);
                    }
                }
            });
            timeout_handle.set(Some(fallback));

            // Cancel fallback if the app opens (browser goes hidden)
            if let Some(document) = window.document() {
                let closure = Closure::<dyn Fn()>::new(move || {
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if doc.hidden() {
                            // Take and drop the timeout to cancel it
                            let _ = timeout_handle_clone.take();
                        }
                    }
                });
                let _ = document.add_event_listener_with_callback(
                    "visibilitychange",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget(); // Intentional leak — page lifetime
            }
        });
    }

    // Manual fallback click handler
    #[cfg(feature = "hydrate")]
    let on_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        let uuid = referral_uuid.get();
        if uuid.is_empty() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let platform = detect_platform(&window);
        let deep_link = format!("peer://invite/{}", uuid);
        let origin = window.location().origin().unwrap_or_default();

        // Copy to clipboard first
        let text_to_copy = match platform {
            Platform::Android | Platform::Ios => &deep_link,
            Platform::Desktop => &uuid,
        };
        let _ = window.navigator().clipboard().write_text(text_to_copy);

        // Redirect to platform-appropriate URL
        match platform {
            Platform::Android => {
                let _ = window.location().set_href(ANDROID_STORE);
            }
            Platform::Ios => {
                let _ = window.location().set_href(IOS_STORE);
            }
            Platform::Desktop => {
                let url = format!("{}/register?referralUuid={}", origin, uuid);
                let _ = window.location().set_href(&url);
            }
        }
    };

    // SSR fallback: no-op click handler
    #[cfg(not(feature = "hydrate"))]
    let on_click = move |_ev: leptos::ev::MouseEvent| {
        let _ = &referral_uuid;
    };

    view! {
        <Title text="PeerNetwork Invite"/>
        <div class="invite-page">
            <p class="invite-prompt">
                "Having trouble opening the app?"
                <a href="#" class="invite-link" on:click=on_click>"Click Here"</a>
                " to continue manually."
            </p>
        </div>
    }
}

/// Detect platform from user agent.
#[cfg(feature = "hydrate")]
fn detect_platform(window: &web_sys::Window) -> Platform {
    let ua = window
        .navigator()
        .user_agent()
        .unwrap_or_default()
        .to_lowercase();

    if ua.contains("android") {
        Platform::Android
    } else if ua.contains("iphone") || ua.contains("ipad") || ua.contains("ipod") {
        Platform::Ios
    } else {
        Platform::Desktop
    }
}
