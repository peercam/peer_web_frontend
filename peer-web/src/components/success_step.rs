//! Registration Step 3: Success confirmation screen.
//!
//! Shown after a successful registration + account verification.
//! Stores the new user's email in sessionStorage and provides
//! a link to the login page.

use leptos::prelude::*;

/// Props for the SuccessStep component.
#[component]
pub fn SuccessStep(
    /// The email address the user registered with.
    /// Used to persist to sessionStorage for login auto-fill.
    email: Signal<String>,
) -> impl IntoView {
    // On mount: persist email to sessionStorage for login page auto-fill
    Effect::new(move |_| {
        let email_val = email.get();
        if !email_val.is_empty() {
            store_email_in_session_storage(&email_val);
        }
    });

    view! {
        <div class="success-message">
            <div class="step-header">
                <span class="icon" aria-hidden="true">
                    <i class="peer-icon peer-icon-good-tick-circle"></i>
                </span>
                <h2 class="x_large_font">
                    "Welcome to " <strong>"peer!"</strong>
                </h2>
                <p class="large_font">
                    "Your account is ready! Start exploring and earn your first token today."
                </p>
            </div>

            <a class="btn btn-primary" href="/login">
                "Continue to Login"
            </a>
        </div>
    }
}

/// Persist the registered email to sessionStorage.
///
/// The login page reads `newUserEmail` to auto-fill the email field,
/// providing a smoother post-registration experience.
fn store_email_in_session_storage(email: &str) {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.session_storage() {
                let _ = storage.set_item("newUserEmail", email);
            }
        }
    }
    // No-op on server side — sessionStorage is a browser-only API.
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = email;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_email_noop_on_server() {
        // On the server (non-hydrate), this should be a no-op and not panic.
        store_email_in_session_storage("test@example.com");
    }
}
