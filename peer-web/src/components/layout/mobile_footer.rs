//! Route-aware mobile navigation footer.
//!
//! Replaces ten private `fn MobileFooter` copies and one inline
//! `<footer class="mobile-footer">` block (in `peer_shop.rs`). The
//! canonical nav follows the Dashboard preset (Home / Search / New Post
//! / Alerts / Profile) — see Open Question 1 in
//! [`docs/plans/layout/layout-shell-implementation.md`].
//!
//! The `active` class is computed reactively against
//! [`leptos_router::hooks::use_location`] so SSR (request URI) and
//! post-hydration (`window.location`) produce identical markup. The
//! precedent for this pattern is
//! [`crate::components::widgets::main_menu`].

use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// Shared mobile navigation footer.
#[component]
pub fn MobileFooter() -> impl IntoView {
    let location = use_location();

    // Helpers — return a reactive closure so `class:active` re-evaluates
    // after hydration without baking the SSR path into the markup.
    let is_exact = move |target: &'static str| {
        let path = location.pathname.get();
        path == target
    };
    let is_prefix = move |target: &'static str| {
        let path = location.pathname.get();
        path == target || path.starts_with(&format!("{target}/"))
    };

    let active_dashboard = move || is_exact("/dashboard");
    let active_search = move || is_prefix("/search");
    let active_alerts = move || is_prefix("/notifications");
    let active_profile = move || is_prefix("/profile");

    view! {
        <footer class="mobile-footer">
            <nav class="mobile-nav">
                <a href="/dashboard" class="nav-item" class:active=active_dashboard>
                    <i class="peer-icon peer-icon-home"/>
                    <span>"Home"</span>
                </a>
                <a href="/search" class="nav-item" class:active=active_search>
                    <i class="peer-icon peer-icon-search"/>
                    <span>"Search"</span>
                </a>
                <a href="/newpost" class="nav-item add-post">
                    <i class="peer-icon peer-icon-plus"/>
                </a>
                <a href="/notifications" class="nav-item" class:active=active_alerts>
                    <i class="peer-icon peer-icon-bell"/>
                    <span>"Alerts"</span>
                </a>
                <a href="/profile" class="nav-item" class:active=active_profile>
                    <i class="peer-icon peer-icon-user"/>
                    <span>"Profile"</span>
                </a>
            </nav>
        </footer>
    }
}
