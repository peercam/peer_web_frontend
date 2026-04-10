//! Authenticated API call wrapper with automatic token refresh.
//!
//! This module provides `auth_fetch` which wraps API calls and handles
//! 401 responses by refreshing the access token and retrying.

use std::future::Future;

use leptos::prelude::*;

use crate::api::auth::refresh_access_token;
use crate::models::common::ApiError;

/// Check if an error indicates an unauthorized (401) response.
pub fn is_unauthorized(error: &ServerFnError) -> bool {
    let msg = error.to_string().to_lowercase();
    msg.contains("401")
        || msg.contains("unauthorized")
        || msg.contains("expired")
        || msg.contains("invalid token")
}

/// Wrapper for authenticated API calls that handles token refresh on 401.
///
/// This function:
/// 1. Executes the provided async request
/// 2. If it fails with a 401/unauthorized error, refreshes the token
/// 3. Retries the original request once
///
/// # Example
///
/// ```rust,ignore
/// use crate::api::auth_fetch::auth_fetch;
///
/// let profile = auth_fetch(|| get_user_profile(user_id)).await?;
/// ```
pub async fn auth_fetch<T, F, Fut>(request: F) -> Result<T, ServerFnError>
where
    F: Fn() -> Fut + Clone,
    Fut: Future<Output = Result<T, ServerFnError>>,
{
    // First attempt
    match request().await {
        Ok(result) => Ok(result),
        Err(e) if is_unauthorized(&e) => {
            // Attempt to refresh the token
            match refresh_access_token().await {
                Ok(payload) if payload.is_success() => {
                    // Token refreshed, retry the original request
                    request().await
                }
                Ok(_) => {
                    // Refresh returned but wasn't successful
                    Err(ServerFnError::new("Session expired. Please log in again."))
                }
                Err(refresh_err) => {
                    // Refresh failed - return original error with context
                    leptos::logging::warn!(
                        "Token refresh failed during 401 recovery: {:?}",
                        refresh_err
                    );
                    Err(ServerFnError::new("Session expired. Please log in again."))
                }
            }
        }
        Err(e) => Err(e),
    }
}

/// Version of auth_fetch that takes an ApiError instead of ServerFnError.
///
/// Useful when working directly with GraphQL utilities.
pub async fn auth_fetch_api<T, F, Fut>(request: F) -> Result<T, ApiError>
where
    F: Fn() -> Fut + Clone,
    Fut: Future<Output = Result<T, ApiError>>,
{
    // First attempt
    match request().await {
        Ok(result) => Ok(result),
        Err(e) if is_unauthorized_api(&e) => {
            // Attempt to refresh the token
            match refresh_access_token().await {
                Ok(payload) if payload.is_success() => {
                    // Token refreshed, retry the original request
                    request().await
                }
                Ok(_) => Err(ApiError::Unauthorized("Session expired. Please log in again.".into())),
                Err(e) => {
                    leptos::logging::warn!("Token refresh failed: {:?}", e);
                    Err(ApiError::Unauthorized("Session expired. Please log in again.".into()))
                }
            }
        }
        Err(e) => Err(e),
    }
}

/// Check if an ApiError indicates unauthorized access.
fn is_unauthorized_api(error: &ApiError) -> bool {
    matches!(error, ApiError::Unauthorized(_))
        || error.to_string().to_lowercase().contains("401")
        || error.to_string().to_lowercase().contains("unauthorized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_unauthorized() {
        assert!(is_unauthorized(&ServerFnError::new("401 Unauthorized")));
        assert!(is_unauthorized(&ServerFnError::new("HTTP 401")));
        assert!(is_unauthorized(&ServerFnError::new("Token expired")));
        assert!(is_unauthorized(&ServerFnError::new("invalid token")));
        assert!(!is_unauthorized(&ServerFnError::new("Internal server error")));
        assert!(!is_unauthorized(&ServerFnError::new("Network error")));
    }
}
