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

    // Check existing session on mount
    let session_check = Resource::new(|| (), |_| async move { check_session().await.ok() });

    Effect::new(move |_| {
        if let Some(Some(has_session)) = session_check.get() {
            is_authenticated.set(has_session);
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
