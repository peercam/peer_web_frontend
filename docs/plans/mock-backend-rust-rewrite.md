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

## 2. Phase 0 — Project Skeleton & Parity

**Outcome:** A `cargo build`-able Rust crate that passes the same 7 test scenarios as the current Node.js mock.

### Step 0.1 — Initialise crate

Create `tests/mock_backend/` as a Rust project (alongside, or replacing, the Node.js files):

```
tests/mock_backend/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── state.rs
│   ├── seed.rs
│   ├── error.rs
│   ├── types/
│   │   ├── mod.rs
│   │   └── response.rs
│   └── schema/
│       ├── mod.rs
│       ├── query.rs
│       └── mutation/
│           ├── mod.rs
│           └── registration.rs
└── tests/
    └── integration.rs
```

**Cargo.toml dependencies:**

```toml
[package]
name = "mock_backend"
version = "0.1.0"
edition = "2024"

[dependencies]
async-graphql = "8"
async-graphql-axum = "8"
axum = "0.8"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
reqwest = { version = "0.13", features = ["json"] }
```

### Step 0.2 — Core types

Define shared response types that mirror the backend's GraphQL schema exactly (field names, casing):

```rust
// types/response.rs

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "PascalCase")]
pub struct DefaultResponse {
    pub status: String,
    pub request_id: String,
    pub response_code: String,
    pub response_message: String,
}

impl DefaultResponse {
    pub fn success(code: &str, message: &str) -> Self { /* ... */ }
    pub fn error(code: &str, message: &str) -> Self { /* ... */ }
}
```

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
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<MockState>>;

pub struct MockState {
    pub known_referrals: HashSet<Uuid>,
    pub registered_emails: HashSet<String>,
    pub verified_users: HashSet<Uuid>,
    pub users: HashMap<Uuid, User>,
    // Extended in later phases:
    // pub posts: Vec<Post>,
    // pub wallets: HashMap<Uuid, Balance>,
    // pub comments: Vec<Comment>,
    // pub chats: Vec<Chat>,
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
            users: HashMap::new(),
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
    async fn _health(&self) -> bool {
        true
    }
}
```

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
    let state = SharedState::default();  // Arc<RwLock<MockState::default()>>
    let schema = build_schema(state.clone());

    Router::new()
        .route("/graphql", post(graphql_handler).get(graphql_handler))
        .route("/reset", post(reset_handler))
        .layer(CorsLayer::permissive())
        .with_state(AppState { schema, mock_state: state })
}
```

- `graphql_handler`: Delegates to `async_graphql_axum::GraphQL`
- `reset_handler`: Acquires write lock, replaces state with `MockState::default()`

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

Port all 7 existing Node.js test scenarios to `tests/integration.rs` using `tower::ServiceExt::oneshot`:

| # | Test | Assert |
|---|------|--------|
| 1 | Valid referral UUID | `status: "success"`, `ResponseCode: "11011"`, `affectedRows[0].uid == "usr_mock_001"` |
| 2 | Invalid referral string | `status: "error"`, `ResponseCode: "31010"`, `affectedRows: null` |
| 3 | Register success | `status: "success"`, `ResponseCode: "10601"`, `userid` is valid UUID |
| 4 | Register duplicate email | `status: "error"`, `ResponseCode: "30601"`, `userid: null` |
| 5 | Verify account success | `status: "success"`, `ResponseCode: "10701"` |
| 6 | Already verified | `status: "success"`, `ResponseCode: "30701"` |
| 7 | Simulated internal error (`fail@`) | `status: "error"`, `ResponseCode: "40601"`, `userid: null` |

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

- [ ] `cargo build` succeeds for the `mock_backend` crate
- [ ] `cargo test` passes all 7 parity tests (in-process, no port binding)
- [ ] `cargo run` starts the HTTP server on `:4000` and serves GraphQL
- [ ] `POST /reset` clears state
- [ ] `POST /graphql` with the 3 mutations returns identical response shapes to the Node.js mock
- [ ] `peer-web` E2E tests can import `mock_backend::app()` as a dev-dependency

---

## 3. Phase 1 — Login & Session Flows

**Depends on:** Phase 0
**Driven by:** `peer-web/src/pages/login.rs`, `peer-web/src/api/auth.rs`

### New files

| File | Contents |
|------|----------|
| `src/schema/mutation/auth.rs` | `login`, `refresh_token`, `logout`, `delete_account`, `request_password_reset`, `reset_password_token_verify`, `reset_password`, `change_password`, `contactus` |
| `src/types/auth.rs` | `AuthPayload`, `ResetPasswordRequestResponse`, `ContactusResponse`, `ContactusResponsePayload` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub user_passwords: HashMap<Uuid, String>,      // uid → plaintext (mock only)
    pub access_tokens: HashMap<String, Uuid>,        // token → uid
    pub refresh_tokens: HashMap<String, Uuid>,       // token → uid
    pub password_reset_tokens: HashMap<String, Uuid>, // token → uid
    pub deleted_users: HashSet<Uuid>,
}
```

### Resolvers to implement

| Mutation | Input | Logic | Key response codes |
|----------|-------|-------|--------------------|
| `login(email, password)` | email + password | Look up user by email → verify password → generate mock JWT tokens | `10801` success, `30801` invalid creds, `60801` not verified |
| `refreshToken(refreshToken)` | refresh token string | Validate token in `refresh_tokens` map → issue new pair | `10901` success, `30901` invalid token |
| `logout` | (auth header) | Remove tokens from maps | `11001` success |
| `deleteAccount(password)` | password | Verify password → set user status to deleted | `11012` success, `31001` wrong password |
| `requestPasswordReset(email)` | email | Generate reset token, store in map | `11901` always (prevents enumeration) |
| `resetPasswordTokenVerify(token)` | token | Check `password_reset_tokens` | `11902` valid, `31904` invalid |
| `resetPassword(token, password)` | token + new password | Update password, clear tokens | `11005` success, `31904` invalid token |
| `changePassword(oldPassword, newPassword)` | old + new (auth'd) | Verify old → update | `11001` success |
| `contactus(name, email, message)` | name + email + message | No-op, return success | `10401` success |

### Mock JWT strategy

Generate deterministic tokens (no real signing): `mock-access-<uuid>-<timestamp>` and `mock-refresh-<uuid>-<timestamp>`. The mock validates by presence in the `access_tokens` / `refresh_tokens` maps, not by cryptographic verification.

### Auth middleware

Add an Axum middleware or async-graphql guard that extracts the `Authorization: Bearer <token>` header, resolves it to a `Uuid` via `access_tokens`, and injects the current user ID into the async-graphql context. Unauthenticated requests to protected resolvers return `60501`.

### Tests (≥12 new)

| # | Scenario |
|---|----------|
| 1 | Register → verify → login success |
| 2 | Login with wrong password → `30801` |
| 3 | Login unverified account → `60801` |
| 4 | Login deleted account → `30801` |
| 5 | Refresh token success |
| 6 | Refresh with invalid token → `30901` |
| 7 | Logout invalidates tokens |
| 8 | Delete account → re-login fails |
| 9 | Password reset flow (request → verify → reset → login with new password) |
| 10 | Reset with bad token → `31904` |
| 11 | Change password (authenticated) |
| 12 | Contact us returns `10401` |

### Phase 1 definition of done

- [ ] All auth mutations return correct response shapes
- [ ] Auth middleware blocks unauthenticated calls to protected resolvers
- [ ] Token lifecycle (issue → refresh → logout/invalidate) works end-to-end
- [ ] ≥12 integration tests pass

---

## 4. Phase 2 — Users & Profiles

**Depends on:** Phase 1 (requires auth)
**Driven by:** `peer-web/src/pages/profile.rs`, `peer-web/src/pages/view_profile.rs`, `peer-web/src/api/profile.rs`

### New files

| File | Contents |
|------|----------|
| `src/schema/query/users.rs` | `listUsersV2`, `getProfile`, `listFollowRelations`, `listFriends`, `getUserPreferences` |
| `src/schema/mutation/profile.rs` | `editProfile`, `followUser`/`unfollowUser` (toggleFollow), `blockUser`/`unblockUser` (toggleBlock), `reportUser` |
| `src/types/user.rs` | `User`, `UserListResponse`, `Profile`, `UserPreferences`, `FollowRelation`, `Friend` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub profiles: HashMap<Uuid, Profile>,
    pub follows: HashSet<(Uuid, Uuid)>,       // (follower, followed)
    pub blocks: HashSet<(Uuid, Uuid)>,         // (blocker, blocked)
    pub reports: Vec<UserReport>,
    pub preferences: HashMap<Uuid, UserPreferences>,
}
```

### Seed data additions

- 3–5 pre-populated user profiles with varying fields (biography, avatar, etc.)
- 2 pre-existing follow relationships
- 1 pre-existing block relationship
- Default preferences for each seeded user

### Resolvers to implement

| Operation | Type | Key response codes |
|-----------|------|--------------------|
| `listUsersV2(userid?, username?, offset, limit)` | Query | `11101` found, `21101` empty |
| `getProfile(userid)` | Query | `11301` found, `31007` not found |
| `listFollowRelations(userid, type, offset, limit)` | Query | `11101` success |
| `listFriends(offset, limit)` | Query | `11101` success |
| `getUserPreferences` | Query | `11401` success |
| `editProfile(input)` | Mutation | `11301` success |
| `followUser(userid)` / `unfollowUser(userid)` | Mutation | `11101` follow, `11102` unfollow |
| `blockUser(userid)` / `unblockUser(userid)` | Mutation | `11103` block, `11104` unblock |
| `reportUser(userid, reason)` | Mutation | `11105` reported |

### Tests (≥10 new)

- Search user by username, by ID, pagination
- Get profile of self, of another user
- Follow → check follower list → unfollow
- Block → verify blocked user excluded from search
- Edit profile → verify changes persisted
- Report user

---

## 5. Phase 3 — Posts & Content

**Depends on:** Phase 2 (requires user profiles)
**Driven by:** `peer-web/src/pages/new_post.rs`, `peer-web/src/pages/view_post.rs`, `peer-web/src/pages/dashboard.rs`, `peer-web/src/api/posts.rs`

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

## 6. Phase 4 — Social (Comments, Chat)

**Depends on:** Phase 3 (requires posts)
**Driven by:** `peer-web/src/api/comments.rs`, `peer-web/src/api/chat.rs`, `peer-web/src/pages/chat.rs`

### New files

| File | Contents |
|------|----------|
| `src/schema/query/comments.rs` | `listComments`, `listChildComments` |
| `src/schema/mutation/comment.rs` | `createComment`, `replyComment`, `likeComment`, `unlikeComment` |
| `src/schema/query/chat.rs` | `listChats`, `getChatMessages` |
| `src/schema/mutation/chat.rs` | `createChat`, `sendChatMessage` |
| `src/types/comment.rs` | `Comment`, `CommentListResponse` |
| `src/types/chat.rs` | `Chat`, `ChatMessage` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub comments: Vec<Comment>,
    pub comment_likes: HashSet<(Uuid, Uuid)>,  // (user, comment)
    pub chats: Vec<Chat>,
    pub chat_messages: Vec<ChatMessage>,
}
```

### Comments — key behaviours

- Single level of nesting: top-level comments and replies (no deeper)
- `listComments(postid)` returns top-level only
- `listChildComments(parent)` returns replies to a specific comment
- Like/unlike toggles
- Content filtering applied

### Chat — key behaviours

- `listChats` returns conversations for the authenticated user
- `createChat(userid)` starts a new 1:1 conversation
- `sendChatMessage(chatid, message)` appends a message
- Messages ordered by timestamp

### Tests (≥10 new)

- Create comment on post → appears in `listComments`
- Reply to comment → appears in `listChildComments`
- Like/unlike comment
- Create chat → send message → list messages
- List chats shows latest message preview

---

## 7. Phase 5 — Economy (Wallet, Tokenomics, Shop, Ads)

**Depends on:** Phase 3 (requires posts for ads)
**Driven by:** `peer-web/src/pages/wallet.rs`, `peer-web/src/api/wallet.rs`

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

### Tests (≥10 new)

- Check balance
- Transfer tokens → both balances update, transaction recorded
- Transfer with insufficient balance → error
- Fee calculation
- Get action prices
- List transaction history with filters
- Create advertisement → appears in `listAdvertisementPosts`
- Purchase shop item → deducts tokens, creates order

---

## 8. Phase 6 — Admin & Moderation

**Depends on:** Phase 3 (requires posts/users to moderate)
**Driven by:** `admin/` PHP admin panel (future Leptos admin, if planned)

### New files

| File | Contents |
|------|----------|
| `src/schema/query/moderation.rs` | `moderationStats`, `moderationItems` |
| `src/schema/mutation/moderation.rs` | `hideContent`, `restoreContent`, `markIllegal` |
| `src/schema/query/admin.rs` | `listUsersAdminV2`, `leaderboard`, `friendshipGraph` |
| `src/types/moderation.rs` | `ModerationStats`, `ModerationTicket`, `ModerationAction` |

### State extensions

```rust
pub struct MockState {
    // ... existing fields ...
    pub moderation_tickets: Vec<ModerationTicket>,
    pub content_visibility: HashMap<Uuid, ContentVisibility>, // content_id → state
}
```

### Role-based access

Implement a simple role-check guard using async-graphql's `Guard` trait:

```rust
pub struct RoleGuard {
    required: u32,  // bitmask
}
```

| Role | Bitmask | Access |
|------|---------|--------|
| User | `0` | Base authenticated queries/mutations |
| Admin | `16` | Admin queries + base |
| Moderator | `256` | Moderation queries + base |

Seeded test users should include one admin and one moderator account.

### Tests (≥6 new)

- Moderator can view moderation stats
- Regular user blocked from moderation endpoints (`62101`)
- Hide content → content marked as hidden
- Admin can search users with extended fields (email, ip)
- Regular user cannot access admin queries

---

## 9. CI Integration

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
