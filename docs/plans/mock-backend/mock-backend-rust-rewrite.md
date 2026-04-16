# Plan: Rewrite Mock Backend in Rust

> Implements ADR: [adr-mock-backend-rust-rewrite.md](../adr-mock-backend-rust-rewrite.md)

**Goal:** Replace the Node.js mock backend (`tests/mock_backend/`) with an idiomatic Rust crate built on async-graphql + Axum, achieving parity with the existing 3 mutations and then incrementally expanding to cover all 10 backend API domains documented in `docs/backend_api/`.

---

## Table of Contents

1. [Current State Inventory](#1-current-state-inventory)
2. [Phase 0 — Project Skeleton & Parity](#2-phase-0--project-skeleton--parity)
3. [Phase 1 — Login & Session Flows](#3-phase-1--login--session-flows)
4. [Phase 2 — Users & Profiles](#4-phase-2--users--profiles)
5. [Phase 3 — Posts & Content](#5-phase-3--posts--content)
6. [Phase 4 — Social (Comments, Chat)](#6-phase-4--social-comments-chat)
7. [Phase 5 — Economy (Wallet, Tokenomics, Shop, Ads)](#7-phase-5--economy-wallet-tokenomics-shop-ads)
8. [Phase 6 — Admin & Moderation](#8-phase-6--admin--moderation)
9. [CI Integration](#9-ci-integration)
10. [Acceptance Criteria](#10-acceptance-criteria)

---

## 1. Current State Inventory

### What exists today (Node.js)

| File | Purpose |
|------|---------|
| `server.js` | Express server, CORS middleware, `/reset` endpoint, `graphql-http` handler |
| `schema.graphql` | SDL: 3 mutations (`verifyReferralString`, `register`, `verifyAccount`), 1 query (`_health`), 5 types |
| `resolvers.js` | JS resolver functions with in-memory logic (~100 LOC) |
| `state.js` | Shared mutable state: `knownReferrals` (2 UUIDs), `registeredEmails`, `verifiedUsers` |
| `test.js` | 7 assertions via bespoke `gql()` + `assert()` runner |
| `fixtures/*.json` | 6 JSON fixture files for referral/register/verify scenarios |

### What the Leptos frontend already calls

From `peer-web/src/api/mod.rs`, the frontend has API modules for:

| Module | GraphQL operations used |
|--------|------------------------|
| `registration.rs` | `verifyReferralString`, `register`, `verifyAccount` |
| `auth.rs` | `login`, `refreshToken`, `logout` |
| `posts.rs` | `listPosts`, `listAdvertisementPosts`, `postAction`, `searchUsers`, `getUser` |
| `comments.rs` | `guestListPost`, `getPost`, `listComments`, `listChildComments`, `createComment`, `likeComment`, `unlikeComment` |
| `profile.rs` | `getProfile`, `listFollowRelations`, `listFriends`, `listUserPosts`, `toggleFollow`, `toggleBlock`, `reportUser` |
| `chat.rs` | `listChats`, `sendChatMessage`, `createChat` |
| `settings.rs` | `updateProfileImage`, `updateBio`, `updateUsername`, `updatePassword`, `updateEmail`, `updatePreferences`, `deleteAccount` |
| `wallet.rs` | `balance`, `transferTokens`, `getTransactionHistory` |

**Priority** for each phase is driven by which Leptos page is actively under development.

---

## 2. Phase 0 — Project Skeleton & Parity ✅

> **Status:** Complete (14 April 2026) — [Detailed plan](./phase-0-mock-backend-skeleton.md)

**Outcome:** A `cargo build`-able Rust crate that passes 9 integration tests (the original 7 parity tests plus health query and reset endpoint tests).

### Step 0.1 — Initialise crate

The Node.js files have been replaced with a Rust project:

```
tests/mock_backend/
├── Cargo.toml
├── README.md
├── fixtures/              # Kept from Node.js as reference
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── state.rs
│   ├── seed.rs
│   ├── types/
│   │   ├── mod.rs
│   │   └── registration.rs
│   └── schema/
│       ├── mod.rs
│       ├── query.rs
│       └── mutation/
│           ├── mod.rs
│           └── registration.rs
└── tests/
    └── integration.rs     # 9 tests
```

> **Note:** `error.rs` and `types/response.rs` from the original plan were not needed — types live in `types/registration.rs` and error handling is inline in resolvers.

**Cargo.toml dependencies (actual):**

```toml
[package]
name = "mock_backend"
version = "0.1.0"
edition = "2024"

[dependencies]
async-graphql = "7"           # v8 still RC at time of implementation
async-graphql-axum = "7"
axum = "0.8"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
regex = "1"

[dev-dependencies]
reqwest = { version = "0.13", features = ["json"] }
tokio-test = "0.4"
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
```

### Step 0.2 — Core types

Define shared response types that mirror the backend's GraphQL schema exactly (field names, casing):

```rust
// types/registration.rs

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct DefaultResponse {
    pub status: String,
    #[graphql(name = "RequestId")]
    pub request_id: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    #[graphql(name = "ResponseMessage")]
    pub response_message: String,
}

impl DefaultResponse {
    pub fn success(code: &str, message: &str) -> Self { /* ... */ }
    pub fn error(code: &str, message: &str) -> Self { /* ... */ }
}
```

> **Deviation:** `#[graphql(rename_fields = "PascalCase")]` was removed because it renamed
> `status` → `Status`, breaking GraphQL queries. Per-field `#[graphql(name)]` is used instead.

Additional types for Phase 0:

| Type | Fields | Used by |
|------|--------|---------|
| `ReferralUser` | `uid`, `username`, `slug`, `img` | `verifyReferralString` |
| `ReferralResponse` | `status`, `ResponseCode`, `affectedRows: Option<Vec<ReferralUser>>`, `meta` | `verifyReferralString` |
| `RegisterResponse` | `status`, `ResponseCode`, `userid: Option<String>`, `meta` | `register` |
| `VerifyAccountResponse` | `status`, `ResponseCode`, `meta` | `verifyAccount` |
| `RegistrationInput` (InputObject) | `email`, `password`, `username`, `pkey: Option<String>`, `referral_uuid: Option<ID>` | `register` |

### Step 0.3 — In-memory state

```rust
// state.rs
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<MockState>>;

pub struct MockState {
    pub known_referrals: HashSet<Uuid>,
    pub registered_emails: HashSet<String>,
    pub verified_users: HashSet<Uuid>,
    // Extended in later phases:
    // pub users: HashMap<Uuid, User>,
    // pub posts: Vec<Post>,
    // pub wallets: HashMap<Uuid, Balance>,
    // pub comments: Vec<Comment>,
    // pub chats: Vec<Chat>,
}

impl MockState {
    /// Reset state to defaults (for test isolation)
    pub fn reset(&mut self) {
        self.registered_emails.clear();
        self.verified_users.clear();
        // Note: known_referrals are NOT cleared — they're seed data
    }
}
```

### Step 0.4 — Seed data

```rust
// seed.rs
use uuid::Uuid;

/// Primary test referral (matches existing Node.js mock)
pub const REFERRAL_PRIMARY: Uuid = uuid!("85d5f836-b1f5-4c4e-9381-1b058e13df93");
/// Secondary test referral
pub const REFERRAL_SECONDARY: Uuid = uuid!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");

impl Default for MockState {
    fn default() -> Self {
        Self {
            known_referrals: HashSet::from([REFERRAL_PRIMARY, REFERRAL_SECONDARY]),
            registered_emails: HashSet::new(),
            verified_users: HashSet::new(),
        }
    }
}

/// Mock referral user data returned on successful verification
pub mod mock_users {
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

### Step 0.5 — Registration resolvers

Implement `schema/mutation/registration.rs` with three resolvers matching the current Node.js logic exactly:

| Resolver | Logic | Response codes |
|----------|-------|----------------|
| `verify_referral_string(referral_string)` | Validate UUID format → check `known_referrals` → return `ReferralResponse` | `11011` success, `31010` invalid |
| `register(input)` | Check `fail@` prefix (→ `40601`), check duplicate email (→ `30601`), otherwise create UUID and add to `registered_emails` (→ `10601`) | `10601`, `30601`, `40601` |
| `verify_account(userid)` | Check `verified_users` for already-verified (→ `30701`), otherwise add to set (→ `10701`) | `10701`, `30701` |

### Step 0.6 — Query root & health

```rust
// schema/query.rs
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[graphql(name = "_health")]
    async fn health(&self) -> bool {
        true
    }
}
```

> **Deviation:** async-graphql strips leading underscores from method names, so `#[graphql(name = "_health")]` is required.

### Step 0.7 — Schema assembly

```rust
// schema/mod.rs
use async_graphql::{Schema, EmptySubscription, MergedObject};

#[derive(MergedObject, Default)]
pub struct MutationRoot(RegistrationMutation);  // extended in later phases

pub fn build_schema(state: SharedState) -> Schema<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(QueryRoot, MutationRoot::default(), EmptySubscription)
        .data(state)
        .finish()
}
```

### Step 0.8 — Axum router (`lib.rs`)

```rust
pub fn app() -> Router {
    let mock_state: SharedState = Arc::new(RwLock::new(MockState::default()));
    let schema = build_schema(mock_state.clone());
    let app_state = AppState { schema: schema.clone(), mock_state };
    build_router(app_state)
}

pub fn app_with_state(mock_state: SharedState) -> Router {
    let schema = build_schema(mock_state.clone());
    let app_state = AppState { schema: schema.clone(), mock_state };
    build_router(app_state)
}
```

- `graphql_handler`: Explicit async fn extracting `State<AppState>` + `GraphQLRequest`, calls `schema.execute()`
- `reset_handler`: Acquires write lock, calls `mock_state.reset()` (clears emails/verified but preserves referral seeds)
- `app_with_state()`: Exposed for tests that need shared state across multiple requests

> **Deviation:** `GraphQL::new()` as a service didn't implement axum 0.8's `Handler` trait, so an explicit handler function is used. GraphQL endpoint is POST-only (sufficient for all use cases).

### Step 0.9 — Binary entry (`main.rs`)

```rust
#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".into());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Mock Peer backend running at http://localhost:{port}/graphql");
    axum::serve(listener, app()).await.unwrap();
}
```

### Step 0.10 — Integration tests

All 9 tests pass using `tower::ServiceExt::oneshot` (in-process, no port binding). Tests using shared state use a `graphql_stateful()` helper with `app_with_state()`.

| # | Test | Assert |
|---|------|--------|
| 1 | `test_valid_referral` | `status: "success"`, `ResponseCode: "11011"`, `affectedRows[0].uid == "usr_mock_001"` |
| 2 | `test_invalid_referral` | `status: "error"`, `ResponseCode: "31010"`, `affectedRows: null` |
| 3 | `test_register_success` | `status: "success"`, `ResponseCode: "10601"`, `userid` is valid UUID |
| 4 | `test_register_duplicate_email` | `status: "error"`, `ResponseCode: "30601"`, `userid: null` |
| 5 | `test_verify_account_success` | `status: "success"`, `ResponseCode: "10701"` |
| 6 | `test_already_verified` | `status: "success"`, `ResponseCode: "30701"` |
| 7 | `test_fail_email_error` | `status: "error"`, `ResponseCode: "40601"`, `userid: null` |
| 8 | `test_reset_endpoint` | POST /reset clears state, re-register succeeds |
| 9 | `test_health_query` | `_health` returns `true` |

### Step 0.11 — Wire into peer-web as dev-dependency

```toml
# peer-web/Cargo.toml
[dev-dependencies]
mock_backend = { path = "../../tests/mock_backend" }
```

### Step 0.12 — Remove Node.js files

Once all 7 tests pass in Rust and CI is green:

- Delete `server.js`, `resolvers.js`, `schema.graphql`, `state.js`, `test.js`, `package.json`, `package-lock.json`, `node_modules/`
- Keep `fixtures/` as reference (or migrate to Rust snapshot files)
- Update `README.md`

### Phase 0 definition of done

- [x] `cargo build` succeeds for the `mock_backend` crate (no warnings)
- [x] `cargo test` passes all 9 integration tests (in-process, no port binding)
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes
- [x] `cargo run` starts the HTTP server on `:4000` and serves GraphQL
- [x] `POST /reset` clears state (preserves seed referrals)
- [x] `POST /graphql` with the 3 mutations returns identical response shapes to the Node.js mock
- [x] Node.js files removed (fixtures/ kept as reference)
- [x] README.md updated with Rust instructions
- [x] `mock_backend::app()` and `build_schema()` exported as public API

---

## 3. Phase 1 — Login & Session Flows ✅

> **Status:** Complete (14 April 2026) — [Detailed plan](./phase-1-login-session-flows.md)

**Depends on:** Phase 0
**Driven by:** `peer-web/src/pages/login.rs`, `peer-web/src/api/auth.rs`

**Outcome:** 9 new auth/account mutations, auth middleware, 2 seeded users, 24 new integration tests (33 total).

### New files

| File | Contents |
|------|----------|
| `src/schema/mutation/auth.rs` | `login`, `refresh_token`, `logout`, `delete_account`, `request_password_reset`, `reset_password_token_verify`, `reset_password`, `update_password`, `contactus` |
| `src/types/auth.rs` | `AuthPayload`, `LogoutPayload`, `ResetPasswordRequestResponse`, `ContactusResponse`, `ContactusResponsePayload` |

### File tree after Phase 1

```
tests/mock_backend/src/
├── lib.rs               # Updated: CurrentUser struct, auth context extraction
├── main.rs
├── state.rs             # Updated: User, ContactMessage, token/password maps
├── seed.rs              # Updated: 2 seeded users with known credentials
├── types/
│   ├── mod.rs           # Updated: pub mod auth
│   ├── registration.rs
│   └── auth.rs          # NEW
└── schema/
    ├── mod.rs           # Updated: MutationRoot(RegistrationMutation, AuthMutation)
    ├── query.rs
    └── mutation/
        ├── mod.rs       # Updated: pub mod auth
        ├── registration.rs  # Updated: register creates User records
        └── auth.rs      # NEW: 9 auth mutations
```

### State extensions (actual)

```rust
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
```

### Seed data

Two pre-populated users are created in `MockState::default()`:

| User | Email | Password | Verified | UUID |
|------|-------|----------|----------|------|
| peerTester | `test@peer.com` | `TestPass123` | ✅ Yes | `00000000-0000-4000-a000-000000000001` |
| newSignup | `unverified@peer.com` | `TestPass456` | ❌ No | `00000000-0000-4000-a000-000000000002` |

### Auth middleware

Auth context is extracted in `lib.rs::graphql_handler`:
- Reads `Authorization: Bearer <token>` header
- Resolves token → user UUID via `access_tokens` map
- Injects `CurrentUser(Option<Uuid>)` into async-graphql context
- Resolvers use `get_current_user(ctx)` / `require_auth(ctx)` helpers

### Mock token strategy

Tokens are deterministic strings with atomic counter for uniqueness: `mock-access-<uuid>-<timestamp>-<seq>`. Validated by map lookup, not cryptographic verification.

### Resolvers implemented

| Mutation | Input | Key response codes |
|----------|-------|--------------------|
| `login(email, password)` | email + password | `10801` success, `30801` invalid creds, `60801` not verified |
| `refreshToken(refreshToken)` | refresh token string | `10901` success, `30901` invalid token |
| `logout(refreshToken)` | refresh token string | `11001` success |
| `deleteAccount(password)` | password (auth required) | `11012` success, `31001` wrong password, `60501` unauth |
| `requestPasswordReset(email)` | email | `11901` always (anti-enumeration) |
| `resetPasswordTokenVerify(token)` | token | `11902` valid, `31904` invalid |
| `resetPassword(token, password)` | token + new password | `11005` success, `31904` invalid token |
| `updatePassword(password, expassword)` | new + old (auth required) | `11001` success, `31001` wrong old, `60501` unauth |
| `contactus(name, email, message)` | name + email + message | `10401` success |

> **Deviation from plan:** The plan called the mutation `changePassword` but the frontend uses `updatePassword(password, expassword)`. The implementation matches the frontend's field names.

### Registration mutation update

The Phase 0 `register` mutation was updated to also create a `User` record and store the password in `user_passwords`, so that newly registered (and verified) users can log in.

### Tests (24 new, 33 total)

| # | Test | Assert |
|---|------|--------|
| 1 | `test_login_seeded_user` | `10801`, tokens returned |
| 2 | `test_register_verify_login_flow` | End-to-end happy path |
| 3 | `test_login_wrong_password` | `30801` |
| 4 | `test_login_nonexistent_email` | `30801` |
| 5 | `test_login_unverified_account` | `60801` |
| 6 | `test_login_deleted_account` | Delete → re-login → `30801` |
| 7 | `test_refresh_token_success` | `10901`, new tokens |
| 8 | `test_refresh_invalid_token` | `30901` |
| 9 | `test_refresh_after_logout` | Logout → refresh fails `30901` |
| 10 | `test_logout_success` | `11001` |
| 11 | `test_delete_account_success` | `11012`, re-login fails |
| 12 | `test_delete_account_wrong_password` | `31001` |
| 13 | `test_delete_account_unauthenticated` | `60501` |
| 14 | `test_password_reset_flow` | Full request → verify → reset → login new |
| 15 | `test_reset_token_verify_invalid` | `31904` |
| 16 | `test_reset_password_invalid_token` | `31904` |
| 17 | `test_reset_password_invalidates_sessions` | Old access token fails after reset |
| 18 | `test_update_password_success` | `11001`, new password works |
| 19 | `test_update_password_wrong_old` | `31001` |
| 20 | `test_update_password_unauthenticated` | `60501` |
| 21 | `test_contactus_success` | `10401` with payload |
| 22 | `test_request_password_reset_unknown_email` | `11901` (anti-enum), no token generated |
| 23 | `test_refresh_then_old_token_invalid` | Old refresh token consumed |
| 24 | `test_protected_mutation_without_auth` | `60501` |

### Phase 1 definition of done

- [x] All 9 auth mutations return correct response shapes
- [x] Auth middleware blocks unauthenticated calls to protected resolvers
- [x] Token lifecycle (issue → refresh → logout/invalidate) works end-to-end
- [x] 24 integration tests pass (exceeds ≥12 target)
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes
- [x] All Phase 0 tests still pass (33 total)

---

## 4. Phase 2 — Users & Profiles ✅

> **Status:** Complete (14 April 2026) — [Detailed plan](./phase-2-users-and-profiles.md) | [Implementation notes](./phase-2-implementation.md)

**Depends on:** Phase 1 (requires auth)
**Driven by:** `peer-web/src/pages/profile.rs`, `peer-web/src/pages/view_profile.rs`, `peer-web/src/api/profile.rs`

**Outcome:** 10 user queries, 8 profile mutations, content filtering, 6 seeded users, 40 new integration tests (73 total).

### New files

| File | Contents |
|------|----------|
| `src/types/user.rs` | 26+ GraphQL types/enums: `ProfileGql`, `ProfileUserGql`, `BasicUserInfoGql`, `SearchUserResult`, `FollowRelationsGql`, `BlockedUsersGql`, `UserInfoGql`, `UserPreferencesGql`, etc. |
| `src/schema/query/mod.rs` | Refactored `QueryRoot` using `MergedObject`: `QueryRoot(HealthQuery, UserQuery)` |
| `src/schema/query/health.rs` | Extracted `_health` query from old `query.rs` |
| `src/schema/query/users.rs` | `getProfile`, `searchUser`, `listUsersV2`, `getUser`, `listFollowRelations`, `listFriends`, `listBlockedUsers`, `getUserInfo`, `getReferralInfo`, `referralList` |
| `src/schema/mutation/profile.rs` | `toggleUserFollowStatus`, `toggleBlockUserStatus`, `reportUser`, `updateProfileImage`, `updateBio`, `updateUsername`, `updateEmail`, `updateUserPreferences` |
| `src/filters.rs` | `filter_users()` pipeline (illegal, system, deleted, blocked) + `paginate()` helper |

### State extensions (actual)

```rust
pub struct MockState {
    // ... Phase 0 + Phase 1 fields ...
    pub follows: HashSet<(Uuid, Uuid)>,              // (follower, followed)
    pub blocks: HashSet<(Uuid, Uuid)>,               // (blocker, blocked)
    pub reports: Vec<UserReport>,
    pub preferences: HashMap<Uuid, UserPreferencesState>,
    pub referral_invitations: HashMap<Uuid, Uuid>,   // invitee → inviter
}
```

`User` struct extended with: `slug_num`, `img`, `biography`, `visibility_status`, `created_at`, `updated_at`.

### Seed data (actual)

- 6 users total: `test@peer.com` (verified), `unverified@peer.com`, alice, bob, carol, dave
- 3 follow edges: alice ↔ bob (mutual), carol → alice
- 1 block edge: carol blocks dave
- 1 referral: alice invited by seed_verified
- Default preferences for all seeded users

### Phase 2 definition of done

- [x] 10 queries resolve correctly with proper response codes
- [x] 8 mutations resolve correctly with proper response codes and side effects
- [x] Content filtering excludes illegal, system, deleted, and blocked users
- [x] 40 new integration tests pass (73 total, exceeds ≥42 target)
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes
- [x] All Phase 0 + Phase 1 tests still pass

---

## 5. Phase 3 — Posts & Content

**Depends on:** Phase 2 (requires user profiles)
**Driven by:** `peer-web/src/pages/new_post.rs`, `peer-web/src/pages/view_post.rs`, `peer-web/src/pages/dashboard.rs`, `peer-web/src/api/posts.rs`
**Detailed plan:** [phase-3-posts-content.md](./phase-3-posts-content.md) — Plan quality: ⭐⭐⭐⭐⭐ (5/5)

### New files

| File | Contents |
|------|----------|
| `src/schema/query/posts.rs` | `listPosts`, `guestListPost`, `getPost`, `listUserPosts` |
| `src/schema/mutation/post.rs` | `createPost`, `postAction` (like/dislike/view/save/share/report), upload stub |
| `src/types/post.rs` | `Post`, `PostListResponse`, `PostInteraction`, `PostFilterType`, `PostSortType`, `ContentType` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub posts: Vec<Post>,
    pub post_likes: HashSet<(Uuid, Uuid)>,    // (user, post)
    pub post_dislikes: HashSet<(Uuid, Uuid)>,
    pub post_saves: HashSet<(Uuid, Uuid)>,
    pub post_views: HashSet<(Uuid, Uuid)>,
    pub post_reports: Vec<PostReport>,
    pub tags: HashMap<String, Vec<Uuid>>,      // tag → post IDs
}
```

### Seed data additions

- 5–10 sample posts (mix of text, image, audio, video types)
- Pre-populated like/view counts
- 3–4 tags with post associations

### Key behaviours to mock

- **Pagination**: `offset` / `limit` on all list queries (max 20)
- **Filtering**: by `PostFilterType` (IMAGE, AUDIO, VIDEO, TEXT, FOLLOWED, etc.)
- **Sorting**: by `PostSortType` (NEWEST, TRENDING, LIKES, etc.)
- **Guest access**: `guestListPost` requires no auth; `listPosts` requires auth
- **Content filtering**: respect `contentFilterBy` parameter
- **Upload stub**: accept multipart POST to `/upload-post`, return a mock file URL

### Tests (≥12 new)

- Create post → appears in `listPosts`
- List posts with pagination (offset/limit)
- Filter by content type
- Sort by likes vs. newest
- Like → re-query shows incremented count
- Save → appears in `filterBy: [SAVED]` (if applicable)
- Guest list post (no auth)
- Get specific post by ID
- Report post
- Upload stub returns mock URL

---

## 6. Phase 4 — Social (Comments, Chat) ✅

> **Status:** Complete (14 April 2026) — [Detailed plan](./phase-4-social-comments-chat.md)

**Depends on:** Phase 3 (requires posts)
**Driven by:** `peer-web/src/api/comments.rs`, `peer-web/src/api/chat.rs`, `peer-web/src/pages/chat.rs`
**Detailed plan:** [phase-4-social-comments-chat.md](./phase-4-social-comments-chat.md) — Plan quality: ⭐⭐⭐⭐⭐ (5/5)

### New files

| File | Contents |
|------|----------|
| `src/types/comment.rs` | `CommentType`, `CommentUser`, `Comment`, `CommentListResponse`, `CreateCommentResponse` |
| `src/types/chat.rs` | `ChatParticipant`, `ChatMessage`, `Chat`, `ListChatsResponse`, `SendMessageResponse`, `CreateChatResponse`, `CreateChatResult` |
| `src/schema/query/comments.rs` | `listComments`, `listChildComments` |
| `src/schema/mutation/comment.rs` | `createComment`, `likeComment`, `unlikeComment`, `reportComment` |
| `src/schema/query/chat.rs` | `listChats` |
| `src/schema/mutation/chat.rs` | `createChat`, `sendChatMessage` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub comments: Vec<CommentRecord>,
    pub comment_likes: HashSet<(Uuid, Uuid)>,       // (user, comment)
    pub comment_reports: HashSet<(Uuid, Uuid)>,      // (user, comment)
    pub daily_comment_count: HashMap<(Uuid, String), u32>, // (user, date) → count
    pub chats: Vec<ChatRecord>,
    pub chat_messages: Vec<ChatMessageRecord>,
}
```

### Comments — key behaviours

- Single level of nesting: top-level comments and replies (no deeper)
- `listComments(postid)` returns top-level only, paginated (offset/limit)
- `listChildComments(parent)` returns replies to a specific comment
- Like/unlike toggles with self-like and duplicate prevention
- Report with self-report and duplicate prevention
- Daily free action: first 4 comments per day are free (`11608`), subsequent are paid (`11605`)
- `amountcomments` on posts computed from actual comment count
- Content filtering: excludes comments from deleted users and non-VISIBLE comments

### Chat — key behaviours

- `listChats` returns conversations for the authenticated user, sorted by most recent activity
- `createChat(input: { name, recipients, image })` creates private (1:1) or group chats
- Private chat deduplication: returns existing chat ID if 1:1 chat already exists (`11803`)
- `sendChatMessage(chatid, content)` appends a message and updates chat `updated_at`
- Messages ordered by timestamp

### Seed data

- 6 comments (4 top-level, 2 replies), 2 comment likes
- 2 chats (1 private, 1 group), 5 chat messages

### Tests (49 new, 170 total)

- 29 comment tests: CRUD, pagination, like/unlike, report, daily free action, auth
- 17 chat tests: list, create (private/group dedup), send, auth, validation
- 3 cross-cutting: post `amountcomments`, reset state, regression

---

## 7. Phase 5 — Economy (Wallet, Tokenomics, Shop, Ads) ✅

> **Status:** Complete — 219 total tests (49 new), 0 clippy warnings

**Depends on:** Phase 3 (requires posts for ads)
**Driven by:** `peer-web/src/pages/wallet.rs`, `peer-web/src/api/wallet.rs`
**Detailed plan:** [phase-5-economy-wallet-tokenomics-shop-ads.md](./phase-5-economy-wallet-tokenomics-shop-ads.md) — Plan quality: ⭐⭐⭐⭐⭐ (5/5)

### New files

| File | Contents |
|------|----------|
| `src/schema/query/wallet.rs` | `balance`, `getTransactionHistory`, `getPaymentHistory` |
| `src/schema/mutation/wallet.rs` | `transferTokens` |
| `src/schema/query/tokenomics.rs` | `getActionPrices`, `getTokenomics`, `getDailyFreeActions`, `getGems`, `getMintingInfo` |
| `src/schema/mutation/tokenomics.rs` | `mintTokens` |
| `src/schema/query/ads.rs` | `listAdvertisementPosts`, `getAdvertisementHistory` |
| `src/schema/mutation/ads.rs` | `createAdvertisement` |
| `src/schema/mutation/shop.rs` | `purchaseShopItem` |
| `src/schema/query/shop.rs` | `shopOrderDetails` |
| `src/types/wallet.rs` | `CurrentLiquidity`, `Transaction`, `TransactionResponse`, `TransferResponse` |
| `src/types/tokenomics.rs` | `ActionPrices`, `Tokenomics`, `Gems`, `MintingInfo` |
| `src/types/ad.rs` | `AdvertisementPost`, `ListAdvertisementPostsResponse` |
| `src/types/shop.rs` | `ShopOrderDetails`, `ShopItemSpecs`, `ShopOrderDeliveryDetails` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub wallets: HashMap<Uuid, Decimal>,          // uid → token balance
    pub transactions: Vec<Transaction>,
    pub daily_actions_used: HashMap<(Uuid, String), u32>,  // (uid, date) → count
    pub gems: HashMap<Uuid, GemState>,
    pub advertisements: Vec<Advertisement>,
    pub shop_orders: Vec<ShopOrder>,
}
```

### Seed data additions

- Each seeded user starts with a default balance (e.g., `1000.0` tokens)
- Pre-populated action prices: post `20.0`, like `3.0`, dislike `3.0`, comment `1.0`
- 2–3 sample transactions in history
- 1 sample advertisement

### Wallet — key behaviours

| Operation | Logic |
|-----------|-------|
| `balance` | Return `wallets[current_user]` |
| `transferTokens(to, amount)` | Deduct from sender, credit receiver, record transaction. Fee = 2% of amount (min 0.01). Reject if insufficient balance. |
| `getTransactionHistory(type?, direction?, limit, offset)` | Filter + paginate `transactions` |

### Tokenomics — key behaviours

| Operation | Logic |
|-----------|-------|
| `getActionPrices` | Return hardcoded prices |
| `getDailyFreeActions` | Return `{ used, limit: 5 }` per action type |
| `getGems` | Return gem totals from post interactions |
| `mintTokens` | Convert gems → tokens at configured rate |

### Tests (49 new, 219 total)

- 16 wallet tests: balance, transfer, fees, insufficient balance, transaction history
- 8 tokenomics tests: action prices, daily free status, gems, minting
- 12 ads tests: create basic/pinned, list, history, cost calculation
- 8 shop tests: purchase, order details, delivery validation
- 5 cross-cutting: token deduction in actions, reset state, regression

---

## 8. Phase 6 — Admin & Moderation ✅

> **Status:** Complete — 266 total tests (47 new), 0 clippy warnings

**Depends on:** Phase 3 (requires posts/users to moderate), Phase 5 (requires wallets/gems)
**Driven by:** `admin/` PHP admin panel (future Leptos admin, if planned)
**Detailed plan:** [phase-6-admin-moderation.md](./phase-6-admin-moderation.md) — Plan quality: ⭐⭐⭐⭐⭐ (5/5)

### New files

| File | Contents |
|------|----------|
| `src/guards.rs` | `RoleGuard`, `UserRolesMask`, role constants, `require_admin()`, `require_moderator()` |
| `src/types/moderation.rs` | `ModerationStatus`, `ModerationContentType`, `ModerationStats`, `ModerationItem`, `BasicUserInfo`, `TargetContent` |
| `src/types/admin.rs` | `AdminUser`, `AllUserInfo`, `PostCommentsData`, `LeaderboardParamsInput` |
| `src/types/admin_gems.rs` | `DailyGemStatusData`, `GemsterResponse`, `GemstersData`, `MintAccount` |
| `src/schema/query/moderation.rs` | `moderationStats`, `moderationItems` |
| `src/schema/mutation/moderation.rs` | `performModeration` (hide/restore/illegal via single mutation) |
| `src/schema/query/admin.rs` | `listUsersAdminV2`, `allfriends`, `postcomments`, `generateLeaderboard` |
| `src/schema/query/admin_gems.rs` | `gemster`, `dailygemstatus`, `dailygemsresults`, `getMintAccount` |
| `src/schema/mutation/admin_gems.rs` | `globalwins`, `distributeTokensForGems`, `gemsters`, `alphaMint` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub moderation_tickets: Vec<ModerationTicketRecord>,
    pub content_visibility: HashMap<Uuid, String>,  // content_id → "NORMAL"/"HIDDEN"/"ILLEGAL"
    pub gem_records: Vec<GemRecord>,
    pub alpha_minted: bool,
    pub minted_dates: HashSet<String>,
}
```

### Role-based access

`RoleGuard` implements async-graphql's `Guard` trait with bitmask checking:

```rust
pub struct RoleGuard {
    required_role: u32,  // bitmask
}
```

| Role | Bitmask | Access |
|------|---------|--------|
| User | `0` | Base authenticated queries/mutations |
| Admin | `16` | Admin queries + base |
| Moderator | `256` | Moderation queries + base |

Seeded test users: one admin (`roles_mask: 16`) and one moderator (`roles_mask: 256`).

### Seed data

- 3 moderation tickets (2 for posts, 1 for comment) in `WaitingForReview` status
- Admin user at IP `192.168.1.1`, Moderator at IP `192.168.1.2`
- Mint account with `MINT_INITIAL_BALANCE: 5_000_000.0`

### Tests (47 new, 266 total)

- 14 moderation tests: stats, item listing, filtering, performModeration actions, auth
- 6 visibility tests: hidden/illegal content filtered from list queries
- 12 admin tests: user search (email, IP, verified, roles), allfriends, postcomments, leaderboard
- 10 gem/mint tests: gemster, dailygemstatus, dailygemsresults, globalwins, distributeTokens, alphaMint
- 5 cross-cutting: report→ticket flow, reset state, regression, duplicate prevention

---

## 9. CI Integration

**Depends on:** Phase 0 (minimum), ideally all phases
**Driven by:** Replacing Node.js CI steps, schema drift detection, in-process E2E testing
**Detailed plan:** [phase-ci-integration.md](./phase-ci-integration.md) — Plan quality: ⭐⭐⭐⭐⭐ (5/5)

### Step 9.1 — Add to workspace (if using Cargo workspace)

If a root `Cargo.toml` workspace exists, add `tests/mock_backend` as a member. Otherwise the crate is standalone.

### Step 9.2 — CI job

```yaml
# In CI pipeline (GitHub Actions, etc.)
- name: Test mock backend
  run: |
    cd tests/mock_backend
    cargo test --all-targets
    cargo clippy -- -D warnings
    cargo fmt --check
```

### Step 9.3 — E2E integration

The Leptos E2E tests (`peer-web/end2end/`) should:

1. Import `mock_backend::app()` as a dev-dependency
2. Start the mock in-process (no port binding needed for tower-based tests)
3. Or bind to a random port for browser-driven E2E tests:

```rust
let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
let port = listener.local_addr().unwrap().port();
tokio::spawn(axum::serve(listener, mock_backend::app()));
// Point Leptos at http://127.0.0.1:{port}/graphql
```

### Step 9.4 — SDL snapshot test

Export the mock's GraphQL SDL and snapshot-test it to catch accidental schema drift:

```rust
#[test]
fn schema_snapshot() {
    let schema = build_schema(default_state());
    let sdl = schema.sdl();
    insta::assert_snapshot!(sdl);
}
```

Periodically diff against the real backend schemas documented in `docs/backend_api/`.

---

## 10. Acceptance Criteria

**Detailed plan:** [acceptance-criteria.md](./acceptance-criteria.md) — Plan quality: ⭐⭐⭐⭐⭐ (5/5)

### Per-phase gates

| Phase | Gate |
|-------|------|
| 0 | `cargo test` passes 7 parity tests; `cargo run` serves GraphQL on `:4000`; Node.js files deleted |
| 1 | Full login/logout/refresh cycle works; auth middleware blocks unauthenticated calls |
| 2 | Profile CRUD, follow/block/report work; search with pagination |
| 3 | Post CRUD, interactions, filtering, sorting, guest access, upload stub |
| 4 | Comment threading, chat messaging |
| 5 | Token balance, transfers, fees, action prices, ads, shop |
| 6 | Role-based access enforced; moderation + admin queries work |

### Cross-cutting requirements (all phases)

- [ ] No `unwrap()` in resolver paths — all errors return proper GraphQL error responses
- [ ] Every resolver has ≥1 success and ≥1 error integration test
- [ ] Response codes match the values in `docs/backend_api/*.md`
- [ ] Field names exactly match the backend schema casing (PascalCase for `DefaultResponse`, camelCase for most others)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] No runtime dependencies on Node.js, npm, or any non-Rust tooling
- [ ] `POST /reset` returns state to `MockState::default()` (test isolation)
- [ ] SDL export available for schema drift detection
