//! Leptos server functions for the authentication flow.
//!
//! These functions bridge client-side WASM components and the backend
//! GraphQL API for login, token refresh, and logout.

use leptos::prelude::*;

use crate::models::auth::{AuthPayload, LogoutPayload};

// ============================================================================
// Server function: Login
// ============================================================================

/// Login user with email and password.
///
/// On success, the response contains access and refresh JWT tokens.
/// The caller is responsible for storing them (via cookies).
#[server(Login, "/api")]
pub async fn login(email: String, password: String) -> Result<AuthPayload, ServerFnError> {
    use crate::api::graphql::{mutate, LoginData, LOGIN_MUTATION};

    // Basic server-side validation
    if email.is_empty() || password.is_empty() {
        return Err(ServerFnError::new("Email and password are required."));
    }

    if !email.contains('@') || !email.contains('.') {
        return Err(ServerFnError::new("Invalid email format."));
    }

    #[derive(serde::Serialize)]
    struct Vars {
        email: String,
        password: String,
    }

    let variables = Vars { email, password };

    let data: LoginData = mutate(LOGIN_MUTATION, variables, None).await?;

    // Set HttpOnly cookies on success (server-side)
    if data.login.is_success() {
        if let (Some(access), Some(refresh)) =
            (&data.login.access_token, &data.login.refresh_token)
        {
            set_auth_cookies_ssr(access, refresh);
        }
    }

    Ok(data.login)
}

// ============================================================================
// Server function: Refresh Token
// ============================================================================

/// Refresh access token using a refresh token.
///
/// Reads the refresh token from the request cookie if not provided explicitly.
#[server(RefreshAccessToken, "/api")]
pub async fn refresh_access_token() -> Result<AuthPayload, ServerFnError> {
    use crate::api::graphql::{mutate, RefreshTokenData, REFRESH_TOKEN_MUTATION};

    let refresh_token = get_cookie_ssr("refresh_token")
        .ok_or_else(|| ServerFnError::new("No refresh token found."))?;

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Vars {
        refresh_token: String,
    }

    let variables = Vars { refresh_token };

    let data: RefreshTokenData = mutate(REFRESH_TOKEN_MUTATION, variables, None).await?;

    // Update cookies on success
    if data.refresh_token.is_success() {
        if let (Some(access), Some(refresh)) = (
            &data.refresh_token.access_token,
            &data.refresh_token.refresh_token,
        ) {
            set_auth_cookies_ssr(access, refresh);
        }
    }

    Ok(data.refresh_token)
}

// ============================================================================
// Server function: Logout
// ============================================================================

/// Logout and invalidate the refresh token.
#[server(LogoutUser, "/api")]
pub async fn logout_user() -> Result<LogoutPayload, ServerFnError> {
    use crate::api::graphql::{mutate, LogoutData, LOGOUT_MUTATION};

    let refresh_token = get_cookie_ssr("refresh_token")
        .ok_or_else(|| ServerFnError::new("No refresh token found."))?;

    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Vars {
        refresh_token: String,
    }

    let variables = Vars { refresh_token };

    let data: LogoutData = mutate(LOGOUT_MUTATION, variables, None).await?;

    // Clear auth cookies
    clear_auth_cookies_ssr();

    Ok(data.logout)
}

// ============================================================================
// Server function: Check session validity
// ============================================================================

/// Check if the current session has a valid refresh token.
///
/// Returns true if a refresh token cookie exists (tokens are validated
/// server-side when actually used).
#[server(CheckSession, "/api")]
pub async fn check_session() -> Result<bool, ServerFnError> {
    Ok(get_cookie_ssr("refresh_token").is_some())
}

// ============================================================================
// SSR Cookie Helpers
// ============================================================================

/// Read a cookie value from the incoming request headers.
#[cfg(feature = "ssr")]
fn get_cookie_ssr(name: &str) -> Option<String> {
    use http::request::Parts;

    let parts = use_context::<Parts>()?;
    parts
        .headers
        .get_all(http::header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(|s| s.trim())
        .find_map(|cookie| {
            let mut kv = cookie.splitn(2, '=');
            let key = kv.next()?.trim();
            let val = kv.next()?.trim();
            if key == name { Some(val.to_string()) } else { None }
        })
}

/// Set auth cookies via the response headers.
#[cfg(feature = "ssr")]
fn set_auth_cookies_ssr(access_token: &str, refresh_token: &str) {
    let response = expect_context::<leptos_axum::ResponseOptions>();

    // Use Secure flag in production (when not localhost)
    let secure_flag = if std::env::var("LEPTOS_ENV").as_deref() == Ok("production") {
        "; Secure"
    } else {
        ""
    };

    // Access token: short-lived (15 minutes)
    response.append_header(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&format!(
            "access_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}{}",
            access_token,
            15 * 60,
            secure_flag
        ))
        .expect("valid header value"),
    );

    // Refresh token: longer-lived (30 days)
    response.append_header(
        http::header::SET_COOKIE,
        http::HeaderValue::from_str(&format!(
            "refresh_token={}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}{}",
            refresh_token,
            30 * 24 * 60 * 60,
            secure_flag
        ))
        .expect("valid header value"),
    );
}

/// Clear all auth-related cookies.
#[cfg(feature = "ssr")]
fn clear_auth_cookies_ssr() {
    let response = expect_context::<leptos_axum::ResponseOptions>();

    // Use Secure flag in production (when not localhost)
    let secure_flag = if std::env::var("LEPTOS_ENV").as_deref() == Ok("production") {
        "; Secure"
    } else {
        ""
    };

    for name in &["access_token", "refresh_token"] {
        response.append_header(
            http::header::SET_COOKIE,
            http::HeaderValue::from_str(&format!(
                "{}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0{}",
                name,
                secure_flag
            ))
            .expect("valid header value"),
        );
    }
}
