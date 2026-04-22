//! Admin-specific header bar.

use leptos::prelude::*;

use crate::api::profile::get_profile;

/// Admin header with logo, logged-in user, and back link.
#[component]
pub fn AdminHeader() -> impl IntoView {
    let username = Resource::new(
        || (),
        |_| async move {
            get_profile(None, None)
                .await
                .ok()
                .map(|p| p.username)
                .unwrap_or_default()
        },
    );

    view! {
        <header class="site-header admin-header">
            <div class="inner-header">
                <div class="logo">
                    <a href="/admin" class="admin-logo-link">"Admin"</a>
                </div>
                <div class="loggedin">
                    <span class="loggedin-label">
                        <Suspense fallback=move || "Admin Dashboard">
                            {move || username.get().map(|name| {
                                if name.is_empty() {
                                    "Admin Dashboard".to_string()
                                } else {
                                    format!("Logged in as {}", name)
                                }
                            })}
                        </Suspense>
                    </span>
                </div>
                <a href="/dashboard" class="button btn-transparent back-to-user">
                    <span class="peer-icon peer-icon-arrow-left"></span>
                    " Back to user mode"
                </a>
            </div>
        </header>
    }
}
