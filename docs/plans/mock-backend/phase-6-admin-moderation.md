# Phase 6: Admin & Moderation

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** [Phase 3 — Posts & Content](./mock-backend-rust-rewrite.md#5-phase-3--posts--content) (requires posts/comments/users to moderate), [Phase 1 — Login & Session Flows](./mock-backend-rust-rewrite.md#3-phase-1--login--session-flows) (requires auth for all operations), [Phase 5 — Economy](./mock-backend-rust-rewrite.md#7-phase-5--economy-wallet-tokenomics-shop-ads) (requires wallets for admin user search, gem/mint operations)
> **Goal:** Add role-based access control (RBAC), content moderation (ticket listing, stats, hide/restore/illegal actions), and admin operations (extended user search, friendship graph, post comments, leaderboard generation, gem/mint administration) so the PHP admin panel and future Leptos admin pages work end-to-end against the mock.
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

**From `admin/js/schema.js` (PHP admin panel — GraphQL queries used via `admin/js/service.js`):**

| Operation | GraphQL SDL | Frontend file | Auth? | Role? |
|-----------|-------------|---------------|-------|-------|
| `moderationStats` | `query ModerationStats` | `schema.js` → `STATS` | Yes | MODERATOR (256) |
| `moderationItems(offset, limit)` | `query ModerationItems(...)` | `schema.js` → `LIST_ITEMS` | Yes | MODERATOR (256) |
| `moderationItems(offset, limit, contentType: post)` | `query ModerationItems(...)` | `schema.js` → `LIST_POST` | Yes | MODERATOR (256) |
| `moderationItems(offset, limit, contentType: comment)` | `query ModerationItems(...)` | `schema.js` → `LIST_COMMENT` | Yes | MODERATOR (256) |
| `moderationItems(offset, limit, contentType: user)` | `query ModerationItems(...)` | `schema.js` → `LIST_USER` | Yes | MODERATOR (256) |
| `performModeration(moderationTicketId, moderationAction)` | `mutation PerformModeration(...)` | `service.js` → `performModeration` | Yes | MODERATOR (256) |

**From `docs/backend_api/10-admin.md` (admin-only operations, not yet wired in any frontend):**

| Operation | Source doc | Purpose | Auth? | Role? |
|-----------|-----------|---------|-------|-------|
| `listUsersAdminV2(...)` | `10-admin.md` | Extended user search (includes email, ip, verified, roles) | Yes | ADMIN (16) |
| `allfriends(offset, limit)` | `10-admin.md` | Platform-wide friendship graph | Yes | ADMIN (16) |
| `postcomments(postid, offset, limit)` | `10-admin.md` | Admin view of all comments on a post (with subcomments) | Yes | ADMIN (16) |
| `generateLeaderboard(leaderboardParams)` | `10-admin.md` | Generate leaderboard CSV for a date range | Yes | ADMIN (16) |

**From `docs/backend_api/06-tokenomics-gems-and-minting.md` (admin gem/mint operations):**

| Operation | Type | Purpose | Auth? | Role? |
|-----------|------|---------|-------|-------|
| `gemster` | Query | Get uncollected gems statistics | Yes | ADMIN (16) |
| `dailygemstatus` | Query | Daily gem status overview | Yes | ADMIN (16) |
| `dailygemsresults(day)` | Query | Per-user gems breakdown for a day | Yes | ADMIN (16) |
| `getMintAccount` | Query | Mint account balance and details | Yes | ADMIN (16) |
| `globalwins` | Mutation | Convert pending interactions to gems | Yes | ADMIN (16) |
| `distributeTokensForGems(date)` | Mutation | Mint and distribute tokens from gems | Yes | ADMIN (16) |
| `gemsters(day)` | Mutation | Same as above but with relative day filter | Yes | ADMIN (16) |
| `alphaMint` | Mutation | One-time alpha token distribution | Yes | ADMIN (16) |

### Target State

After this phase, the mock backend will support:
- Role-based access control via async-graphql `Guard` trait (USER, ADMIN, MODERATOR)
- Moderation ticket lifecycle: report → review → hide/restore/illegal
- Moderation stats query (ticket counts by status)
- Moderation item listing with status/contentType filtering and pagination
- Content visibility state management (NORMAL, HIDDEN, ILLEGAL) with content-filter-aware behaviour
- Admin extended user search (by email, IP, verified, status, roles_mask)
- Admin friendship graph query
- Admin post comments view (with subcomments and visibility status)
- Admin leaderboard generation (mock CSV link)
- Admin gem statistics and daily gem status queries
- Admin gem-to-token minting operations
- Admin mint account balance query
- Automatic moderation ticket creation from existing report actions (Phase 3/4 `reportUser`, `postAction(REPORT)`)
- 11 new GraphQL query resolvers, 5 new mutation resolvers

### New file tree additions

```
packages/mock_backend/src/
├── schema/
│   ├── query/
│   │   ├── moderation.rs   # NEW: moderationStats, moderationItems
│   │   ├── admin.rs        # NEW: listUsersAdminV2, allfriends, postcomments, generateLeaderboard
│   │   └── admin_gems.rs   # NEW: gemster, dailygemstatus, dailygemsresults, getMintAccount
│   └── mutation/
│       ├── moderation.rs   # NEW: performModeration
│       └── admin_gems.rs   # NEW: globalwins, distributeTokensForGems, gemsters, alphaMint
├── types/
│   ├── moderation.rs       # NEW: ModerationStats, ModerationItem, ModerationStatus, etc.
│   ├── admin.rs            # NEW: AdminUser, AllUserFriends, PostCommentsData, LeaderboardResponse, etc.
│   └── admin_gems.rs       # NEW: GemsterResponse, DailyGemStatusData, GemstersResponse, MintAccount, etc.
├── guards.rs               # NEW: RoleGuard, require_role()
└── state.rs                # MODIFIED: new fields for moderation tickets, content visibility, roles
```

---

## 2. Prerequisites

### Phase 3 Completion

- [x] Posts exist in `MockState` (moderation targets posts)
- [x] `PostRecord` type is available with `id`, `visibility_status` fields
- [x] Post reporting (`postAction(REPORT)`) is implemented
- [x] Comment reporting is implemented (Phase 4)
- [x] User reporting (`reportUser`) is implemented (Phase 2)

### Phase 1 & 2 Completion

- [x] Auth middleware from Phase 1 is working (`require_auth()`, `get_current_user()`)
- [x] User profiles exist in `MockState` (admin user search, reporter info)
- [x] `DefaultResponse` type is available from Phase 0
- [x] `BasicUserInfo` type is available from Phase 2

### Phase 5 Completion

- [x] Wallets exist in `MockState` (admin user search shows `liquidity`)
- [x] Gem records exist in `MockState` (admin gem queries read from these)
- [x] `GemRecord` struct is available
- [x] Mint account exists as a system constant
- [x] `daily_actions_used` tracking is in place

### API Reference

All response codes and field names come from:
- `docs/backend_api/09-moderation.md`
- `docs/backend_api/10-admin.md`
- `docs/backend_api/06-tokenomics-gems-and-minting.md` (admin gem/mint sections)
- `admin/js/schema.js` (exact GraphQL field selections used by PHP admin panel)
- `admin/js/service.js` (performModeration mutation shape)

---

## 3. Task Breakdown

### Phase 6.A — Role-Based Access Control (`guards.rs`)

| # | Task | Notes |
|---|------|-------|
| A1 | Create `guards.rs` with `RoleGuard` struct implementing async-graphql `Guard` trait | Takes `required_role: u32` bitmask. Extracts current user from context → checks `roles_mask & required_role != 0`. Returns `FieldError` with code `62101` (not authorized) if denied. |
| A2 | Add `roles_mask: u32` field to the user record in `MockState` | Default `0` for regular users |
| A3 | Define role constants | `ROLE_USER: 0`, `ROLE_SYSTEM: 1`, `ROLE_COMPANY: 2`, `ROLE_BURN: 4`, `ROLE_WEB3_BRIDGE: 8`, `ROLE_ADMIN: 16`, `ROLE_PEER_SHOP: 32`, `ROLE_MODERATOR: 256` |
| A4 | Implement `require_moderator()` convenience guard | Returns `RoleGuard { required_role: 256 }` |
| A5 | Implement `require_admin()` convenience guard | Returns `RoleGuard { required_role: 16 }` |
| A6 | Update auth middleware to inject `roles_mask` into async-graphql context | After resolving `access_token → user_id`, also resolve `user → roles_mask` and insert into context |

### Phase 6.B — Moderation Types (`types/moderation.rs`)

| # | Task | Notes |
|---|------|-------|
| B1 | Create `types/moderation.rs` with `ModerationStatus` enum | `WaitingForReview`, `Hidden`, `Restored`, `Illegal` — maps to `waiting_for_review`, `hidden`, `restored`, `illegal` |
| B2 | Define `ModerationContentType` enum | `Post`, `Comment`, `User` — maps to `post`, `comment`, `user` |
| B3 | Define `ContentVisibilityStatus` enum | `Normal`, `Hidden`, `Illegal` — maps to `NORMAL`, `HIDDEN`, `ILLEGAL` |
| B4 | Define `ModerationStats` struct (SimpleObject) | `AmountAwaitingReview: i32`, `AmountHidden: i32`, `AmountRestored: i32`, `AmountIllegal: i32` — PascalCase per admin schema.js |
| B5 | Define `ModerationStatsResponse` struct | `status: String`, `ResponseCode: Option<String>` (deprecated), `meta: DefaultResponse`, `affectedRows: Option<ModerationStats>` |
| B6 | Define `BasicUserInfo` struct (SimpleObject) | `userid: String`, `img: Option<String>`, `username: String`, `slug: String`, `biography: Option<String>`, `visibilityStatus: Option<String>`, `hasActiveReports: Option<bool>`, `isHiddenForUsers: Option<bool>`, `updatedat: Option<String>` — matches admin schema.js field selections |
| B7 | Define `TargetContent` struct (SimpleObject) | `post: Option<Post>`, `comment: Option<Comment>`, `user: Option<BasicUserInfo>` — only one non-null depending on `targettype` |
| B8 | Define `ModerationItem` struct (SimpleObject) | `moderationTicketId: ID`, `targetContentId: ID`, `targettype: String`, `reportscount: i32`, `status: String`, `createdat: String`, `targetcontent: TargetContent`, `reporters: Vec<BasicUserInfo>`, `moderatedBy: Option<BasicUserInfo>` — matches schema.js `LIST_ITEMS` |
| B9 | Define `ModerationItemListResponse` struct | `status: String`, `ResponseCode: Option<String>` (deprecated), `meta: DefaultResponse`, `affectedRows: Vec<ModerationItem>` |
| B10 | Export from `types/mod.rs` | Add `pub mod moderation;` |

### Phase 6.C — Admin Types (`types/admin.rs`)

| # | Task | Notes |
|---|------|-------|
| C1 | Create `types/admin.rs` with `AdminUser` struct (SimpleObject) | Extends base `User` with `email: Option<String>`, `verified: Option<i32>`, `roles_mask: Option<i32>`, `ip: Option<String>`, `liquidity: Option<Decimal>`, `situation: Option<String>` — matches 10-admin.md extended user type |
| C2 | Define `AdminUserListResponse` struct | `meta: DefaultResponse`, `status: String`, `counter: i32`, `ResponseCode: Option<String>` (deprecated), `affectedRows: Option<Vec<AdminUser>>` |
| C3 | Define `AllUserInfo` struct (SimpleObject) | `followerid: Option<ID>`, `followername: Option<String>`, `followedid: Option<ID>`, `followedname: Option<String>` — matches `allFriends` response |
| C4 | Define `AllUserFriends` struct | `meta: DefaultResponse`, `status: String`, `counter: i32`, `ResponseCode: Option<String>` (deprecated), `affectedRows: Option<Vec<AllUserInfo>>` |
| C5 | Define `PostSubCommentsData` struct (SimpleObject) | `commentid: Option<ID>`, `userid: Option<ID>`, `postid: Option<ID>`, `parentid: Option<ID>`, `content: Option<String>`, `createdat: Option<String>`, `amountlikes: Option<Decimal>`, `amountreplies: Option<Decimal>`, `isliked: Option<bool>`, `user: Option<BasicUserInfo>`, `visibilityStatus: ContentVisibilityStatus`, `isHiddenForUsers: bool` |
| C6 | Define `PostCommentsData` struct (SimpleObject) | Same as C5 plus `subcomments: Option<Vec<PostSubCommentsData>>` |
| C7 | Define `PostCommentsResponse` struct | `meta: DefaultResponse`, `status: String`, `counter: i32`, `ResponseCode: Option<String>` (deprecated), `affectedRows: Option<Vec<PostCommentsData>>` |
| C8 | Define `LeaderboardParamsInput` (InputObject) | `start_date: String`, `end_date: String`, `leaderboardUsersCount: i32` |
| C9 | Define `LeaderboardResponse` struct | `meta: DefaultResponse`, `leaderboardResultLink: Option<String>` |
| C10 | Export from `types/mod.rs` | Add `pub mod admin;` |

### Phase 6.D — Admin Gem Types (`types/admin_gems.rs`)

| # | Task | Notes |
|---|------|-------|
| D1 | Create `types/admin_gems.rs` with `DailyGemStatusData` struct (SimpleObject) | `d0: Decimal`, `d1: Decimal`, `d2: Decimal`, `d3: Decimal`, `d4: Decimal`, `d5: Decimal`, `d6: Decimal`, `d7: Decimal`, `w0: Decimal`, `m0: Decimal`, `y0: Decimal` — matches 06-tokenomics doc |
| D2 | Define `GemsterResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<DailyGemStatusData>` |
| D3 | Define `DailyGemsResultsUserData` struct (SimpleObject) | `userid: Option<ID>`, `pkey: Option<ID>`, `gems: Option<Decimal>` |
| D4 | Define `DailyGemsResultsData` struct (SimpleObject) | `data: Option<Vec<DailyGemsResultsUserData>>`, `totalGems: Option<Decimal>` |
| D5 | Define `DailyGemsResultsResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<DailyGemsResultsData>` |
| D6 | Define `WinStatus` struct (SimpleObject) | `totalGems: Decimal`, `gemsintoken: Decimal`, `bestatigung: Decimal` |
| D7 | Define `GemstersUserStatusDetails` struct (SimpleObject) | `gemid: Option<ID>`, `userid: Option<ID>`, `postid: Option<ID>`, `fromid: Option<ID>`, `gems: Option<Decimal>`, `numbers: Option<Decimal>`, `whereby: Option<Decimal>`, `createdat: Option<String>` |
| D8 | Define `GemstersUserStatus` struct (SimpleObject) | `userid: Option<ID>`, `gems: Option<Decimal>`, `tokens: Option<Decimal>`, `percentage: Option<Decimal>`, `details: Option<Vec<GemstersUserStatusDetails>>` |
| D9 | Define `GemstersData` struct (SimpleObject) | `winStatus: Option<WinStatus>`, `userStatus: Option<Vec<GemstersUserStatus>>` |
| D10 | Define `GemstersResponse` struct | `meta: DefaultResponse`, `counter: i32`, `affectedRows: Option<GemstersData>` |
| D11 | Define `MintAccount` struct (SimpleObject) | `accountid: ID`, `initialBalance: Decimal`, `currentBalance: Decimal`, `updatedat: String` |
| D12 | Define `MintAccountResponse` struct | `meta: DefaultResponse`, `mintAccount: Option<MintAccount>` |
| D13 | Export from `types/mod.rs` | Add `pub mod admin_gems;` |

### Phase 6.E — State Extensions (`state.rs`, `seed.rs`)

| # | Task | Notes |
|---|------|-------|
| E1 | Add `ModerationTicketRecord` struct to state | `id: Uuid`, `target_content_id: Uuid`, `target_type: String` ("post"/"comment"/"user"), `reporter_ids: Vec<Uuid>`, `reports_count: i32`, `status: String`, `moderated_by: Option<Uuid>`, `created_at: String` |
| E2 | Add `moderation_tickets: Vec<ModerationTicketRecord>` to `MockState` | |
| E3 | Add `content_visibility: HashMap<Uuid, String>` to `MockState` | `content_id → "NORMAL"/"HIDDEN"/"ILLEGAL"`. Shared for posts, comments, users. Default is `"NORMAL"` (absence = NORMAL). |
| E4 | Add `roles_mask` field to user records in `MockState` | `HashMap<Uuid, u32>` or inline on the user struct |
| E5 | Add `minted_dates: HashSet<String>` to `MockState` if not already from Phase 5 | Track which dates have been minted to prevent duplicate distribution |
| E6 | Add `alpha_minted: bool` to `MockState` | Tracks whether `alphaMint` has been run |
| E7 | Add `mint_initial_balance: Decimal` constant | `5_000_000.0` — initial mint account balance |
| E8 | Update `MockState::default()` and `MockState::reset()` | Initialize/clear all new fields, preserve seed data |

### Phase 6.F — Seed Data (`seed.rs`)

| # | Task | Notes |
|---|------|-------|
| F1 | Add seed admin user with deterministic UUID | `SEED_USER_ADMIN`, `roles_mask: 16`, username `"admin"`, email `"admin@peerapp.de"`, password `"Admin1234"`, wallet balance `10000.0` |
| F2 | Add seed moderator user with deterministic UUID | `SEED_USER_MODERATOR`, `roles_mask: 256`, username `"moderator"`, email `"mod@peerapp.de"`, password `"Mod1234"`, wallet balance `5000.0` |
| F3 | Add 3 seed moderation tickets | 1 post ticket (`waiting_for_review`), 1 comment ticket (`waiting_for_review`), 1 user ticket (`hidden`, moderated by seed moderator) |
| F4 | Set seed visibility states | The `hidden` ticket's target content should have `content_visibility["HIDDEN"]` |
| F5 | Verify both seed admin and mod are registered, verified, and have active tokens | So they can immediately make authenticated requests in tests |
| F6 | Add `ip` field to seed users | Admin user: `"192.168.1.1"`, others: `"192.168.1.2"`, etc. |

### Phase 6.G — Moderation Query Resolvers (`schema/query/moderation.rs`)

| # | Task | Notes |
|---|------|-------|
| G1 | Create `schema/query/moderation.rs` with `ModerationQuery` struct | Uses `#[Object]` |
| G2 | Implement `moderation_stats` resolver | Guard: `require_moderator()`. Count tickets by status in `moderation_tickets`. Return `ModerationStatsResponse`. Code `12101`. |
| G3 | Implement `moderation_items(status?, contentType?, offset?, limit?)` resolver | Guard: `require_moderator()`. Filter `moderation_tickets` by optional `ModerationStatus` and `ModerationContentType`. Paginate with `offset`/`limit` (max 20). For each ticket, resolve `TargetContent` by looking up the target in posts/comments/users. Resolve `reporters` and `moderatedBy` as `BasicUserInfo`. Code `12102` / `62101`. |
| G4 | Register `ModerationQuery` in `QueryRoot` merged object | |

### Phase 6.H — Moderation Mutation Resolvers (`schema/mutation/moderation.rs`)

| # | Task | Notes |
|---|------|-------|
| H1 | Create `schema/mutation/moderation.rs` with `ModerationMutation` struct | Uses `#[Object]` |
| H2 | Implement `perform_moderation(moderationTicketId, moderationAction)` mutation | Guard: `require_moderator()`. Validate ticket exists (`22103`). Validate action is `hidden`, `restored`, or `illegal` (`32101`). Check ticket not already in a terminal state with same action (`32103`). Update ticket status, set `moderated_by` to current user. Update `content_visibility` for the target content. Reset report counts. Code `12103`. |
| H3 | Implement visibility side effects | **Hidden**: set `content_visibility[target_id] = "HIDDEN"`, reset reports. **Illegal**: set `content_visibility[target_id] = "ILLEGAL"`, reset reports. **Restored**: set `content_visibility[target_id] = "NORMAL"`, reset reports. |
| H4 | Register `ModerationMutation` in `MutationRoot` merged object | |

### Phase 6.I — Admin Query Resolvers (`schema/query/admin.rs`)

| # | Task | Notes |
|---|------|-------|
| I1 | Create `schema/query/admin.rs` with `AdminQuery` struct | Uses `#[Object]` |
| I2 | Implement `list_users_admin_v2(contentFilterBy?, userid?, email?, username?, status?, verified?, ip?, offset?, limit?)` resolver | Guard: `require_admin()`. Validate `userid` and `username` not both provided (`31012`). Validate UUID format (`30201`), username format (`30202`), offset (`30203`), limit (`30204`), IP format (`30257`). Search users, return extended `AdminUser` with `email`, `verified`, `roles_mask`, `ip`, `liquidity`. Code `11009` / `31007`. |
| I3 | Implement `allfriends(offset?, limit?)` resolver | Guard: `require_admin()`. Iterate all follow relationships. Return `AllUserFriends` with `AllUserInfo` entries (follower/followed IDs and names). Paginate. |
| I4 | Implement `postcomments(postid, offset?, limit?)` resolver | Guard: `require_admin()`. List all comments on the specified post. For each top-level comment, include `subcomments` (replies). Include `visibilityStatus` and `isHiddenForUsers` fields. Paginate top-level comments. Code `11101` / `21101`. |
| I5 | Implement `generate_leaderboard(leaderboardParams)` resolver | Guard: `require_admin()`. Validate date format and `end_date >= start_date` (`33002`). Generate a mock CSV link (no actual file creation). Compute mock leaderboard from user post interaction data. Return `LeaderboardResponse` with `leaderboardResultLink`. Code `12301` / `22201`. |
| I6 | Register `AdminQuery` in `QueryRoot` merged object | |

### Phase 6.J — Admin Gem Query Resolvers (`schema/query/admin_gems.rs`)

| # | Task | Notes |
|---|------|-------|
| J1 | Create `schema/query/admin_gems.rs` with `AdminGemQuery` struct | Uses `#[Object]` |
| J2 | Implement `gemster` resolver | Guard: `require_admin()`. Aggregate uncollected gems from `gems` state grouped by day (d0–d7), week (w0), month (m0), year (y0). Return `GemsterResponse`. Code `11207`. |
| J3 | Implement `dailygemstatus` resolver | Guard: `require_admin()`. Same aggregation as `gemster` — return `DailyGemStatusData`. Code `11207`. |
| J4 | Implement `dailygemsresults(day)` resolver | Guard: `require_admin()`. For the specified `DayFilterType`, compute per-user gem totals. Return `DailyGemsResultsResponse` with `totalGems` and per-user `[DailyGemsResultsUserData]`. Code `11207` / `21206`. |
| J5 | Implement `get_mint_account` resolver | Guard: `require_admin()`. Return `MintAccountResponse` with `MintAccount { accountid: SYSTEM_MINT_ACCOUNT, initialBalance: 5_000_000.0, currentBalance: wallets[SYSTEM_MINT_ACCOUNT], updatedat }`. Code `0`. |
| J6 | Register `AdminGemQuery` in `QueryRoot` merged object | |

### Phase 6.K — Admin Gem Mutation Resolvers (`schema/mutation/admin_gems.rs`)

| # | Task | Notes |
|---|------|-------|
| K1 | Create `schema/mutation/admin_gems.rs` with `AdminGemMutation` struct | Uses `#[Object]` |
| K2 | Implement `globalwins` mutation | Guard: `require_admin()`. For each user's posts, convert pending interactions (views, likes, dislikes, comments since last run) into `GemRecord` entries using gem return rates (view 0.25, like 5.0, dislike -3.0, comment 2.0). Mark interactions as processed. Code `11206` / `21205`. |
| K3 | Implement `distribute_tokens_for_gems(date)` mutation | Guard: `require_admin()`. Validate date format (`30105`). Validate not future date. Check `minted_dates` for duplicate (`31204`). Calculate `totalGems` for date. Calculate `gemsintoken = 5000.0 / totalGems`. For each user: `tokens = user_gems × gemsintoken`. Credit wallets from `SYSTEM_MINT_ACCOUNT`. Record in `minted_dates`. Return `GemstersResponse`. Code `11208` / `21206`. |
| K4 | Implement `gemsters(day)` mutation | Guard: `require_admin()`. Convert `DayFilterType` to a date string, then delegate to same logic as K3. |
| K5 | Implement `alpha_mint` mutation | Guard: `require_admin()`. Check `alpha_minted` flag (`31204` if already run). Mock: credit each seeded non-system user with 100.0 tokens from mint account. Set `alpha_minted = true`. Code `200`. |
| K6 | Register `AdminGemMutation` in `MutationRoot` merged object | |

### Phase 6.L — Report-to-Ticket Integration

| # | Task | Notes |
|---|------|-------|
| L1 | Update Phase 2's `reportUser` resolver | After recording the report, check if a moderation ticket already exists for the target user. If yes, increment `reports_count` and add reporter to `reporter_ids`. If no, create a new `ModerationTicketRecord` with `target_type: "user"`, `status: "waiting_for_review"`. |
| L2 | Update Phase 3's `postAction(REPORT)` resolver | Same deduplication logic: find or create ticket with `target_type: "post"`. |
| L3 | Update Phase 4's comment report logic (if exists) | Same deduplication logic: find or create ticket with `target_type: "comment"`. |
| L4 | Implement report deduplication | Use `target_content_id` as the deduplication key. Same user cannot report the same content twice. Restored content cannot be re-reported (`32104`). |

### Phase 6.M — Content Visibility Integration

| # | Task | Notes |
|---|------|-------|
| M1 | Update Phase 3's `list_posts` / `get_post` resolvers | Check `content_visibility[post_id]`. If `ILLEGAL`, filter out completely. If `HIDDEN`, include `visibilityStatus: "HIDDEN"` and `isHiddenForUsers: true` in response. |
| M2 | Update Phase 4's `list_comments` / `list_child_comments` resolvers | Same visibility filtering for comments. |
| M3 | Update Phase 2's profile/user listing resolvers | Same visibility filtering for users. |
| M4 | Add `visibilityStatus` and `isHiddenForUsers` fields to `Post`, `Comment`, `User` GraphQL types if not already present | Ensure these fields are returned in all relevant query responses. |

### Phase 6.N — Schema Assembly Updates

| # | Task | Notes |
|---|------|-------|
| N1 | Add `ModerationQuery`, `AdminQuery`, `AdminGemQuery` to `QueryRoot` | |
| N2 | Add `ModerationMutation`, `AdminGemMutation` to `MutationRoot` | |

### Phase 6.O — Cleanup & Validation

| # | Task | Notes |
|---|------|-------|
| O1 | Run `cargo clippy -- -D warnings` | Fix all warnings |
| O2 | Run `cargo fmt --check` | Fix formatting |
| O3 | Run full test suite (`cargo test --all-targets`) | All Phase 0–6 tests pass |
| O4 | Manual smoke test with `cargo run` + curl | Verify HTTP layer for all new endpoints with moderator/admin tokens |

---

## 4. Implementation Details

### 4.1 Role-Based Access Control (`guards.rs`)

```rust
use async_graphql::{Context, Guard, Result};

/// Role bitmask constants.
pub const ROLE_USER: u32 = 0;
pub const ROLE_SYSTEM: u32 = 1;
pub const ROLE_COMPANY: u32 = 2;
pub const ROLE_BURN: u32 = 4;
pub const ROLE_WEB3_BRIDGE: u32 = 8;
pub const ROLE_ADMIN: u32 = 16;
pub const ROLE_PEER_SHOP: u32 = 32;
pub const ROLE_MODERATOR: u32 = 256;

/// Guard that checks whether the authenticated user has the required role bitmask.
pub struct RoleGuard {
    pub required_role: u32,
}

impl RoleGuard {
    pub fn new(required_role: u32) -> Self {
        Self { required_role }
    }
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        // First check authentication
        let roles_mask = ctx
            .data_opt::<UserRolesMask>()
            .ok_or_else(|| {
                async_graphql::Error::new("Not authenticated")
                    .extend_with(|_, e| e.set("code", "60501"))
            })?
            .0;

        // Check role
        if self.required_role == ROLE_USER || (roles_mask & self.required_role) != 0 {
            Ok(())
        } else {
            Err(async_graphql::Error::new("Not authorized")
                .extend_with(|_, e| e.set("code", "62101")))
        }
    }
}

/// Wrapper for the user's roles bitmask, inserted into context by auth middleware.
pub struct UserRolesMask(pub u32);

/// Convenience: builds a guard requiring MODERATOR role.
pub fn require_moderator() -> RoleGuard {
    RoleGuard::new(ROLE_MODERATOR)
}

/// Convenience: builds a guard requiring ADMIN role.
pub fn require_admin() -> RoleGuard {
    RoleGuard::new(ROLE_ADMIN)
}
```

### 4.2 Moderation Types (`types/moderation.rs`)

```rust
use async_graphql::{Enum, SimpleObject, ID};
use serde::{Deserialize, Serialize};

use super::post::Post;
use super::comment::Comment;
use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Status of a moderation ticket.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum ModerationStatus {
    WaitingForReview,
    Hidden,
    Restored,
    Illegal,
}

impl ModerationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WaitingForReview => "waiting_for_review",
            Self::Hidden => "hidden",
            Self::Restored => "restored",
            Self::Illegal => "illegal",
        }
    }
}

/// Type of content being moderated.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum ModerationContentType {
    Post,
    Comment,
    User,
}

/// Visibility status applied to content after moderation.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ContentVisibilityStatus {
    Normal,
    Hidden,
    Illegal,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Aggregate counts of moderation tickets by status.
///
/// Field names are PascalCase to match the admin schema.js `STATS` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ModerationStats {
    #[graphql(name = "AmountAwaitingReview")]
    pub amount_awaiting_review: i32,
    #[graphql(name = "AmountHidden")]
    pub amount_hidden: i32,
    #[graphql(name = "AmountRestored")]
    pub amount_restored: i32,
    #[graphql(name = "AmountIllegal")]
    pub amount_illegal: i32,
}

/// Response for `moderationStats` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ModerationStatsResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ModerationStats>,
}

/// Basic user info for reporters, moderators, and user-type moderation targets.
///
/// Must match `admin/js/schema.js` field selections for `reporters` and `moderatedBy`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BasicUserInfo {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: String,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
    pub updatedat: Option<String>,
}

/// The reported content, only one field is non-null depending on `targettype`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TargetContent {
    pub post: Option<Post>,
    pub comment: Option<Comment>,
    pub user: Option<BasicUserInfo>,
}

/// A single moderation ticket.
///
/// Must match `admin/js/schema.js` `LIST_ITEMS` field selections.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ModerationItem {
    #[graphql(name = "moderationTicketId")]
    pub moderation_ticket_id: ID,
    #[graphql(name = "targetContentId")]
    pub target_content_id: ID,
    pub targettype: String,
    pub reportscount: i32,
    pub status: String,
    pub createdat: String,
    pub targetcontent: TargetContent,
    pub reporters: Vec<BasicUserInfo>,
    #[graphql(name = "moderatedBy")]
    pub moderated_by: Option<BasicUserInfo>,
}

/// Response for `moderationItems` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ModerationItemListResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Vec<ModerationItem>,
}
```

### 4.3 Admin Types (`types/admin.rs`)

```rust
use async_graphql::{InputObject, SimpleObject, ID};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::moderation::{BasicUserInfo, ContentVisibilityStatus};
use super::registration::DefaultResponse;

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Extended user type for admin search results.
///
/// Includes fields not visible to regular users: email, verified, roles_mask, ip, liquidity.
/// Matches `10-admin.md` extended User type.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdminUser {
    pub id: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    pub biography: Option<String>,
    pub status: Option<i32>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
    // Admin-only fields:
    pub email: Option<String>,
    pub verified: Option<i32>,
    pub roles_mask: Option<i32>,
    pub ip: Option<String>,
    pub liquidity: Option<Decimal>,
    pub situation: Option<String>,
}

/// Response for `listUsersAdminV2` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdminUserListResponse {
    pub meta: DefaultResponse,
    pub status: String,
    pub counter: i32,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AdminUser>>,
}

/// A single follow relationship for the friendship graph.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AllUserInfo {
    pub followerid: Option<ID>,
    pub followername: Option<String>,
    pub followedid: Option<ID>,
    pub followedname: Option<String>,
}

/// Response for `allfriends` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AllUserFriends {
    pub meta: DefaultResponse,
    pub status: String,
    pub counter: i32,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AllUserInfo>>,
}

/// A subcomment (reply) in the admin post comments view.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostSubCommentsData {
    pub commentid: Option<ID>,
    pub userid: Option<ID>,
    pub postid: Option<ID>,
    pub parentid: Option<ID>,
    pub content: Option<String>,
    pub createdat: Option<String>,
    pub amountlikes: Option<Decimal>,
    pub amountreplies: Option<Decimal>,
    pub isliked: Option<bool>,
    pub user: Option<BasicUserInfo>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
}

/// A top-level comment in the admin post comments view (with subcomments).
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostCommentsData {
    pub commentid: Option<ID>,
    pub userid: Option<ID>,
    pub postid: Option<ID>,
    pub parentid: Option<ID>,
    pub content: Option<String>,
    pub createdat: Option<String>,
    pub amountlikes: Option<Decimal>,
    pub isliked: Option<bool>,
    pub user: Option<BasicUserInfo>,
    pub subcomments: Option<Vec<PostSubCommentsData>>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
}

/// Response for `postcomments` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostCommentsResponse {
    pub meta: DefaultResponse,
    pub status: String,
    pub counter: i32,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<PostCommentsData>>,
}

/// Input for `generateLeaderboard` query.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct LeaderboardParamsInput {
    pub start_date: String,
    pub end_date: String,
    #[graphql(name = "leaderboardUsersCount")]
    pub leaderboard_users_count: i32,
}

/// Response for `generateLeaderboard` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct LeaderboardResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "leaderboardResultLink")]
    pub leaderboard_result_link: Option<String>,
}
```

### 4.4 Admin Gem Types (`types/admin_gems.rs`)

```rust
use async_graphql::{SimpleObject, ID};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Daily gem status data aggregated across all users.
///
/// Each field is a Decimal representing gems for the corresponding time period.
/// Matches `DailyGemStatusData` in 06-tokenomics-gems-and-minting.md.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemStatusData {
    pub d0: Decimal,    // Today
    pub d1: Decimal,    // Yesterday
    pub d2: Decimal,    // 2 days ago
    pub d3: Decimal,
    pub d4: Decimal,
    pub d5: Decimal,
    pub d6: Decimal,
    pub d7: Decimal,    // 7 days ago
    pub w0: Decimal,    // This week total
    pub m0: Decimal,    // This month total
    pub y0: Decimal,    // This year total
}

/// Response for `gemster` and `dailygemstatus` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemsterResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<DailyGemStatusData>,
}

/// Per-user gem data for a specific day.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemsResultsUserData {
    pub userid: Option<ID>,
    pub pkey: Option<ID>,
    pub gems: Option<Decimal>,
}

/// Aggregated gems results for a day.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemsResultsData {
    pub data: Option<Vec<DailyGemsResultsUserData>>,
    #[graphql(name = "totalGems")]
    pub total_gems: Option<Decimal>,
}

/// Response for `dailygemsresults` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemsResultsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<DailyGemsResultsData>,
}

/// Win status from a gem-to-token distribution.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct WinStatus {
    #[graphql(name = "totalGems")]
    pub total_gems: Decimal,
    pub gemsintoken: Decimal,
    pub bestatigung: Decimal,
}

/// Detailed gem source for a single distribution entry.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersUserStatusDetails {
    pub gemid: Option<ID>,
    pub userid: Option<ID>,
    pub postid: Option<ID>,
    pub fromid: Option<ID>,
    pub gems: Option<Decimal>,
    pub numbers: Option<Decimal>,
    pub whereby: Option<Decimal>,
    pub createdat: Option<String>,
}

/// Per-user distribution result.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersUserStatus {
    pub userid: Option<ID>,
    pub gems: Option<Decimal>,
    pub tokens: Option<Decimal>,
    pub percentage: Option<Decimal>,
    pub details: Option<Vec<GemstersUserStatusDetails>>,
}

/// Container for distribution results.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersData {
    #[graphql(name = "winStatus")]
    pub win_status: Option<WinStatus>,
    #[graphql(name = "userStatus")]
    pub user_status: Option<Vec<GemstersUserStatus>>,
}

/// Response for `distributeTokensForGems` / `gemsters` mutations.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<GemstersData>,
}

/// Mint account details.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct MintAccount {
    pub accountid: ID,
    #[graphql(name = "initialBalance")]
    pub initial_balance: Decimal,
    #[graphql(name = "currentBalance")]
    pub current_balance: Decimal,
    pub updatedat: String,
}

/// Response for `getMintAccount` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct MintAccountResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "mintAccount")]
    pub mint_account: Option<MintAccount>,
}
```

### 4.5 State Extensions (`state.rs`)

```rust
// Added to MockState:

/// A moderation ticket record.
pub struct ModerationTicketRecord {
    pub id: Uuid,
    pub target_content_id: Uuid,
    pub target_type: String,        // "post", "comment", "user"
    pub reporter_ids: Vec<Uuid>,
    pub reports_count: i32,
    pub status: String,             // "waiting_for_review", "hidden", "restored", "illegal"
    pub moderated_by: Option<Uuid>,
    pub created_at: String,
}

pub struct MockState {
    // ... existing fields from Phase 0–5 ...

    // Phase 6 additions:
    pub moderation_tickets: Vec<ModerationTicketRecord>,
    pub content_visibility: HashMap<Uuid, String>,  // content_id → "NORMAL"/"HIDDEN"/"ILLEGAL"
    pub alpha_minted: bool,
}
```

### 4.6 Seed Data (`seed.rs`)

```rust
use uuid::uuid;

// ---- Admin & Moderator seed users ----

pub const SEED_USER_ADMIN: Uuid = uuid!("ad000000-0000-0000-0000-000000000001");
pub const SEED_USER_MODERATOR: Uuid = uuid!("ad000000-0000-0000-0000-000000000002");

// ---- Seed moderation tickets ----

pub const SEED_MOD_TICKET_POST: Uuid = uuid!("ad000000-0000-0000-0000-ticket000001");
pub const SEED_MOD_TICKET_COMMENT: Uuid = uuid!("ad000000-0000-0000-0000-ticket000002");
pub const SEED_MOD_TICKET_USER: Uuid = uuid!("ad000000-0000-0000-0000-ticket000003");

// In MockState::default():
// - Register admin user: username "admin", email "admin@peerapp.de", roles_mask: 16
//   password "Admin1234", verified, wallet 10000.0, ip "192.168.1.1"
// - Register moderator user: username "moderator", email "mod@peerapp.de", roles_mask: 256
//   password "Mod1234", verified, wallet 5000.0, ip "192.168.1.2"
// - Create 3 moderation tickets:
//   1. Post ticket (SEED_MOD_TICKET_POST): targets SEED_POST_1, status "waiting_for_review",
//      reporter: SEED_USER_VERIFIED, reports_count: 1
//   2. Comment ticket (SEED_MOD_TICKET_COMMENT): targets seed comment, status "waiting_for_review",
//      reporter: SEED_USER_2, reports_count: 1
//   3. User ticket (SEED_MOD_TICKET_USER): targets SEED_USER_2, status "hidden",
//      moderated_by: SEED_USER_MODERATOR, reports_count: 2
// - Set content_visibility for SEED_USER_2 → "HIDDEN" (matching ticket 3)
```

### 4.7 Moderation Query Resolvers (`schema/query/moderation.rs`)

```rust
use async_graphql::{Context, Object, Result};

use crate::guards::require_moderator;
use crate::state::SharedState;
use crate::types::moderation::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct ModerationQuery;

#[Object]
impl ModerationQuery {
    /// Get moderation dashboard statistics.
    #[graphql(guard = "require_moderator()")]
    async fn moderation_stats(&self, ctx: &Context<'_>) -> Result<ModerationStatsResponse> {
        let state = ctx.data::<SharedState>()?.read().await;

        let mut awaiting = 0;
        let mut hidden = 0;
        let mut restored = 0;
        let mut illegal = 0;

        for ticket in &state.moderation_tickets {
            match ticket.status.as_str() {
                "waiting_for_review" => awaiting += 1,
                "hidden" => hidden += 1,
                "restored" => restored += 1,
                "illegal" => illegal += 1,
                _ => {}
            }
        }

        Ok(ModerationStatsResponse {
            status: "success".into(),
            response_code: Some("12101".into()),
            meta: DefaultResponse::success("12101", "Stats retrieved"),
            affected_rows: Some(ModerationStats {
                amount_awaiting_review: awaiting,
                amount_hidden: hidden,
                amount_restored: restored,
                amount_illegal: illegal,
            }),
        })
    }

    /// List moderation tickets with optional filters.
    #[graphql(guard = "require_moderator()")]
    async fn moderation_items(
        &self,
        ctx: &Context<'_>,
        status: Option<ModerationStatus>,
        content_type: Option<ModerationContentType>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<ModerationItemListResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(10).clamp(1, 20) as usize;

        let mut tickets: Vec<_> = state.moderation_tickets.iter().collect();

        // Filter by status
        if let Some(s) = &status {
            let s_str = s.as_str();
            tickets.retain(|t| t.status == s_str);
        }

        // Filter by content type
        if let Some(ct) = &content_type {
            let ct_str = match ct {
                ModerationContentType::Post => "post",
                ModerationContentType::Comment => "comment",
                ModerationContentType::User => "user",
            };
            tickets.retain(|t| t.target_type == ct_str);
        }

        // Paginate
        let total = tickets.len();
        let page: Vec<_> = tickets.into_iter().skip(offset).take(limit).collect();

        // Resolve each ticket to ModerationItem
        let items = page
            .iter()
            .map(|ticket| {
                state.resolve_moderation_item(ticket) // helper on MockState
            })
            .collect();

        Ok(ModerationItemListResponse {
            status: "success".into(),
            response_code: Some("12102".into()),
            meta: DefaultResponse::success("12102", "Items retrieved"),
            affected_rows: items,
        })
    }
}
```

### 4.8 Moderation Mutation Resolvers (`schema/mutation/moderation.rs`)

```rust
use async_graphql::{Context, Object, Result, ID};

use crate::guards::require_moderator;
use crate::state::SharedState;
use crate::types::moderation::ModerationStatus;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct ModerationMutation;

#[Object]
impl ModerationMutation {
    /// Perform a moderation action on a ticket.
    #[graphql(guard = "require_moderator()")]
    async fn perform_moderation(
        &self,
        ctx: &Context<'_>,
        moderation_ticket_id: ID,
        moderation_action: ModerationStatus,
    ) -> Result<DefaultResponse> {
        let current_user_id = get_current_user(ctx)?;
        let mut state = ctx.data::<SharedState>()?.write().await;

        // Validate action is not "waiting_for_review"
        if moderation_action == ModerationStatus::WaitingForReview {
            return Ok(DefaultResponse::error("32101", "Invalid moderation action"));
        }

        // Find ticket
        let ticket_id_str = moderation_ticket_id.to_string();
        let ticket = state
            .moderation_tickets
            .iter_mut()
            .find(|t| t.id.to_string() == ticket_id_str);

        let ticket = match ticket {
            Some(t) => t,
            None => return Ok(DefaultResponse::error("22103", "Ticket not found")),
        };

        // Check if already processed with same action
        if ticket.status == moderation_action.as_str() {
            return Ok(DefaultResponse::error("32103", "Ticket already processed"));
        }

        // Update ticket
        ticket.status = moderation_action.as_str().to_string();
        ticket.moderated_by = Some(current_user_id);

        // Update content visibility
        let target_id = ticket.target_content_id;
        match moderation_action {
            ModerationStatus::Hidden => {
                state.content_visibility.insert(target_id, "HIDDEN".into());
            }
            ModerationStatus::Illegal => {
                state.content_visibility.insert(target_id, "ILLEGAL".into());
            }
            ModerationStatus::Restored => {
                state.content_visibility.insert(target_id, "NORMAL".into());
            }
            _ => {}
        }

        // Reset report counts on the target (mock: set to 0)
        // This is tracked in the moderation ticket itself
        // Also clear any per-content report counters from earlier phases

        Ok(DefaultResponse::success("12103", "Moderation action performed"))
    }
}
```

### 4.9 Admin Query Resolvers (`schema/query/admin.rs`)

```rust
use async_graphql::{Context, Object, Result, ID};

use crate::guards::require_admin;
use crate::state::SharedState;
use crate::types::admin::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct AdminQuery;

#[Object]
impl AdminQuery {
    /// Extended user search with admin-specific filter fields.
    #[graphql(guard = "require_admin()")]
    async fn list_users_admin_v2(
        &self,
        ctx: &Context<'_>,
        content_filter_by: Option<String>,
        userid: Option<ID>,
        email: Option<String>,
        username: Option<String>,
        status: Option<i32>,
        verified: Option<i32>,
        ip: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<AdminUserListResponse> {
        let state = ctx.data::<SharedState>()?.read().await;

        // Validate: userid and username cannot both be present
        if userid.is_some() && username.is_some() {
            return Ok(AdminUserListResponse {
                meta: DefaultResponse::error("31012", "userid and username cannot be used together"),
                status: "error".into(),
                counter: 0,
                response_code: Some("31012".into()),
                affected_rows: None,
            });
        }

        // Validate UUID format if provided
        if let Some(ref uid) = userid {
            if uuid::Uuid::parse_str(&uid.to_string()).is_err() {
                return Ok(AdminUserListResponse {
                    meta: DefaultResponse::error("30201", "Invalid UUID format"),
                    status: "error".into(),
                    counter: 0,
                    response_code: Some("30201".into()),
                    affected_rows: None,
                });
            }
        }

        // Validate IP format if provided
        if let Some(ref ip_str) = ip {
            if ip_str.parse::<std::net::IpAddr>().is_err() {
                return Ok(AdminUserListResponse {
                    meta: DefaultResponse::error("30257", "Invalid IP address"),
                    status: "error".into(),
                    counter: 0,
                    response_code: Some("30257".into()),
                    affected_rows: None,
                });
            }
        }

        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(10).clamp(1, 20) as usize;

        // Filter users based on search criteria
        let results = state.search_users_admin(
            userid.as_ref(),
            email.as_deref(),
            username.as_deref(),
            status,
            verified,
            ip.as_deref(),
            offset,
            limit,
        );

        if results.is_empty() {
            return Ok(AdminUserListResponse {
                meta: DefaultResponse::error("31007", "No users found"),
                status: "error".into(),
                counter: 0,
                response_code: Some("31007".into()),
                affected_rows: None,
            });
        }

        Ok(AdminUserListResponse {
            meta: DefaultResponse::success("11009", "User data prepared"),
            status: "success".into(),
            counter: results.len() as i32,
            response_code: Some("11009".into()),
            affected_rows: Some(results),
        })
    }

    /// List all follow relationships across the platform.
    #[graphql(guard = "require_admin()")]
    async fn allfriends(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<AllUserFriends> {
        let state = ctx.data::<SharedState>()?.read().await;
        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(20).clamp(1, 100) as usize;

        let all_follows: Vec<AllUserInfo> = state
            .follows
            .iter()
            .map(|(follower_id, followed_id)| {
                let follower_name = state.get_username(*follower_id);
                let followed_name = state.get_username(*followed_id);
                AllUserInfo {
                    followerid: Some(follower_id.to_string().into()),
                    followername: Some(follower_name),
                    followedid: Some(followed_id.to_string().into()),
                    followedname: Some(followed_name),
                }
            })
            .collect();

        let total = all_follows.len();
        let page: Vec<_> = all_follows.into_iter().skip(offset).take(limit).collect();

        Ok(AllUserFriends {
            meta: DefaultResponse::success("11101", "Friends retrieved"),
            status: "success".into(),
            counter: total as i32,
            response_code: Some("11101".into()),
            affected_rows: Some(page),
        })
    }

    /// Admin view of all comments on a post (with subcomments).
    #[graphql(guard = "require_admin()")]
    async fn postcomments(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<PostCommentsResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(10).clamp(1, 20) as usize;

        // Get top-level comments for the post, resolve subcomments,
        // include visibilityStatus and isHiddenForUsers from content_visibility map
        let comments = state.get_admin_post_comments(&postid.to_string(), offset, limit);

        Ok(PostCommentsResponse {
            meta: DefaultResponse::success("11101", "Comments retrieved"),
            status: "success".into(),
            counter: comments.len() as i32,
            response_code: Some("11101".into()),
            affected_rows: Some(comments),
        })
    }

    /// Generate a leaderboard CSV (mock: returns a fake download link).
    #[graphql(guard = "require_admin()")]
    async fn generate_leaderboard(
        &self,
        ctx: &Context<'_>,
        leaderboard_params: LeaderboardParamsInput,
    ) -> Result<LeaderboardResponse> {
        // Validate date format
        let start_ok = chrono::NaiveDate::parse_from_str(&leaderboard_params.start_date, "%Y-%m-%d");
        let end_ok = chrono::NaiveDate::parse_from_str(&leaderboard_params.end_date, "%Y-%m-%d");

        match (start_ok, end_ok) {
            (Ok(start), Ok(end)) => {
                if end < start {
                    return Ok(LeaderboardResponse {
                        meta: DefaultResponse::error("33002", "Invalid date range"),
                        leaderboard_result_link: None,
                    });
                }

                let n = leaderboard_params.leaderboard_users_count;
                let link = format!(
                    "runtime-data/media/other/power_power_contest_leaderboards_data/leaderboard_{}_{}_top{}.csv",
                    leaderboard_params.start_date.replace('-', ""),
                    leaderboard_params.end_date.replace('-', ""),
                    n
                );

                Ok(LeaderboardResponse {
                    meta: DefaultResponse::success("12301", "Leaderboard generated"),
                    leaderboard_result_link: Some(link),
                })
            }
            _ => Ok(LeaderboardResponse {
                meta: DefaultResponse::error("30301", "Invalid parameters"),
                leaderboard_result_link: None,
            }),
        }
    }
}
```

### 4.10 Admin Gem Query Resolvers (`schema/query/admin_gems.rs`)

```rust
use async_graphql::{Context, Object, Result};
use rust_decimal::Decimal;

use crate::guards::require_admin;
use crate::state::SharedState;
use crate::types::admin_gems::*;
use crate::types::registration::DefaultResponse;
use crate::types::wallet::DayFilterType;

#[derive(Default)]
pub struct AdminGemQuery;

#[Object]
impl AdminGemQuery {
    /// Get uncollected gems statistics by time period.
    #[graphql(guard = "require_admin()")]
    async fn gemster(&self, ctx: &Context<'_>) -> Result<GemsterResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let data = state.aggregate_gems_by_period();
        Ok(GemsterResponse {
            meta: DefaultResponse::success("11207", "Gems data loaded"),
            affected_rows: Some(data),
        })
    }

    /// Daily gem status overview.
    #[graphql(guard = "require_admin()")]
    async fn dailygemstatus(&self, ctx: &Context<'_>) -> Result<GemsterResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let data = state.aggregate_gems_by_period();
        Ok(GemsterResponse {
            meta: DefaultResponse::success("11207", "Gems data loaded"),
            affected_rows: Some(data),
        })
    }

    /// Per-user gems breakdown for a specific day.
    #[graphql(guard = "require_admin()")]
    async fn dailygemsresults(
        &self,
        ctx: &Context<'_>,
        day: DayFilterType,
    ) -> Result<DailyGemsResultsResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let (user_data, total) = state.get_gems_for_day(&day);

        if user_data.is_empty() {
            return Ok(DailyGemsResultsResponse {
                meta: DefaultResponse::error("21206", "No gems for given day"),
                affected_rows: None,
            });
        }

        Ok(DailyGemsResultsResponse {
            meta: DefaultResponse::success("11207", "Gems data loaded"),
            affected_rows: Some(DailyGemsResultsData {
                data: Some(user_data),
                total_gems: Some(total),
            }),
        })
    }

    /// Get mint account balance and details.
    #[graphql(guard = "require_admin()")]
    async fn get_mint_account(&self, ctx: &Context<'_>) -> Result<MintAccountResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let current_balance = state
            .wallets
            .get(&crate::seed::SYSTEM_MINT_ACCOUNT)
            .copied()
            .unwrap_or(Decimal::ZERO);

        Ok(MintAccountResponse {
            meta: DefaultResponse::success("0", "Account retrieved"),
            mint_account: Some(MintAccount {
                accountid: crate::seed::SYSTEM_MINT_ACCOUNT.to_string().into(),
                initial_balance: crate::seed::MINT_INITIAL_BALANCE,
                current_balance,
                updatedat: crate::state::today_date_string(),
            }),
        })
    }
}
```

### 4.11 Admin Gem Mutation Resolvers (`schema/mutation/admin_gems.rs`)

```rust
use async_graphql::{Context, Object, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::guards::require_admin;
use crate::state::SharedState;
use crate::types::admin_gems::*;
use crate::types::registration::DefaultResponse;
use crate::types::wallet::DayFilterType;

/// Daily token mint amount (total distributed per day).
const DAILY_NUMBER_TOKEN: f64 = 5000.0;

#[derive(Default)]
pub struct AdminGemMutation;

#[Object]
impl AdminGemMutation {
    /// Convert all pending post interactions into gems.
    #[graphql(guard = "require_admin()")]
    async fn globalwins(&self, ctx: &Context<'_>) -> Result<DefaultResponse> {
        let mut state = ctx.data::<SharedState>()?.write().await;
        let converted = state.convert_interactions_to_gems();

        if converted == 0 {
            return Ok(DefaultResponse::error("21205", "No interactions to convert"));
        }

        Ok(DefaultResponse::success("11206", "Interactions converted to gems"))
    }

    /// Mint and distribute tokens from gems for a specific date.
    #[graphql(guard = "require_admin()")]
    async fn distribute_tokens_for_gems(
        &self,
        ctx: &Context<'_>,
        date: String,
    ) -> Result<GemstersResponse> {
        // Validate date format
        let parsed = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d");
        if parsed.is_err() {
            return Ok(GemstersResponse {
                meta: DefaultResponse::error("30105", "Invalid date format"),
                counter: 0,
                affected_rows: None,
            });
        }

        let today = chrono::Local::now().date_naive();
        if parsed.unwrap() > today {
            return Ok(GemstersResponse {
                meta: DefaultResponse::error("30105", "Cannot mint for future dates"),
                counter: 0,
                affected_rows: None,
            });
        }

        let mut state = ctx.data::<SharedState>()?.write().await;

        // Check if already minted for this date
        if state.minted_dates.contains(&date) {
            return Ok(GemstersResponse {
                meta: DefaultResponse::error("31204", "Already minted for date"),
                counter: 0,
                affected_rows: None,
            });
        }

        // Distribute tokens
        let result = state.distribute_gems_to_tokens(&date, DAILY_NUMBER_TOKEN);

        match result {
            Some((data, counter)) => {
                state.minted_dates.insert(date);
                Ok(GemstersResponse {
                    meta: DefaultResponse::success("11208", "Tokens distributed"),
                    counter,
                    affected_rows: Some(data),
                })
            }
            None => Ok(GemstersResponse {
                meta: DefaultResponse::error("21206", "No gems for date"),
                counter: 0,
                affected_rows: None,
            }),
        }
    }

    /// Mint and distribute tokens from gems for a relative day.
    #[graphql(guard = "require_admin()")]
    async fn gemsters(
        &self,
        ctx: &Context<'_>,
        day: DayFilterType,
    ) -> Result<GemstersResponse> {
        let date = crate::state::day_filter_to_date(&day);
        // Delegate to same logic as distribute_tokens_for_gems
        self.distribute_tokens_for_gems(ctx, date).await
    }

    /// One-time alpha token distribution.
    #[graphql(guard = "require_admin()")]
    async fn alpha_mint(&self, ctx: &Context<'_>) -> Result<DefaultResponse> {
        let mut state = ctx.data::<SharedState>()?.write().await;

        if state.alpha_minted {
            return Ok(DefaultResponse::error("31204", "Alpha mint already completed"));
        }

        // Credit each non-system user with 100.0 tokens from mint account
        let user_ids: Vec<_> = state
            .users
            .keys()
            .filter(|uid| !state.is_system_account(**uid))
            .copied()
            .collect();

        let amount = Decimal::from(100);
        for uid in &user_ids {
            *state.wallets.entry(*uid).or_insert(Decimal::ZERO) += amount;
            *state
                .wallets
                .entry(crate::seed::SYSTEM_MINT_ACCOUNT)
                .or_insert(Decimal::ZERO) -= amount;
        }

        state.alpha_minted = true;
        Ok(DefaultResponse::success("200", "Alpha mint complete"))
    }
}
```

### 4.12 Helper Functions on `MockState`

```rust
impl MockState {
    /// Resolve a moderation ticket record into its GraphQL representation.
    pub fn resolve_moderation_item(&self, ticket: &ModerationTicketRecord) -> ModerationItem {
        let target_content = match ticket.target_type.as_str() {
            "post" => TargetContent {
                post: self.find_post_graphql(&ticket.target_content_id),
                comment: None,
                user: None,
            },
            "comment" => TargetContent {
                post: None,
                comment: self.find_comment_graphql(&ticket.target_content_id),
                user: None,
            },
            "user" => TargetContent {
                post: None,
                comment: None,
                user: self.find_user_basic_info(&ticket.target_content_id),
            },
            _ => TargetContent { post: None, comment: None, user: None },
        };

        let reporters: Vec<BasicUserInfo> = ticket
            .reporter_ids
            .iter()
            .filter_map(|uid| self.find_user_basic_info(uid))
            .collect();

        let moderated_by = ticket
            .moderated_by
            .and_then(|uid| self.find_user_basic_info(&uid));

        ModerationItem {
            moderation_ticket_id: ticket.id.to_string().into(),
            target_content_id: ticket.target_content_id.to_string().into(),
            targettype: ticket.target_type.clone(),
            reportscount: ticket.reports_count,
            status: ticket.status.clone(),
            createdat: ticket.created_at.clone(),
            targetcontent: target_content,
            reporters,
            moderated_by,
        }
    }

    /// Find or create a moderation ticket for reported content.
    /// Returns true if a new ticket was created, false if an existing one was updated.
    pub fn report_content(
        &mut self,
        target_id: Uuid,
        target_type: &str,
        reporter_id: Uuid,
    ) -> Result<bool, &'static str> {
        // Check if restored content (cannot be re-reported)
        if let Some(ticket) = self.moderation_tickets.iter().find(|t| {
            t.target_content_id == target_id && t.status == "restored"
        }) {
            return Err("32104"); // Already restored
        }

        // Check if reporter already reported this content
        if let Some(ticket) = self.moderation_tickets.iter().find(|t| {
            t.target_content_id == target_id && t.reporter_ids.contains(&reporter_id)
        }) {
            return Err("32104"); // Already reported by this user
        }

        // Find existing ticket for this content
        if let Some(ticket) = self.moderation_tickets.iter_mut().find(|t| {
            t.target_content_id == target_id && t.status == "waiting_for_review"
        }) {
            ticket.reporter_ids.push(reporter_id);
            ticket.reports_count += 1;
            return Ok(false);
        }

        // Create new ticket
        let ticket = ModerationTicketRecord {
            id: Uuid::new_v4(),
            target_content_id: target_id,
            target_type: target_type.to_string(),
            reporter_ids: vec![reporter_id],
            reports_count: 1,
            status: "waiting_for_review".to_string(),
            moderated_by: None,
            created_at: today_date_string(),
        };
        self.moderation_tickets.push(ticket);
        Ok(true)
    }

    /// Get content visibility status (defaults to NORMAL).
    pub fn get_visibility(&self, content_id: &Uuid) -> &str {
        self.content_visibility
            .get(content_id)
            .map(|s| s.as_str())
            .unwrap_or("NORMAL")
    }

    /// Check if content is visible (not ILLEGAL, and not HIDDEN for strict filter).
    pub fn is_content_visible(&self, content_id: &Uuid) -> bool {
        self.get_visibility(content_id) != "ILLEGAL"
    }

    /// Search users with admin-specific filters.
    pub fn search_users_admin(
        &self,
        userid: Option<&ID>,
        email: Option<&str>,
        username: Option<&str>,
        status: Option<i32>,
        verified: Option<i32>,
        ip: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Vec<AdminUser> {
        // Filter users based on provided criteria, return extended AdminUser objects
        // with email, verified, roles_mask, ip, liquidity fields
        // ... implementation ...
        vec![]
    }

    /// Aggregate gem records into DailyGemStatusData (d0–d7, w0, m0, y0).
    pub fn aggregate_gems_by_period(&self) -> DailyGemStatusData {
        // Group gems by date, calculate totals for each period
        // ... implementation ...
        DailyGemStatusData { /* ... */ }
    }

    /// Get per-user gem totals for a specific day filter.
    pub fn get_gems_for_day(&self, day: &DayFilterType) -> (Vec<DailyGemsResultsUserData>, Decimal) {
        // ... implementation ...
        (vec![], Decimal::ZERO)
    }

    /// Convert pending interactions to gem records.
    /// Returns count of interactions converted.
    pub fn convert_interactions_to_gems(&mut self) -> usize {
        // ... implementation ...
        0
    }

    /// Distribute tokens from gems for a specific date.
    /// Returns distribution data and user count if gems exist, None otherwise.
    pub fn distribute_gems_to_tokens(
        &mut self,
        date: &str,
        daily_token_amount: f64,
    ) -> Option<(GemstersData, i32)> {
        // ... implementation ...
        None
    }

    /// Convert a DayFilterType to a date string.
    pub fn day_filter_to_date(day: &DayFilterType) -> String {
        // ... implementation ...
        String::new()
    }
}

/// Convert DayFilterType to a YYYY-MM-DD date string.
pub fn day_filter_to_date(day: &DayFilterType) -> String {
    let today = chrono::Local::now().date_naive();
    let target = match day {
        DayFilterType::D0 => today,
        DayFilterType::D1 => today - chrono::Duration::days(1),
        DayFilterType::D2 => today - chrono::Duration::days(2),
        DayFilterType::D3 => today - chrono::Duration::days(3),
        DayFilterType::D4 => today - chrono::Duration::days(4),
        DayFilterType::D5 => today - chrono::Duration::days(5),
        DayFilterType::D6 => today - chrono::Duration::days(6),
        DayFilterType::D7 => today - chrono::Duration::days(7),
        DayFilterType::W0 => today, // Use today; aggregate function handles range
        DayFilterType::M0 => today,
        DayFilterType::Y0 => today,
    };
    target.format("%Y-%m-%d").to_string()
}
```

### 4.13 Schema Assembly Updates

```rust
// schema/mod.rs — updated merged objects

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    HealthQuery,
    // Phase 2:
    UserQuery,
    // Phase 3:
    PostQuery,
    // Phase 4:
    CommentQuery,
    ChatQuery,
    // Phase 5:
    WalletQuery,
    TokenomicsQuery,
    AdQuery,
    ShopQuery,
    // Phase 6:
    ModerationQuery,
    AdminQuery,
    AdminGemQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    RegistrationMutation,   // Phase 0
    AuthMutation,           // Phase 1
    ProfileMutation,        // Phase 2
    PostMutation,           // Phase 3
    CommentMutation,        // Phase 4
    ChatMutation,           // Phase 4
    WalletMutation,         // Phase 5
    AdMutation,             // Phase 5
    ShopMutation,           // Phase 5
    ModerationMutation,     // Phase 6
    AdminGemMutation,       // Phase 6
);
```

---

## 5. Testing Strategy

### Moderation Tests (≥14)

| # | Test | Assert |
|---|------|--------|
| 1 | Get moderation stats as moderator | `12101`, counts match seed data (2 waiting, 0 restored, 1 hidden, 0 illegal) |
| 2 | Get moderation stats as regular user → `62101` | Not authorized |
| 3 | Get moderation stats without auth → `60501` | Not authenticated |
| 4 | List all moderation items as moderator | `12102`, returns 3 seed tickets |
| 5 | List items filtered by status `waiting_for_review` | Returns 2 tickets |
| 6 | List items filtered by contentType `post` | Returns 1 ticket |
| 7 | List items filtered by contentType `user` | Returns 1 ticket (hidden) with `moderatedBy` populated |
| 8 | List items with pagination (offset 1, limit 1) | Returns 1 ticket |
| 9 | List items as regular user → `62101` | Not authorized |
| 10 | Perform moderation: hide post ticket | `12103`, ticket status → `hidden`, `content_visibility[post_id] = "HIDDEN"` |
| 11 | Perform moderation: restore comment ticket | `12103`, ticket status → `restored`, `content_visibility[comment_id] = "NORMAL"` |
| 12 | Perform moderation: mark illegal | `12103`, ticket status → `illegal`, `content_visibility = "ILLEGAL"` |
| 13 | Perform moderation on non-existent ticket → `22103` | Not found |
| 14 | Perform moderation: duplicate action on same ticket → `32103` | Already processed |

### Content Visibility Tests (≥6)

| # | Test | Assert |
|---|------|--------|
| 15 | Hidden post excluded from `listPosts` (or shows `isHiddenForUsers: true`) | Post visibility reflected in query |
| 16 | Illegal post completely filtered from `listPosts` | Post does not appear at all |
| 17 | Restored post visible again in `listPosts` | Post reappears with `visibilityStatus: "NORMAL"` |
| 18 | Hidden comment shows `isHiddenForUsers: true` in `listComments` | Comment visibility reflected |
| 19 | Hidden user shows `visibilityStatus: "HIDDEN"` in profile queries | User visibility reflected |
| 20 | Report user → moderation ticket created | `report_content` creates ticket with correct `target_type` |

### Admin Tests (≥12)

| # | Test | Assert |
|---|------|--------|
| 21 | Admin search users by email | `11009`, returns user with email visible |
| 22 | Admin search users by IP | `11009`, returns matching user |
| 23 | Admin search users by verified status | `11009`, filtered correctly |
| 24 | Admin search with both userid and username → `31012` | Error |
| 25 | Admin search with invalid UUID → `30201` | Error |
| 26 | Admin search with invalid IP → `30257` | Error |
| 27 | Admin search — extended fields present (email, roles_mask, liquidity) | Fields populated |
| 28 | Admin search as regular user → `62101` | Not authorized |
| 29 | `allfriends` returns follow relationships | Counter matches total follows, entries have follower/followed info |
| 30 | `postcomments` returns comments with subcomments and visibility | Comments include `visibilityStatus`, `isHiddenForUsers`, `subcomments` |
| 31 | `generateLeaderboard` returns CSV link | `12301`, link format correct |
| 32 | `generateLeaderboard` with invalid date range → `33002` | Error |

### Admin Gem/Mint Tests (≥10)

| # | Test | Assert |
|---|------|--------|
| 33 | `gemster` returns gem statistics | `11207`, `DailyGemStatusData` populated from seed gems |
| 34 | `dailygemstatus` returns same shape | `11207` |
| 35 | `dailygemsresults(D0)` returns per-user gems | `11207`, user data with gem totals |
| 36 | `dailygemsresults` for day with no gems → `21206` | Empty |
| 37 | `getMintAccount` returns balance | `0`, `initialBalance: 5_000_000.0`, `currentBalance` matches |
| 38 | `globalwins` converts interactions to gems | `11206`, gem records created |
| 39 | `globalwins` with no pending interactions → `21205` | Nothing to convert |
| 40 | `distributeTokensForGems` mints tokens | `11208`, user wallets credited, mint account debited |
| 41 | `distributeTokensForGems` for already-minted date → `31204` | Duplicate |
| 42 | `alphaMint` credits users | `200`, all non-system users get 100.0 tokens |

### Cross-Cutting Tests (≥4)

| # | Test | Assert |
|---|------|--------|
| 43 | `alphaMint` twice → `31204` | Already run |
| 44 | Report post → ticket created → moderator can see it in `moderationItems` | Full flow |
| 45 | Reset clears all moderation state | POST /reset → tickets cleared, visibility reset, alpha_minted reset |
| 46 | All Phase 0–5 tests still pass | Regression check |

---

## 6. Definition of Done

### Phase 6 gate

- [x] `RoleGuard` correctly blocks unauthenticated (`60501`) and unauthorized (`62101`) access
- [x] `moderationStats` returns accurate ticket counts by status
- [x] `moderationItems` returns filtered, paginated tickets with resolved `TargetContent`, `reporters`, and `moderatedBy`
- [x] `performModeration` transitions ticket status and updates `content_visibility` for hide/restore/illegal actions
- [x] Content visibility states (`NORMAL`, `HIDDEN`, `ILLEGAL`) are respected in Phase 3/4 list queries
- [x] Report actions (Phase 2/3/4) create or update moderation tickets with proper deduplication
- [x] `listUsersAdminV2` returns extended user data with email, IP, verified, roles_mask, liquidity
- [x] `listUsersAdminV2` validates all inputs (UUID format, IP format, userid+username exclusivity)
- [x] `allfriends` returns all follow relationships with pagination
- [x] `postcomments` returns comments with subcomments and visibility fields
- [x] `generateLeaderboard` validates date range and returns mock CSV link
- [x] `gemster` and `dailygemstatus` return gem aggregations by time period (d0–d7, w0, m0, y0)
- [x] `dailygemsresults` returns per-user gem breakdown for a specific day
- [x] `getMintAccount` returns mint account balance
- [x] `globalwins` converts pending interactions to gem records
- [x] `distributeTokensForGems` / `gemsters` distribute tokens proportional to gems, prevent duplicate minting
- [x] `alphaMint` credits non-system users once, rejects duplicate runs
- [x] `POST /reset` clears all moderation and admin state back to seed data
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes
- [x] ≥46 integration tests pass (14 moderation + 6 visibility + 12 admin + 10 gem/mint + 4 cross-cutting) — **47 actual**
- [x] All Phase 0–5 tests still pass (regression)

### Response code summary

| Code | Meaning | Operation |
|------|---------|-----------|
| `0` | Mint account retrieved | `getMintAccount` |
| `200` | Alpha mint complete | `alphaMint` |
| `11009` | Users found | `listUsersAdminV2` |
| `11101` | Data retrieved | `allfriends`, `postcomments` |
| `11206` | Interactions converted to gems | `globalwins` |
| `11207` | Gems data loaded | `gemster`, `dailygemstatus`, `dailygemsresults` |
| `11208` | Tokens distributed | `distributeTokensForGems`, `gemsters` |
| `12101` | Stats retrieved | `moderationStats` |
| `12102` | Items retrieved | `moderationItems` |
| `12103` | Moderation action performed | `performModeration` |
| `12301` | Leaderboard generated | `generateLeaderboard` |
| `21205` | No interactions to convert | `globalwins` |
| `21206` | No gems for date/day | `dailygemsresults`, `distributeTokensForGems` |
| `22103` | Ticket not found | `performModeration` |
| `22201` | No users found for criteria | `generateLeaderboard` |
| `30105` | Invalid date format | `distributeTokensForGems` |
| `30201` | Invalid UUID format | `listUsersAdminV2` |
| `30202` | Invalid username | `listUsersAdminV2` |
| `30203` | Invalid offset | `listUsersAdminV2` |
| `30204` | Invalid limit | `listUsersAdminV2` |
| `30257` | Invalid IP address | `listUsersAdminV2` |
| `30301` | Invalid parameters | `generateLeaderboard` |
| `31007` | No users found | `listUsersAdminV2` |
| `31012` | userid and username both provided | `listUsersAdminV2` |
| `31204` | Already minted for date / alpha already run | `distributeTokensForGems`, `alphaMint` |
| `32101` | Invalid moderation action | `performModeration` |
| `32103` | Ticket already processed | `performModeration` |
| `32104` | Already reported / restored content | Report actions |
| `33002` | Invalid date range | `generateLeaderboard` |
| `60501` | Not authenticated | All operations |
| `62101` | Not authorized (wrong role) | All admin/moderation operations |
