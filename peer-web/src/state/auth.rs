//! Global authentication state and context provider.
//!
//! Provides `AuthContext` at the application root so any component
//! can check authentication status and trigger login/logout/refresh.

use leptos::prelude::*;

use crate::api::auth::{check_session, login, logout_user, refresh_access_token};
use crate::models::auth::AuthPayload;

/// Global auth context available to all components.
#[derive(Clone, Copy)]
pub struct AuthContext {
    /// Whether the user is currently authenticated.
    pub is_authenticated: RwSignal<bool>,
    /// Whether the initial session check has resolved.
    /// Components should avoid making auth-based redirect decisions until
    /// this is `true`, otherwise authenticated users will briefly be
    /// treated as unauthenticated on first paint.
    pub is_session_checked: RwSignal<bool>,
    /// The login action — dispatched with (email, password).
    pub login_action: Action<(String, String), Result<AuthPayload, ServerFnError>>,
    /// The logout action.
    pub logout_action: Action<(), Result<(), ServerFnError>>,
    /// The refresh action — refreshes the access token silently.
    pub refresh_action: Action<(), Result<AuthPayload, ServerFnError>>,
}

/// Provide the auth context at the application root.
///
/// Call this once in `App` or a top-level provider component.
pub fn provide_auth_context() {
    let is_authenticated = RwSignal::new(false);
    let is_session_checked = RwSignal::new(false);

    // Check existing session on mount
    let session_check = Resource::new(|| (), |_| async move { check_session().await.ok() });

    Effect::new(move |_| {
        if let Some(maybe_session) = session_check.get() {
            if let Some(has_session) = maybe_session {
                is_authenticated.set(has_session);
            }
            is_session_checked.set(true);
        }
    });

    // Login action
    let login_action = Action::new(move |args: &(String, String)| {
        let (email, password) = args.clone();
        async move { login(email, password).await }
    });

    // Handle login results
    Effect::new(move |_| {
        if let Some(Ok(payload)) = login_action.value().get() {
            is_authenticated.set(payload.is_success());
            is_session_checked.set(true);
        }
    });

    // Logout action
    let logout_action = Action::new(move |_: &()| async move {
        let result = logout_user().await;
        if result.is_ok() {
            is_authenticated.set(false);
        }
        result.map(|_| ())
    });

    // Refresh action
    let refresh_action = Action::new(move |_: &()| async move {
        let result = refresh_access_token().await;
        if let Ok(ref payload) = result {
            is_authenticated.set(payload.is_success());
        }
        result
    });

    let ctx = AuthContext {
        is_authenticated,
        is_session_checked,
        login_action,
        logout_action,
        refresh_action,
    };

    provide_context(ctx);
}

/// Get the auth context from the nearest provider.
pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}

/// Percent-encode a value for use as a `redirect=` query parameter.
///
/// Uses an unreserved-character allow-list (`A–Z a–z 0–9 - _ . ~ /`); every
/// other byte of the UTF-8 encoding is percent-encoded as `%XX`. Shared by
/// `AuthGuard` and `HomePage` so the two redirect paths can never drift in
/// their encoding behaviour.
pub(crate) fn encode_redirect(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for &b in value.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char);
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(out, "%{:02X}", b);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::encode_redirect;

    #[test]
    fn ascii_unreserved_passes_through() {
        assert_eq!(encode_redirect("/wallet"), "/wallet");
        assert_eq!(encode_redirect("/u/test-user_1.0~"), "/u/test-user_1.0~");
    }

    #[test]
    fn ascii_reserved_is_percent_encoded() {
        assert_eq!(encode_redirect("/a b"), "/a%20b");
        assert_eq!(encode_redirect("/?x=1&y=2"), "/%3Fx%3D1%26y%3D2");
    }

    #[test]
    fn non_ascii_uses_utf8_bytes() {
        // U+00E9 'é' = 0xC3 0xA9
        assert_eq!(encode_redirect("/é"), "/%C3%A9");
        // U+1F600 '😀' = 0xF0 0x9F 0x98 0x80
        assert_eq!(encode_redirect("/😀"), "/%F0%9F%98%80");
    }
}
