use async_graphql::{Context, Object};
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

use crate::CurrentUser;
use crate::state::SharedState;
use crate::types::auth::*;
use crate::types::registration::DefaultResponse;

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a mock access token.
fn generate_access_token(uid: &Uuid) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    let seq = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mock-access-{uid}-{ts}-{seq}")
}

/// Generate a mock refresh token.
fn generate_refresh_token(uid: &Uuid) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    let seq = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mock-refresh-{uid}-{ts}-{seq}")
}

/// Generate a mock password reset token.
fn generate_reset_token(uid: &Uuid) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    let seq = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mock-reset-{uid}-{ts}-{seq}")
}

/// Extract the current authenticated user ID from the GraphQL context.
pub fn get_current_user(ctx: &Context<'_>) -> Option<Uuid> {
    ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0)
}

/// Require authentication. Returns Err with a 60501 DefaultResponse if not authenticated.
pub fn require_auth(ctx: &Context<'_>) -> Result<Uuid, DefaultResponse> {
    get_current_user(ctx).ok_or_else(|| DefaultResponse::error("60501", "Authentication required"))
}

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
    /// Authenticate with email and password. Returns JWT token pair on success.
    async fn login(&self, ctx: &Context<'_>, email: String, password: String) -> AuthPayload {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Find user by email
        let user = match state_write.find_user_by_email(&email) {
            Some(u) => u.clone(),
            None => return AuthPayload::error("30801"),
        };

        // Check if account is deleted
        if user.status == 6 || state_write.deleted_users.contains(&user.uid) {
            return AuthPayload::error("30801");
        }

        // Check if account is verified
        if !state_write.verified_users.contains(&user.uid) {
            return AuthPayload::error("60801");
        }

        // Verify password
        let stored_password = match state_write.user_passwords.get(&user.uid) {
            Some(p) => p.clone(),
            None => return AuthPayload::error("30801"),
        };

        if stored_password != password {
            return AuthPayload::error("30801");
        }

        // Generate tokens
        let access = generate_access_token(&user.uid);
        let refresh = generate_refresh_token(&user.uid);

        // Store tokens
        state_write.access_tokens.insert(access.clone(), user.uid);
        state_write.refresh_tokens.insert(refresh.clone(), user.uid);

        AuthPayload::success("10801", &access, &refresh)
    }

    /// Exchange a valid refresh token for a new token pair.
    async fn refresh_token(&self, ctx: &Context<'_>, refresh_token: String) -> AuthPayload {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Look up the refresh token
        let uid = match state_write.refresh_tokens.remove(&refresh_token) {
            Some(uid) => uid,
            None => return AuthPayload::error("30901"),
        };

        // Verify user still exists and is active
        if state_write.deleted_users.contains(&uid) {
            return AuthPayload::error("30901");
        }

        // Invalidate old access tokens for this user
        state_write.access_tokens.retain(|_, v| *v != uid);

        // Generate new token pair
        let new_access = generate_access_token(&uid);
        let new_refresh = generate_refresh_token(&uid);

        state_write.access_tokens.insert(new_access.clone(), uid);
        state_write.refresh_tokens.insert(new_refresh.clone(), uid);

        AuthPayload::success("10901", &new_access, &new_refresh)
    }

    /// Invalidate a refresh token (and associated access tokens).
    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> LogoutPayload {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        if let Some(uid) = state_write.refresh_tokens.remove(&refresh_token) {
            state_write.invalidate_user_tokens(&uid);
        }

        LogoutPayload {
            status: "success".to_string(),
            response_code: Some("11001".to_string()),
        }
    }

    /// Soft-delete the authenticated user's account.
    async fn delete_account(&self, ctx: &Context<'_>, password: String) -> DefaultResponse {
        let uid = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Verify password
        let stored = match state_write.user_passwords.get(&uid) {
            Some(p) => p.clone(),
            None => return DefaultResponse::error("31001", "Password does not match"),
        };

        if stored != password {
            return DefaultResponse::error("31001", "Password does not match");
        }

        // Soft-delete: mark status = 6, add to deleted set
        if let Some(user) = state_write.users.get_mut(&uid) {
            user.status = 6;
        }
        state_write.deleted_users.insert(uid);

        // Invalidate all tokens
        state_write.invalidate_user_tokens(&uid);

        DefaultResponse::success("11012", "Account deleted successfully")
    }

    /// Initiate a password reset flow. Always returns success (anti-enumeration).
    async fn request_password_reset(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> ResetPasswordRequestResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Only generate token if user exists (but always return success)
        if let Some(user) = state_write.find_user_by_email(&email).cloned() {
            let token = generate_reset_token(&user.uid);
            state_write.password_reset_tokens.insert(token, user.uid);
        }

        ResetPasswordRequestResponse {
            meta: DefaultResponse::success("11901", "Email sent if account exists"),
            status: "success".to_string(),
            response_code: Some("11901".to_string()),
            next_attempt_at: Some(
                (chrono::Utc::now() + chrono::Duration::seconds(60)).to_rfc3339(),
            ),
        }
    }

    /// Validate a password reset token.
    async fn reset_password_token_verify(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> DefaultResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        if state_read.password_reset_tokens.contains_key(&token) {
            DefaultResponse::success("11902", "Token is valid")
        } else {
            DefaultResponse::error("31904", "Invalid or expired reset token")
        }
    }

    /// Set a new password using a valid reset token.
    async fn reset_password(
        &self,
        ctx: &Context<'_>,
        token: String,
        password: String,
    ) -> DefaultResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Consume the token
        let uid = match state_write.password_reset_tokens.remove(&token) {
            Some(uid) => uid,
            None => return DefaultResponse::error("31904", "Invalid or expired reset token"),
        };

        // Update password
        state_write.user_passwords.insert(uid, password);

        // Invalidate all existing tokens for this user
        state_write.invalidate_user_tokens(&uid);

        // Clear any remaining reset tokens for this user
        state_write.password_reset_tokens.retain(|_, v| *v != uid);

        DefaultResponse::success("11005", "Password updated successfully")
    }

    /// Change password for the authenticated user.
    async fn update_password(
        &self,
        ctx: &Context<'_>,
        password: String,
        expassword: String,
    ) -> DefaultResponse {
        let uid = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Verify old password
        let stored = match state_write.user_passwords.get(&uid) {
            Some(p) => p.clone(),
            None => return DefaultResponse::error("31001", "Old password does not match"),
        };

        if stored != expassword {
            return DefaultResponse::error("31001", "Old password does not match");
        }

        // Update to new password
        state_write.user_passwords.insert(uid, password);

        // Invalidate all existing tokens
        state_write.invalidate_user_tokens(&uid);

        DefaultResponse::success("11001", "Password changed successfully")
    }

    /// Submit a contact form message. Guest-accessible.
    async fn contactus(
        &self,
        ctx: &Context<'_>,
        name: String,
        email: String,
        message: String,
    ) -> ContactusResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        state_write
            .contact_messages
            .push(crate::state::ContactMessage {
                name: name.clone(),
                email: email.clone(),
                message: message.clone(),
            });

        let ts = chrono::Utc::now().to_rfc3339();

        ContactusResponse {
            meta: DefaultResponse::success("10401", "Message sent successfully"),
            status: "success".to_string(),
            response_code: Some("10401".to_string()),
            affected_rows: Some(ContactusResponsePayload {
                msgid: "1".to_string(),
                email,
                name,
                message,
                ip: "127.0.0.1".to_string(),
                createdat: ts,
            }),
        }
    }
}
