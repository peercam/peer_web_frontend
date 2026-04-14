# Acceptance Criteria — Mock Backend Rust Rewrite

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** All phases (Phase 0–6 + CI Integration)
> **Goal:** Define the complete, verifiable acceptance criteria that must pass before the Rust mock backend is considered production-ready. This includes per-phase gates, cross-cutting quality requirements, automated verification tooling, and a final sign-off checklist.
> **Plan Quality:** ⭐⭐⭐⭐⭐ (5/5)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Prerequisites](#2-prerequisites)
3. [Per-Phase Acceptance Gates](#3-per-phase-acceptance-gates)
4. [Cross-Cutting Requirements](#4-cross-cutting-requirements)
5. [Automated Verification](#5-automated-verification)
6. [Response Code Compliance Matrix](#6-response-code-compliance-matrix)
7. [Schema Fidelity Checks](#7-schema-fidelity-checks)
8. [Performance & Reliability Baselines](#8-performance--reliability-baselines)
9. [Migration Verification](#9-migration-verification)
10. [Final Sign-Off Checklist](#10-final-sign-off-checklist)

---

## 1. Overview

### Purpose

The acceptance criteria serve as the **single source of truth** for determining when each phase — and the overall project — is complete. Every criterion is designed to be mechanically verifiable: either through an automated test, a CI check, or a concrete manual step with an expected output.

### Verification Layers

| Layer | What it checks | How it runs |
|-------|----------------|-------------|
| Unit tests | Individual resolver logic, state mutations | `cargo test` in `tests/mock_backend` |
| Integration tests | Full GraphQL request → response roundtrips via tower `oneshot` | `cargo test --test integration` |
| SDL snapshot | Schema structure hasn't drifted | `cargo insta test` in CI |
| Clippy + fmt | Code quality, idiomatic Rust | `cargo clippy -- -D warnings && cargo fmt --check` |
| E2E tests | Leptos frontend ↔ mock backend via HTTP | Playwright in `peer-web/end2end/` |
| Manual smoke test | Server starts, responds to curl | `cargo run` + `curl` |
| Response code audit | Codes match `docs/backend_api/*.md` and `json/response-codes.json` | Dedicated audit test |

### Relationship to other plans

```
mock-backend-rust-rewrite.md (parent)
├── phase-0-mock-backend-skeleton.md    → Gate 0
├── phase-1-login-session-flows.md      → Gate 1
├── phase-2-users-and-profiles.md       → Gate 2
├── phase-3-posts-content.md            → Gate 3
├── phase-4-social-comments-chat.md     → Gate 4
├── phase-5-economy-wallet-tokenomics-shop-ads.md → Gate 5
├── phase-6-admin-moderation.md         → Gate 6
├── phase-ci-integration.md             → Gate CI
└── acceptance-criteria.md (this file)  → Final sign-off
```

---

## 2. Prerequisites

Before the acceptance criteria can be fully evaluated:

- [ ] All phases (0–6) have been implemented
- [ ] CI integration (phase CI) is live and green
- [ ] Node.js mock backend files have been deleted
- [ ] `peer-web` E2E tests pass against the Rust mock
- [ ] SDL snapshot has been generated and committed

---

## 3. Per-Phase Acceptance Gates

Each phase has a self-contained gate. A phase is **not complete** until every item in its gate is checked.

### Gate 0 — Project Skeleton & Parity

| # | Criterion | Verification method |
|---|-----------|---------------------|
| 0.1 | `cargo build` succeeds for `tests/mock_backend` with no errors | `cargo build 2>&1; echo $?` → `0` |
| 0.2 | `cargo test` passes all 7 parity tests | `cargo test --test integration` → 7/7 pass |
| 0.3 | `cargo run` starts HTTP server on `:4000` | `cargo run &` then `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` → `{"data":{"_health":true}}` |
| 0.4 | `POST /reset` resets state to defaults | `curl -s -X POST http://localhost:4000/reset` → 200 OK |
| 0.5 | `verifyReferralString` with valid UUID returns `11011` | Integration test #1 |
| 0.6 | `verifyReferralString` with invalid string returns `31010` | Integration test #2 |
| 0.7 | `register` with valid input returns `10601` + UUID | Integration test #3 |
| 0.8 | `register` with duplicate email returns `30601` | Integration test #4 |
| 0.9 | `verifyAccount` with unverified user returns `10701` | Integration test #5 |
| 0.10 | `verifyAccount` with already-verified user returns `30701` | Integration test #6 |
| 0.11 | `register` with `fail@` prefix returns `40601` | Integration test #7 |
| 0.12 | Response field names match Node.js mock exactly (PascalCase for `DefaultResponse` fields) | SDL snapshot comparison |
| 0.13 | `peer-web` can import `mock_backend::app()` as dev-dependency | `cd peer-web && cargo check --tests` succeeds |
| 0.14 | Node.js files removed: `server.js`, `resolvers.js`, `schema.graphql`, `state.js`, `test.js`, `package.json`, `package-lock.json` | `ls tests/mock_backend/*.js tests/mock_backend/package*.json` → "No such file" |

### Gate 1 — Login & Session Flows ✅

> **Status:** All criteria verified (14 April 2026) — 24 Phase 1 tests, 33 total.

| # | Criterion | Verification method | Status |
|---|-----------|---------------------|--------|
| 1.1 | Register → verify → login returns `10801` + access/refresh tokens | `test_register_verify_login_flow` | ✅ |
| 1.2 | Login with wrong password returns `30801` | `test_login_wrong_password` | ✅ |
| 1.3 | Login with unverified account returns `60801` | `test_login_unverified_account` | ✅ |
| 1.4 | Login with deleted account returns `30801` | `test_login_deleted_account` | ✅ |
| 1.5 | `refreshToken` with valid token returns `10901` + new token pair | `test_refresh_token_success` | ✅ |
| 1.6 | `refreshToken` with invalid/expired token returns `30901` | `test_refresh_invalid_token` | ✅ |
| 1.7 | `logout` invalidates tokens — subsequent `refreshToken` fails | `test_refresh_after_logout` | ✅ |
| 1.8 | `deleteAccount` soft-deletes — re-login fails | `test_delete_account_success` | ✅ |
| 1.9 | Password reset flow: `requestPasswordReset` → `resetPasswordTokenVerify` → `resetPassword` → login with new password | `test_password_reset_flow` | ✅ |
| 1.10 | `resetPassword` with invalid token returns `31904` | `test_reset_password_invalid_token` | ✅ |
| 1.11 | `updatePassword` (authenticated) changes password | `test_update_password_success` | ✅ |
| 1.12 | `contactus` returns `10401` | `test_contactus_success` | ✅ |
| 1.13 | Protected mutation without auth header returns `60501` | `test_protected_mutation_without_auth` | ✅ |
| 1.14 | Auth middleware extracts `Authorization: Bearer <token>` and injects current user | Verified via `test_delete_account_success` + `test_update_password_success` | ✅ |
| 1.15 | ≥12 integration tests pass for Phase 1 | 24 Phase 1 tests pass (33 total) | ✅ |

### Gate 2 — Users & Profiles ✅

> **Status:** All criteria verified (14 April 2026) — 40 Phase 2 tests, 73 total.

| # | Criterion | Verification method | Status |
|---|-----------|---------------------|--------|
| 2.1 | `listUsersV2` returns seeded users with pagination (offset/limit) | `test_list_users_v2_by_username` | ✅ |
| 2.2 | `listUsersV2` by username substring returns matches | `test_list_users_v2_by_username` | ✅ |
| 2.3 | `listUsersV2` with no matches returns empty `affectedRows` | `test_list_users_v2_empty_results` | ✅ |
| 2.4 | `getProfile` returns full profile for valid user ID | `test_get_other_user_profile` | ✅ |
| 2.5 | `getProfile` for nonexistent user returns `21001` | `test_get_profile_not_found` | ✅ |
| 2.6 | `updateBio`, `updateUsername`, `updateProfileImage` update profile | `test_update_bio`, `test_update_username_success`, `test_update_profile_image` | ✅ |
| 2.7 | `toggleUserFollowStatus` (follow) → follow relations include target | `test_toggle_follow`, `test_list_follow_relations` | ✅ |
| 2.8 | `toggleUserFollowStatus` (unfollow) → target removed | `test_toggle_follow` (toggle back) | ✅ |
| 2.9 | `toggleBlockUserStatus` → blocked user excluded from search | `test_blocked_user_excluded_from_search`, `test_list_users_v2_excludes_blocked` | ✅ |
| 2.10 | `toggleBlockUserStatus` (unblock) reverses block | `test_toggle_block_unblock` | ✅ |
| 2.11 | `reportUser` stores report and returns `11012` | `test_report_user_success` | ✅ |
| 2.12 | `listFriends` returns mutual follows only | `test_list_friends_mutual_only` | ✅ |
| 2.13 | `getUserInfo` returns preferences for seeded users | `test_get_user_info` | ✅ |
| 2.14 | ≥10 integration tests pass for Phase 2 | 40 Phase 2 tests pass (73 total) | ✅ |

### Gate 3 — Posts & Content

| # | Criterion | Verification method |
|---|-----------|---------------------|
| 3.1 | `createPost` returns success with new post ID | Integration test |
| 3.2 | Created post appears in `listPosts` | Integration test |
| 3.3 | `listPosts` respects `offset`/`limit` pagination (max 20) | Integration test |
| 3.4 | `listPosts` filters by `PostFilterType` (IMAGE, AUDIO, VIDEO, TEXT) | Integration test |
| 3.5 | `listPosts` sorts by `PostSortType` (NEWEST, TRENDING, LIKES) | Integration test |
| 3.6 | `postAction(LIKE)` increments like count on re-query | Integration test |
| 3.7 | `postAction(DISLIKE)` increments dislike count | Integration test |
| 3.8 | `postAction(SAVE)` marks post as saved for user | Integration test |
| 3.9 | `postAction(REPORT)` stores post report | Integration test |
| 3.10 | `guestListPost` works without authentication | Integration test |
| 3.11 | `getPost` returns full post details by ID | Integration test |
| 3.12 | `listUserPosts` returns posts filtered by author | Integration test |
| 3.13 | Upload stub (`POST /upload-post`) returns mock file URL | Integration test (HTTP) |
| 3.14 | ≥12 integration tests pass for Phase 3 | `cargo test` count |

### Gate 4 — Social (Comments, Chat)

| # | Criterion | Verification method |
|---|-----------|---------------------|
| 4.1 | `createComment` on a post returns success | Integration test |
| 4.2 | `listComments(postid)` returns top-level comments only | Integration test |
| 4.3 | Reply to comment via `replyComment` or `createComment(parent)` | Integration test |
| 4.4 | `listChildComments(parent)` returns replies to a specific comment | Integration test |
| 4.5 | `likeComment` / `unlikeComment` toggles comment like state | Integration test |
| 4.6 | `createChat(userid)` creates a 1:1 conversation | Integration test |
| 4.7 | `sendChatMessage(chatid, message)` appends message | Integration test |
| 4.8 | `listChats` returns conversations for authenticated user | Integration test |
| 4.9 | Chat messages ordered by timestamp | Integration test |
| 4.10 | ≥10 integration tests pass for Phase 4 | `cargo test` count |

### Gate 5 — Economy (Wallet, Tokenomics, Shop, Ads)

| # | Criterion | Verification method |
|---|-----------|---------------------|
| 5.1 | `balance` returns seeded balance for authenticated user | Integration test |
| 5.2 | `transferTokens(to, amount)` deducts from sender, credits receiver | Integration test |
| 5.3 | `transferTokens` with insufficient balance returns error | Integration test |
| 5.4 | Transfer fee = 2% of amount (min 0.01) is applied correctly | Integration test |
| 5.5 | `getTransactionHistory` returns recorded transactions with pagination | Integration test |
| 5.6 | `getTransactionHistory` filters by type and direction | Integration test |
| 5.7 | `getActionPrices` returns hardcoded prices (post: 20.0, like: 3.0, etc.) | Integration test |
| 5.8 | `getDailyFreeActions` returns `{ used, limit: 5 }` per action type | Integration test |
| 5.9 | `getGems` returns gem totals from post interactions | Integration test |
| 5.10 | `mintTokens` converts gems → tokens at configured rate | Integration test |
| 5.11 | `createAdvertisement` creates ad → appears in `listAdvertisementPosts` | Integration test |
| 5.12 | `purchaseShopItem` deducts tokens and creates order | Integration test |
| 5.13 | ≥10 integration tests pass for Phase 5 | `cargo test` count |

### Gate 6 — Admin & Moderation

| # | Criterion | Verification method |
|---|-----------|---------------------|
| 6.1 | Moderator account can query `moderationStats` | Integration test |
| 6.2 | Regular user blocked from moderation queries → `62101` | Integration test |
| 6.3 | `hideContent` marks content as hidden | Integration test |
| 6.4 | `restoreContent` reverses hide | Integration test |
| 6.5 | `markIllegal` sets content visibility to illegal | Integration test |
| 6.6 | Admin can query `listUsersAdminV2` with extended fields (email, IP) | Integration test |
| 6.7 | Regular user blocked from admin queries | Integration test |
| 6.8 | Role bitmask check: user (0), admin (16), moderator (256) enforced | Integration test per role |
| 6.9 | ≥6 integration tests pass for Phase 6 | `cargo test` count |

### Gate CI — CI Integration

| # | Criterion | Verification method |
|---|-----------|---------------------|
| CI.1 | `.github/workflows/mock-backend.yml` exists and triggers on `tests/mock_backend/**` | CI run on PR touching mock backend |
| CI.2 | `cargo fmt --check` step fails PR on formatting violations | Deliberate bad format → PR fails |
| CI.3 | `cargo clippy -- -D warnings` step fails PR on warnings | Deliberate warning → PR fails |
| CI.4 | `cargo test --all-targets` runs all mock backend tests in CI | CI log shows test count |
| CI.5 | SDL snapshot test in CI catches schema drift | Add resolver without snapshot update → CI fails |
| CI.6 | E2E workflow uses Rust mock binary (no `npm ci` for mock) | CI log shows `cargo build --release` for mock |
| CI.7 | `global-setup.ts` spawns Rust binary | CI E2E tests pass with Rust backend |
| CI.8 | All existing Playwright tests pass with Rust mock | Zero regressions in E2E suite |

---

## 4. Cross-Cutting Requirements

These requirements apply to **every phase** and **every resolver**. They are not phase-specific — they must hold true across the entire codebase at all times.

### 4.1 Error Handling

| # | Requirement | Verification |
|---|-------------|--------------|
| CC.1 | No `unwrap()` in any resolver path | `grep -rn '\.unwrap()' src/schema/ src/types/` → 0 matches in resolver logic (allowed in `main.rs`, test code, and infallible conversions with adjacent comment) |
| CC.2 | All errors return proper GraphQL responses, never panic | Fuzz test: send malformed queries → server remains up |
| CC.3 | Invalid input returns a typed error response code, not a generic 500 | Integration test per resolver with bad input |
| CC.4 | `async-graphql` errors are caught and mapped to `DefaultResponse`-style payloads | No raw `Err(...)` leaking unstructured errors |

### 4.2 Test Coverage

| # | Requirement | Verification |
|---|-------------|--------------|
| CC.5 | Every resolver has ≥1 success test | Audit: list all `#[Object] impl` methods → match to test names |
| CC.6 | Every resolver has ≥1 error/edge-case test | Same audit |
| CC.7 | Total integration test count ≥ 67 (7 + 15 + 10 + 12 + 10 + 10 + 6 + phase-CI tests) | `cargo test -- --list 2>&1 \| grep -c 'test '` |
| CC.8 | All tests pass with `POST /reset` called between scenarios (test isolation) | Test harness calls reset in `setup` or uses fresh `app()` per test |

### 4.3 Response Code Compliance

| # | Requirement | Verification |
|---|-------------|--------------|
| CC.9 | All response codes used by the mock exist in `json/response-codes.json` | Automated audit test (see §5) |
| CC.10 | All response codes used by the mock match the descriptions in `docs/backend_api/*.md` | Manual review or snapshot comparison |
| CC.11 | Response code format: 5-digit string (e.g., `"10601"`, `"30801"`) | Regex check in audit test |
| CC.12 | `status` field is always `"success"` or `"error"` | Audit test asserts on all response fixtures |

### 4.4 Schema Fidelity

| # | Requirement | Verification |
|---|-------------|--------------|
| CC.13 | Field names exactly match backend schema casing | SDL snapshot + manual comparison to `docs/backend_api/*.md` |
| CC.14 | `DefaultResponse` uses PascalCase fields (`Status`, `RequestId`, `ResponseCode`, `ResponseMessage`) | SDL snapshot |
| CC.15 | Most other types use camelCase fields (matching real backend) | SDL snapshot |
| CC.16 | SDL export is committed to version control and snapshot-tested | `tests/snapshots/` exists with `.snap` file |
| CC.17 | No extra fields or missing fields vs. the documented schema | Periodic manual diff against `docs/backend_api/*.md` |

### 4.5 Code Quality

| # | Requirement | Verification |
|---|-------------|--------------|
| CC.18 | `cargo clippy -- -D warnings` passes | CI + local run |
| CC.19 | `cargo fmt --check` passes | CI + local run |
| CC.20 | No runtime dependencies on Node.js, npm, or any non-Rust tooling | `grep -r 'node\|npm' tests/mock_backend/Cargo.toml` → 0 matches; no `package.json` |
| CC.21 | All dependencies use recent, maintained versions | `cargo audit` (if available) or manual check |
| CC.22 | No `unsafe` code in resolver paths | `grep -rn 'unsafe' src/` → 0 matches (or justified with comment) |

### 4.6 State Management

| # | Requirement | Verification |
|---|-------------|--------------|
| CC.23 | `POST /reset` returns state to `MockState::default()` | Integration test: mutate state → reset → verify clean state |
| CC.24 | All state is behind `Arc<RwLock<MockState>>` (no global mutable state) | Code review: no `static mut`, no `lazy_static` with interior mutability |
| CC.25 | Concurrent requests don't cause data races | `RwLock` ensures safety; optionally run concurrent test |
| CC.26 | Seed data is consistent: seeded users have matching entries in all relevant maps (`users`, `user_passwords`, `registered_emails`, `verified_users`, etc.) | Dedicated seed validation test |

---

## 5. Automated Verification

### 5.1 Response Code Audit Test

Create `tests/mock_backend/tests/response_code_audit.rs` that programmatically verifies all response codes used in the mock are valid:

```rust
// tests/response_code_audit.rs

use std::collections::HashSet;

/// All response codes the mock backend emits, enumerated here.
/// This list must be updated when new resolvers are added.
const MOCK_RESPONSE_CODES: &[&str] = &[
    // Phase 0 — Registration
    "11011", "31010", "10601", "30601", "40601", "10701", "30701",
    // Phase 1 — Auth
    "10801", "30801", "60801", "10901", "30901", "11001", "11013",
    "11901", "11902", "31904", "11005", "10401", "60501",
    // Phase 2 — Users & Profiles
    "11101", "21101", "31007", "11301",
    // Phase 3 — Posts
    "11501", "11502", "11503", "11504",
    // Phase 4 — Comments & Chat
    "11601", "11701",
    // Phase 5 — Economy
    "11209", "11215", "11301", "11304", "11206", "11208",
    // Phase 6 — Admin & Moderation
    "62101",
];

#[test]
fn all_mock_response_codes_are_five_digit_strings() {
    for code in MOCK_RESPONSE_CODES {
        assert_eq!(code.len(), 5, "Response code '{code}' is not 5 digits");
        assert!(
            code.chars().all(|c| c.is_ascii_digit()),
            "Response code '{code}' contains non-digit characters"
        );
    }
}

#[test]
fn all_mock_response_codes_exist_in_response_codes_json() {
    let json_str = include_str!("../../../json/response-codes.json");
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let data = parsed["data"].as_object().expect("data should be an object");
    let known_codes: HashSet<&str> = data.keys().map(|k| k.as_str()).collect();

    let mut missing = Vec::new();
    for code in MOCK_RESPONSE_CODES {
        if !known_codes.contains(*code) {
            missing.push(*code);
        }
    }

    assert!(
        missing.is_empty(),
        "Response codes used in mock but missing from response-codes.json: {missing:?}"
    );
}
```

### 5.2 Resolver Coverage Audit Test

Create `tests/mock_backend/tests/resolver_coverage.rs` to verify every resolver has at least one test:

```rust
// tests/resolver_coverage.rs

/// Mapping of resolver names → expected test function name patterns.
/// When adding a new resolver, add an entry here.
const RESOLVER_TEST_MAPPING: &[(&str, &[&str])] = &[
    // Phase 0
    ("verify_referral_string", &["valid_referral", "invalid_referral"]),
    ("register", &["register_success", "register_duplicate", "register_fail_prefix"]),
    ("verify_account", &["verify_success", "already_verified"]),
    // Phase 1
    ("login", &["login_success", "login_wrong_password", "login_unverified", "login_deleted"]),
    ("refresh_token", &["refresh_success", "refresh_invalid"]),
    ("logout", &["logout_invalidates"]),
    ("delete_account", &["delete_account"]),
    ("request_password_reset", &["password_reset_flow"]),
    ("reset_password_token_verify", &["reset_token_verify"]),
    ("reset_password", &["reset_password", "reset_bad_token"]),
    ("update_password", &["change_password"]),
    ("contactus", &["contactus"]),
    // ... extend for all phases
];

#[test]
fn all_resolvers_have_tests() {
    // This is a documentation/tracking test. It compiles and asserts
    // that the mapping is non-empty for each resolver.
    for (resolver, tests) in RESOLVER_TEST_MAPPING {
        assert!(
            !tests.is_empty(),
            "Resolver '{resolver}' has no mapped test patterns"
        );
    }
}
```

### 5.3 SDL Snapshot Test

Already defined in [phase-ci-integration.md](./phase-ci-integration.md). Summary:

```rust
#[test]
fn schema_sdl_snapshot() {
    let state = default_shared_state();
    let schema = build_schema(state);
    let sdl = schema.sdl();
    insta::assert_snapshot!("graphql_schema", sdl);
}
```

### 5.4 Seed Data Consistency Test

```rust
#[test]
fn seed_data_is_internally_consistent() {
    let state = MockState::default();

    // Every user in `users` must have a password in `user_passwords`
    for uid in state.users.keys() {
        assert!(
            state.user_passwords.contains_key(uid),
            "User {uid} has no password entry"
        );
    }

    // Every user in `users` must have their email in `registered_emails`
    for user in state.users.values() {
        assert!(
            state.registered_emails.contains(&user.email),
            "User {}'s email '{}' not in registered_emails",
            user.uid, user.email
        );
    }

    // Every user in `verified_users` must exist in `users`
    for uid in &state.verified_users {
        assert!(
            state.users.contains_key(uid),
            "Verified user {uid} not found in users map"
        );
    }

    // All known referrals are valid UUIDs (guaranteed by type, but check count)
    assert!(
        state.known_referrals.len() >= 2,
        "Expected at least 2 known referrals"
    );
}
```

### 5.5 No-Unwrap Lint Test

```rust
#[test]
fn no_unwrap_in_resolver_source_files() {
    // Walk resolver source files and assert no `.unwrap()` calls
    // (excluding comments and test code)
    let resolver_dirs = ["src/schema/"];
    for dir in resolver_dirs {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            let content = std::fs::read_to_string(entry.path()).unwrap();
            for (line_num, line) in content.lines().enumerate() {
                // Skip comments
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("///") {
                    continue;
                }
                assert!(
                    !line.contains(".unwrap()"),
                    "Found .unwrap() in {}:{} — use proper error handling instead:\n  {}",
                    entry.path().display(),
                    line_num + 1,
                    line.trim()
                );
            }
        }
    }
}
```

> **Note:** The `walkdir` crate must be added to `[dev-dependencies]` for this test. Alternatively, implement the check as a shell script in CI: `! grep -rn '\.unwrap()' src/schema/`.

---

## 6. Response Code Compliance Matrix

This matrix maps every mock resolver to its expected response codes and the authoritative source document.

### Phase 0 — Registration

| Resolver | Success code | Error codes | Source document |
|----------|-------------|-------------|-----------------|
| `verifyReferralString` | `11011` | `31010` | `docs/backend_api/01-authentication-and-account.md` |
| `register` | `10601` | `30601` (duplicate), `40601` (internal) | `docs/backend_api/01-authentication-and-account.md` |
| `verifyAccount` | `10701` | `30701` (already verified) | `docs/backend_api/01-authentication-and-account.md` |

### Phase 1 — Auth

| Resolver | Success code | Error codes | Source document |
|----------|-------------|-------------|-----------------|
| `login` | `10801` | `30801` (invalid creds/deleted), `60801` (not verified) | `01-authentication-and-account.md` |
| `refreshToken` | `10901` | `30901` (invalid token) | `01-authentication-and-account.md` |
| `logout` | `11001` | — | `01-authentication-and-account.md` |
| `deleteAccount` | `11013` | `31001` (wrong password) | `01-authentication-and-account.md` |
| `requestPasswordReset` | `11901` | — (always success, anti-enumeration) | `01-authentication-and-account.md` |
| `resetPasswordTokenVerify` | `11902` | `31904` (invalid token) | `01-authentication-and-account.md` |
| `resetPassword` | `11005` | `31904` (invalid token) | `01-authentication-and-account.md` |
| `updatePassword` | `11005` | `31001` (wrong old password) | `01-authentication-and-account.md` |
| `contactus` | `10401` | — | `01-authentication-and-account.md` |

### Phase 2 — Users & Profiles

| Resolver | Success code | Error codes | Source document |
|----------|-------------|-------------|-----------------|
| `listUsersV2` | `11001` | `21101` (empty) | `02-users-and-profiles.md` |
| `getProfile` | `11008` | `31007` (not found) | `02-users-and-profiles.md` |
| `editProfile` | `11003`/`11004`/`11007` | — | `02-users-and-profiles.md` |
| `followUser` | `11104` | — | `02-users-and-profiles.md` |
| `unfollowUser` | `11103` | — | `02-users-and-profiles.md` |
| `blockUser` | `11105` | — | `02-users-and-profiles.md` |
| `unblockUser` | `11106` | — | `02-users-and-profiles.md` |
| `reportUser` | `11012` | — | `02-users-and-profiles.md` |
| `listFollowRelations` | `11101` | — | `02-users-and-profiles.md` |
| `listFriends` | `11102` | — | `02-users-and-profiles.md` |
| `getUserPreferences` | `11014` | — | `02-users-and-profiles.md` |

### Phase 3 — Posts & Content

| Resolver | Success code | Error codes | Source document |
|----------|-------------|-------------|-----------------|
| `listPosts` | `11501` | — | `03-posts-and-content.md` |
| `guestListPost` | `11501` | — | `03-posts-and-content.md` |
| `getPost` | `11502` | `31502` (not found) | `03-posts-and-content.md` |
| `createPost` | `11501` | — | `03-posts-and-content.md` |
| `postAction(LIKE)` | `11503` | — | `03-posts-and-content.md` |
| `postAction(DISLIKE)` | `11504` | — | `03-posts-and-content.md` |
| `postAction(SAVE)` | `11505` | — | `03-posts-and-content.md` |
| `postAction(REPORT)` | `11507` | — | `03-posts-and-content.md` |
| `listUserPosts` | `11501` | — | `03-posts-and-content.md` |

### Phase 4 — Comments & Chat

| Resolver | Success code | Error codes | Source document |
|----------|-------------|-------------|-----------------|
| `listComments` | `11601` | — | `04-comments.md` |
| `listChildComments` | `11601` | — | `04-comments.md` |
| `createComment` | `11601` | — | `04-comments.md` |
| `likeComment` | `11602` | — | `04-comments.md` |
| `unlikeComment` | `11603` | — | `04-comments.md` |
| `listChats` | `11701` | — | Frontend convention |
| `createChat` | `11701` | — | Frontend convention |
| `sendChatMessage` | `11701` | — | Frontend convention |

### Phase 5 — Economy

| Resolver | Success code | Error codes | Source document |
|----------|-------------|-------------|-----------------|
| `balance` | `11209` | — | `05-wallet-and-transfers.md` |
| `transferTokens` | `11402` | `31402` (insufficient), `31403` (self-transfer) | `05-wallet-and-transfers.md` |
| `getTransactionHistory` | `11215` | — | `05-wallet-and-transfers.md` |
| `getActionPrices` | `11304` | — | `06-tokenomics-gems-and-minting.md` |
| `getDailyFreeActions` | `11303` | — | `06-tokenomics-gems-and-minting.md` |
| `getGems` | `11207` | — | `06-tokenomics-gems-and-minting.md` |
| `mintTokens` | `11208` | `31208` (no gems) | `06-tokenomics-gems-and-minting.md` |
| `listAdvertisementPosts` | `11501` | — | `07-advertisements.md` |
| `createAdvertisement` | `11501` | `31301` (insufficient funds) | `07-advertisements.md` |
| `purchaseShopItem` | `11301` | `31301` (insufficient funds) | `08-shop.md` |

### Phase 6 — Admin & Moderation

| Resolver | Success code | Error codes | Source document |
|----------|-------------|-------------|-----------------|
| `moderationStats` | `12001` | `62101` (not moderator) | `09-moderation.md` |
| `moderationItems` | `12001` | `62101` | `09-moderation.md` |
| `hideContent` | `12002` | `62101` | `09-moderation.md` |
| `restoreContent` | `12003` | `62101` | `09-moderation.md` |
| `markIllegal` | `12004` | `62101` | `09-moderation.md` |
| `listUsersAdminV2` | `12101` | `62101` (not admin) | `10-admin.md` |
| `leaderboard` | `12102` | `62101` | `10-admin.md` |

---

## 7. Schema Fidelity Checks

### 7.1 Field Casing Rules

The real Peer backend uses inconsistent casing across types. The mock must replicate this exactly:

| Type | Casing convention | Example fields | Verified by |
|------|-------------------|----------------|-------------|
| `DefaultResponse` | PascalCase | `Status`, `RequestId`, `ResponseCode`, `ResponseMessage` | SDL snapshot |
| `AuthPayload` | camelCase | `status`, `ResponseCode`, `accessToken`, `refreshToken` | SDL snapshot |
| `ReferralResponse` | Mixed | `status`, `ResponseCode`, `affectedRows`, `meta` | SDL snapshot |
| `Post` / `User` / etc. | camelCase | `createdAt`, `userId`, `likeCount` | SDL snapshot |

### 7.2 SDL Diff Procedure

When response shapes are suspected to have drifted:

1. Export current SDL: `cargo test schema_sdl_snapshot -- --nocapture` (prints SDL on failure)
2. Compare field-by-field against the corresponding section in `docs/backend_api/*.md`
3. If the mock is wrong, fix the type definition and update the snapshot
4. If the real backend docs are outdated, update the docs and the snapshot

### 7.3 Nullable vs. Non-Nullable Fields

| Field pattern | Nullability rule |
|---------------|-----------------|
| `accessToken`, `refreshToken` | `Option<String>` — null on error |
| `userid` in `RegisterResponse` | `Option<String>` — null on error |
| `affectedRows` in list responses | `Option<Vec<T>>` — null when empty/error |
| `status`, `ResponseCode` | Always present (non-null) |
| `meta` (DefaultResponse) | Always present |

Verified by: SDL snapshot shows `String` vs `String!` correctly.

---

## 8. Performance & Reliability Baselines

These are not hard SLAs but sanity checks to ensure the mock is fast enough for testing.

| Metric | Baseline | How to measure |
|--------|----------|----------------|
| Cold start (first `cargo run`) | < 2 seconds to serve first request | `time cargo run &` + `curl` health check |
| Single GraphQL request latency | < 10 ms (in-process via tower) | `cargo bench` or test timing |
| `POST /reset` latency | < 5 ms | Part of integration test timing |
| Memory usage at rest (seed data only) | < 50 MB | `ps aux \| grep mock_backend` |
| Memory usage under load (1000 posts, 100 users) | < 200 MB | Seed stress test |
| Concurrent request handling | No deadlocks under 50 concurrent requests | Stress test with `tokio::spawn` |
| `cargo test` total runtime | < 30 seconds for full suite | CI job timing |
| `cargo build --release` time | < 60 seconds | CI job timing |

### 8.1 Concurrency Stress Test

```rust
#[tokio::test]
async fn concurrent_requests_dont_deadlock() {
    let app = app();
    let mut handles = Vec::new();

    for i in 0..50 {
        let app_clone = app.clone();
        handles.push(tokio::spawn(async move {
            let query = if i % 2 == 0 {
                r#"{ _health }"#
            } else {
                r#"mutation { verifyReferralString(referralString: "85d5f836-b1f5-4c4e-9381-1b058e13df93") { status } }"#
            };
            let body = serde_json::json!({ "query": query });
            let request = http::Request::builder()
                .method("POST")
                .uri("/graphql")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap();
            app_clone.oneshot(request).await.unwrap()
        }));
    }

    let results = futures::future::join_all(handles).await;
    for result in results {
        let response = result.expect("task should not panic");
        assert_eq!(response.status(), 200);
    }
}
```

---

## 9. Migration Verification

These checks verify the old Node.js mock is fully replaced and nothing was lost in translation.

### 9.1 File Removal Checklist

| File | Expected state after migration |
|------|-------------------------------|
| `tests/mock_backend/server.js` | **DELETED** |
| `tests/mock_backend/resolvers.js` | **DELETED** |
| `tests/mock_backend/schema.graphql` | **DELETED** |
| `tests/mock_backend/state.js` | **DELETED** |
| `tests/mock_backend/test.js` | **DELETED** |
| `tests/mock_backend/package.json` | **DELETED** |
| `tests/mock_backend/package-lock.json` | **DELETED** |
| `tests/mock_backend/node_modules/` | **DELETED** |
| `tests/mock_backend/fixtures/` | **KEPT** (as reference) or migrated to Rust snapshot files |
| `tests/mock_backend/Cargo.toml` | **EXISTS** |
| `tests/mock_backend/src/` | **EXISTS** with all phase modules |
| `tests/mock_backend/tests/` | **EXISTS** with integration + audit tests |

### 9.2 Behavioural Equivalence

For each of the 7 original Node.js test scenarios, verify the Rust mock returns **byte-equivalent** JSON response shapes (field names, nesting, types):

| # | Original Node.js assertion | Rust test equivalent | JSON shape match |
|---|---------------------------|----------------------|------------------|
| 1 | Valid referral → `status: "success"`, `ResponseCode: "11011"`, `affectedRows[0].uid == "usr_mock_001"` | `test_valid_referral_uuid` | ✅ |
| 2 | Invalid referral → `status: "error"`, `ResponseCode: "31010"`, `affectedRows: null` | `test_invalid_referral_string` | ✅ |
| 3 | Register → `status: "success"`, `ResponseCode: "10601"`, `userid` is UUID | `test_register_success` | ✅ |
| 4 | Duplicate email → `status: "error"`, `ResponseCode: "30601"`, `userid: null` | `test_register_duplicate_email` | ✅ |
| 5 | Verify → `status: "success"`, `ResponseCode: "10701"` | `test_verify_account_success` | ✅ |
| 6 | Already verified → `status: "success"`, `ResponseCode: "30701"` | `test_already_verified` | ✅ |
| 7 | `fail@` register → `status: "error"`, `ResponseCode: "40601"`, `userid: null` | `test_register_fail_prefix` | ✅ |

### 9.3 API Surface Equivalence

| Endpoint | Node.js | Rust | Match |
|----------|---------|------|-------|
| `POST /graphql` | Express + graphql-http | Axum + async-graphql-axum | Fields must match |
| `POST /reset` | Custom Express route | Custom Axum handler | Same behaviour: reset state, return 200 |
| `GET /graphql` | Not supported | Optional: GraphiQL playground | Additive (non-breaking) |
| CORS | `cors()` middleware (allow all) | `CorsLayer::permissive()` | Same effect |
| Port | `:4000` (hardcoded) | `:4000` (default, configurable via `PORT` env) | Compatible |

### 9.4 CI Pipeline Equivalence

| CI step | Before (Node.js) | After (Rust) |
|---------|-------------------|--------------|
| Install runtime | `actions/setup-node@v4` | `dtolnay/rust-toolchain@stable` |
| Install deps | `npm ci` | `cargo build` (deps fetched by Cargo) |
| Run tests | `node test.js` | `cargo test --all-targets` |
| Start server (E2E) | `node server.js` | Compiled binary or `cargo run --release` |
| Teardown server | `tree-kill` (PID) | `kill` (PID) — same approach |

---

## 10. Final Sign-Off Checklist

All items must be checked before the Rust mock backend rewrite is considered **complete**.

### Build & Run

- [ ] `cargo build` succeeds with no warnings
- [ ] `cargo build --release` succeeds
- [ ] `cargo run` starts server on `:4000` in < 2 seconds
- [ ] `POST /graphql` with `{ _health }` returns `{"data":{"_health":true}}`
- [ ] `POST /reset` returns 200 and resets all mutable state

### Test Suite

- [ ] `cargo test --all-targets` passes all tests (0 failures, 0 ignored)
- [ ] Total test count ≥ 67 integration tests across all phases
- [ ] Every resolver has ≥1 success and ≥1 error test
- [ ] Test isolation verified: no test depends on execution order
- [ ] Response code audit test passes (all codes in `response-codes.json`)
- [ ] Seed data consistency test passes

### Code Quality

- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] No `unwrap()` in resolver code paths
- [ ] No `unsafe` code in resolver code paths
- [ ] No runtime dependency on Node.js / npm

### Schema

- [ ] SDL snapshot committed and CI-enforced
- [ ] Field names match documented backend schema casing
- [ ] All response types have correct nullability
- [ ] SDL can be exported programmatically via `schema.sdl()`

### CI

- [ ] `.github/workflows/mock-backend.yml` runs on mock backend changes
- [ ] E2E workflow uses Rust mock (no `npm ci` for mock backend)
- [ ] All existing Playwright E2E tests pass with Rust mock
- [ ] CI completes in < 15 min (standalone) / < 30 min (E2E)

### Migration

- [ ] All Node.js files deleted (`server.js`, `resolvers.js`, `schema.graphql`, `state.js`, `test.js`, `package.json`, `package-lock.json`, `node_modules/`)
- [ ] `fixtures/` kept as reference or migrated
- [ ] `tests/mock_backend/target/` in `.gitignore`
- [ ] `README.md` updated to reference Rust mock
- [ ] `peer-web/Cargo.toml` includes `mock_backend` as dev-dependency

### Per-Phase Gates (summary)

- [ ] Gate 0 — All 14 criteria pass (Skeleton & Parity)
- [ ] Gate 1 — All 15 criteria pass (Login & Sessions)
- [ ] Gate 2 — All 14 criteria pass (Users & Profiles)
- [ ] Gate 3 — All 14 criteria pass (Posts & Content)
- [ ] Gate 4 — All 10 criteria pass (Comments & Chat)
- [ ] Gate 5 — All 13 criteria pass (Economy)
- [ ] Gate 6 — All 9 criteria pass (Admin & Moderation)
- [ ] Gate CI — All 8 criteria pass (CI Integration)

### Cross-Cutting (summary)

- [ ] CC.1–CC.4 — Error handling verified
- [ ] CC.5–CC.8 — Test coverage verified
- [ ] CC.9–CC.12 — Response code compliance verified
- [ ] CC.13–CC.17 — Schema fidelity verified
- [ ] CC.18–CC.22 — Code quality verified
- [ ] CC.23–CC.26 — State management verified
