//! Login page component.
//!
//! Renders the full login page with:
//! - Left panel: phone mockup with image
//! - Right panel: login form with query-param messages
//!
//! ## Query parameters
//!
//! - `?message=unauthorized` → "You do not have access. Please log in to continue."
//! - `?message=sessionExpired` → "Your session has expired. Please log in again."
//! - `?message=mustLogin` → "Please log in to access your dashboard."
//! - `?message=walletAccessDenied` → "Please log in to access your wallet."

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_query_map;

use crate::components::left_panel::LeftPanel;
use crate::components::login_form::LoginForm;

#[cfg(feature = "hydrate")]
use crate::api::auth::refresh_access_token;
#[cfg(feature = "hydrate")]
use crate::utils::cookies::get_remember_me;

/// Map query parameter message keys to user-friendly display text.
fn query_message(key: &str) -> Option<&'static str> {
    match key {
        "unauthorized" => Some("You do not have access. Please log in to continue."),
        "sessionExpired" => Some("Your session has expired. Please log in again."),
        "mustLogin" => Some("Please log in to access your dashboard."),
        "walletAccessDenied" => Some("Please log in to access your wallet."),
        _ => None,
    }
}

/// The login page component.
#[component]
pub fn LoginPage() -> impl IntoView {
    let query = use_query_map();

    // Extract query param message
    let top_message = Memo::new(move |_| {
        let params = query.get();
        params
            .get("message")
            .and_then(|key| query_message(&key))
            .map(|s| s.to_string())
    });

    // Auto-login: attempt silent refresh on mount if remember_me is set
    let auto_login_loading = RwSignal::new(false);

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            if get_remember_me() {
                auto_login_loading.set(true);
                leptos::task::spawn_local(async move {
                    match refresh_access_token().await {
                        Ok(payload) if payload.is_success() => {
                            // Redirect to dashboard
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href("/dashboard");
                            }
                        }
                        _ => {
                            // Refresh failed — stay on login page
                            auto_login_loading.set(false);
                        }
                    }
                });
            }
        }
    });

    view! {
        <Title text="Peer Network - Login"/>
        <Meta name="description" content="Log in to your Peer Network account."/>

        // Skip navigation link
        <a href="#login-form" class="skip-link sr-only">
            "Skip to login form"
        </a>

        <div class="container large_font">
            // Left panel: phone mockup
            <LeftPanel
                image_src="/img/register.webp"
                image_alt="Login preview"
            />

            // Right panel: login form
            <div class="container_right">
                <div class="container_inner">
                    // Top message area (query param messages)
                    <div class="top_head_area">
                        <Show when=move || top_message.get().is_some()>
                            <div class="query-message info-message" role="status">
                                {move || top_message.get().unwrap_or_default()}
                            </div>
                        </Show>
                    </div>

                    // Center area: login form
                    <div class="center_area" id="login-form" tabindex="-1">
                        <div class="step-header">
                            <h2 class="x_large_font">
                                "Welcome back!"
                            </h2>
                            <p class="large_font">
                                "Log in to your account to continue."
                            </p>
                        </div>

                        <Show
                            when=move || !auto_login_loading.get()
                            fallback=|| view! {
                                <div class="auto-login-loading" role="status">
                                    <span class="spinner"></span>
                                    " Signing you in..."
                                </div>
                            }
                        >
                            <LoginForm />
                        </Show>
                    </div>

                    // Footer area
                    <div class="footer_area medium_font">
                        <p class="version version-number"></p>
                    </div>
                </div>
            </div>
        </div>
    }
}
