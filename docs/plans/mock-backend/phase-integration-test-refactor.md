# Integration Test Refactor

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** All phases (1–6) completed — the refactor touches tests from every phase.
> **Goal:** Break up the monolithic 7,675-line `tests/mock_backend/tests/integration.rs` into a well-organised, maintainable multi-module test suite with shared helpers, reduced boilerplate, and clear domain boundaries.
> **Plan Quality:** ⭐⭐⭐⭐⭐ (5/5)

---

## Table of Contents

1. [Motivation](#1-motivation)
2. [Current State Analysis](#2-current-state-analysis)
3. [Target Architecture](#3-target-architecture)
4. [Task Breakdown](#4-task-breakdown)
5. [Implementation Details](#5-implementation-details)
6. [Migration Strategy](#6-migration-strategy)
7. [Testing Strategy](#7-testing-strategy)
8. [Definition of Done](#8-definition-of-done)

---

## 1. Motivation

### Why refactor?

| Problem | Impact |
|---------|--------|
| **Single 7,675-line file** | Hard to navigate; every contributor touches the same file; merge conflicts inevitable |
| **Duplicated helper code** | `graphql()`, `graphql_stateful()`, `graphql_with_auth()` inlined at the top with near-identical bodies; ad-hoc `login_*` helpers scattered mid-file |
| **No module structure** | Tests for auth, profiles, posts, comments, chat, wallet, tokenomics, ads, shop, moderation, and admin all live in one file with only comment banners separating them |
| **Inconsistent assertion style** | Mix of `assert_eq!` on JSON paths, nested `if let`, and manual `.as_str().unwrap()` chains — no consistent pattern |
| **GraphQL query strings written inline** | Identical field selections are repeated tens of times (e.g., `meta { ResponseCode }`) — fragile when the schema changes |
| **No test data builders** | Tests that need specific state (e.g., "carol with 0 balance") do it via long sequences of mutations, making tests slow and hard to read |
| **Dead code warnings** | `#[allow(dead_code)]` on `login_bob` — signals the helpers were grown organically |
| **No shared test utilities crate/module** | Everything sits in one integration test binary, meaning all tests must recompile together |

### Goals of the refactor

1. **Domain-organised test modules** — one file per feature domain, easy to find and extend.
2. **Shared test harness module** — `graphql()`, `graphql_with_auth()`, `login_as()`, `default_shared_state()`, assertion helpers, and reusable GraphQL fragments all live in one place.
3. **Macro-based or builder-based boilerplate reduction** — reduce the per-test ceremony for "login, execute query, assert ResponseCode."
4. **Zero test loss** — every existing test case is preserved (renamed if necessary) with identical coverage.
5. **All tests green** — `cargo test` passes identically before and after the refactor.

---

## 2. Current State Analysis

### File statistics

| Metric | Value |
|--------|-------|
| Total lines | 7,675 |
| `#[tokio::test]` functions | ~140 |
| Helper functions | 8 (`graphql`, `graphql_stateful`, `graphql_with_auth`, `default_shared_state`, `decimal_val`, `login_as`, `login_default`, `login_alice`, `login_bob`, `login_admin`, `login_moderator`, `do_post_action`) |
| Inline constants | 1 (`COMMENT_FIELDS`) |
| Comment section banners | 17 |

### Domain distribution (approximate test counts)

| Domain | Tests | Line range |
|--------|-------|------------|
| Referral & Registration | 5 | 1–215 |
| Account Verification | 2 | 216–310 |
| Login & Session | 10 | 370–640 |
| Token Refresh & Logout | 5 | 640–830 |
| Account Deletion | 3 | 830–950 |
| Password Reset & Update | 7 | 950–1,200 |
| Contact Us | 1 | ~1,200 |
| Profile (getProfile, searchUser, listUsersV2, getUser) | 12 | 1,250–1,700 |
| Follow / Block / Report | 12 | 1,700–2,100 |
| Profile Updates (image, bio, username, email, preferences) | 8 | 2,100–2,400 |
| User Info & Referral | 3 | 2,400–2,500 |
| Posts (CRUD, list, filter, sort, search, guest) | 22 | 2,500–3,400 |
| Post Actions (like, dislike, view, save, share, report) | 12 | 3,400–3,700 |
| Post Eligibility & Tags | 4 | 3,700–3,850 |
| Upload & Ads in Posts | 4 | 3,850–4,050 |
| Comments (CRUD, like, unlike, report) | 17 | 4,050–4,700 |
| Daily Free Actions | 2 | 4,700–4,800 |
| Chat (list, create, send, validation) | 14 | 4,800–5,300 |
| Cross-cutting (Phase 4) | 3 | 5,300–5,500 |
| Wallet (balance, transfer, history) | 12 | 5,500–5,900 |
| Tokenomics (prices, daily free, gems) | 8 | 5,900–6,200 |
| Advertisements (create, history, visibility) | 6 | 6,200–6,450 |
| Shop (purchase, details, validation) | 5 | 6,450–6,600 |
| Economy Cross-cutting | 5 | 6,600–6,850 |
| Moderation (stats, items, actions, visibility) | 15 | 6,850–7,400 |
| Admin (search, friends, comments, leaderboard, gems, mint) | 12 | 7,400–7,675 |

### Key helper functions and their usage

| Helper | Calls | Notes |
|--------|-------|-------|
| `graphql()` | ~15 | Stateless, no auth — used for simple smoke tests and unauthenticated error paths |
| `graphql_stateful()` | ~25 | Shared state, no auth header |
| `graphql_with_auth()` | ~95 | Shared state + Bearer token — the workhorse |
| `login_as()` | ~10 (direct) + via wrappers | Generic login helper |
| `login_default()` / `login_alice()` / `login_bob()` | ~80 combined | Convenience wrappers |
| `do_post_action()` | ~15 | Post action helper |
| `decimal_val()` | ~12 | Parses Decimal-as-string from JSON |

---

## 3. Target Architecture

### File tree

```
tests/mock_backend/tests/
├── common/
│   ├── mod.rs              # Re-exports everything below
│   ├── client.rs           # graphql(), graphql_stateful(), graphql_with_auth()
│   ├── auth.rs             # login_as(), login_default(), login_alice(), login_bob(),
│   │                       #   login_admin(), login_moderator()
│   ├── state.rs            # default_shared_state(), custom state builders
│   ├── assertions.rs       # assert_response_code!(), assert_gql_error_code!(), decimal_val()
│   └── fragments.rs        # COMMENT_FIELDS, META_FIELDS, and other reusable field selections
│
├── auth_registration.rs    # Referral, register, verify, fail-email
├── auth_session.rs         # Login, logout, refresh, delete account, password reset/update
├── profiles.rs             # getProfile, searchUser, listUsersV2, getUser, getUserInfo, referralInfo
├── social.rs               # Follow/unfollow, block/unblock, friends, blocked users, reportUser
├── profile_updates.rs      # updateProfileImage, updateBio, updateUsername, updateEmail, updatePreferences
├── posts.rs                # listPosts (feed, pagination, filters, sort, search), single post, guest post
├── post_actions.rs         # Like, unlike, dislike, view, save, share, report, eligibility, tags
├── post_upload.rs          # Upload endpoint tests
├── comments.rs             # listComments, listChildComments, createComment, like/unlike/report comment
├── chat.rs                 # listChats, createChat, sendChatMessage
├── wallet.rs               # balance, transferV2, transactionHistory
├── tokenomics.rs           # actionPrices, getTokenomics, dailyFreeStatus, todaysInteractions
├── advertisements.rs       # advertisePostBasic, advertisePostPinned, advertisementHistory, listAdPosts
├── shop.rs                 # performShopOrder, shopOrderDetails
├── moderation.rs           # moderationStats, moderationItems, performModeration, content visibility
├── admin.rs                # listUsersAdminV2, allfriends, postcomments, generateLeaderboard
├── admin_gems.rs           # gemster, dailygemstatus, dailygemsresults, getMintAccount, globalwins,
│                           #   distributeTokensForGems, alphaMint
└── cross_cutting.rs        # Economy cross-cutting, phase regressions, reset-clears-state tests
```

### Why this split?

- Each file maps to a clear **feature domain** matching the phase plan structure and the `docs/backend_api/` documentation numbering.
- Files stay under ~500 lines each — easily scannable.
- `common/` is a proper Rust test utility module (not a `#[cfg(test)]` module in `lib.rs`), which follows the [Rust integration test conventions](https://doc.rust-lang.org/book/ch11-03-test-organization.html#submodules-in-integration-tests).
- Splitting `admin.rs` / `admin_gems.rs` keeps the admin section manageable since it covers two distinct concerns (user admin vs. gem/mint administration).

---

## 4. Task Breakdown

### Phase A: Scaffold the shared test harness (common/)

| # | Task | Details |
|---|------|---------|
| A1 | Create `tests/common/mod.rs` | Declare submodules: `client`, `auth`, `state`, `assertions`, `fragments` |
| A2 | Create `tests/common/client.rs` | Move `graphql()`, `graphql_stateful()`, `graphql_with_auth()`. Make them `pub`. Unify the three functions to reduce duplication — `graphql_with_auth` becomes the base, others delegate to it with `None` for optional params. |
| A3 | Create `tests/common/auth.rs` | Move `login_as()`, `login_default()`, `login_alice()`, `login_bob()`, `login_admin()`, `login_moderator()`. All `pub`. Remove `#[allow(dead_code)]` from `login_bob` — it will be used now. |
| A4 | Create `tests/common/state.rs` | Move `default_shared_state()`. Add a `pub fn fresh_state() -> Arc<RwLock<MockState>>` alias if naming diverges. |
| A5 | Create `tests/common/assertions.rs` | Move `decimal_val()`. Add `assert_response_code!(json_value, path, expected_code)` macro to reduce the 3-line pattern of `&res["data"]["foo"]["meta"]["ResponseCode"]` + `assert_eq!`. Add `assert_gql_error_code!(res, expected_code)` for the GraphQL-errors-array pattern used in moderation/admin permission tests. |
| A6 | Create `tests/common/fragments.rs` | Move `COMMENT_FIELDS`. Add `META_FIELDS`, `USER_FIELDS`, `POST_FIELDS` as reusable partial GraphQL selections that tests can interpolate via `format!()`. |
| A7 | Verify `common/` compiles | `cargo test --no-run` — the common module is compiled but not executed on its own. |

### Phase B: Extract domain test files

Each sub-task:
1. Create the new file (e.g., `tests/auth_registration.rs`).
2. Add `mod common;` at the top.
3. `use common::{client::*, auth::*, state::*, assertions::*, fragments::*};`
4. `use` the needed seed constants and library types.
5. Move the corresponding `#[tokio::test]` functions from `integration.rs`, updating helper calls to use the `common::` paths.
6. Update any locally-defined helper (e.g., `do_post_action`) — either move it to `common/` if shared, or keep it file-local if used by only one domain.

| # | Task | Source section | Tests moved | New file |
|---|------|---------------|-------------|----------|
| B1 | Extract registration & referral tests | Referral, Register, Verify, FailEmail | ~5 | `auth_registration.rs` |
| B2 | Extract session tests | Login, Logout, Refresh, Delete, PasswordReset, UpdatePassword, ContactUs | ~18 | `auth_session.rs` |
| B3 | Extract profile read tests | getProfile, searchUser, listUsersV2, getUser, getUserInfo, referralInfo, referralList | ~15 | `profiles.rs` |
| B4 | Extract social interaction tests | Follow, Block, Friends, BlockedUsers, ReportUser | ~12 | `social.rs` |
| B5 | Extract profile update tests | updateProfileImage, updateBio, updateUsername, updateEmail, updateUserPreferences | ~8 | `profile_updates.rs` |
| B6 | Extract post list/query tests | listPosts (all variants), guestListPost, listUserPosts | ~16 | `posts.rs` |
| B7 | Extract post action tests | Like, dislike, view, save, share, report, eligibility, searchTags + `do_post_action` helper | ~16 | `post_actions.rs` |
| B8 | Extract upload tests | Upload endpoint tests | ~3 | `post_upload.rs` |
| B9 | Extract comment tests | listComments, listChildComments, createComment, likeComment, unlikeComment, reportComment, daily free action | ~19 | `comments.rs` |
| B10 | Extract chat tests | listChats, createChat, sendChatMessage | ~14 | `chat.rs` |
| B11 | Extract wallet tests | balance, resolveTransferV2, transactionHistory | ~12 | `wallet.rs` |
| B12 | Extract tokenomics tests | getActionPrices, getTokenomics, getDailyFreeStatus, listTodaysInteractions | ~8 | `tokenomics.rs` |
| B13 | Extract advertisement tests | advertisePostBasic, advertisePostPinned, advertisementHistory, listAdvertisementPosts | ~6 | `advertisements.rs` |
| B14 | Extract shop tests | performShopOrder, shopOrderDetails | ~5 | `shop.rs` |
| B15 | Extract moderation tests | moderationStats, moderationItems, performModeration, content visibility | ~15 | `moderation.rs` |
| B16 | Extract admin tests | listUsersAdminV2, allfriends, postcomments, generateLeaderboard | ~8 | `admin.rs` |
| B17 | Extract admin gem/mint tests | gemster, dailygemstatus, dailygemsresults, getMintAccount, globalwins, distributeTokensForGems, alphaMint | ~7 | `admin_gems.rs` |
| B18 | Extract cross-cutting tests | Economy cross-cutting, phase regressions, reset-clears-state | ~8 | `cross_cutting.rs` |

### Phase C: Cleanup & Verification

| # | Task | Details |
|---|------|---------|
| C1 | Delete `integration.rs` | Remove the original monolith after confirming all tests moved. |
| C2 | Run full test suite | `cargo test` — all ~140 tests must pass with zero failures. |
| C3 | Run `cargo clippy` | No new warnings. Remove all `#[allow(dead_code)]` that are no longer needed. |
| C4 | Run `cargo fmt` | Ensure consistent formatting across all new files. |
| C5 | Verify test count parity | Compare `cargo test 2>&1 | grep "test result"` before and after — the total test count must be identical. |

---

## 5. Implementation Details

### 5.1 The `common/` module pattern

Rust's integration test convention places shared code in `tests/common/mod.rs` so it isn't compiled as its own test binary. Each integration test file (`tests/foo.rs`) includes:

```rust
mod common;
use common::prelude::*;
```

We add a `prelude` re-export in `common/mod.rs`:

```rust
pub mod assertions;
pub mod auth;
pub mod client;
pub mod fragments;
pub mod state;

pub mod prelude {
    pub use super::assertions::*;
    pub use super::auth::*;
    pub use super::client::*;
    pub use super::fragments::*;
    pub use super::state::*;

    // Re-export commonly used external types
    pub use mock_backend::seed::*;
    pub use mock_backend::{app, app_with_state, state::MockState};
    pub use serde_json::{json, Value};
    pub use std::sync::Arc;
    pub use tokio::sync::RwLock;
}
```

### 5.2 Unified GraphQL client (`client.rs`)

Collapse the three functions into one base + two convenience wrappers:

```rust
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use mock_backend::{app, app_with_state, state::MockState};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

/// Core GraphQL request — supports optional shared state and optional auth token.
pub async fn graphql_request(
    state: Option<&Arc<RwLock<MockState>>>,
    query: &str,
    token: Option<&str>,
) -> Value {
    let app = match state {
        Some(s) => app_with_state(s.clone()),
        None => app(),
    };

    let body = json!({ "query": query });

    let mut builder = Request::builder()
        .method("POST")
        .uri("/graphql")
        .header("Content-Type", "application/json");

    if let Some(t) = token {
        builder = builder.header("Authorization", format!("Bearer {}", t));
    }

    let request = builder
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body_bytes).unwrap()
}

/// Stateless, no auth.
pub async fn graphql(query: &str) -> Value {
    graphql_request(None, query, None).await
}

/// Shared state, no auth.
pub async fn graphql_stateful(state: &Arc<RwLock<MockState>>, query: &str) -> Value {
    graphql_request(Some(state), query, None).await
}

/// Shared state + Bearer token.
pub async fn graphql_with_auth(
    state: &Arc<RwLock<MockState>>,
    query: &str,
    token: &str,
) -> Value {
    graphql_request(Some(state), query, Some(token)).await
}
```

### 5.3 Assertion helpers (`assertions.rs`)

```rust
use serde_json::Value;

/// Extract a Decimal value (serialized as JSON string) as f64.
pub fn decimal_val(v: &Value) -> f64 {
    v.as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| v.as_f64())
        .expect("Expected a numeric value (string or number)")
}

/// Assert a ResponseCode at a given JSON path.
///
/// Usage: `assert_response_code!(res, "data.login", "10801");`
#[macro_export]
macro_rules! assert_response_code {
    ($res:expr, $path:expr, $expected:expr) => {{
        let parts: Vec<&str> = $path.split('.').collect();
        let mut val = &$res;
        for part in &parts {
            val = &val[*part];
        }
        // Try nested meta.ResponseCode first, then direct ResponseCode
        let code = if val["meta"]["ResponseCode"].is_string() {
            &val["meta"]["ResponseCode"]
        } else if val["ResponseCode"].is_string() {
            &val["ResponseCode"]
        } else {
            panic!(
                "No ResponseCode found at path '{}': {:?}",
                $path, val
            );
        };
        assert_eq!(
            code.as_str().unwrap(),
            $expected,
            "ResponseCode mismatch at '{}'",
            $path
        );
    }};
}

/// Assert that the GraphQL response contains an error with the given code.
///
/// Usage: `assert_gql_error_code!(res, "62101");`
#[macro_export]
macro_rules! assert_gql_error_code {
    ($res:expr, $expected:expr) => {{
        assert!(
            $res["errors"].is_array(),
            "Expected errors array in response"
        );
        let has_code = $res["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| {
                e["extensions"]["code"]
                    .as_str()
                    .map(|c| c.contains($expected))
                    .unwrap_or(false)
            });
        assert!(has_code, "Expected error code containing '{}' in errors: {:?}", $expected, $res["errors"]);
    }};
}
```

### 5.4 Fragment constants (`fragments.rs`)

```rust
pub const META_FIELDS: &str = "meta { status ResponseCode }";
pub const COMMENT_FIELDS: &str = "commentid userid postid parentid content createdat amountlikes amountreplies isliked user { id username slug img isfollowed isfollowing }";
pub const POST_CORE_FIELDS: &str = "id contenttype title amountlikes amountviews isliked isviewed tags user { id username slug }";
```

### 5.5 `do_post_action` placement

The `do_post_action` helper is used only in post-action and cross-cutting tests. Move it to `common/client.rs` as a `pub` function so both `post_actions.rs` and `cross_cutting.rs` can use it.

### 5.6 What NOT to change

- **Test logic** — no assertions change, no test scenarios added or removed.
- **Test names** — preserve existing names for `cargo test` grep compatibility. If renamed for module clarity, the old name pattern must remain findable.
- **Seed data** — no changes to `seed.rs` or `state.rs` in the `src/` crate.
- **GraphQL schema** — no resolver or type changes.
- **Cargo.toml** — no new dependencies (the `common/` module pattern uses the same `[dev-dependencies]`).

---

## 6. Migration Strategy

### Incremental extraction approach

To avoid a big-bang change that's impossible to debug when tests fail:

1. **Phase A** — Create `common/` and populate it. Leave `integration.rs` untouched. Verify `cargo test --no-run` passes.
2. **Phase B** — Extract one domain at a time. After each extraction:
   - Delete the moved tests from `integration.rs`.
   - Run `cargo test` and confirm the total test count stays the same.
   - Commit.
3. **Phase C** — After all domains extracted, `integration.rs` should be empty. Delete it. Run final verification.

### Rollback safety

Each commit in Phase B is atomic — if a domain extraction breaks something, reverting one commit gets back to a working state. The incremental approach means the old `integration.rs` and new files coexist during the migration.

---

## 7. Testing Strategy

### Before refactor (baseline)

```bash
cd tests/mock_backend
cargo test 2>&1 | tail -1
# Expected: test result: ok. ~140 passed; 0 failed; 0 ignored
```

Record the exact count.

### After each Phase B task

```bash
cargo test 2>&1 | tail -1
# Must match the baseline count exactly
```

### After Phase C (final)

```bash
cargo test 2>&1 | tail -1          # Same count, 0 failures
cargo clippy -- -D warnings        # Clean
cargo fmt -- --check                # Clean
```

### CI considerations

No CI changes needed — `cargo test` already runs all files in `tests/`. The new multi-file layout is automatically discovered by Cargo's test harness.

---

## 8. Definition of Done

- [ ] `tests/common/mod.rs` exists with `client`, `auth`, `state`, `assertions`, `fragments` submodules
- [ ] All ~140 tests moved to domain-specific files (18 files total)
- [ ] `tests/integration.rs` deleted
- [ ] `cargo test` — all tests pass, count matches pre-refactor baseline
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt -- --check` — clean
- [ ] No `#[allow(dead_code)]` in test files
- [ ] Every test file starts with `mod common; use common::prelude::*;`
- [ ] `graphql()` / `graphql_stateful()` / `graphql_with_auth()` deduplicated into one base function in `common/client.rs`
- [ ] `decimal_val()` and assertion macros live in `common/assertions.rs`
- [ ] No test logic changed — only moved and re-imported
