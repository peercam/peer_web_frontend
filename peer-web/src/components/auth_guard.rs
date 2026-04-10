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

    view! {
        <Show
            when=move || auth.is_authenticated.get()
            fallback=move || view! { <Redirect path=redirect_url.get()/> }
        >
            {children()}
        </Show>
    }
}
