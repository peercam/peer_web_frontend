# Phase 1: Login & Session Flows

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** [Phase 0 — Project Skeleton & Parity](./phase-0-mock-backend-skeleton.md)
> **Goal:** Add all authentication and account management mutations so the Leptos login page, token refresh, logout, password reset, and contact-us flows work end-to-end against the mock.
> **Status:** ✅ Complete (14 April 2026)
> **Plan Quality:** ⭐⭐⭐⭐⭐ (5/5)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Prerequisites](#2-prerequisites)
3. [Task Breakdown](#3-task-breakdown)
4. [Implementation Details](#4-implementation-details)
5. [Testing Strategy](#5-testing-strategy)
6. [Definition of Done](#6-definition-of-done)

---

## 1. Overview

### What the frontend calls today

From `src/api/auth.rs` and `src/api/graphql.rs`, the Leptos app uses three core auth mutations:

| Mutation | GraphQL SDL | Frontend file |
|----------|-------------|---------------|
| `login(email, password)` | `mutation Login($email: String!, $password: String!) { login(...) { status ResponseCode accessToken refreshToken } }` | `auth.rs` → `Login` server fn |
| `refreshToken(refreshToken)` | `mutation RefreshToken($refreshToken: String!) { refreshToken(...) { status ResponseCode accessToken refreshToken } }` | `auth.rs` → `RefreshAccessToken` server fn |
| `logout(refreshToken)` | `mutation Logout($refreshToken: String!) { logout(...) { status ResponseCode } }` | `auth.rs` → `LogoutUser` server fn |

From `src/api/settings.rs` and `src/api/graphql.rs`:

| Mutation | GraphQL SDL | Frontend file |
|----------|-------------|---------------|
| `deleteAccount(password)` | `mutation DeleteAccount($password: String!) { deleteAccount(...) { status ResponseCode } }` | `settings.rs` → `delete_account` server fn |
| `updatePassword(password, expassword)` | `mutation UpdatePassword($password: String!, $expassword: String!) { updatePassword(...) { status ResponseCode } }` | `settings.rs` → `update_password` server fn |

From `docs/backend_api/01-authentication-and-account.md` (not yet wired in frontend but part of the backend schema):

| Mutation | Purpose | Guest? |
|----------|---------|--------|
| `requestPasswordReset(email)` | Initiate password reset email flow | Yes |
| `resetPasswordTokenVerify(token)` | Validate a password reset token | Yes |
| `resetPassword(token, password)` | Set new password via reset token | Yes |
| `contactus(name, email, message)` | Submit contact form | Yes |

### Target State

After this phase, the mock backend will support:
- Full login → token refresh → logout lifecycle
- Authenticated mutation guard (rejects unauthenticated calls with `60501`)
- Password change (authenticated) and password reset (guest) flows
- Account deletion (soft-delete, prevents re-login)
- Contact form submission
- 9 new GraphQL mutations + state extensions to `MockState`

### New file tree additions

```
packages/mock_backend/src/
├── schema/
│   └── mutation/
│       └── auth.rs          # NEW: 9 auth/account mutations
├── types/
│   └── auth.rs              # NEW: AuthPayload, LogoutPayload, etc.
└── state.rs                 # MODIFIED: new fields for tokens, passwords, etc.
```

---

## 2. Prerequisites

### Phase 0 Completion

- [ ] All 7 parity tests pass (`cargo test`)
- [ ] `MockState` has `known_referrals`, `registered_emails`, `verified_users`
- [ ] `DefaultResponse` type exists in `types/registration.rs`
- [ ] `MutationRoot` uses `MergedObject` and can accept additional mutation structs
- [ ] `app()` and `app_with_state()` router builders exist in `lib.rs`
- [ ] `/reset` endpoint resets state to defaults

### API Reference

All response codes and field names in this plan come from:
- `docs/backend_api/01-authentication-and-account.md`
- `src/models/auth.rs` (frontend deserialization types)

---

## 3. Task Breakdown

### Phase 1.A — Auth Types (`types/auth.rs`)

| # | Task | Notes |
|---|------|-------|
| A1 | Create `types/auth.rs` with `AuthPayload` struct | Must match frontend's `AuthPayload` deserialization exactly |
| A2 | Define `LogoutPayload` struct | `status` + `ResponseCode` only |
| A3 | Define `ResetPasswordRequestResponse` struct | Includes `nextAttemptAt` field |
| A4 | Define `ContactusResponse` and `ContactusResponsePayload` structs | Nested `affectedRows` |
| A5 | Export new types from `types/mod.rs` | Add `pub mod auth;` |

### Phase 1.B — State Extensions (`state.rs`, `seed.rs`)

| # | Task | Notes |
|---|------|-------|
| B1 | Add `User` struct to state (if not already present) | `uid`, `email`, `username`, `slug`, `password`, `verified`, `status`, `role` |
| B2 | Add `users: HashMap<Uuid, User>` to `MockState` | Central user store |
| B3 | Add `user_passwords: HashMap<Uuid, String>` to `MockState` | Plaintext passwords (mock only, never do this in production) |
| B4 | Add `access_tokens: HashMap<String, Uuid>` to `MockState` | Token → user mapping |
| B5 | Add `refresh_tokens: HashMap<String, Uuid>` to `MockState` | Token → user mapping |
| B6 | Add `password_reset_tokens: HashMap<String, Uuid>` to `MockState` | Token → user mapping |
| B7 | Add `deleted_users: HashSet<Uuid>` to `MockState` | Soft-deleted account tracking |
| B8 | Add `contact_messages: Vec<ContactMessage>` to `MockState` | Store contact form submissions |
| B9 | Update `MockState::reset()` to clear all new fields | Preserve seed data only |
| B10 | Update `MockState::default()` seed data | Add 2 pre-populated users (one verified, one unverified) with known credentials |
| B11 | Refactor `register` mutation to also insert into `users` and `user_passwords` | Wire up existing registration to create full user records |

### Phase 1.C — Auth Resolvers (`schema/mutation/auth.rs`)

| # | Task | Notes |
|---|------|-------|
| C1 | Create `schema/mutation/auth.rs` with `AuthMutation` struct | Uses `#[Object]` |
| C2 | Implement `login(email, password)` resolver | Lookup by email → verify password → check verified → check not deleted → generate tokens |
| C3 | Implement `refresh_token(refresh_token)` resolver | Validate token in map → issue new pair → remove old tokens |
| C4 | Implement `logout(refresh_token)` resolver | Remove tokens from both maps |
| C5 | Implement `delete_account(password)` resolver | Requires auth → verify password → soft-delete (status=6) → invalidate tokens |
| C6 | Implement `request_password_reset(email)` resolver | Generate token → store mapping → always return success (anti-enumeration) |
| C7 | Implement `reset_password_token_verify(token)` resolver | Check `password_reset_tokens` map |
| C8 | Implement `reset_password(token, password)` resolver | Validate token → update password → clear all tokens for user |
| C9 | Implement `update_password(password, expassword)` resolver | Requires auth → verify old → update → clear tokens |
| C10 | Implement `contactus(name, email, message)` resolver | Store message → return success |

### Phase 1.D — Auth Middleware & Guard

| # | Task | Notes |
|---|------|-------|
| D1 | Create auth context extractor function | Reads `Authorization: Bearer <token>` from request headers |
| D2 | Inject current user ID into async-graphql context | Via custom `Request` extension or async-graphql data |
| D3 | Create `get_current_user()` helper for resolvers | Returns `Option<Uuid>` from context |
| D4 | Create `require_auth()` helper that returns error response | Returns `60501` if not authenticated |
| D5 | Update `graphql_handler` in `lib.rs` | Extract auth header and add to async-graphql request data |

### Phase 1.E — Schema Assembly

| # | Task | Notes |
|---|------|-------|
| E1 | Add `pub mod auth;` to `schema/mutation/mod.rs` | Register the new module |
| E2 | Add `AuthMutation` to `MutationRoot` merged object | `MutationRoot(RegistrationMutation, AuthMutation)` |

### Phase 1.F — Integration Tests

| # | Task | Notes |
|---|------|-------|
| F1 | Add `graphql_with_auth()` test helper | Sends `Authorization: Bearer <token>` header |
| F2 | Test: Register → verify → login success | Full happy path, end-to-end |
| F3 | Test: Login with wrong password → `30801` | |
| F4 | Test: Login with unverified account → `60801` | |
| F5 | Test: Login with deleted account → `30801` | |
| F6 | Test: Refresh token success → new tokens | |
| F7 | Test: Refresh with invalid token → `30901` | |
| F8 | Test: Logout invalidates tokens (refresh fails after) | |
| F9 | Test: Delete account → re-login fails | |
| F10 | Test: Password reset flow (request → verify → reset → login with new) | |
| F11 | Test: Reset with invalid token → `31904` | |
| F12 | Test: Change password (auth'd) → login with new password | |
| F13 | Test: Change password with wrong old password → error | |
| F14 | Test: Contactus returns `10401` | |
| F15 | Test: Protected mutation without auth → `60501` | |
| F16 | Test: Login with seeded user credentials | Uses pre-populated seed data |

### Phase 1.G — Cleanup & Validation

| # | Task | Notes |
|---|------|-------|
| G1 | Run `cargo clippy -- -D warnings` | Fix all warnings |
| G2 | Run `cargo fmt --check` | Fix formatting |
| G3 | Run full test suite (`cargo test --all-targets`) | All Phase 0 + Phase 1 tests pass |
| G4 | Manual smoke test with `cargo run` + curl | Verify HTTP layer works |

---

## 4. Implementation Details

### 4.1 Auth Types (`types/auth.rs`)

```rust
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

/// Response for login and refreshToken mutations.
///
/// Must match the frontend's deserialization in `src/models/auth.rs`:
/// ```
/// AuthPayload { status, ResponseCode, accessToken, refreshToken }
/// ```
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AuthPayload {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "accessToken")]
    pub access_token: Option<String>,
    #[graphql(name = "refreshToken")]
    pub refresh_token: Option<String>,
}

impl AuthPayload {
    pub fn success(code: &str, access: &str, refresh: &str) -> Self {
        Self {
            status: "success".to_string(),
            response_code: Some(code.to_string()),
            access_token: Some(access.to_string()),
            refresh_token: Some(refresh.to_string()),
        }
    }

    pub fn error(code: &str) -> Self {
        Self {
            status: "error".to_string(),
            response_code: Some(code.to_string()),
            access_token: None,
            refresh_token: None,
        }
    }
}

/// Response for logout mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct LogoutPayload {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
}

/// Response for requestPasswordReset mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ResetPasswordRequestResponse {
    pub meta: DefaultResponse,
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "nextAttemptAt")]
    pub next_attempt_at: Option<String>,
}

/// Response for contactus mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ContactusResponse {
    pub meta: DefaultResponse,
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ContactusResponsePayload>,
}

/// Payload within ContactusResponse.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ContactusResponsePayload {
    pub msgid: String,
    pub email: String,
    pub name: String,
    pub message: String,
    pub ip: String,
    pub createdat: String,
}
```

### 4.2 User Struct & State Extensions (`state.rs`)

```rust
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type SharedState = Arc<RwLock<MockState>>;

/// A mock user record
#[derive(Debug, Clone)]
pub struct User {
    pub uid: Uuid,
    pub email: String,
    pub username: String,
    pub slug: String,
    pub role: u32,      // 0 = user, 16 = admin, 256 = moderator
    pub status: u32,    // 0 = active, 6 = deleted
}

/// Contact message record
#[derive(Debug, Clone)]
pub struct ContactMessage {
    pub name: String,
    pub email: String,
    pub message: String,
}

/// In-memory mock backend state
#[derive(Debug, Clone)]
pub struct MockState {
    // --- Phase 0 fields ---
    pub known_referrals: HashSet<Uuid>,
    pub registered_emails: HashSet<String>,
    pub verified_users: HashSet<Uuid>,

    // --- Phase 1 fields ---
    pub users: HashMap<Uuid, User>,
    pub user_passwords: HashMap<Uuid, String>,       // uid → plaintext (mock only!)
    pub access_tokens: HashMap<String, Uuid>,         // token → uid
    pub refresh_tokens: HashMap<String, Uuid>,        // token → uid
    pub password_reset_tokens: HashMap<String, Uuid>, // token → uid
    pub deleted_users: HashSet<Uuid>,
    pub contact_messages: Vec<ContactMessage>,
}

impl MockState {
    /// Reset all mutable state for test isolation.
    /// Preserves seed data (known_referrals, seeded users).
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Look up a user by email address.
    pub fn find_user_by_email(&self, email: &str) -> Option<&User> {
        self.users.values().find(|u| u.email == email)
    }

    /// Invalidate all tokens belonging to a user.
    pub fn invalidate_user_tokens(&mut self, uid: &Uuid) {
        self.access_tokens.retain(|_, v| v != uid);
        self.refresh_tokens.retain(|_, v| v != uid);
    }
}
```

### 4.3 Seed Data Extensions (`seed.rs`)

```rust
use std::collections::{HashMap, HashSet};
use uuid::{uuid, Uuid};

use crate::state::{ContactMessage, MockState, User};

// --- Existing Phase 0 constants ---
pub const REFERRAL_PRIMARY: Uuid = uuid!("85d5f836-b1f5-4c4e-9381-1b058e13df93");
pub const REFERRAL_SECONDARY: Uuid = uuid!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");

// --- Phase 1 seed users ---
pub const SEED_USER_VERIFIED: Uuid = uuid!("00000000-0000-4000-a000-000000000001");
pub const SEED_USER_UNVERIFIED: Uuid = uuid!("00000000-0000-4000-a000-000000000002");

/// Pre-known credentials for seeded test users
pub mod credentials {
    /// Verified user: test@peer.com / TestPass123
    pub const VERIFIED_EMAIL: &str = "test@peer.com";
    pub const VERIFIED_PASSWORD: &str = "TestPass123";
    pub const VERIFIED_USERNAME: &str = "peerTester";

    /// Unverified user: unverified@peer.com / TestPass456
    pub const UNVERIFIED_EMAIL: &str = "unverified@peer.com";
    pub const UNVERIFIED_PASSWORD: &str = "TestPass456";
    pub const UNVERIFIED_USERNAME: &str = "newSignup";
}

impl Default for MockState {
    fn default() -> Self {
        use credentials::*;

        let mut users = HashMap::new();
        let mut user_passwords = HashMap::new();
        let mut registered_emails = HashSet::new();
        let mut verified_users = HashSet::new();

        // Seed verified user
        users.insert(SEED_USER_VERIFIED, User {
            uid: SEED_USER_VERIFIED,
            email: VERIFIED_EMAIL.to_string(),
            username: VERIFIED_USERNAME.to_string(),
            slug: "peertester".to_string(),
            role: 0,
            status: 0,
        });
        user_passwords.insert(SEED_USER_VERIFIED, VERIFIED_PASSWORD.to_string());
        registered_emails.insert(VERIFIED_EMAIL.to_string());
        verified_users.insert(SEED_USER_VERIFIED);

        // Seed unverified user
        users.insert(SEED_USER_UNVERIFIED, User {
            uid: SEED_USER_UNVERIFIED,
            email: UNVERIFIED_EMAIL.to_string(),
            username: UNVERIFIED_USERNAME.to_string(),
            slug: "newsignup".to_string(),
            role: 0,
            status: 0,
        });
        user_passwords.insert(SEED_USER_UNVERIFIED, UNVERIFIED_PASSWORD.to_string());
        registered_emails.insert(UNVERIFIED_EMAIL.to_string());
        // Note: SEED_USER_UNVERIFIED is NOT added to verified_users

        Self {
            known_referrals: HashSet::from([REFERRAL_PRIMARY, REFERRAL_SECONDARY]),
            registered_emails,
            verified_users,
            users,
            user_passwords,
            access_tokens: HashMap::new(),
            refresh_tokens: HashMap::new(),
            password_reset_tokens: HashMap::new(),
            deleted_users: HashSet::new(),
            contact_messages: Vec::new(),
        }
    }
}

// --- Existing Phase 0 mock user for referral responses ---
pub mod mock_users {
    use async_graphql::ID;
    use crate::types::registration::ReferralUser;

    pub fn referral_user() -> ReferralUser {
        ReferralUser {
            uid: ID::from("usr_mock_001"),
            username: "peerTester".to_string(),
            slug: "peertester".to_string(),
            img: Some("https://via.placeholder.com/96".to_string()),
        }
    }
}
```

### 4.4 Registration Mutation Update

The existing `register` mutation in `schema/mutation/registration.rs` must be updated to also create a `User` record and store the password, so that login can work:

```rust
// In register() resolver — after generating userid, BEFORE returning:

// Create user record
state_write.users.insert(userid, User {
    uid: userid,
    email: input.email.clone(),
    username: input.username.clone(),
    slug: input.username.to_lowercase(),
    role: 0,
    status: 0,
});
state_write.user_passwords.insert(userid, input.password.clone());
```

### 4.5 Mock Token Generation

Tokens are deterministic strings — no real JWT signing. The mock validates by map lookup, not cryptographic verification.

```rust
/// Generate a mock access token.
/// Format: `mock-access-<user_uuid>-<timestamp_ms>`
fn generate_access_token(uid: &Uuid) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    format!("mock-access-{uid}-{ts}")
}

/// Generate a mock refresh token.
/// Format: `mock-refresh-<user_uuid>-<timestamp_ms>`
fn generate_refresh_token(uid: &Uuid) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    format!("mock-refresh-{uid}-{ts}")
}

/// Generate a mock password reset token.
/// Format: `mock-reset-<user_uuid>-<timestamp_ms>`
fn generate_reset_token(uid: &Uuid) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    format!("mock-reset-{uid}-{ts}")
}
```

### 4.6 Auth Context Extraction

The GraphQL handler must extract the `Authorization` header and make the current user available to resolvers:

```rust
// In lib.rs — new handler that passes auth context

use axum::http::HeaderMap;
use async_graphql::http::GraphiQLSource;

/// Optional current-user identifier injected by auth extraction
#[derive(Clone, Debug)]
pub struct CurrentUser(pub Option<Uuid>);

async fn graphql_handler(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    // Extract bearer token from Authorization header
    let current_user = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let mock_state = state.mock_state.read().await;
                mock_state.access_tokens.get(token).copied()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut request = req.into_inner();
    request = request.data(CurrentUser(current_user));
    state.schema.execute(request).await.into()
}
```

Update the router to use this handler instead of `GraphQL::new(schema)`:

```rust
Router::new()
    .route("/graphql", get(graphql_playground).post(graphql_handler))
    .route("/reset", post(reset_handler))
    .layer(cors)
    .with_state(app_state)
```

### 4.7 Auth Resolver Helpers

```rust
// In schema/mutation/auth.rs — helper functions

use crate::state::SharedState;

/// Extract the current authenticated user ID from the GraphQL context.
/// Returns None if no valid auth token was provided.
fn get_current_user(ctx: &Context<'_>) -> Option<Uuid> {
    ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0)
}

/// Require authentication. Returns Err with a 60501 DefaultResponse if not authenticated.
fn require_auth(ctx: &Context<'_>) -> Result<Uuid, DefaultResponse> {
    get_current_user(ctx).ok_or_else(|| {
        DefaultResponse::error("60501", "Authentication required")
    })
}
```

### 4.8 Auth Mutation Resolvers (`schema/mutation/auth.rs`)

```rust
use async_graphql::{Context, Object};
use uuid::Uuid;

use crate::state::SharedState;
use crate::types::auth::*;
use crate::types::registration::DefaultResponse;

pub struct AuthMutation;

#[Object]
impl AuthMutation {
    // ========================================================================
    // login
    // ========================================================================

    /// Authenticate with email and password. Returns JWT token pair on success.
    ///
    /// Response codes:
    /// - 10801: Login successful
    /// - 30801: Invalid credentials (wrong email, wrong password, deleted account)
    /// - 60801: Account not verified
    async fn login(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> AuthPayload {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Find user by email
        let user = match state_write.find_user_by_email(&email) {
            Some(u) => u.clone(),
            None => return AuthPayload::error("30801"),
        };

        // Check if account is deleted (status == 6)
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

    // ========================================================================
    // refreshToken
    // ========================================================================

    /// Exchange a valid refresh token for a new token pair.
    ///
    /// Response codes:
    /// - 10901: Token refreshed successfully
    /// - 30901: Invalid, expired, or revoked refresh token
    async fn refresh_token(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> AuthPayload {
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

    // ========================================================================
    // logout
    // ========================================================================

    /// Invalidate a refresh token (and associated access tokens).
    ///
    /// Response codes:
    /// - 11001: Logout successful
    async fn logout(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> LogoutPayload {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Remove the refresh token and its associated access tokens
        if let Some(uid) = state_write.refresh_tokens.remove(&refresh_token) {
            state_write.invalidate_user_tokens(&uid);
        }

        LogoutPayload {
            status: "success".to_string(),
            response_code: Some("11001".to_string()),
        }
    }

    // ========================================================================
    // deleteAccount
    // ========================================================================

    /// Soft-delete the authenticated user's account.
    ///
    /// Response codes:
    /// - 11012: Account deleted successfully
    /// - 60501: Not authenticated
    /// - 31001: Password does not match
    async fn delete_account(
        &self,
        ctx: &Context<'_>,
        password: String,
    ) -> DefaultResponse {
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

    // ========================================================================
    // requestPasswordReset
    // ========================================================================

    /// Initiate a password reset flow. Always returns success (anti-enumeration).
    ///
    /// Response codes:
    /// - 11901: Email sent (always, regardless of whether email exists)
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
                (chrono::Utc::now() + chrono::Duration::seconds(60))
                    .to_rfc3339()
            ),
        }
    }

    // ========================================================================
    // resetPasswordTokenVerify
    // ========================================================================

    /// Validate a password reset token.
    ///
    /// Response codes:
    /// - 11902: Token is valid
    /// - 31904: Invalid or expired reset token
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

    // ========================================================================
    // resetPassword
    // ========================================================================

    /// Set a new password using a valid reset token.
    ///
    /// Response codes:
    /// - 11005: Password updated successfully
    /// - 31904: Invalid or expired reset token
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

        // Invalidate all existing tokens for this user (security: force re-login)
        state_write.invalidate_user_tokens(&uid);

        // Clear any remaining reset tokens for this user
        state_write.password_reset_tokens.retain(|_, v| *v != uid);

        DefaultResponse::success("11005", "Password updated successfully")
    }

    // ========================================================================
    // updatePassword (change password — authenticated)
    // ========================================================================

    /// Change password for the authenticated user.
    /// Note: The backend names the params `password` (new) and `expassword` (old).
    ///
    /// Response codes:
    /// - 11001: Password changed successfully
    /// - 60501: Not authenticated
    /// - 31001: Old password does not match
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

        // Invalidate all existing tokens (force re-login with new password)
        state_write.invalidate_user_tokens(&uid);

        DefaultResponse::success("11001", "Password changed successfully")
    }

    // ========================================================================
    // contactus
    // ========================================================================

    /// Submit a contact form message. Guest-accessible.
    ///
    /// Response codes:
    /// - 10401: Message sent successfully
    async fn contactus(
        &self,
        ctx: &Context<'_>,
        name: String,
        email: String,
        message: String,
    ) -> ContactusResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        state_write.contact_messages.push(ContactMessage {
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
```

### 4.9 Schema Assembly Update (`schema/mod.rs`)

```rust
pub mod mutation;
pub mod query;

use async_graphql::{EmptySubscription, MergedObject, Schema};

use crate::state::SharedState;
use mutation::auth::AuthMutation;
use mutation::registration::RegistrationMutation;
use query::QueryRoot;

/// Combined mutation root — extends as new phases are added
#[derive(MergedObject, Default)]
pub struct MutationRoot(pub RegistrationMutation, pub AuthMutation);

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn build_schema(state: SharedState) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
        .data(state)
        .finish()
}
```

### 4.10 Router Update (`lib.rs`)

Replace the `GraphQL::new(schema)` handler with the custom `graphql_handler` from §4.6:

```rust
use axum::http::HeaderMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct CurrentUser(pub Option<Uuid>);

async fn graphql_handler(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    let current_user = extract_bearer_user(&state, &headers).await;
    let mut request = req.into_inner();
    request = request.data(CurrentUser(current_user));
    state.schema.execute(request).await.into()
}

async fn extract_bearer_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Option<Uuid> {
    let auth_header = headers.get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    let token = auth_str.strip_prefix("Bearer ")?;
    let mock_state = state.mock_state.read().await;
    mock_state.access_tokens.get(token).copied()
}

// Router now uses the custom handler:
pub fn app() -> Router {
    // ...
    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/reset", post(reset_handler))
        .layer(cors)
        .with_state(app_state)
}
```

---

## 5. Testing Strategy

### 5.1 Test Harness Extensions

Add an authenticated request helper alongside the existing `graphql()` helper:

```rust
/// Send a GraphQL mutation with an Authorization: Bearer header
async fn graphql_with_auth(app: axum::Router, query: &str, token: &str) -> Value {
    let body = json!({ "query": query });

    let request = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body_bytes).unwrap()
}

/// Helper: register → verify → login, returning (access_token, refresh_token, userid)
async fn register_verify_login(
    state: SharedState,
    email: &str,
    password: &str,
    username: &str,
) -> (String, String, String) {
    // 1. Register
    let app = app_with_state(state.clone());
    let res = graphql_stateful(app, &format!(r#"
        mutation {{
            register(input: {{
                email: "{email}"
                password: "{password}"
                username: "{username}"
            }}) {{ status ResponseCode userid }}
        }}
    "#)).await;
    let userid = res["data"]["register"]["userid"].as_str().unwrap().to_string();

    // 2. Verify
    let app = app_with_state(state.clone());
    graphql_stateful(app, &format!(r#"
        mutation {{ verifyAccount(userid: "{userid}") {{ status ResponseCode }} }}
    "#)).await;

    // 3. Login
    let app = app_with_state(state.clone());
    let res = graphql_stateful(app, &format!(r#"
        mutation {{
            login(email: "{email}", password: "{password}") {{
                status ResponseCode accessToken refreshToken
            }}
        }}
    "#)).await;
    let login_data = &res["data"]["login"];
    let access = login_data["accessToken"].as_str().unwrap().to_string();
    let refresh = login_data["refreshToken"].as_str().unwrap().to_string();

    (access, refresh, userid)
}
```

### 5.2 Test Cases

All tests use `app_with_state(state.clone())` to share state across sequential operations within a single test.

| # | Test name | Scenario | Key assertions |
|---|-----------|----------|----------------|
| 1 | `test_login_seeded_user` | Login with seeded verified user (`test@peer.com` / `TestPass123`) | `status: "success"`, `ResponseCode: "10801"`, `accessToken` is non-null, `refreshToken` is non-null |
| 2 | `test_register_verify_login_flow` | Register new user → verify → login | End-to-end happy path succeeds, tokens returned |
| 3 | `test_login_wrong_password` | Login with correct email, wrong password | `ResponseCode: "30801"`, no tokens |
| 4 | `test_login_nonexistent_email` | Login with email not in system | `ResponseCode: "30801"`, no tokens |
| 5 | `test_login_unverified_account` | Login with seeded unverified user | `ResponseCode: "60801"`, no tokens |
| 6 | `test_login_deleted_account` | Delete account → attempt login | `ResponseCode: "30801"` |
| 7 | `test_refresh_token_success` | Login → refresh with returned refresh token | `ResponseCode: "10901"`, new `accessToken`, new `refreshToken` |
| 8 | `test_refresh_invalid_token` | Refresh with a made-up token | `ResponseCode: "30901"` |
| 9 | `test_refresh_after_logout` | Login → logout → refresh with old token | Refresh returns `30901` (token was invalidated) |
| 10 | `test_logout_success` | Login → logout | `ResponseCode: "11001"` |
| 11 | `test_delete_account_success` | Login → deleteAccount with correct password | `ResponseCode: "11012"`, subsequent login fails with `30801` |
| 12 | `test_delete_account_wrong_password` | Login → deleteAccount with wrong password | `ResponseCode: "31001"` |
| 13 | `test_delete_account_unauthenticated` | deleteAccount without auth header | `ResponseCode: "60501"` |
| 14 | `test_password_reset_flow` | requestPasswordReset → verify token → resetPassword → login with new password | Full happy path; old password no longer works |
| 15 | `test_reset_token_verify_invalid` | resetPasswordTokenVerify with bad token | `ResponseCode: "31904"` |
| 16 | `test_reset_password_invalid_token` | resetPassword with bad token | `ResponseCode: "31904"` |
| 17 | `test_reset_password_invalidates_sessions` | Login → reset password via token → old access token no longer works | Auth'd mutation fails after password reset |
| 18 | `test_update_password_success` | Login → updatePassword with correct old password | `ResponseCode: "11001"`, login works with new password, fails with old |
| 19 | `test_update_password_wrong_old` | Login → updatePassword with wrong old password | `ResponseCode: "31001"` |
| 20 | `test_update_password_unauthenticated` | updatePassword without auth | `ResponseCode: "60501"` |
| 21 | `test_contactus_success` | contactus(name, email, message) | `ResponseCode: "10401"`, `affectedRows.email` matches input |
| 22 | `test_request_password_reset_unknown_email` | requestPasswordReset with non-existent email | Still returns `ResponseCode: "11901"` (anti-enumeration) |

### 5.3 Example Test Implementations

```rust
#[tokio::test]
async fn test_login_seeded_user() {
    let state = default_shared_state();
    let app = app_with_state(state.clone());

    let res = graphql_stateful(app, r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                status
                ResponseCode
                accessToken
                refreshToken
            }
        }
    "#).await;

    let data = &res["data"]["login"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10801");
    assert!(data["accessToken"].is_string());
    assert!(data["refreshToken"].is_string());

    // Verify token format
    let access = data["accessToken"].as_str().unwrap();
    assert!(access.starts_with("mock-access-"), "Token should have mock prefix");
}

#[tokio::test]
async fn test_login_wrong_password() {
    let state = default_shared_state();
    let app = app_with_state(state.clone());

    let res = graphql_stateful(app, r#"
        mutation {
            login(email: "test@peer.com", password: "WrongPassword") {
                status
                ResponseCode
                accessToken
                refreshToken
            }
        }
    "#).await;

    let data = &res["data"]["login"];
    assert_eq!(data["status"], "error");
    assert_eq!(data["ResponseCode"], "30801");
    assert!(data["accessToken"].is_null());
    assert!(data["refreshToken"].is_null());
}

#[tokio::test]
async fn test_login_unverified() {
    let state = default_shared_state();
    let app = app_with_state(state.clone());

    let res = graphql_stateful(app, r#"
        mutation {
            login(email: "unverified@peer.com", password: "TestPass456") {
                status
                ResponseCode
                accessToken
            }
        }
    "#).await;

    let data = &res["data"]["login"];
    assert_eq!(data["ResponseCode"], "60801");
    assert!(data["accessToken"].is_null());
}

#[tokio::test]
async fn test_refresh_then_old_token_invalid() {
    let state = default_shared_state();

    // Login
    let app1 = app_with_state(state.clone());
    let res = graphql_stateful(app1, r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken refreshToken
            }
        }
    "#).await;
    let old_refresh = res["data"]["login"]["refreshToken"].as_str().unwrap().to_string();

    // Refresh
    let app2 = app_with_state(state.clone());
    let res = graphql_stateful(app2, &format!(r#"
        mutation {{
            refreshToken(refreshToken: "{}") {{
                status ResponseCode accessToken refreshToken
            }}
        }}
    "#, old_refresh)).await;

    assert_eq!(res["data"]["refreshToken"]["ResponseCode"], "10901");

    // Old refresh token should now be invalid
    let app3 = app_with_state(state.clone());
    let res = graphql_stateful(app3, &format!(r#"
        mutation {{
            refreshToken(refreshToken: "{}") {{
                ResponseCode
            }}
        }}
    "#, old_refresh)).await;

    assert_eq!(res["data"]["refreshToken"]["ResponseCode"], "30901");
}

#[tokio::test]
async fn test_delete_account_then_login_fails() {
    let state = default_shared_state();

    // Login
    let app1 = app_with_state(state.clone());
    let res = graphql_stateful(app1, r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                accessToken refreshToken
            }
        }
    "#).await;
    let access = res["data"]["login"]["accessToken"].as_str().unwrap().to_string();

    // Delete account
    let app2 = app_with_state(state.clone());
    let res = graphql_with_auth(app2, r#"
        mutation {
            deleteAccount(password: "TestPass123") {
                status ResponseCode ResponseMessage
            }
        }
    "#, &access).await;

    assert_eq!(res["data"]["deleteAccount"]["ResponseCode"], "11012");

    // Attempt login → should fail
    let app3 = app_with_state(state.clone());
    let res = graphql_stateful(app3, r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                ResponseCode
            }
        }
    "#).await;

    assert_eq!(res["data"]["login"]["ResponseCode"], "30801");
}

#[tokio::test]
async fn test_password_reset_full_flow() {
    let state = default_shared_state();

    // 1. Request password reset
    let app1 = app_with_state(state.clone());
    let res = graphql_stateful(app1, r#"
        mutation {
            requestPasswordReset(email: "test@peer.com") {
                status ResponseCode nextAttemptAt
            }
        }
    "#).await;
    assert_eq!(res["data"]["requestPasswordReset"]["ResponseCode"], "11901");

    // 2. Extract the reset token from state (test-only introspection)
    let token = {
        let st = state.read().await;
        st.password_reset_tokens
            .keys()
            .next()
            .cloned()
            .expect("Reset token should exist")
    };

    // 3. Verify token
    let app2 = app_with_state(state.clone());
    let res = graphql_stateful(app2, &format!(r#"
        mutation {{
            resetPasswordTokenVerify(token: "{}") {{
                status ResponseCode
            }}
        }}
    "#, token)).await;
    assert_eq!(res["data"]["resetPasswordTokenVerify"]["ResponseCode"], "11902");

    // 4. Reset password
    let app3 = app_with_state(state.clone());
    let res = graphql_stateful(app3, &format!(r#"
        mutation {{
            resetPassword(token: "{}", password: "NewPass789") {{
                status ResponseCode
            }}
        }}
    "#, token)).await;
    assert_eq!(res["data"]["resetPassword"]["ResponseCode"], "11005");

    // 5. Login with new password
    let app4 = app_with_state(state.clone());
    let res = graphql_stateful(app4, r#"
        mutation {
            login(email: "test@peer.com", password: "NewPass789") {
                ResponseCode accessToken
            }
        }
    "#).await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "10801");
    assert!(res["data"]["login"]["accessToken"].is_string());

    // 6. Old password should no longer work
    let app5 = app_with_state(state.clone());
    let res = graphql_stateful(app5, r#"
        mutation {
            login(email: "test@peer.com", password: "TestPass123") {
                ResponseCode
            }
        }
    "#).await;
    assert_eq!(res["data"]["login"]["ResponseCode"], "30801");
}

#[tokio::test]
async fn test_contactus_success() {
    let res = graphql(r#"
        mutation {
            contactus(name: "Alice", email: "alice@test.com", message: "Hello!") {
                status
                ResponseCode
                affectedRows { msgid email name message }
            }
        }
    "#).await;

    let data = &res["data"]["contactus"];
    assert_eq!(data["status"], "success");
    assert_eq!(data["ResponseCode"], "10401");
    assert_eq!(data["affectedRows"]["email"], "alice@test.com");
    assert_eq!(data["affectedRows"]["name"], "Alice");
    assert_eq!(data["affectedRows"]["message"], "Hello!");
}

#[tokio::test]
async fn test_protected_mutation_without_auth() {
    let res = graphql(r#"
        mutation {
            deleteAccount(password: "anything") {
                status ResponseCode ResponseMessage
            }
        }
    "#).await;

    let data = &res["data"]["deleteAccount"];
    assert_eq!(data["ResponseCode"], "60501");
}
```

---

## 6. Definition of Done

### Build & Test Gates

- [x] `cargo build` succeeds without warnings
- [x] `cargo test --all-targets` passes all Phase 0 tests (9) AND all Phase 1 tests (24 new) — **33 total**
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes

### Functional Requirements — Auth Mutations

- [x] `login(email, password)` returns `AuthPayload` with `accessToken` + `refreshToken` on success (`10801`)
- [x] `login` returns `30801` for wrong password, non-existent email, and deleted accounts
- [x] `login` returns `60801` for unverified accounts
- [x] `refreshToken(refreshToken)` returns new token pair (`10901`)
- [x] `refreshToken` returns `30901` for invalid/used token
- [x] `logout(refreshToken)` returns `11001` and invalidates tokens
- [x] `deleteAccount(password)` returns `11012` and soft-deletes user
- [x] `deleteAccount` returns `60501` when unauthenticated
- [x] `deleteAccount` returns `31001` when password is wrong

### Functional Requirements — Password Reset

- [x] `requestPasswordReset(email)` always returns `11901` (anti-enumeration)
- [x] `requestPasswordReset` generates a token for existing users (no token for unknown emails)
- [x] `resetPasswordTokenVerify(token)` returns `11902` for valid tokens, `31904` for invalid
- [x] `resetPassword(token, password)` updates password and returns `11005`
- [x] `resetPassword` returns `31904` for invalid tokens
- [x] `resetPassword` invalidates all existing sessions for the user
- [x] `resetPassword` consumes the reset token (single-use)

### Functional Requirements — Password Change

- [x] `updatePassword(password, expassword)` changes password when old password matches (`11001`)
- [x] `updatePassword` returns `31001` when old password is wrong
- [x] `updatePassword` returns `60501` when unauthenticated
- [x] After password change, login works with new password and fails with old

### Functional Requirements — Contact

- [x] `contactus(name, email, message)` returns `10401` with payload

### Auth Middleware

- [x] `Authorization: Bearer <token>` header is parsed and resolved to a user ID
- [x] Authenticated user ID is available in resolver context via `CurrentUser`
- [x] Protected mutations return `60501` when no valid token is provided
- [x] Guest mutations (`login`, `refreshToken`, `logout`, `requestPasswordReset`, `resetPasswordTokenVerify`, `resetPassword`, `contactus`) work without authentication

### Response Shape Compatibility

- [x] `AuthPayload` fields: `status`, `ResponseCode` (PascalCase), `accessToken`, `refreshToken` (camelCase)
- [x] `LogoutPayload` fields: `status`, `ResponseCode`
- [x] `DefaultResponse` fields: `status`, `RequestId`, `ResponseCode`, `ResponseMessage` (all PascalCase)
- [x] `ResetPasswordRequestResponse` includes `nextAttemptAt`
- [x] `ContactusResponse` includes nested `affectedRows` with `ContactusResponsePayload`

### State & Isolation

- [x] `POST /reset` resets all auth state (tokens, passwords, deleted users) back to seed defaults
- [x] Seeded verified user (`test@peer.com` / `TestPass123`) works for login out of the box
- [x] Seeded unverified user (`unverified@peer.com` / `TestPass456`) returns `60801` on login
- [x] Registering new users via Phase 0's `register` mutation creates records that `login` can authenticate

### Cross-Cutting (carried from parent plan)

- [x] No `unwrap()` in resolver paths — all errors return proper GraphQL responses
- [x] Every resolver has ≥1 success and ≥1 error integration test
- [x] Response codes match values in `docs/backend_api/01-authentication-and-account.md`
- [x] No runtime dependencies on Node.js, npm, or non-Rust tooling
