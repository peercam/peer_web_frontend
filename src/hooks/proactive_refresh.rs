//! Proactive token refresh hook.
//!
//! Automatically refreshes the access token before it expires to avoid
//! jarring UX interruptions from expired tokens.

#[cfg(feature = "hydrate")]
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::state::auth::use_auth;

/// Access token lifetime minus refresh buffer (14 minutes in milliseconds).
///
/// Access tokens last 15 minutes; we refresh 1 minute before expiry.
#[cfg(feature = "hydrate")]
const REFRESH_INTERVAL_MS: u32 = 14 * 60 * 1000;

/// Set up proactive token refresh when the user is authenticated.
///
/// This hook schedules a refresh just before the access token expires,
/// preventing mid-session authentication failures.
///
/// # Usage
///
/// Call this in your app root or a layout component:
///
/// ```rust,ignore
/// use crate::hooks::use_proactive_refresh;
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     use_proactive_refresh();
///     // ...
/// }
/// ```
pub fn use_proactive_refresh() {
    #[cfg(feature = "hydrate")]
    {
        let auth = use_auth();

        use std::sync::{Arc, Mutex};

        // Track the current timeout handle so we can cancel it
        let timeout_handle: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));
        let timeout_handle_clone = timeout_handle.clone();

        // Schedule refresh when authenticated
        Effect::new(move |_| {
            let is_auth = auth.is_authenticated.get();

            // Clear any existing timeout
            if let Ok(mut guard) = timeout_handle.lock() {
                if let Some(handle) = guard.take() {
                    if let Some(window) = web_sys::window() {
                        window.clear_timeout_with_handle(handle);
                    }
                }
            }

            if is_auth {
                schedule_refresh(auth, timeout_handle.clone());
            }
        });

        // Cleanup on unmount
        on_cleanup(move || {
            if let Ok(mut guard) = timeout_handle_clone.lock() {
                if let Some(handle) = guard.take() {
                    if let Some(window) = web_sys::window() {
                        window.clear_timeout_with_handle(handle);
                    }
                }
            }
        });
    }
}

/// Schedule the next token refresh.
#[cfg(feature = "hydrate")]
fn schedule_refresh(
    auth: crate::state::auth::AuthContext,
    timeout_handle: std::sync::Arc<std::sync::Mutex<Option<i32>>>,
) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let Some(window) = web_sys::window() else {
        return;
    };

    // Create callback for the refresh
    let callback = Closure::once(Box::new(move || {
        // Dispatch refresh action
        auth.refresh_action.dispatch(());

        // Schedule next refresh (if still authenticated, the effect will re-trigger)
        // The effect watching is_authenticated will handle rescheduling
    }) as Box<dyn FnOnce()>);

    // Schedule the timeout
    match window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        REFRESH_INTERVAL_MS as i32,
    ) {
        Ok(handle) => {
            if let Ok(mut guard) = timeout_handle.lock() {
                *guard = Some(handle);
            }
        }
        Err(e) => {
            leptos::logging::error!("Failed to schedule token refresh: {:?}", e);
        }
    }

    // Prevent the closure from being dropped
    callback.forget();
}
