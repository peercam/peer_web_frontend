//! Auth guard component for protected routes.
//!
//! Wraps child components and redirects to `/login` if the user
//! is not authenticated.

use leptos::prelude::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_location;

use crate::state::auth::use_auth;

/// Protects child routes by requiring authentication.
///
/// If the user is not authenticated, redirects to `/login` with
/// a `message=mustLogin` query parameter and preserves the original
/// path for redirect after login.
#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let auth = use_auth();
    let location = use_location();

    let redirect_url = Memo::new(move |_| {
        let path = location.pathname.get();
        if path == "/" || path == "/login" {
            "/login?message=mustLogin".to_string()
        } else {
            // Simple percent-encoding of path for the redirect parameter
            let encoded: String = path
                .chars()
                .map(|c| match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | '/' => {
                        c.to_string()
                    }
                    _ => format!("%{:02X}", c as u32),
                })
                .collect();
            format!("/login?message=mustLogin&redirect={}", encoded)
        }
    });

    // `children` is `ChildrenFn` (Fn) but the nested `Show` requires the inner
    // body to itself be `Fn`, which moves `children`. Wrap in `StoredValue` so
    // both layers can call it without ownership issues.
    let children = StoredValue::new(children);

    view! {
        // Avoid redirecting before the initial session check resolves —
        // otherwise authenticated users get bounced to /login on first paint.
        <Show
            when=move || auth.is_session_checked.get()
            fallback=|| view! { <div class="auth-guard-loading" role="status" aria-busy="true"></div> }
        >
            <Show
                when=move || auth.is_authenticated.get()
                fallback=move || view! { <Redirect path=redirect_url.get()/> }
            >
                {children.with_value(|c| c())}
            </Show>
        </Show>
    }
}
