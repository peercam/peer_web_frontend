//! Mobile deep-link redirect hook.
//!
//! Detects mobile devices and attempts to redirect guests to the native app.

use leptos::prelude::*;

/// Detect mobile and attempt app deep link redirect for guest users.
///
/// On Android/iOS, attempts to open `peer://post/{id}` deep link.
/// This is a fire-and-forget redirect attempt - if the app isn't installed,
/// the browser will simply ignore the deep link.
///
/// # Arguments
///
/// * `post_id` - The post ID to deep link to
/// * `is_guest` - Whether the user is a guest (only redirects for guests)
pub fn use_mobile_redirect(post_id: Signal<String>, is_guest: bool) {
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            if !is_guest {
                return;
            }

            let id = post_id.get();
            if id.is_empty() {
                return;
            }

            let Some(window) = web_sys::window() else {
                return;
            };

            let ua = window
                .navigator()
                .user_agent()
                .unwrap_or_default()
                .to_lowercase();

            let is_android = ua.contains("android");
            let is_ios = ua.contains("iphone") || ua.contains("ipad") || ua.contains("ipod");

            if is_android || is_ios {
                let deep_link = format!("peer://post/{}", id);
                let _ = window.location().set_href(&deep_link);
            }
        });
    }

    // No-op on server side
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (post_id, is_guest);
    }
}
