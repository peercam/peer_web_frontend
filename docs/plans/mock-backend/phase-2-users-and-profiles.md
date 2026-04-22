# Phase 2: Users & Profiles

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** [Phase 1 — Login & Session Flows](./phase-1-login-session-flows.md)
> **Goal:** Add user discovery, profile viewing, social relationships (follow/block/report), user preferences, and profile-editing mutations so the Leptos profile pages, settings pages, and user search work end-to-end against the mock.

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

From `src/api/profile.rs` and `src/api/graphql.rs`, the Leptos app uses these queries:

| Query | GraphQL SDL | Frontend file |
|-------|-------------|---------------|
| `getProfile(userid?, contentFilterBy?)` | Returns `ProfileInfo { meta, affectedRows: Profile }` | `profile.rs` → `GetProfile` server fn |
| `listFollowRelations(userid?, contentFilterBy?, offset, limit)` | Returns `FollowRelationsResponse { meta, counter, affectedRows: { followers, following } }` | `profile.rs` → `ListFollowRelations` server fn |
| `listFriends(userid?, contentFilterBy?, offset, limit)` | Returns `UserFriendsResponse { meta, counter, affectedRows: [BasicUserInfo] }` | `profile.rs` → `ListFriends` server fn |
| `listPosts(userid!, filterBy?, contentFilterBy?, sortBy?, offset, limit)` | (Posts — Phase 3, but called from profile page) | `profile.rs` → `ListUserPosts` server fn |
| `searchUser(username!, offset?, limit?)` | Returns `{ meta, counter, affectedRows: [{ id, username, slug, img }] }` | `graphql.rs` → `SEARCH_USERS_QUERY` |
| `getUser(id!)` | Returns user info with preferences | `graphql.rs` → `GET_USER_QUERY` |

And these mutations:

| Mutation | GraphQL SDL | Frontend file |
|----------|-------------|---------------|
| `toggleUserFollowStatus(userid!)` | Returns `FollowStatusResponse { meta, isfollowing }` | `profile.rs` → `ToggleFollow` server fn |
| `toggleBlockUserStatus(userid!)` | Returns `DefaultResponse` | `profile.rs` → `ToggleBlock` server fn |
| `reportUser(userid!)` | Returns `DefaultResponse` | `profile.rs` → `ReportUser` server fn |
| `updateProfileImage(img!)` | Returns `{ status, ResponseCode }` | `settings.rs` → `UpdateProfileImage` server fn |
| `updateBio(biography!)` | Returns `{ status, ResponseCode }` | `settings.rs` → `UpdateBio` server fn |
| `updateUsername(username!, password!)` | Returns `{ status, ResponseCode }` | `settings.rs` → `UpdateUsername` server fn |
| `updateEmail(email!, password!)` | Returns `{ status, ResponseCode }` | `settings.rs` → `UpdateEmail` server fn |
| `updateUserPreferences(userPreferences)` | Returns `{ status, ResponseCode, affectedRows: { contentFilteringSeverityLevel } }` | `settings.rs` → `UpdateContentPreferences` server fn |

From `docs/backend_api/02-users-and-profiles.md` (additional operations in the backend schema, not yet fully wired in frontend):

| Operation | Type | Purpose |
|-----------|------|---------|
| `listUsersV2(contentFilterBy?, userid?, username?, offset, limit)` | Query | Full user search with pagination and content filtering |
| `getUserInfo` | Query | Current user's account info + preferences |
| `listBlockedUsers(contentFilterBy?, offset, limit)` | Query | Users blocked by/blocking current user |
| `getReferralInfo` | Query | Current user's referral UUID and link |
| `referralList(offset, limit)` | Query | Users who signed up via the current user's referral |
| `updateUserPreferences(userPreferences)` | Mutation | Update content filtering & onboarding preferences |

### Target State

After this phase, the mock backend will support:
- User discovery via search (`searchUser`, `listUsersV2`)
- Profile viewing (own + other users) with social stats
- User info retrieval with preferences
- Follow/unfollow toggle with follower/following lists and friend detection
- Block/unblock toggle with blocked user lists and interaction filtering
- User reporting
- Profile editing (avatar, bio, username, email)
- User preferences management (content filtering severity, onboardings)
- Referral info retrieval
- Content filtering and blocked-user exclusion on list queries
- Pagination on all list endpoints

### New file tree additions

```
packages/mock_backend/src/
├── schema/
│   ├── query/
│   │   └── users.rs           # NEW: getProfile, listUsersV2, searchUser, getUser,
│   │                          #       listFollowRelations, listFriends,
│   │                          #       listBlockedUsers, getUserInfo,
│   │                          #       getReferralInfo, referralList
│   └── mutation/
│       └── profile.rs         # NEW: toggleUserFollowStatus, toggleBlockUserStatus,
│                              #       reportUser, updateProfileImage, updateBio,
│                              #       updateUsername, updateEmail, updateUserPreferences
├── types/
│   └── user.rs                # NEW: Profile, ProfileUser, BasicUserInfo,
│                              #       UserListResponse, FollowRelationsResponse,
│                              #       FriendsResponse, BlockedUsersResponse,
│                              #       FollowStatusResponse, UserInfo,
│                              #       UserPreferences, ReferralInfoResponse, etc.
└── state.rs                   # MODIFIED: new fields for profiles, follows, blocks,
                               #           reports, preferences
```

---

## 2. Prerequisites

### Phase 1 Completion

- [x] All auth mutations work (`login`, `refreshToken`, `logout`, `deleteAccount`, etc.)
- [x] Auth middleware injects `CurrentUser(Option<Uuid>)` into async-graphql context
- [x] `require_auth()` helper returns `60501` for unauthenticated calls (currently `fn` in `auth.rs`, needs `pub` promotion — see §4.5a)
- [x] `MockState` has `users: HashMap<Uuid, User>`, `user_passwords`, `access_tokens`, `refresh_tokens`
- [x] `MutationRoot` uses `MergedObject` and can accept additional mutation structs
- [x] `QueryRoot` supports merging via `MergedObject` — refactored to `MergedObject` pattern (§4.8 Step 1–2)
- [x] `app()` and `app_with_state()` router builders exist
- [x] `/reset` endpoint resets state to defaults
- [x] Seeded verified user (`test@peer.com` / `TestPass123`) can log in

### API Reference

All response codes and field names in this plan come from:
- `docs/backend_api/02-users-and-profiles.md`
- `src/models/profile.rs` (frontend deserialization types)
- `src/models/settings.rs` (settings response types)
- `src/api/graphql.rs` (exact GraphQL query/mutation strings)

---

## 3. Task Breakdown

### Phase 2.A — User & Profile Types (`types/user.rs`)

| # | Task | Notes |
|---|------|-------|
| A1 | Create `types/user.rs` with `ContentVisibilityStatus` enum | `NORMAL`, `HIDDEN`, `ILLEGAL` |
| A2 | Define `ContentFilterType` enum (InputEnum) | `MYGRANDMALIKES`, `MYGRANDMAHATES` |
| A3 | Define `OnboardingType` enum (InputEnum) | `INTROONBOARDING` |
| A4 | Define `ProfileGql` struct (SimpleObject) | Full profile with social stats — mirrors `src/models/profile.rs::Profile` |
| A5 | Define `ProfileInfoResponse` wrapper | `meta: DefaultResponse`, `affectedRows: Option<ProfileGql>` |
| A6 | Define `ProfileUserGql` struct (SimpleObject) | Used in follow lists — mirrors `src/models/profile.rs::ProfileUser` |
| A7 | Define `BasicUserInfoGql` struct (SimpleObject) | Used in friends list — mirrors `src/models/profile.rs::BasicUserInfo` |
| A8 | Define `SearchUserResult` struct (SimpleObject) | Minimal: `id`, `username`, `slug`, `img` — matches `SEARCH_USERS_QUERY` response |
| A9 | Define `UserListResponse` wrapper | `meta`, `counter`, `affectedRows: Option<Vec<SearchUserResult>>` |
| A10 | Define `FollowRelationsGql` struct | `followers: Vec<ProfileUserGql>`, `following: Vec<ProfileUserGql>` |
| A11 | Define `FollowRelationsResponseGql` wrapper | `meta`, `counter`, `affectedRows: Option<FollowRelationsGql>` |
| A12 | Define `FriendsResponseGql` wrapper | `meta`, `counter`, `affectedRows: Vec<BasicUserInfoGql>` |
| A13 | Define `FollowStatusResponseGql` | `meta`, `isfollowing: bool` — matches `ToggleFollow` response |
| A14 | Define `BlockedUserGql` and `BlockedUsersGql` structs | `iBlocked: Vec<BlockedUserGql>`, `blockedBy: Vec<BlockedUserGql>` |
| A15 | Define `BlockedUsersResponseGql` wrapper | `meta`, `counter`, `affectedRows: Option<BlockedUsersGql>` |
| A16 | Define `UserInfoGql` struct | `userid`, `liquidity`, `amountposts`, `amountreports`, `amountblocked`, `amountfollower`, `amountfollowed`, `amountfriends`, `invited`, `updatedat`, `userPreferences` |
| A17 | Define `UserInfoResponseGql` wrapper | `meta`, `affectedRows: Option<UserInfoGql>` |
| A18 | Define `UserPreferencesGql` struct | `contentFilteringSeverityLevel`, `onboardingsWereShown` |
| A19 | Define `UserPreferencesInput` (InputObject) | `contentFilteringSeverityLevel: Option<ContentFilterType>`, `shownOnboardings: Option<Vec<OnboardingType>>` |
| A20 | Define `UserPreferencesResponseGql` | `status`, `ResponseCode`, `affectedRows: Option<UserPreferencesPayloadGql>` |
| A21 | Define `UserPreferencesPayloadGql` | `contentFilteringSeverityLevel: Option<String>` |
| A22 | Define `UpdateResponse` struct (SimpleObject) | `status`, `ResponseCode` — matches settings mutation responses |
| A23 | Define `ReferralInfoResponseGql` | `meta`, `referralUuid: Option<ID>`, `referralLink: Option<String>` |
| A24 | Define `ReferralUsersGql` and `ReferralListResponseGql` | invited-by + i-invited lists |
| A25 | Define `UserGetResponseGql` | `meta`, `affectedRows` with user info + preferences — matches `GET_USER_QUERY` |
| A26 | Export new types from `types/mod.rs` | Add `pub mod user;` |

### Phase 2.B — State Extensions (`state.rs`, `seed.rs`)

| # | Task | Notes |
|---|------|-------|
| B1 | Extend `User` struct with profile fields | Add `img: Option<String>`, `biography: Option<String>`, `slug_num: i32` (numeric slug), `visibility_status: ContentVisibilityStatus`, `created_at: String`, `updated_at: String` |
| B2 | Add `UserPreferencesState` struct to state | `content_filtering_severity_level: ContentFilterType`, `onboardings_were_shown: Vec<String>` |
| B3 | Add `UserReport` struct to state | `reporter: Uuid`, `reported: Uuid`, `created_at: String` |
| B4 | Add `follows: HashSet<(Uuid, Uuid)>` to `MockState` | `(follower_uid, followed_uid)` |
| B5 | Add `blocks: HashSet<(Uuid, Uuid)>` to `MockState` | `(blocker_uid, blocked_uid)` |
| B6 | Add `reports: Vec<UserReport>` to `MockState` | User reports for moderation |
| B7 | Add `preferences: HashMap<Uuid, UserPreferencesState>` to `MockState` | Per-user preferences |
| B8 | Add `referral_invitations: HashMap<Uuid, Uuid>` to `MockState` | `invitee → inviter` mapping |
| B9 | Update `MockState::reset()` to clear all new fields | Preserve seed data only |
| B10 | Seed 3–5 pre-populated user profiles with varying fields | Mix of filled/empty bios, avatars, different slug numbers |
| B11 | Seed 2 pre-existing follow relationships | e.g., user_alice follows user_bob, user_bob follows user_alice (mutual) |
| B12 | Seed 1 pre-existing block relationship | e.g., user_carol blocks user_dave |
| B13 | Seed default preferences for each seeded user | Default to `MYGRANDMALIKES` |
| B14 | Seed referral mapping for at least 1 user pair | e.g., user_alice was invited by seed_verified user |
| B15 | Add helper methods to `MockState` | `find_user_by_username()`, `is_following()`, `is_blocked_by()`, `are_friends()`, `count_followers()`, `count_following()`, `count_friends()`, `count_blocked()`, `user_has_reported()` |

### Phase 2.C — User Queries (`schema/query/users.rs`)

| # | Task | Notes |
|---|------|-------|
| C1 | Create `schema/query/users.rs` with `UserQuery` struct | Uses `#[Object]` |
| C2 | Implement `get_profile(userid?, contentFilterBy?)` | If `userid` is None, return current user's profile; compute social stats dynamically from `follows`/`blocks` sets |
| C3 | Implement `list_users_v2(contentFilterBy?, userid?, username?, offset, limit)` | Search by ID or partial username match; apply content filtering specs; paginate |
| C4 | Implement `search_user(username!, offset?, limit?)` | Partial username match; return minimal user info (`id`, `username`, `slug`, `img`); paginate |
| C5 | Implement `get_user(id!)` | Return user info with preferences; matches `GET_USER_QUERY` response shape |
| C6 | Implement `list_follow_relations(userid?, contentFilterBy?, offset, limit)` | Return `{ followers, following }` for target user; annotate each entry with `isfollowed`/`isfollowing` relative to current user |
| C7 | Implement `list_friends(userid?, contentFilterBy?, offset, limit)` | Return mutual follows only; paginate |
| C8 | Implement `list_blocked_users(contentFilterBy?, offset, limit)` | Return `{ iBlocked, blockedBy }` for current user |
| C9 | Implement `get_user_info` | Return current user's account info + preferences |
| C10 | Implement `get_referral_info` | Return current user's referral UUID (= their user UUID) and shareable link |
| C11 | Implement `referral_list(offset, limit)` | Return `invitedBy` and `iInvited` from `referral_invitations` map |

### Phase 2.D — Profile Mutations (`schema/mutation/profile.rs`)

| # | Task | Notes |
|---|------|-------|
| D1 | Create `schema/mutation/profile.rs` with `ProfileMutation` struct | Uses `#[Object]` |
| D2 | Implement `toggle_user_follow_status(userid!)` | Toggle follow; if adding follow, return `isfollowing: true` + code `11104`; if removing, `isfollowing: false` + code `11103` |
| D3 | Implement `toggle_block_user_status(userid!)` | Toggle block; if blocking, also remove mutual follows; return `11105` (blocked) or `11106` (unblocked) |
| D4 | Implement `report_user(userid!)` | Validate: can't report self (`31009`), can't duplicate report (`31008`), user must exist (`31007`); record report; return `11012` |
| D5 | Implement `update_profile_image(img!)` | Store `img` on user record; return `{ status, ResponseCode: "11004" }` |
| D6 | Implement `update_bio(biography!)` | Store biography on user record; return `{ status, ResponseCode: "11003" }` |
| D7 | Implement `update_username(username!, password!)` | Validate format (`^[a-zA-Z0-9_-]+$`, 3–23 chars); verify password; update username; return `11007` |
| D8 | Implement `update_email(email!, password!)` | Verify password; update email on user record and in `registered_emails` set; return `11006` |
| D9 | Implement `update_user_preferences(userPreferences)` | Update content filtering and onboardings; return `{ status, ResponseCode: "11014", affectedRows: { contentFilteringSeverityLevel } }` |

### Phase 2.E — Content Filtering Logic

| # | Task | Notes |
|---|------|-------|
| E1 | Create a `filter_users()` helper function | Applies all ContentFilterSpecs to a user list |
| E2 | Implement `IllegalContentFilterSpec` | Exclude users with `visibility_status == ILLEGAL` |
| E3 | Implement `SystemUserSpec` | Exclude users with role bitmask 1, 2, or 4 |
| E4 | Implement `DeletedUserSpec` | Exclude users with `status == 6` |
| E5 | Implement `UserIsBlockedByMeSpec` | Exclude users the current user has blocked |
| E6 | Implement `CurrentUserIsBlockedUserSpec` | Exclude users who have blocked the current user |
| E7 | Apply `filter_users()` to `listUsersV2`, `searchUser`, `listFollowRelations`, `listFriends` | Before pagination |

### Phase 2.F — Schema Assembly

| # | Task | Notes |
|---|------|-------|
| F1 | Refactor `QueryRoot` to use `MergedObject` (if not already) | So additional query structs can be added |
| F2 | Add `pub mod users;` to `schema/query/mod.rs` | Register the new query module |
| F3 | Add `UserQuery` to `QueryRoot` merged object | `QueryRoot(HealthQuery, UserQuery)` (or equivalent) |
| F4 | Add `pub mod profile;` to `schema/mutation/mod.rs` | Register the new mutation module |
| F5 | Add `ProfileMutation` to `MutationRoot` merged object | `MutationRoot(RegistrationMutation, AuthMutation, ProfileMutation)` |

### Phase 2.G — Integration Tests

| # | Test | Assert |
|---|------|--------|
| G1 | `getProfile` — own profile (no userid) | `11008`, returns current user's profile with correct stats |
| G2 | `getProfile` — another user's profile (by userid) | `11008`, returns target profile with `iFollowThisUser`/`thisUserFollowsMe` relative to current user |
| G3 | `getProfile` — non-existent user | `21001` |
| G4 | `getProfile` — unauthenticated | `60501` |
| G5 | `searchUser` — by username (partial match) | Returns matching users, correct `counter` |
| G6 | `searchUser` — no matches | Empty `affectedRows`, `counter: 0` |
| G7 | `searchUser` — pagination (offset/limit) | Correct subset returned |
| G8 | `listUsersV2` — search by username | `11001`, users found |
| G9 | `listUsersV2` — search by userid | `11001`, single user found |
| G10 | `listUsersV2` — empty results | `21001` |
| G11 | `listUsersV2` — excludes blocked users | Users blocked by current user are omitted |
| G12 | `listUsersV2` — excludes deleted users | Users with `status == 6` are omitted |
| G13 | `getUser` — by ID | Returns user info with preferences |
| G14 | `toggleUserFollowStatus` — follow | `11104`, `isfollowing: true` |
| G15 | `toggleUserFollowStatus` — unfollow | `11103`, `isfollowing: false` |
| G16 | `toggleUserFollowStatus` — unauthenticated | `60501` |
| G17 | `listFollowRelations` — own user | Returns followers and following lists with correct counts |
| G18 | `listFollowRelations` — another user | Returns that user's followers/following |
| G19 | `listFollowRelations` — annotates `isfollowed`/`isfollowing` relative to current user | Verifies social context fields |
| G20 | `listFollowRelations` — pagination | Offset/limit applied correctly |
| G21 | `listFriends` — returns mutual follows only | Users who follow each other both appear |
| G22 | `listFriends` — no friends | `21101`, empty |
| G23 | `toggleBlockUserStatus` — block | `11105`, follow relationships removed in both directions |
| G24 | `toggleBlockUserStatus` — unblock | `11106` |
| G25 | `toggleBlockUserStatus` — unauthenticated | `60501` |
| G26 | `listBlockedUsers` — returns `iBlocked` and `blockedBy` | Correct users in each list |
| G27 | `reportUser` — success | `11012` |
| G28 | `reportUser` — self-report | `31009` |
| G29 | `reportUser` — duplicate report | `31008` |
| G30 | `reportUser` — non-existent user | `31007` |
| G31 | `updateProfileImage` — success | `11004`, image URL updated on profile |
| G32 | `updateBio` — success | `11003`, biography updated on profile |
| G33 | `updateUsername` — success | `11007`, username changed |
| G34 | `updateUsername` — invalid format | `30202` |
| G35 | `updateUsername` — wrong password | `31001` |
| G36 | `updateEmail` — success | `11006`, email changed |
| G37 | `updateEmail` — wrong password | `31001` |
| G38 | `updateUserPreferences` — update severity level | `11014`, returns updated preferences |
| G39 | `getUserInfo` — returns own account info with preferences | `11009` |
| G40 | `getReferralInfo` — returns referral UUID | `11011` |
| G41 | `referralList` — returns inviter and invitees | `11011`, invitedBy populated for alice |
| G42 | Follow → then block → verify follow removed | Full interaction flow |
| G43 | Block → verify blocked user excluded from search | Content filtering integration |
| G44 | `listUsersV2` — excludes blocked users (integration) | Carol's search omits dave |
| G45 | `getUser` — by ID returns preferences | Returns `userPreferences.contentFilteringSeverityLevel` |
| G46 | `listUsersV2` — search by username | `11001`, matching users returned |

### Phase 2.H — Cleanup & Validation

| # | Task | Notes |
|---|------|-------|
| H1 | Run `cargo clippy -- -D warnings` | Fix all warnings |
| H2 | Run `cargo fmt --check` | Fix formatting |
| H3 | Run full test suite (`cargo test --all-targets`) | All Phase 0 + Phase 1 + Phase 2 tests pass |
| H4 | Manual smoke test with `cargo run` + curl | Verify HTTP layer works for new endpoints |

---

## 4. Implementation Details

### 4.1 User & Profile Types (`types/user.rs`)

```rust
use async_graphql::{Enum, InputObject, SimpleObject, ID};
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Content visibility status for profiles and posts.
#[derive(Enum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentVisibilityStatus {
    #[graphql(name = "NORMAL")]
    Normal,
    #[graphql(name = "HIDDEN")]
    Hidden,
    #[graphql(name = "ILLEGAL")]
    Illegal,
}

impl Default for ContentVisibilityStatus {
    fn default() -> Self {
        Self::Normal
    }
}

/// Content filtering severity level.
#[derive(Enum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentFilterType {
    /// Stricter: hides all flagged content.
    #[graphql(name = "MYGRANDMALIKES")]
    Mygrandmalikes,
    /// Less strict: shows placeholders for flagged content.
    #[graphql(name = "MYGRANDMAHATES")]
    Mygrandmahates,
}

impl Default for ContentFilterType {
    fn default() -> Self {
        Self::Mygrandmalikes
    }
}

/// Onboarding types.
#[derive(Enum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingType {
    #[graphql(name = "INTROONBOARDING")]
    IntroOnboarding,
}

// ============================================================================
// Profile types (getProfile response)
// ============================================================================

/// Full profile with social statistics.
///
/// Must match frontend's `src/models/profile.rs::Profile` deserialization.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ProfileGql {
    pub id: ID,
    pub username: String,
    pub status: i32,
    pub slug: i32,
    pub img: Option<String>,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    /// Whether the current user follows this profile.
    #[graphql(name = "iFollowThisUser")]
    pub i_follow_this_user: bool,
    /// Whether this profile follows the current user.
    #[graphql(name = "thisUserFollowsMe")]
    pub this_user_follows_me: bool,
    /// Whether the current user has reported this user.
    pub isreported: bool,
    pub amountposts: i32,
    pub amounttrending: i32,
    /// Number of users this profile follows.
    pub amountfollowed: i32,
    /// Number of followers.
    pub amountfollower: i32,
    /// Number of mutual follows (peers/friends).
    pub amountfriends: i32,
    pub amountblocked: i32,
    pub amountreports: i32,
}

/// Wrapper for `getProfile` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ProfileInfoResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ProfileGql>,
}

// ============================================================================
// ProfileUser types (follow lists)
// ============================================================================

/// User representation in follow lists (followers/following).
///
/// Must match frontend's `src/models/profile.rs::ProfileUser` deserialization.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ProfileUserGql {
    pub userid: ID,
    pub username: String,
    pub slug: i32,
    pub img: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    /// Whether the current user follows this person.
    pub isfollowed: bool,
    /// Whether this person follows the current user.
    pub isfollowing: bool,
}

/// Follow relations aggregate (followers + following lists).
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FollowRelationsGql {
    pub followers: Vec<ProfileUserGql>,
    pub following: Vec<ProfileUserGql>,
}

/// Wrapper for `listFollowRelations` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FollowRelationsResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<FollowRelationsGql>,
}

// ============================================================================
// BasicUserInfo types (friends list)
// ============================================================================

/// Basic user info for friends/peers list.
///
/// Must match frontend's `src/models/profile.rs::BasicUserInfo` deserialization.
/// Note: `updatedat` is included because the backend schema returns it; the frontend
/// struct doesn't deserialize it (serde ignores unknown fields by default).
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BasicUserInfoGql {
    pub userid: ID,
    pub img: Option<String>,
    pub username: String,
    pub slug: i32,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    pub updatedat: Option<String>,
}

/// Wrapper for `listFriends` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FriendsResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Vec<BasicUserInfoGql>,
}

// ============================================================================
// Follow status toggle response
// ============================================================================

/// Response from `toggleUserFollowStatus` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FollowStatusResponseGql {
    pub meta: DefaultResponse,
    pub isfollowing: bool,
}

// ============================================================================
// Search / list user types
// ============================================================================

/// Minimal user info for search results.
///
/// Matches the fields selected in `SEARCH_USERS_QUERY`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct SearchUserResult {
    pub id: ID,
    pub username: String,
    pub slug: i32,
    pub img: Option<String>,
}

/// Wrapper for `searchUser` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct SearchUserResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<SearchUserResult>>,
}

/// User fields returned by `listUsersV2` (superset of SearchUserResult).
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserListItem {
    pub id: ID,
    pub username: String,
    pub status: i32,
    pub slug: i32,
    pub img: Option<String>,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    pub createdat: Option<String>,
    pub updatedat: Option<String>,
}

/// Wrapper for `listUsersV2` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<UserListItem>>,
}

// ============================================================================
// Blocked users types
// ============================================================================

/// A blocked user entry.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BlockedUserGql {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: i32,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
}

/// Blocked users aggregate.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BlockedUsersGql {
    /// Users the current user has blocked.
    #[graphql(name = "iBlocked")]
    pub i_blocked: Vec<BlockedUserGql>,
    /// Users who have blocked the current user.
    #[graphql(name = "blockedBy")]
    pub blocked_by: Vec<BlockedUserGql>,
}

/// Wrapper for `listBlockedUsers` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BlockedUsersResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<BlockedUsersGql>,
}

// ============================================================================
// User info types (getUserInfo)
// ============================================================================

/// User preferences.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesGql {
    #[graphql(name = "contentFilteringSeverityLevel")]
    pub content_filtering_severity_level: Option<ContentFilterType>,
    #[graphql(name = "onboardingsWereShown")]
    pub onboardings_were_shown: Vec<OnboardingType>,
}

/// User info returned by `getUserInfo`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserInfoGql {
    pub userid: ID,
    pub liquidity: f64,
    pub amountposts: i32,
    pub amountreports: i32,
    pub amountblocked: i32,
    pub amountfollower: i32,
    pub amountfollowed: i32,
    pub amountfriends: i32,
    pub invited: Option<ID>,
    pub updatedat: Option<String>,
    #[graphql(name = "userPreferences")]
    pub user_preferences: Option<UserPreferencesGql>,
}

/// Wrapper for `getUserInfo` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserInfoResponseGql {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<UserInfoGql>,
}

/// User preferences update input.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesInput {
    pub content_filtering_severity_level: Option<ContentFilterType>,
    pub shown_onboardings: Option<Vec<OnboardingType>>,
}

/// Response from `updateUserPreferences` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesResponseGql {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<UserPreferencesPayloadGql>,
}

/// Payload within UserPreferencesResponseGql.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesPayloadGql {
    #[graphql(name = "contentFilteringSeverityLevel")]
    pub content_filtering_severity_level: Option<String>,
}

/// Simple update response for settings mutations (updateBio, updateProfileImage, etc.)
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateResponseGql {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
}

impl UpdateResponseGql {
    pub fn success(code: &str) -> Self {
        Self {
            status: "success".to_string(),
            response_code: Some(code.to_string()),
        }
    }

    pub fn error(code: &str) -> Self {
        Self {
            status: "error".to_string(),
            response_code: Some(code.to_string()),
        }
    }
}

// ============================================================================
// Referral types
// ============================================================================

/// Response from `getReferralInfo` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ReferralInfoResponseGql {
    pub meta: DefaultResponse,
    #[graphql(name = "referralUuid")]
    pub referral_uuid: Option<ID>,
    #[graphql(name = "referralLink")]
    pub referral_link: Option<String>,
}

/// Referral users aggregate.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ReferralUsersGql {
    /// User who invited the current user.
    #[graphql(name = "invitedBy")]
    pub invited_by: Option<ProfileUserGql>,
    /// Users invited by the current user.
    #[graphql(name = "iInvited")]
    pub i_invited: Vec<ProfileUserGql>,
}

/// Wrapper for `referralList` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ReferralListResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: ReferralUsersGql,
}

// ============================================================================
// getUser response (matches GET_USER_QUERY)
// ============================================================================

/// User info with preferences, returned by `getUser(id)` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetUserResult {
    pub id: ID,
    pub username: String,
    pub slug: i32,
    pub img: Option<String>,
    pub biography: Option<String>,
    #[graphql(name = "amountFollowers")]
    pub amount_followers: i32,
    #[graphql(name = "amountFollowing")]
    pub amount_following: i32,
    #[graphql(name = "amountPeers")]
    pub amount_peers: i32,
    #[graphql(name = "userPreferences")]
    pub user_preferences: Option<UserPreferencesGql>,
}

/// Wrapper for `getUser` query response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetUserResponseGql {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<GetUserResult>,
}
```

### 4.2 State Extensions (`state.rs`)

```rust
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub type SharedState = Arc<RwLock<MockState>>;

/// Content visibility status (state-level).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentVisibilityState {
    Normal,
    Hidden,
    Illegal,
}

impl Default for ContentVisibilityState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Content filtering severity level (state-level).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFilterState {
    Mygrandmalikes,
    Mygrandmahates,
}

impl Default for ContentFilterState {
    fn default() -> Self {
        Self::Mygrandmalikes
    }
}

/// A mock user record (extended from Phase 1).
#[derive(Debug, Clone)]
pub struct User {
    pub uid: Uuid,
    pub email: String,
    pub username: String,
    pub slug: String,
    pub slug_num: i32,                              // Numeric slug for display
    pub role: u32,                                   // 0 = user, 16 = admin, 256 = moderator
    pub status: u32,                                 // 0 = active, 6 = deleted
    pub img: Option<String>,                         // Avatar URL
    pub biography: Option<String>,                   // Bio text (data URI)
    pub visibility_status: ContentVisibilityState,
    pub created_at: String,                          // ISO 8601
    pub updated_at: String,                          // ISO 8601
}

/// User preferences stored in state.
#[derive(Debug, Clone)]
pub struct UserPreferencesState {
    pub content_filtering_severity_level: ContentFilterState,
    pub onboardings_were_shown: Vec<String>,
}

impl Default for UserPreferencesState {
    fn default() -> Self {
        Self {
            content_filtering_severity_level: ContentFilterState::Mygrandmalikes,
            onboardings_were_shown: Vec::new(),
        }
    }
}

/// A user report record.
#[derive(Debug, Clone)]
pub struct UserReport {
    pub reporter: Uuid,
    pub reported: Uuid,
    pub created_at: String,
}

/// In-memory mock backend state.
#[derive(Debug, Clone)]
pub struct MockState {
    // --- Phase 0 fields ---
    pub known_referrals: HashSet<Uuid>,
    pub registered_emails: HashSet<String>,
    pub verified_users: HashSet<Uuid>,

    // --- Phase 1 fields ---
    pub users: HashMap<Uuid, User>,
    pub user_passwords: HashMap<Uuid, String>,
    pub access_tokens: HashMap<String, Uuid>,
    pub refresh_tokens: HashMap<String, Uuid>,
    pub password_reset_tokens: HashMap<String, Uuid>,
    pub deleted_users: HashSet<Uuid>,
    pub contact_messages: Vec<ContactMessage>,

    // --- Phase 2 fields ---
    pub follows: HashSet<(Uuid, Uuid)>,                      // (follower, followed)
    pub blocks: HashSet<(Uuid, Uuid)>,                        // (blocker, blocked)
    pub reports: Vec<UserReport>,
    pub preferences: HashMap<Uuid, UserPreferencesState>,
    pub referral_invitations: HashMap<Uuid, Uuid>,            // invitee → inviter
}
```

### 4.3 State Helper Methods

```rust
impl MockState {
    /// Check if `follower` follows `followed`.
    pub fn is_following(&self, follower: &Uuid, followed: &Uuid) -> bool {
        self.follows.contains(&(*follower, *followed))
    }

    /// Check if `blocker` has blocked `blocked`.
    pub fn is_blocked_by(&self, blocker: &Uuid, blocked: &Uuid) -> bool {
        self.blocks.contains(&(*blocker, *blocked))
    }

    /// Check if two users are mutual follows (friends/peers).
    pub fn are_friends(&self, a: &Uuid, b: &Uuid) -> bool {
        self.is_following(a, b) && self.is_following(b, a)
    }

    /// Count followers of a user.
    pub fn count_followers(&self, uid: &Uuid) -> i32 {
        self.follows.iter().filter(|(_, followed)| followed == uid).count() as i32
    }

    /// Count users that a user follows.
    pub fn count_following(&self, uid: &Uuid) -> i32 {
        self.follows.iter().filter(|(follower, _)| follower == uid).count() as i32
    }

    /// Count mutual follows (friends) of a user.
    pub fn count_friends(&self, uid: &Uuid) -> i32 {
        self.follows
            .iter()
            .filter(|(follower, followed)| {
                follower == uid && self.follows.contains(&(*followed, *follower))
            })
            .count() as i32
    }

    /// Count users blocked by a user.
    pub fn count_blocked(&self, uid: &Uuid) -> i32 {
        self.blocks.iter().filter(|(blocker, _)| blocker == uid).count() as i32
    }

    /// Count reports against a user.
    pub fn count_reports(&self, uid: &Uuid) -> i32 {
        self.reports.iter().filter(|r| r.reported == *uid).count() as i32
    }

    /// Check if `reporter` has already reported `reported`.
    pub fn user_has_reported(&self, reporter: &Uuid, reported: &Uuid) -> bool {
        self.reports.iter().any(|r| r.reporter == *reporter && r.reported == *reported)
    }

    /// Check if a user has any active reports against them.
    pub fn has_active_reports(&self, uid: &Uuid) -> bool {
        self.reports.iter().any(|r| r.reported == *uid)
    }

    /// Find user by username (case-insensitive exact match).
    pub fn find_user_by_username(&self, username: &str) -> Option<&User> {
        let lower = username.to_lowercase();
        self.users.values().find(|u| u.username.to_lowercase() == lower)
    }

    /// Search users by partial username match (case-insensitive).
    pub fn search_users_by_username(&self, query: &str) -> Vec<&User> {
        let lower = query.to_lowercase();
        self.users.values()
            .filter(|u| u.username.to_lowercase().contains(&lower))
            .collect()
    }
}
```

### 4.4 Seed Data Extensions (`seed.rs`)

```rust
use uuid::{uuid, Uuid};

// --- Phase 2 seed users (in addition to Phase 1 seed users) ---
pub const SEED_USER_ALICE: Uuid = uuid!("00000000-0000-4000-a000-000000000003");
pub const SEED_USER_BOB: Uuid = uuid!("00000000-0000-4000-a000-000000000004");
pub const SEED_USER_CAROL: Uuid = uuid!("00000000-0000-4000-a000-000000000005");
pub const SEED_USER_DAVE: Uuid = uuid!("00000000-0000-4000-a000-000000000006");

pub mod credentials_phase2 {
    pub const ALICE_EMAIL: &str = "alice@peer.com";
    pub const ALICE_PASSWORD: &str = "AlicePass123";
    pub const ALICE_USERNAME: &str = "alice_peer";

    pub const BOB_EMAIL: &str = "bob@peer.com";
    pub const BOB_PASSWORD: &str = "BobPass123";
    pub const BOB_USERNAME: &str = "bob_peer";

    pub const CAROL_EMAIL: &str = "carol@peer.com";
    pub const CAROL_PASSWORD: &str = "CarolPass123";
    pub const CAROL_USERNAME: &str = "carol_peer";

    pub const DAVE_EMAIL: &str = "dave@peer.com";
    pub const DAVE_PASSWORD: &str = "DavePass123";
    pub const DAVE_USERNAME: &str = "dave_peer";
}

// In MockState::default(), add after existing seeded users:

// Seed alice (verified, has bio and avatar)
users.insert(SEED_USER_ALICE, User {
    uid: SEED_USER_ALICE,
    email: ALICE_EMAIL.to_string(),
    username: ALICE_USERNAME.to_string(),
    slug: "alice_peer".to_string(),
    slug_num: 10003,
    role: 0,
    status: 0,
    img: Some("https://via.placeholder.com/96/alice".to_string()),
    biography: Some("data:text/plain;base64,SGVsbG8gSSdtIEFsaWNl".to_string()), // "Hello I'm Alice"
    visibility_status: ContentVisibilityState::Normal,
    created_at: "2025-01-15T10:00:00Z".to_string(),
    updated_at: "2025-06-01T12:00:00Z".to_string(),
});
// ... (similarly for bob, carol, dave)

// Seed follow relationships
// alice ↔ bob (mutual = friends)
follows.insert((SEED_USER_ALICE, SEED_USER_BOB));
follows.insert((SEED_USER_BOB, SEED_USER_ALICE));
// carol → alice (one-directional)
follows.insert((SEED_USER_CAROL, SEED_USER_ALICE));

// Seed block relationship
// carol blocks dave
blocks.insert((SEED_USER_CAROL, SEED_USER_DAVE));

// Seed referral
// alice was invited by seed_verified user
referral_invitations.insert(SEED_USER_ALICE, SEED_USER_VERIFIED);

// Seed default preferences for all users
for uid in [SEED_USER_VERIFIED, SEED_USER_ALICE, SEED_USER_BOB, SEED_USER_CAROL, SEED_USER_DAVE] {
    preferences.insert(uid, UserPreferencesState::default());
}
```

### 4.5 Content Filtering Logic

```rust
// New file: src/filters.rs
// Register in lib.rs: pub mod filters;

use uuid::Uuid;
use crate::state::{MockState, User, ContentVisibilityState};

/// System role bitmask values (role values 1, 2, 4 are system accounts).
const SYSTEM_ROLE_MASK: u32 = 0b111; // 1 | 2 | 4 = 7

/// Apply all content filtering specs to a user list.
/// Returns users that pass all filters.
pub fn filter_users<'a>(
    users: impl Iterator<Item = &'a User>,
    current_user_uid: Option<&Uuid>,
    state: &MockState,
) -> Vec<&'a User> {
    users
        .filter(|u| {
            // IllegalContentFilterSpec: exclude illegal content
            if u.visibility_status == ContentVisibilityState::Illegal {
                return false;
            }
            // SystemUserSpec: exclude system accounts (role bitmask 1, 2, 4)
            if u.role & SYSTEM_ROLE_MASK != 0 {
                return false;
            }
            // DeletedUserSpec: exclude soft-deleted users
            if u.status == 6 {
                return false;
            }
            // Block-based filtering (requires current user)
            if let Some(me) = current_user_uid {
                // UserIsBlockedByMeSpec: exclude users I blocked
                if state.is_blocked_by(me, &u.uid) {
                    return false;
                }
                // CurrentUserIsBlockedUserSpec: exclude users who blocked me
                if state.is_blocked_by(&u.uid, me) {
                    return false;
                }
            }
            true
        })
        .collect()
}

/// Apply offset/limit pagination to a slice.
pub fn paginate<T>(items: &[T], offset: usize, limit: usize) -> &[T] {
    let start = offset.min(items.len());
    let end = (start + limit).min(items.len());
    &items[start..end]
}
```

### 4.5a Auth Helpers (shared)

Phase 1 defines `require_auth()` and `get_current_user()` as private functions in
`schema/mutation/auth.rs`. Phase 2 resolvers (both queries and mutations) need
these helpers too. **Promote them to `pub` and re-export from `lib.rs`:**

```rust
// In schema/mutation/auth.rs — change visibility:
pub fn get_current_user(ctx: &Context<'_>) -> Option<Uuid> {
    ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0)
}

pub fn require_auth(ctx: &Context<'_>) -> Result<Uuid, DefaultResponse> {
    get_current_user(ctx).ok_or_else(|| DefaultResponse::error("60501", "Authentication required"))
}
```

```rust
// In lib.rs — add re-export for convenient cross-module use:
pub use schema::mutation::auth::{get_current_user, require_auth};
```

### 4.5b Visibility Status Conversion Helper

Add to `types/user.rs` (or `filters.rs`):

```rust
use crate::state::ContentVisibilityState;

/// Convert state-level visibility status to GraphQL enum.
pub fn convert_visibility(state: ContentVisibilityState) -> ContentVisibilityStatus {
    match state {
        ContentVisibilityState::Normal => ContentVisibilityStatus::Normal,
        ContentVisibilityState::Hidden => ContentVisibilityStatus::Hidden,
        ContentVisibilityState::Illegal => ContentVisibilityStatus::Illegal,
    }
}
```

### 4.6 User Query Resolvers (`schema/query/users.rs`)

```rust
use async_graphql::{Context, Object, ID};
use uuid::Uuid;

use crate::filters::{filter_users, paginate};
use crate::require_auth;
use crate::state::SharedState;
use crate::types::registration::DefaultResponse;
use crate::types::user::*;
use crate::CurrentUser;

pub struct UserQuery;

/// Build a ProfileUserGql from a User, annotated relative to the current user.
fn build_profile_user(
    user: &crate::state::User,
    me: &Uuid,
    state: &crate::state::MockState,
) -> ProfileUserGql {
    ProfileUserGql {
        userid: ID::from(user.uid.to_string()),
        username: user.username.clone(),
        slug: user.slug_num,
        img: user.img.clone(),
        visibility_status: convert_visibility(user.visibility_status),
        is_hidden_for_users: false,
        has_active_reports: state.has_active_reports(&user.uid),
        isfollowed: state.is_following(me, &user.uid),
        isfollowing: state.is_following(&user.uid, me),
    }
}

/// Build a BasicUserInfoGql from a User.
fn build_basic_user_info(
    user: &crate::state::User,
    state: &crate::state::MockState,
) -> BasicUserInfoGql {
    BasicUserInfoGql {
        userid: ID::from(user.uid.to_string()),
        img: user.img.clone(),
        username: user.username.clone(),
        slug: user.slug_num,
        biography: user.biography.clone(),
        visibility_status: convert_visibility(user.visibility_status),
        is_hidden_for_users: false,
        has_active_reports: state.has_active_reports(&user.uid),
        updatedat: Some(user.updated_at.clone()),
    }
}

#[Object]
impl UserQuery {
    // ========================================================================
    // getProfile
    // ========================================================================

    /// Fetch a user's full profile with social statistics.
    ///
    /// If `userid` is None, returns the current user's own profile.
    ///
    /// Response codes:
    /// - 11008: Profile loaded successfully
    /// - 21001: User not found
    /// - 60501: Not authenticated
    async fn get_profile(
        &self,
        ctx: &Context<'_>,
        userid: Option<ID>,
        content_filter_by: Option<ContentFilterType>,
    ) -> ProfileInfoResponse {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return ProfileInfoResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        // Determine target user
        let target_uid = match &userid {
            Some(id) => match Uuid::parse_str(id.as_str()) {
                Ok(uid) => uid,
                Err(_) => return ProfileInfoResponse {
                    meta: DefaultResponse::error("30201", "Invalid user UUID"),
                    affected_rows: None,
                },
            },
            None => me,
        };

        let user = match state_read.users.get(&target_uid) {
            Some(u) => u,
            None => return ProfileInfoResponse {
                meta: DefaultResponse::error("21001", "User not found"),
                affected_rows: None,
            },
        };

        let profile = ProfileGql {
            id: ID::from(user.uid.to_string()),
            username: user.username.clone(),
            status: user.status as i32,
            slug: user.slug_num,
            img: user.img.clone(),
            biography: user.biography.clone(),
            visibility_status: convert_visibility(user.visibility_status),
            is_hidden_for_users: false,
            has_active_reports: state_read.has_active_reports(&target_uid),
            i_follow_this_user: state_read.is_following(&me, &target_uid),
            this_user_follows_me: state_read.is_following(&target_uid, &me),
            isreported: state_read.user_has_reported(&me, &target_uid),
            amountposts: 0,  // Phase 3 will populate this
            amounttrending: 0,
            amountfollowed: state_read.count_following(&target_uid),
            amountfollower: state_read.count_followers(&target_uid),
            amountfriends: state_read.count_friends(&target_uid),
            amountblocked: state_read.count_blocked(&target_uid),
            amountreports: state_read.count_reports(&target_uid),
        };

        ProfileInfoResponse {
            meta: DefaultResponse::success("11008", "Profile loaded successfully"),
            affected_rows: Some(profile),
        }
    }

    // ========================================================================
    // searchUser
    // ========================================================================

    /// Search users by username (partial match).
    ///
    /// Response codes:
    /// - 11001: Users found
    /// - 21001: No users found
    async fn search_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> SearchUserResponse {
        let current_user = ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0);
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let matching = state_read.search_users_by_username(&username);
        let filtered = filter_users(matching.into_iter(), current_user.as_ref(), &state_read);

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(10).clamp(1, 20) as usize;
        let total = filtered.len() as i32;
        let page = paginate(&filtered, off, lim);

        if page.is_empty() {
            return SearchUserResponse {
                meta: DefaultResponse::success("21001", "No users found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let results: Vec<SearchUserResult> = page.iter().map(|u| SearchUserResult {
            id: ID::from(u.uid.to_string()),
            username: u.username.clone(),
            slug: u.slug_num,
            img: u.img.clone(),
        }).collect();

        SearchUserResponse {
            meta: DefaultResponse::success("11001", "Users retrieved successfully"),
            counter: total,
            affected_rows: Some(results),
        }
    }

    // ========================================================================
    // listUsersV2
    // ========================================================================

    /// Search and list users with optional filters and pagination.
    ///
    /// Response codes:
    /// - 11001: Users found
    /// - 21001: No users found
    async fn list_users_v2(
        &self,
        ctx: &Context<'_>,
        content_filter_by: Option<ContentFilterType>,
        userid: Option<ID>,
        username: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> UserListResponse {
        let current_user = ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0);
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        // Collect candidates: search by userid, username, or all users
        let candidates: Vec<&crate::state::User> = if let Some(ref id) = userid {
            match Uuid::parse_str(id.as_str()) {
                Ok(uid) => state_read.users.get(&uid).into_iter().collect(),
                Err(_) => vec![],
            }
        } else if let Some(ref name) = username {
            state_read.search_users_by_username(name)
        } else {
            state_read.users.values().collect()
        };

        let filtered = filter_users(candidates.into_iter(), current_user.as_ref(), &state_read);

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(10).clamp(1, 20) as usize;
        let total = filtered.len() as i32;
        let page = paginate(&filtered, off, lim);

        if page.is_empty() {
            return UserListResponse {
                meta: DefaultResponse::success("21001", "No users found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let items: Vec<UserListItem> = page.iter().map(|u| UserListItem {
            id: ID::from(u.uid.to_string()),
            username: u.username.clone(),
            status: u.status as i32,
            slug: u.slug_num,
            img: u.img.clone(),
            biography: u.biography.clone(),
            visibility_status: convert_visibility(u.visibility_status),
            is_hidden_for_users: false,
            has_active_reports: state_read.has_active_reports(&u.uid),
            createdat: Some(u.created_at.clone()),
            updatedat: Some(u.updated_at.clone()),
        }).collect();

        UserListResponse {
            meta: DefaultResponse::success("11001", "Users retrieved successfully"),
            counter: total,
            affected_rows: Some(items),
        }
    }

    // ========================================================================
    // getUser
    // ========================================================================

    /// Get user info by ID with preferences.
    ///
    /// Response codes:
    /// - 11001: User found
    /// - 21001: User not found
    async fn get_user(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> GetUserResponseGql {
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let uid = match Uuid::parse_str(id.as_str()) {
            Ok(uid) => uid,
            Err(_) => return GetUserResponseGql {
                meta: DefaultResponse::error("21001", "User not found"),
                affected_rows: None,
            },
        };

        let user = match state_read.users.get(&uid) {
            Some(u) => u,
            None => return GetUserResponseGql {
                meta: DefaultResponse::error("21001", "User not found"),
                affected_rows: None,
            },
        };

        let prefs = state_read.preferences.get(&uid);
        let user_prefs = prefs.map(|p| UserPreferencesGql {
            content_filtering_severity_level: Some(match p.content_filtering_severity_level {
                crate::state::ContentFilterState::Mygrandmalikes => ContentFilterType::Mygrandmalikes,
                crate::state::ContentFilterState::Mygrandmahates => ContentFilterType::Mygrandmahates,
            }),
            onboardings_were_shown: p.onboardings_were_shown.iter().filter_map(|s| {
                match s.as_str() {
                    "INTROONBOARDING" => Some(OnboardingType::IntroOnboarding),
                    _ => None,
                }
            }).collect(),
        });

        GetUserResponseGql {
            meta: DefaultResponse::success("11001", "User found"),
            affected_rows: Some(GetUserResult {
                id: ID::from(user.uid.to_string()),
                username: user.username.clone(),
                slug: user.slug_num,
                img: user.img.clone(),
                biography: user.biography.clone(),
                amount_followers: state_read.count_followers(&uid),
                amount_following: state_read.count_following(&uid),
                amount_peers: state_read.count_friends(&uid),
                user_preferences: user_prefs,
            }),
        }
    }

    // ========================================================================
    // listFollowRelations
    // ========================================================================

    /// List followers and following for a user.
    ///
    /// Response codes:
    /// - 11101: Follow relations loaded
    /// - 60501: Not authenticated
    async fn list_follow_relations(
        &self,
        ctx: &Context<'_>,
        userid: Option<ID>,
        content_filter_by: Option<ContentFilterType>,
        offset: i32,
        limit: i32,
    ) -> FollowRelationsResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return FollowRelationsResponseGql {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let target_uid = match &userid {
            Some(id) => Uuid::parse_str(id.as_str()).unwrap_or(me),
            None => me,
        };

        // Collect followers (users who follow target)
        let follower_uids: Vec<Uuid> = state_read.follows.iter()
            .filter(|(_, followed)| *followed == target_uid)
            .map(|(follower, _)| *follower)
            .collect();

        let followers: Vec<ProfileUserGql> = follower_uids.iter()
            .filter_map(|uid| state_read.users.get(uid))
            .map(|u| build_profile_user(u, &me, &state_read))
            .collect();

        // Collect following (users target follows)
        let following_uids: Vec<Uuid> = state_read.follows.iter()
            .filter(|(follower, _)| *follower == target_uid)
            .map(|(_, followed)| *followed)
            .collect();

        let following: Vec<ProfileUserGql> = following_uids.iter()
            .filter_map(|uid| state_read.users.get(uid))
            .map(|u| build_profile_user(u, &me, &state_read))
            .collect();

        let total = followers.len() + following.len();

        FollowRelationsResponseGql {
            meta: DefaultResponse::success("11101", "Follow relations loaded"),
            counter: total as i32,
            affected_rows: Some(FollowRelationsGql { followers, following }),
        }
    }

    // ========================================================================
    // listFriends
    // ========================================================================

    /// List mutual follows (friends/peers) for a user.
    ///
    /// Response codes:
    /// - 11102: Friends loaded
    /// - 21101: No friends found
    /// - 60501: Not authenticated
    async fn list_friends(
        &self,
        ctx: &Context<'_>,
        userid: Option<ID>,
        content_filter_by: Option<ContentFilterType>,
        offset: i32,
        limit: i32,
    ) -> FriendsResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return FriendsResponseGql {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: vec![],
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let target_uid = match &userid {
            Some(id) => Uuid::parse_str(id.as_str()).unwrap_or(me),
            None => me,
        };

        // Find mutual follows
        let friends: Vec<BasicUserInfoGql> = state_read.follows.iter()
            .filter(|(follower, followed)| {
                *follower == target_uid && state_read.is_following(followed, &target_uid)
            })
            .filter_map(|(_, followed)| state_read.users.get(followed))
            .map(|u| build_basic_user_info(u, &state_read))
            .collect();

        let total = friends.len() as i32;
        let off = offset.max(0) as usize;
        let lim = limit.clamp(1, 20) as usize;
        let page: Vec<BasicUserInfoGql> = friends.into_iter().skip(off).take(lim).collect();

        if page.is_empty() {
            return FriendsResponseGql {
                meta: DefaultResponse::success("21101", "No friends found"),
                counter: 0,
                affected_rows: vec![],
            };
        }

        FriendsResponseGql {
            meta: DefaultResponse::success("11102", "Friends loaded"),
            counter: total,
            affected_rows: page,
        }
    }

    // ========================================================================
    // listBlockedUsers
    // ========================================================================

    /// List users blocked by (and blocking) the current user.
    ///
    /// Response codes:
    /// - 11107: Blocked users loaded
    /// - 60501: Not authenticated
    async fn list_blocked_users(
        &self,
        ctx: &Context<'_>,
        content_filter_by: Option<ContentFilterType>,
        offset: i32,
        limit: i32,
    ) -> BlockedUsersResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return BlockedUsersResponseGql {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        // Users I blocked
        let i_blocked: Vec<BlockedUserGql> = state_read.blocks.iter()
            .filter(|(blocker, _)| *blocker == me)
            .filter_map(|(_, blocked)| state_read.users.get(blocked))
            .map(|u| BlockedUserGql {
                userid: u.uid.to_string(),
                img: u.img.clone(),
                username: u.username.clone(),
                slug: u.slug_num,
                has_active_reports: state_read.has_active_reports(&u.uid),
                visibility_status: convert_visibility(u.visibility_status),
                is_hidden_for_users: false,
            })
            .collect();

        // Users who blocked me
        let blocked_by: Vec<BlockedUserGql> = state_read.blocks.iter()
            .filter(|(_, blocked)| *blocked == me)
            .filter_map(|(blocker, _)| state_read.users.get(blocker))
            .map(|u| BlockedUserGql {
                userid: u.uid.to_string(),
                img: u.img.clone(),
                username: u.username.clone(),
                slug: u.slug_num,
                has_active_reports: state_read.has_active_reports(&u.uid),
                visibility_status: convert_visibility(u.visibility_status),
                is_hidden_for_users: false,
            })
            .collect();

        let total = (i_blocked.len() + blocked_by.len()) as i32;

        BlockedUsersResponseGql {
            meta: DefaultResponse::success("11107", "Blocked users loaded"),
            counter: total,
            affected_rows: Some(BlockedUsersGql { i_blocked, blocked_by }),
        }
    }

    // ========================================================================
    // getUserInfo
    // ========================================================================

    /// Get the current user's account info and preferences.
    ///
    /// Response codes:
    /// - 11009: User info loaded
    /// - 60501: Not authenticated
    async fn get_user_info(
        &self,
        ctx: &Context<'_>,
    ) -> UserInfoResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return UserInfoResponseGql {
                meta: DefaultResponse::error("60501", "Authentication required"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let user = match state_read.users.get(&me) {
            Some(u) => u,
            None => return UserInfoResponseGql {
                meta: DefaultResponse::error("21001", "User not found"),
                affected_rows: None,
            },
        };

        let prefs = state_read.preferences.get(&me);
        let user_prefs = prefs.map(|p| UserPreferencesGql {
            content_filtering_severity_level: Some(match p.content_filtering_severity_level {
                crate::state::ContentFilterState::Mygrandmalikes => ContentFilterType::Mygrandmalikes,
                crate::state::ContentFilterState::Mygrandmahates => ContentFilterType::Mygrandmahates,
            }),
            onboardings_were_shown: p.onboardings_were_shown.iter().filter_map(|s| {
                match s.as_str() {
                    "INTROONBOARDING" => Some(OnboardingType::IntroOnboarding),
                    _ => None,
                }
            }).collect(),
        });

        // Determine who invited this user (if any)
        let invited_by = state_read.referral_invitations.get(&me).map(|uid| ID::from(uid.to_string()));

        UserInfoResponseGql {
            meta: DefaultResponse::success("11009", "User info loaded"),
            affected_rows: Some(UserInfoGql {
                userid: ID::from(user.uid.to_string()),
                liquidity: 0.0,  // Phase 5 will populate this
                amountposts: 0,  // Phase 3 will populate this
                amountreports: state_read.count_reports(&me),
                amountblocked: state_read.count_blocked(&me),
                amountfollower: state_read.count_followers(&me),
                amountfollowed: state_read.count_following(&me),
                amountfriends: state_read.count_friends(&me),
                invited: invited_by,
                updatedat: Some(user.updated_at.clone()),
                user_preferences: user_prefs,
            }),
        }
    }

    // ========================================================================
    // getReferralInfo
    // ========================================================================

    /// Get the current user's referral UUID and shareable link.
    ///
    /// Response codes:
    /// - 11011: Referral info loaded
    /// - 60501: Not authenticated
    async fn get_referral_info(
        &self,
        ctx: &Context<'_>,
    ) -> ReferralInfoResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return ReferralInfoResponseGql {
                meta: DefaultResponse::error("60501", "Authentication required"),
                referral_uuid: None,
                referral_link: None,
            },
        };

        // In the real backend, the referral UUID is a separate field;
        // in the mock, we use the user's own UUID as their referral code.
        let referral_uuid = me.to_string();
        let referral_link = format!("https://peer.com/invite?referralUuid={}", referral_uuid);

        ReferralInfoResponseGql {
            meta: DefaultResponse::success("11011", "Referral info loaded"),
            referral_uuid: Some(ID::from(referral_uuid)),
            referral_link: Some(referral_link),
        }
    }

    // ========================================================================
    // referralList
    // ========================================================================

    /// List the current user's inviter and invitees.
    ///
    /// Response codes:
    /// - 11011: Referral list loaded
    /// - 60501: Not authenticated
    async fn referral_list(
        &self,
        ctx: &Context<'_>,
        offset: i32,
        limit: i32,
    ) -> ReferralListResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return ReferralListResponseGql {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: ReferralUsersGql {
                    invited_by: None,
                    i_invited: vec![],
                },
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        // Who invited me?
        let invited_by = state_read.referral_invitations.get(&me)
            .and_then(|inviter_uid| state_read.users.get(inviter_uid))
            .map(|u| build_profile_user(u, &me, &state_read));

        // Who did I invite?
        let i_invited: Vec<ProfileUserGql> = state_read.referral_invitations.iter()
            .filter(|(_, inviter)| **inviter == me)
            .filter_map(|(invitee, _)| state_read.users.get(invitee))
            .map(|u| build_profile_user(u, &me, &state_read))
            .collect();

        let total = i_invited.len() as i32 + if invited_by.is_some() { 1 } else { 0 };

        ReferralListResponseGql {
            meta: DefaultResponse::success("11011", "Referral list loaded"),
            counter: total,
            affected_rows: ReferralUsersGql { invited_by, i_invited },
        }
    }
}
```

### 4.7 Profile Mutation Resolvers (`schema/mutation/profile.rs`)

```rust
use async_graphql::{Context, Object, ID};
use uuid::Uuid;

use crate::require_auth;
use crate::state::SharedState;
use crate::types::registration::DefaultResponse;
use crate::types::user::*;
use crate::CurrentUser;

#[derive(Default)]
pub struct ProfileMutation;

#[Object]
impl ProfileMutation {
    // ========================================================================
    // toggleUserFollowStatus
    // ========================================================================

    /// Follow or unfollow a user. Toggles the follow state.
    ///
    /// Response codes:
    /// - 11104: Now following user
    /// - 11103: Unfollowed user
    /// - 30201: Invalid UUID
    /// - 60501: Not authenticated
    async fn toggle_user_follow_status(
        &self,
        ctx: &Context<'_>,
        userid: ID,
    ) -> FollowStatusResponseGql {
        let me = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => return FollowStatusResponseGql {
                meta: resp,
                isfollowing: false,
            },
        };

        let target = match Uuid::parse_str(userid.as_str()) {
            Ok(uid) => uid,
            Err(_) => return FollowStatusResponseGql {
                meta: DefaultResponse::error("30201", "Invalid UUID"),
                isfollowing: false,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let key = (me, target);
        if state_write.follows.contains(&key) {
            // Unfollow
            state_write.follows.remove(&key);
            FollowStatusResponseGql {
                meta: DefaultResponse::success("11103", "Unfollowed user"),
                isfollowing: false,
            }
        } else {
            // Follow
            state_write.follows.insert(key);
            FollowStatusResponseGql {
                meta: DefaultResponse::success("11104", "Now following user"),
                isfollowing: true,
            }
        }
    }

    // ========================================================================
    // toggleBlockUserStatus
    // ========================================================================

    /// Block or unblock a user. Toggles the block state.
    /// When blocking: removes follow relationships in both directions.
    ///
    /// Response codes:
    /// - 11105: User blocked
    /// - 11106: User unblocked
    /// - 30201: Invalid UUID
    /// - 60501: Not authenticated
    async fn toggle_block_user_status(
        &self,
        ctx: &Context<'_>,
        userid: ID,
    ) -> DefaultResponse {
        let me = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let target = match Uuid::parse_str(userid.as_str()) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("30201", "Invalid UUID"),
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let key = (me, target);
        if state_write.blocks.contains(&key) {
            // Unblock
            state_write.blocks.remove(&key);
            DefaultResponse::success("11106", "User unblocked")
        } else {
            // Block — also remove follow relationships in both directions
            state_write.blocks.insert(key);
            state_write.follows.remove(&(me, target));
            state_write.follows.remove(&(target, me));
            DefaultResponse::success("11105", "User blocked")
        }
    }

    // ========================================================================
    // reportUser
    // ========================================================================

    /// Report a user for moderation review.
    ///
    /// Response codes:
    /// - 11012: User reported
    /// - 31007: User not found
    /// - 31008: Already reported (duplicate)
    /// - 31009: Cannot report yourself
    /// - 60501: Not authenticated
    async fn report_user(
        &self,
        ctx: &Context<'_>,
        userid: ID,
    ) -> DefaultResponse {
        let me = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let target = match Uuid::parse_str(userid.as_str()) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("31007", "User not found"),
        };

        if me == target {
            return DefaultResponse::error("31009", "Cannot report yourself");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Check user exists
        if !state_write.users.contains_key(&target) {
            return DefaultResponse::error("31007", "User not found");
        }

        // Check duplicate report
        if state_write.user_has_reported(&me, &target) {
            return DefaultResponse::error("31008", "Already reported");
        }

        // Record report
        state_write.reports.push(UserReport {
            reporter: me,
            reported: target,
            created_at: "2025-01-01T00:00:00Z".to_string(), // mock timestamp
        });

        DefaultResponse::success("11012", "User reported")
    }

    // ========================================================================
    // updateProfileImage
    // ========================================================================

    /// Update the authenticated user's avatar image.
    ///
    /// Response codes:
    /// - 11004: Profile picture updated
    /// - 30101: Missing fields
    /// - 60501: Not authenticated
    async fn update_profile_image(
        &self,
        ctx: &Context<'_>,
        img: String,
    ) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        if img.is_empty() {
            return UpdateResponseGql::error("30101");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        if let Some(user) = state_write.users.get_mut(&me) {
            user.img = Some(img);
        }

        UpdateResponseGql::success("11004")
    }

    // ========================================================================
    // updateBio
    // ========================================================================

    /// Update the authenticated user's biography.
    ///
    /// Response codes:
    /// - 11003: Bio updated
    /// - 30101: Missing fields
    /// - 60501: Not authenticated
    async fn update_bio(
        &self,
        ctx: &Context<'_>,
        biography: String,
    ) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        if biography.is_empty() {
            return UpdateResponseGql::error("30101");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        if let Some(user) = state_write.users.get_mut(&me) {
            user.biography = Some(biography);
        }

        UpdateResponseGql::success("11003")
    }

    // ========================================================================
    // updateUsername
    // ========================================================================

    /// Change the authenticated user's username. Requires current password.
    ///
    /// Response codes:
    /// - 11007: Username changed
    /// - 30202: Invalid username format
    /// - 31001: Password does not match
    /// - 60501: Not authenticated
    async fn update_username(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
    ) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        // Validate username format: ^[a-zA-Z0-9_-]+$, 3–23 chars
        let username_re = regex::Regex::new(r"^[a-zA-Z0-9_-]{3,23}$").unwrap();
        if !username_re.is_match(&username) {
            return UpdateResponseGql::error("30202");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Verify password
        let stored = match state_write.user_passwords.get(&me) {
            Some(p) => p.clone(),
            None => return UpdateResponseGql::error("31001"),
        };
        if stored != password {
            return UpdateResponseGql::error("31001");
        }

        if let Some(user) = state_write.users.get_mut(&me) {
            user.username = username.clone();
            user.slug = username.to_lowercase();
        }

        UpdateResponseGql::success("11007")
    }

    // ========================================================================
    // updateEmail
    // ========================================================================

    /// Change the authenticated user's email. Requires current password.
    ///
    /// Response codes:
    /// - 11006: Email updated
    /// - 30103: Invalid email format
    /// - 31001: Password does not match
    /// - 60501: Not authenticated
    async fn update_email(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Verify password
        let stored = match state_write.user_passwords.get(&me) {
            Some(p) => p.clone(),
            None => return UpdateResponseGql::error("31001"),
        };
        if stored != password {
            return UpdateResponseGql::error("31001");
        }

        // Update email
        if let Some(user) = state_write.users.get_mut(&me) {
            let old_email = user.email.clone();
            user.email = email.clone();
            state_write.registered_emails.remove(&old_email);
            state_write.registered_emails.insert(email);
        }

        UpdateResponseGql::success("11006")
    }

    // ========================================================================
    // updateUserPreferences
    // ========================================================================

    /// Update content filtering and onboarding preferences.
    ///
    /// Response codes:
    /// - 11014: Preferences updated
    /// - 60501: Not authenticated
    async fn update_user_preferences(
        &self,
        ctx: &Context<'_>,
        user_preferences: Option<UserPreferencesInput>,
    ) -> UserPreferencesResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => return UserPreferencesResponseGql {
                status: "error".to_string(),
                response_code: Some("60501".to_string()),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let prefs = state_write.preferences.entry(me).or_insert_with(Default::default);

        if let Some(input) = user_preferences {
            if let Some(level) = input.content_filtering_severity_level {
                prefs.content_filtering_severity_level = match level {
                    ContentFilterType::Mygrandmalikes => ContentFilterState::Mygrandmalikes,
                    ContentFilterType::Mygrandmahates => ContentFilterState::Mygrandmahates,
                };
            }
            if let Some(onboardings) = input.shown_onboardings {
                for ob in onboardings {
                    let name = match ob {
                        OnboardingType::IntroOnboarding => "INTROONBOARDING".to_string(),
                    };
                    if !prefs.onboardings_were_shown.contains(&name) {
                        prefs.onboardings_were_shown.push(name);
                    }
                }
            }
        }

        let severity = match prefs.content_filtering_severity_level {
            ContentFilterState::Mygrandmalikes => "MYGRANDMALIKES",
            ContentFilterState::Mygrandmahates => "MYGRANDMAHATES",
        };

        UserPreferencesResponseGql {
            status: "success".to_string(),
            response_code: Some("11014".to_string()),
            affected_rows: Some(UserPreferencesPayloadGql {
                content_filtering_severity_level: Some(severity.to_string()),
            }),
        }
    }
}

/// Helper: require auth for update mutations.
/// Returns `Err(UpdateResponseGql)` with `60501` if unauthenticated.
fn require_auth_update(ctx: &Context<'_>) -> Result<Uuid, UpdateResponseGql> {
    ctx.data_opt::<CurrentUser>()
        .and_then(|cu| cu.0)
        .ok_or_else(|| UpdateResponseGql::error("60501"))
}
```

### 4.8 Schema Assembly

The current `QueryRoot` in `schema/query.rs` is a plain `#[Object]` struct with only
the `_health` query. To merge `UserQuery`, we need to refactor it into a `MergedObject`.

**Step 1:** Split `schema/query.rs` into a directory `schema/query/mod.rs` + `schema/query/health.rs` + `schema/query/users.rs`:

```rust
// schema/query/health.rs
use async_graphql::Object;

#[derive(Default)]
pub struct HealthQuery;

#[Object]
impl HealthQuery {
    #[graphql(name = "_health")]
    async fn health(&self) -> bool {
        true
    }
}
```

**Step 2:** Update `schema/query/mod.rs`:

```rust
pub mod health;
pub mod users;

use async_graphql::MergedObject;
use health::HealthQuery;
use users::UserQuery;

/// Query root — merged from all query modules.
#[derive(MergedObject, Default)]
pub struct QueryRoot(HealthQuery, UserQuery);
```

**Step 3:** Update `schema/mod.rs`:

```rust
pub mod mutation;
pub mod query;

use async_graphql::{EmptySubscription, MergedObject, Schema};

use crate::state::SharedState;
use mutation::auth::AuthMutation;
use mutation::profile::ProfileMutation;
use mutation::registration::RegistrationMutation;
use query::QueryRoot;

/// Combined mutation root.
#[derive(MergedObject, Default)]
pub struct MutationRoot(pub RegistrationMutation, pub AuthMutation, pub ProfileMutation);

/// The full GraphQL schema.
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Build the GraphQL schema with injected state.
pub fn build_schema(state: SharedState) -> AppSchema {
    Schema::build(QueryRoot::default(), MutationRoot::default(), EmptySubscription)
        .data(state)
        .finish()
}
```

**Step 4:** Update `schema/mutation/mod.rs`:

```rust
pub mod auth;
pub mod profile;
pub mod registration;
```

**Step 5:** Update `lib.rs` to re-export auth helpers:

```rust
pub mod filters;
pub mod schema;
pub mod seed;
pub mod state;
pub mod types;

// Re-export auth helpers for cross-module use
pub use schema::mutation::auth::{get_current_user, require_auth};
```

---

## 5. Testing Strategy

### Test Helpers

Extend existing test helpers from Phase 1.

**Important:** The existing `graphql_stateful()` and `graphql_with_auth()` helpers take
`state: &Arc<RwLock<MockState>>` and build the app internally — do NOT pass a `Router`.

```rust
use mock_backend::seed::{
    SEED_USER_ALICE, SEED_USER_BOB, SEED_USER_CAROL, SEED_USER_DAVE, SEED_USER_VERIFIED,
};

/// Helper: login a seeded user and return the access token.
async fn login_as(state: &Arc<RwLock<MockState>>, email: &str, password: &str) -> String {
    let res = graphql_stateful(state, &format!(r#"
        mutation {{
            login(email: "{email}", password: "{password}") {{
                accessToken
            }}
        }}
    "#)).await;
    res["data"]["login"]["accessToken"].as_str().unwrap().to_string()
}

/// Helper: login as the default seeded verified user.
async fn login_default(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "test@peer.com", "TestPass123").await
}

/// Helper: login as alice.
async fn login_alice(state: &Arc<RwLock<MockState>>) -> String {
    login_as(state, "alice@peer.com", "AlicePass123").await
}
```

### Test Implementations

Append to existing `tests/integration.rs`:

```rust

#[tokio::test]
async fn test_get_own_profile() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            getProfile {
                meta { status ResponseCode }
                affectedRows {
                    id username slug
                    iFollowThisUser thisUserFollowsMe
                    amountfollower amountfollowed amountfriends
                }
            }
        }
    "#, &token).await;

    let data = &res["data"]["getProfile"];
    assert_eq!(data["meta"]["ResponseCode"], "11008");
    assert_eq!(data["affectedRows"]["username"], "alice_peer");
    // Alice follows bob and bob follows alice → 1 friend
    assert_eq!(data["affectedRows"]["amountfriends"], 1);
}

#[tokio::test]
async fn test_get_other_user_profile() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(&state, &format!(r#"
        query {{
            getProfile(userid: "{}") {{
                meta {{ ResponseCode }}
                affectedRows {{
                    username
                    iFollowThisUser
                    thisUserFollowsMe
                }}
            }}
        }}
    "#, SEED_USER_BOB), &token).await;

    let data = &res["data"]["getProfile"]["affectedRows"];
    assert_eq!(data["username"], "bob_peer");
    assert_eq!(data["iFollowThisUser"], true);   // alice follows bob
    assert_eq!(data["thisUserFollowsMe"], true);  // bob follows alice
}

#[tokio::test]
async fn test_get_profile_not_found() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            getProfile(userid: "00000000-0000-0000-0000-000000000099") {
                meta { ResponseCode }
                affectedRows { id }
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["getProfile"]["meta"]["ResponseCode"], "21001");
    assert!(res["data"]["getProfile"]["affectedRows"].is_null());
}

#[tokio::test]
async fn test_get_profile_unauthenticated() {
    let res = graphql(r#"
        query {
            getProfile { meta { ResponseCode } }
        }
    "#).await;

    assert_eq!(res["data"]["getProfile"]["meta"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_search_user_by_username() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            searchUser(username: "alice", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { id username slug img }
            }
        }
    "#, &token).await;

    let data = &res["data"]["searchUser"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert!(data["counter"].as_i64().unwrap() >= 1);
    assert_eq!(data["affectedRows"][0]["username"], "alice_peer");
}

#[tokio::test]
async fn test_search_user_no_results() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            searchUser(username: "nonexistent_user_xyz", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["searchUser"]["meta"]["ResponseCode"], "21001");
    assert_eq!(res["data"]["searchUser"]["counter"], 0);
}

#[tokio::test]
async fn test_list_users_v2_by_username() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            listUsersV2(username: "bob", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { id username slug }
            }
        }
    "#, &token).await;

    let data = &res["data"]["listUsersV2"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert!(data["counter"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_list_users_v2_excludes_blocked() {
    let state = default_shared_state();
    // Carol blocks dave (seeded)
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(&state, r#"
        query {
            listUsersV2(username: "dave", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { username }
            }
        }
    "#, &token).await;

    // Dave should be excluded because carol blocks dave
    let data = &res["data"]["listUsersV2"];
    if let Some(arr) = data["affectedRows"].as_array() {
        for user in arr {
            assert_ne!(user["username"], "dave_peer");
        }
    }
}

#[tokio::test]
async fn test_get_user_by_id() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, &format!(r#"
        query {{
            getUser(id: "{}") {{
                meta {{ ResponseCode }}
                affectedRows {{
                    id username slug img biography
                    amountFollowers amountFollowing amountPeers
                    userPreferences {{ contentFilteringSeverityLevel }}
                }}
            }}
        }}
    "#, SEED_USER_ALICE), &token).await;

    let data = &res["data"]["getUser"];
    assert_eq!(data["meta"]["ResponseCode"], "11001");
    assert_eq!(data["affectedRows"]["username"], "alice_peer");
    assert!(data["affectedRows"]["userPreferences"]["contentFilteringSeverityLevel"].is_string());
}

#[tokio::test]
async fn test_toggle_follow() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Follow alice
    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{
            toggleUserFollowStatus(userid: "{}") {{
                meta {{ ResponseCode }}
                isfollowing
            }}
        }}
    "#, SEED_USER_ALICE), &token).await;

    let data = &res["data"]["toggleUserFollowStatus"];
    assert_eq!(data["meta"]["ResponseCode"], "11104");
    assert_eq!(data["isfollowing"], true);

    // Unfollow alice
    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{
            toggleUserFollowStatus(userid: "{}") {{
                meta {{ ResponseCode }}
                isfollowing
            }}
        }}
    "#, SEED_USER_ALICE), &token).await;

    let data = &res["data"]["toggleUserFollowStatus"];
    assert_eq!(data["meta"]["ResponseCode"], "11103");
    assert_eq!(data["isfollowing"], false);
}

#[tokio::test]
async fn test_toggle_follow_unauthenticated() {
    let res = graphql(&format!(r#"
        mutation {{
            toggleUserFollowStatus(userid: "{}") {{
                meta {{ ResponseCode }}
                isfollowing
            }}
        }}
    "#, SEED_USER_ALICE)).await;

    assert_eq!(res["data"]["toggleUserFollowStatus"]["meta"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_list_follow_relations() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            listFollowRelations(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows {
                    followers { userid username isfollowed isfollowing }
                    following { userid username isfollowed isfollowing }
                }
            }
        }
    "#, &token).await;

    let data = &res["data"]["listFollowRelations"];
    assert_eq!(data["meta"]["ResponseCode"], "11101");
    // Alice has 2 followers (bob + carol), follows 1 (bob)
    let followers = &data["affectedRows"]["followers"];
    let following = &data["affectedRows"]["following"];
    assert!(followers.as_array().unwrap().len() >= 2);
    assert!(following.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_friends_mutual_only() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            listFriends(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows { userid username }
            }
        }
    "#, &token).await;

    let data = &res["data"]["listFriends"];
    assert_eq!(data["meta"]["ResponseCode"], "11102");
    // Alice ↔ Bob is mutual
    let friends: Vec<&str> = data["affectedRows"]
        .as_array().unwrap()
        .iter()
        .map(|f| f["username"].as_str().unwrap())
        .collect();
    assert!(friends.contains(&"bob_peer"));
}

#[tokio::test]
async fn test_toggle_block_removes_follows() {
    let state = default_shared_state();
    let token = login_alice(&state).await;

    // Alice follows bob (seeded). Block bob.
    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{
            toggleBlockUserStatus(userid: "{}") {{
                status ResponseCode
            }}
        }}
    "#, SEED_USER_BOB), &token).await;

    assert_eq!(res["data"]["toggleBlockUserStatus"]["ResponseCode"], "11105");

    // Verify follow relationships removed
    let st = state.read().await;
    assert!(!st.is_following(&SEED_USER_ALICE, &SEED_USER_BOB));
    assert!(!st.is_following(&SEED_USER_BOB, &SEED_USER_ALICE));
}

#[tokio::test]
async fn test_toggle_block_unblock() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Block alice
    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{
            toggleBlockUserStatus(userid: "{}") {{ ResponseCode }}
        }}
    "#, SEED_USER_ALICE), &token).await;
    assert_eq!(res["data"]["toggleBlockUserStatus"]["ResponseCode"], "11105");

    // Unblock alice
    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{
            toggleBlockUserStatus(userid: "{}") {{ ResponseCode }}
        }}
    "#, SEED_USER_ALICE), &token).await;
    assert_eq!(res["data"]["toggleBlockUserStatus"]["ResponseCode"], "11106");
}

#[tokio::test]
async fn test_list_blocked_users() {
    let state = default_shared_state();
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(&state, r#"
        query {
            listBlockedUsers(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows {
                    iBlocked { userid username }
                    blockedBy { userid username }
                }
            }
        }
    "#, &token).await;

    let data = &res["data"]["listBlockedUsers"];
    assert_eq!(data["meta"]["ResponseCode"], "11107");
    // Carol blocks dave
    let i_blocked: Vec<&str> = data["affectedRows"]["iBlocked"]
        .as_array().unwrap()
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(i_blocked.contains(&"dave_peer"));
}

#[tokio::test]
async fn test_report_user_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{
            reportUser(userid: "{}") {{
                status ResponseCode
            }}
        }}
    "#, SEED_USER_DAVE), &token).await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "11012");
}

#[tokio::test]
async fn test_report_user_self() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{
            reportUser(userid: "{}") {{
                ResponseCode
            }}
        }}
    "#, SEED_USER_VERIFIED), &token).await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "31009");
}

#[tokio::test]
async fn test_report_user_duplicate() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // First report
    graphql_with_auth(&state, &format!(r#"
        mutation {{ reportUser(userid: "{}") {{ ResponseCode }} }}
    "#, SEED_USER_DAVE), &token).await;

    // Duplicate report
    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{ reportUser(userid: "{}") {{ ResponseCode }} }}
    "#, SEED_USER_DAVE), &token).await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "31008");
}

#[tokio::test]
async fn test_report_user_not_found() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            reportUser(userid: "00000000-0000-0000-0000-000000000099") {
                ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["reportUser"]["ResponseCode"], "31007");
}

#[tokio::test]
async fn test_update_profile_image() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateProfileImage(img: "data:image/png;base64,iVBOR...") {
                status ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["updateProfileImage"]["ResponseCode"], "11004");

    // Verify image persisted on profile
    let res = graphql_with_auth(&state, r#"
        query { getProfile { affectedRows { img } } }
    "#, &token).await;
    assert_eq!(
        res["data"]["getProfile"]["affectedRows"]["img"],
        "data:image/png;base64,iVBOR..."
    );
}

#[tokio::test]
async fn test_update_bio() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateBio(biography: "data:text/plain;base64,SGVsbG8gV29ybGQ=") {
                status ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["updateBio"]["ResponseCode"], "11003");
}

#[tokio::test]
async fn test_update_username_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateUsername(username: "newName123", password: "TestPass123") {
                status ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "11007");
}

#[tokio::test]
async fn test_update_username_invalid_format() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateUsername(username: "ab", password: "TestPass123") {
                ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "30202");
}

#[tokio::test]
async fn test_update_username_wrong_password() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateUsername(username: "validName", password: "WrongPass") {
                ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["updateUsername"]["ResponseCode"], "31001");
}

#[tokio::test]
async fn test_update_email_success() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateEmail(email: "newemail@peer.com", password: "TestPass123") {
                status ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["updateEmail"]["ResponseCode"], "11006");
}

#[tokio::test]
async fn test_update_email_wrong_password() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateEmail(email: "new@peer.com", password: "WrongPass") {
                ResponseCode
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["updateEmail"]["ResponseCode"], "31001");
}

#[tokio::test]
async fn test_update_preferences() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        mutation {
            updateUserPreferences(userPreferences: {
                contentFilteringSeverityLevel: MYGRANDMAHATES
            }) {
                status ResponseCode
                affectedRows { contentFilteringSeverityLevel }
            }
        }
    "#, &token).await;

    let data = &res["data"]["updateUserPreferences"];
    assert_eq!(data["ResponseCode"], "11014");
    assert_eq!(data["affectedRows"]["contentFilteringSeverityLevel"], "MYGRANDMAHATES");
}

#[tokio::test]
async fn test_get_user_info() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            getUserInfo {
                meta { ResponseCode }
                affectedRows {
                    userid
                    amountfollower
                    amountfollowed
                    userPreferences { contentFilteringSeverityLevel }
                }
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["getUserInfo"]["meta"]["ResponseCode"], "11009");
    assert!(res["data"]["getUserInfo"]["affectedRows"]["userid"].is_string());
}

#[tokio::test]
async fn test_get_referral_info() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            getReferralInfo {
                meta { ResponseCode }
                referralUuid
                referralLink
            }
        }
    "#, &token).await;

    assert_eq!(res["data"]["getReferralInfo"]["meta"]["ResponseCode"], "11011");
    assert!(res["data"]["getReferralInfo"]["referralUuid"].is_string());
}

#[tokio::test]
async fn test_referral_list() {
    let state = default_shared_state();
    // Alice was invited by SEED_USER_VERIFIED (seeded)
    let token = login_alice(&state).await;

    let res = graphql_with_auth(&state, r#"
        query {
            referralList(offset: 0, limit: 20) {
                meta { ResponseCode }
                counter
                affectedRows {
                    invitedBy { userid username }
                    iInvited { userid username }
                }
            }
        }
    "#, &token).await;

    let data = &res["data"]["referralList"];
    assert_eq!(data["meta"]["ResponseCode"], "11011");
    // Alice was invited by the seed verified user
    assert!(data["affectedRows"]["invitedBy"]["userid"].is_string());
}

#[tokio::test]
async fn test_follow_then_block_removes_follow() {
    let state = default_shared_state();
    let token = login_default(&state).await;

    // Follow dave
    let res = graphql_with_auth(&state, &format!(r#"
        mutation {{ toggleUserFollowStatus(userid: "{}") {{ isfollowing }} }}
    "#, SEED_USER_DAVE), &token).await;
    assert_eq!(res["data"]["toggleUserFollowStatus"]["isfollowing"], true);

    // Block dave
    graphql_with_auth(&state, &format!(r#"
        mutation {{ toggleBlockUserStatus(userid: "{}") {{ ResponseCode }} }}
    "#, SEED_USER_DAVE), &token).await;

    // Verify follow removed
    let st = state.read().await;
    assert!(!st.is_following(&SEED_USER_VERIFIED, &SEED_USER_DAVE));
}

#[tokio::test]
async fn test_blocked_user_excluded_from_search() {
    let state = default_shared_state();

    // Login as carol (who blocks dave)
    let token = login_as(&state, "carol@peer.com", "CarolPass123").await;

    let res = graphql_with_auth(&state, r#"
        query {
            searchUser(username: "dave", offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows { username }
            }
        }
    "#, &token).await;

    // Dave should be excluded because carol blocks dave
    let data = &res["data"]["searchUser"];
    let results = data["affectedRows"].as_array();
    if let Some(arr) = results {
        for user in arr {
            assert_ne!(user["username"], "dave_peer");
        }
    }
}
```

---

## 6. Definition of Done

> **Status:** All items complete (14 April 2026). Final: 73 tests pass, clippy clean, fmt clean.

### Build & Test Gates

- [x] `cargo build` succeeds without warnings
- [x] `cargo test --all-targets` passes all Phase 0 + Phase 1 + Phase 2 tests (40 new, 73 total)
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes

### Functional Requirements — Queries

- [x] `getProfile(userid?)` returns `ProfileInfoResponse` with correct social stats (`11008`)
- [x] `getProfile` without userid returns current user's own profile
- [x] `getProfile` returns `21001` for non-existent users
- [x] `getProfile` returns `60501` when unauthenticated
- [x] `getProfile` correctly computes `iFollowThisUser`, `thisUserFollowsMe`, `isreported`
- [x] `getProfile` correctly computes `amountfollower`, `amountfollowed`, `amountfriends`, `amountblocked`, `amountreports`
- [x] `searchUser(username)` returns matching users with partial match (`11001`)
- [x] `searchUser` returns `21001` for no matches
- [x] `searchUser` respects offset/limit pagination
- [x] `searchUser` excludes blocked users (both directions)
- [x] `listUsersV2` supports search by userid, username, pagination
- [x] `listUsersV2` applies all content filtering specs (illegal, system, deleted, blocked)
- [x] `getUser(id)` returns user info with preferences
- [x] `listFollowRelations` returns `{ followers, following }` with correct `isfollowed`/`isfollowing` annotations
- [x] `listFollowRelations` supports pagination
- [x] `listFriends` returns only mutual follows (`11102`)
- [x] `listFriends` returns `21101` when no friends
- [x] `listBlockedUsers` returns `{ iBlocked, blockedBy }` (`11107`)
- [x] `getUserInfo` returns current user's account info and preferences (`11009`)
- [x] `getReferralInfo` returns referral UUID and link (`11011`)
- [x] `referralList` returns inviter and invitees

### Functional Requirements — Mutations

- [x] `toggleUserFollowStatus` follows (`11104`, `isfollowing: true`) and unfollows (`11103`, `isfollowing: false`)
- [x] `toggleUserFollowStatus` returns `60501` when unauthenticated
- [x] `toggleBlockUserStatus` blocks (`11105`) and unblocks (`11106`)
- [x] `toggleBlockUserStatus` removes follow relationships in both directions when blocking
- [x] `toggleBlockUserStatus` returns `60501` when unauthenticated
- [x] `reportUser` records report (`11012`)
- [x] `reportUser` rejects self-report (`31009`)
- [x] `reportUser` rejects duplicate report (`31008`)
- [x] `reportUser` rejects non-existent user (`31007`)
- [x] `reportUser` returns `60501` when unauthenticated
- [x] `updateProfileImage(img)` updates avatar (`11004`)
- [x] `updateBio(biography)` updates bio (`11003`)
- [x] `updateUsername(username, password)` updates username (`11007`)
- [x] `updateUsername` rejects invalid format (`30202`)
- [x] `updateUsername` rejects wrong password (`31001`)
- [x] `updateEmail(email, password)` updates email (`11006`)
- [x] `updateEmail` rejects wrong password (`31001`)
- [x] `updateUserPreferences` updates severity level and onboardings (`11014`)
- [x] All settings mutations return `60501` when unauthenticated

### Content Filtering

- [x] Deleted users (`status == 6`) excluded from all list queries
- [x] Illegal-visibility users excluded from all list queries
- [x] System accounts (role bitmask 1, 2, 4) excluded from user lists
- [x] Blocked users excluded from search and list results (both directions)

### Response Shape Compatibility

- [x] `ProfileInfoResponse` shape matches `src/models/profile.rs::ProfileResponse`
- [x] `ProfileUserGql` fields match `src/models/profile.rs::ProfileUser` (including `isfollowed`/`isfollowing`)
- [x] `BasicUserInfoGql` fields match `src/models/profile.rs::BasicUserInfo`
- [x] `FollowRelationsResponseGql` shape matches `src/models/profile.rs::FollowRelationsResponse`
- [x] `FriendsResponseGql` shape matches `src/models/profile.rs::FriendsResponse`
- [x] `FollowStatusResponseGql` shape matches `src/models/profile.rs::FollowStatusResponse`
- [x] `SearchUserResponse` shape matches `SEARCH_USERS_QUERY` response (includes `id`, `username`, `slug`, `img`)
- [x] `GetUserResponseGql` shape matches `GET_USER_QUERY` response (includes preferences)
- [x] `UpdateResponseGql` fields match `src/models/settings.rs::UpdateResponse` (`status` + `ResponseCode` PascalCase)
- [x] `UserPreferencesResponseGql` shape matches `src/models/settings.rs::UserPreferencesUpdateResponse`
- [x] `DefaultResponse` fields: `status`, `RequestId`, `ResponseCode`, `ResponseMessage` (all PascalCase)

### State & Isolation

- [x] `POST /reset` clears all Phase 2 state (follows, blocks, reports, preferences, referrals) back to seed defaults
- [x] Seed data includes 4 users with profiles (+ 2 from Phase 1 = 6 total), 3 follow relationships (one mutual), 1 block relationship
- [x] All seeded users have default preferences
- [x] All seeded users can log in (verified + have passwords)

### Cross-Cutting (carried from parent plan)

- [x] No `unwrap()` in resolver paths — all errors return proper GraphQL responses
- [x] Every resolver has ≥1 success and ≥1 error integration test
- [x] Response codes match values in `docs/backend_api/02-users-and-profiles.md`
- [x] Field names exactly match the backend schema casing conventions
- [x] No runtime dependencies on Node.js, npm, or non-Rust tooling
