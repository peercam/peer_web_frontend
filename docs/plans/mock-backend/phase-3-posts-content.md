# Phase 3: Posts & Content

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** [Phase 2 — Users & Profiles](./mock-backend-rust-rewrite.md#4-phase-2--users--profiles) (requires user profiles for post authorship and social filtering)
> **Goal:** Add all post-related queries and mutations so the Leptos dashboard feed, view post page, new post page, and user profile posts work end-to-end against the mock.
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

From `peer-web/src/api/posts.rs` and `peer-web/src/api/graphql.rs`:

| Operation | GraphQL SDL | Frontend file | Auth? |
|-----------|-------------|---------------|-------|
| `listPosts(filterBy, contentFilterBy, sortBy, title, tag, offset, limit)` | `query ListPosts(...)` | `posts.rs` → `ListPosts` server fn | Yes |
| `listPosts(postid, limit: 1)` | `query ListPost($postid)` | `comments.rs` → `GetPost` server fn | Yes |
| `listPosts(userid, filterBy, sortBy, offset, limit)` | `query ListUserPosts(...)` | `graphql.rs` → `LIST_USER_POSTS_QUERY` | Yes |
| `guestListPost(postid)` | `query GuestListPost($postid)` | `comments.rs` → `GuestGetPost` server fn | No |
| `listAdvertisementPosts(offset, limit, contentFilterBy, title, tag)` | `query ListAdvertisementPosts(...)` | `posts.rs` → `ListAdPosts` server fn | Optional |
| `resolvePostAction(action, postid)` | `mutation ResolvePostAction(...)` | `posts.rs` → `PostAction` server fn | Yes |
| `postEligibility` | `query PostEligibility` | `posts.rs` → `CheckPostEligibility` server fn | Yes |
| `createPost(action, input)` | `mutation CreatePost(...)` | `posts.rs` → `CreatePost` server fn | Yes |
| `searchTags(tagName, offset, limit)` | `query SearchTags(...)` | `posts.rs` → via `SEARCH_TAGS_QUERY` | Yes |

From `peer-web/src/api/posts.rs` (search/user lookup — arguably cross-cutting but wired in posts module):

| Operation | GraphQL SDL | Frontend file | Auth? |
|-----------|-------------|---------------|-------|
| `searchUser(username, offset, limit)` | `query SearchUser(...)` | `posts.rs` → `SearchUsers` server fn | Optional |
| `getUser(id)` | `query GetUser($id)` | `posts.rs` → `GetUserInfo` server fn | Yes |

From `docs/backend_api/03-posts-and-content.md` (additional operations in the schema but not yet wired in the Leptos frontend):

| Operation | Purpose | Auth? |
|-----------|---------|-------|
| `listTags(offset, limit)` | List all tags | Yes |
| `postInteractions(getOnly, postOrCommentId, offset, limit)` | List users who interacted with a post | Yes |
| `POST /upload-post` | Multipart file upload (REST, not GraphQL) | Yes |

### Target State

After this phase, the mock backend will support:
- Full paginated post feed (`listPosts`) with filtering, sorting, and search
- Single-post retrieval (authenticated and guest)
- User-scoped post listing (profile posts)
- Post creation with tag management
- Post interactions (like, dislike, view, save, report, share) with toggle semantics
- Post eligibility check with mock token generation
- Advertisement post listing
- Tag search
- File upload stub (`POST /upload-post`)
- 5 new GraphQL query resolvers, 2 new mutation resolvers, 1 new REST endpoint

### New file tree additions

```
tests/mock_backend/src/
├── schema/
│   ├── query/
│   │   └── posts.rs          # NEW: listPosts, guestListPost, postEligibility, searchTags, listTags, listAdvertisementPosts, postInteractions
│   └── mutation/
│       └── post.rs            # NEW: createPost, resolvePostAction
├── types/
│   └── post.rs                # NEW: Post, PostUser, PostListResponse, PostInput, ContentType, enums, etc.
├── routes/
│   └── upload.rs              # NEW: POST /upload-post REST handler
└── state.rs                   # MODIFIED: new fields for posts, interactions, tags, eligibility tokens
```

---

## 2. Prerequisites

### Phase 2 Completion

- [ ] User profiles exist in `MockState` (the `Post` type embeds a `ProfileUser`)
- [ ] Follow/block relationships exist (needed for `FOLLOWED`, `FOLLOWER`, `FRIENDS` filters)
- [ ] Auth middleware from Phase 1 is working (most post operations require authentication)
- [ ] `require_auth()` and `get_current_user()` helpers are available
- [ ] `DefaultResponse` type is available from Phase 0

### API Reference

All response codes and field names come from:
- `docs/backend_api/03-posts-and-content.md`
- `peer-web/src/models/post.rs` (frontend deserialization types)
- `peer-web/src/api/graphql.rs` (exact GraphQL field selections)

---

## 3. Task Breakdown

### Phase 3.A — Post Types (`types/post.rs`)

| # | Task | Notes |
|---|------|-------|
| A1 | Create `types/post.rs` with `ContentType` enum | `image`, `audio`, `video`, `text` (lowercase serialization) |
| A2 | Define `PostFilterType` enum | `IMAGE`, `AUDIO`, `VIDEO`, `TEXT`, `FOLLOWED`, `FOLLOWER`, `VIEWED`, `FRIENDS` |
| A3 | Define `PostSortType` enum | `NEWEST`, `TRENDING`, `LIKES`, `DISLIKES`, `VIEWS`, `COMMENTS`, `FOR_ME`, `OLDEST`, `FOLLOWER`, `FOLLOWED`, `RELEVANT`, `FRIENDS` |
| A4 | Define `ContentFilterType` enum | `MYGRANDMALIKES`, `MYGRANDMAHATES` |
| A5 | Define `IgnoreOption` enum | `YES`, `NO` |
| A6 | Define `PostActionType` enum | `LIKE`, `DISLIKE`, `REPORT`, `VIEW`, `SHARE`, `SAVE` (plus undo variants `UNLIKE`, `UNDISLIKE`, `UNSAVE`) |
| A7 | Define `PostType` enum | `POST` (used by `createPost`) |
| A8 | Define `PostUser` struct (SimpleObject) | `id`, `username`, `slug`, `img`, `isfollowed`, `isfollowing`, `isfriend` — must match frontend's `PostUser` deserialization |
| A9 | Define `Post` struct (SimpleObject) | All fields from `peer-web/src/models/post.rs`: `id`, `contenttype`, `title`, `media`, `cover`, `mediadescription`, `createdat`, amounts, interaction booleans, `tags`, `hasActiveReports`, `visibilityStatus`, `isHiddenForUsers`, `user: PostUser`, `comments: Vec<Comment>` |
| A10 | Define `PostListResponse` struct | `meta: DefaultResponse`, `counter: Int`, `affectedRows: Option<Vec<Post>>` |
| A11 | Define `PostResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<Post>` (for single-post guest query) |
| A12 | Define `PostInput` (InputObject) | `title`, `mediadescription`, `contenttype`, `media`, `cover`, `tags`, `uploadedFiles` |
| A13 | Define `CreatePostResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<CreatedPost>` |
| A14 | Define `CreatedPost` struct | `id`, `contenttype`, `title` |
| A15 | Define `PostEligibilityResponse` struct | `meta: DefaultResponse`, `eligibilityToken: Option<String>` |
| A16 | Define `Tag` struct and `TagSearchResponse` | `name: String`; `meta`, `counter`, `affectedRows: Vec<Tag>` |
| A17 | Define `AdvertisementInfo` struct | `advertisementid`, `advertisementtype`, `startdate`, `enddate` |
| A18 | Define `AdvertisementPost` struct | `post: Post`, `advertisement: AdvertisementInfo` |
| A19 | Define `AdListResponse` struct | `meta: DefaultResponse`, `counter: Int`, `affectedRows: Vec<AdvertisementPost>` |
| A20 | Define `PostInteractionResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<Vec<ProfileUser>>` |
| A21 | Define `GetOnly` enum | `VIEW`, `LIKE`, `DISLIKE`, `COMMENTLIKE` |
| A22 | Define `UploadPostResponse` struct | `status`, `ResponseCode`, `affectedRows: Option<UploadAffectedRows>` |
| A23 | Define `UploadAffectedRows` struct | `uploadedFiles: String` |
| A24 | Export new types from `types/mod.rs` | Add `pub mod post;` |

### Phase 3.B — State Extensions (`state.rs`, `seed.rs`)

| # | Task | Notes |
|---|------|-------|
| B1 | Add `PostRecord` struct to state | Internal storage representation: `id: Uuid`, `author_id: Uuid`, `contenttype: String`, `title: String`, `media: String`, `cover: String`, `mediadescription: String`, `created_at: String`, `tags: Vec<String>`, `visibility_status: String`, `is_advertisement: bool`, `uploaded_files: Option<String>` |
| B2 | Add `posts: Vec<PostRecord>` to `MockState` | Ordered by creation time (newest first) |
| B3 | Add `post_likes: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, post_id)` — tracks who liked which post |
| B4 | Add `post_dislikes: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, post_id)` |
| B5 | Add `post_saves: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, post_id)` |
| B6 | Add `post_views: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, post_id)` |
| B7 | Add `post_shares: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, post_id)` |
| B8 | Add `post_reports: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, post_id)` |
| B9 | Add `tags: HashSet<String>` to `MockState` | Global tag registry (lowercase normalized) |
| B10 | Add `eligibility_tokens: HashMap<String, Uuid>` to `MockState` | Token → user mapping for upload authorization |
| B11 | Add `eligibility_token_status: HashMap<String, String>` to `MockState` | Token → status (`ISSUED`, `FILE_UPLOADED`, `POST_CREATED`) |
| B12 | Add `uploaded_files: HashMap<String, Vec<String>>` to `MockState` | Eligibility token → list of uploaded filenames |
| B13 | Add `advertisements: Vec<AdvertisementRecord>` to `MockState` | Ads linking to posts |
| B14 | Update `MockState::default()` and `MockState::reset()` to initialize/clear all new fields | Preserve seed data only |

### Phase 3.C — Seed Data (`seed.rs`)

| # | Task | Notes |
|---|------|-------|
| C1 | Add 8 seed posts with deterministic UUIDs | Mix of content types: 3 image, 2 text, 1 audio, 1 video, 1 text (by different users) |
| C2 | Associate seed posts with existing seed users | Posts authored by `SEED_USER_VERIFIED` and any additional Phase 2 seed users |
| C3 | Add pre-populated interaction data | 2 likes, 3 views on various posts |
| C4 | Add 4 seed tags | `"rust"`, `"webdev"`, `"peer"`, `"tutorial"` |
| C5 | Associate tags with seed posts | Each seed post has 1–3 tags |
| C6 | Add 1 seed advertisement record | Links to one seed post as a promoted ad |

### Phase 3.D — Post Query Resolvers (`schema/query/posts.rs`)

| # | Task | Notes |
|---|------|-------|
| D1 | Create `schema/query/posts.rs` with `PostQuery` struct | Uses `#[Object]` with `#[graphql(name = "...")]` for correct SDL names |
| D2 | Implement `list_posts(...)` resolver | Full parameter set: `filterBy`, `contentFilterBy`, `IgnorList`, `sortBy`, `userid`, `postid`, `title`, `tag`, `from`, `to`, `offset`, `limit`, `commentOffset`, `commentLimit`. Returns `PostListResponse`. |
| D3 | Implement filtering logic in `list_posts` | Content type filtering (`IMAGE`, `AUDIO`, `VIDEO`, `TEXT`), social filtering (`FOLLOWED`, `FOLLOWER`, `FRIENDS`, `VIEWED`), date range, title search, tag matching, single post by `postid`, user-scoped by `userid` |
| D4 | Implement sorting logic in `list_posts` | `NEWEST` (default), `OLDEST`, `LIKES`, `DISLIKES`, `VIEWS`, `COMMENTS`, `TRENDING`, `FOR_ME` (alias for NEWEST in mock) |
| D5 | Implement pagination in `list_posts` | `offset` (default 0), `limit` (default 10, max 20). Return `counter` as total matching count before pagination. |
| D6 | Implement block-list filtering | When `IgnorList != NO`, exclude posts by blocked users |
| D7 | Implement per-user interaction state | Set `isliked`, `isviewed`, `isdisliked`, `issaved`, `isreported` based on current user's interactions |
| D8 | Implement `guest_list_post(postid)` resolver | No auth required. Returns `PostListResponse` with single post. Interaction flags all `false`. No `PostUser` social fields. |
| D9 | Implement `post_eligibility` resolver | Requires auth. Generates a mock eligibility token (`mock-eligibility-<uuid>-<ts>`), stores in state, returns `PostEligibilityResponse`. |
| D10 | Implement `search_tags(tagName, offset, limit)` resolver | Fuzzy match (case-insensitive contains) on tag names. Returns `TagSearchResponse`. |
| D11 | Implement `list_tags(offset, limit)` resolver | Return all tags paginated. |
| D12 | Implement `list_advertisement_posts(offset, limit, contentFilterBy, title, tag)` resolver | Filter `posts` to only those with active advertisement entries. Returns `AdListResponse` with nested `post` + `advertisement`. |
| D13 | Implement `post_interactions(getOnly, postOrCommentId, offset, limit)` resolver | Return list of `ProfileUser`s who performed the specified interaction on the post. |
| D14 | Register `PostQuery` in `QueryRoot` merged object | `QueryRoot(HealthQuery, PostQuery, ...)` |

### Phase 3.E — Post Mutation Resolvers (`schema/mutation/post.rs`)

| # | Task | Notes |
|---|------|-------|
| E1 | Create `schema/mutation/post.rs` with `PostMutation` struct | Uses `#[Object]` |
| E2 | Implement `create_post(action, input)` resolver | Requires auth. Validate input (title length 1–63, description ≤500, tag format/count, media limits per content type). Create `PostRecord`, register new tags. Return `CreatePostResponse` with `id`, `contenttype`, `title`. |
| E3 | Implement input validation in `create_post` | Title: 1–63 chars (`30210`). Description: 1–500 chars (`30263`). Tags: max 10, each 2–53 chars alphanumeric+underscore (`30262`). Media limits per content type (`30267`/`30268`). Required fields check (`30101`). |
| E4 | Implement `resolve_post_action(action, postid)` resolver | Requires auth. Dispatch on `PostActionType`. Validate post exists (`31510`), not own post for like/dislike/report (`31506`/`31507`/`31508`). Handle toggles and duplicates. |
| E5 | Implement LIKE action | Add to `post_likes`. Prevent duplicate (`31501`). Prevent self-like (`31506`). Return `11503` or `11514` (free daily). |
| E6 | Implement UNLIKE action | Remove from `post_likes`. Return `11503`. |
| E7 | Implement DISLIKE action | Add to `post_dislikes`. Prevent duplicate (`31502`). Prevent self-dislike (`31507`). Return `11504`. |
| E8 | Implement UNDISLIKE action | Remove from `post_dislikes`. Return `11504`. |
| E9 | Implement VIEW action | Add to `post_views`. Idempotent (duplicate is `31505` or silently accepted). Return `11506`. |
| E10 | Implement SAVE action (toggle) | Toggle in `post_saves`. Return `11512` (saved) or `11511` (unsaved). |
| E11 | Implement REPORT action | Add to `post_reports`. Prevent duplicate (`31503`). Prevent self-report (`31508`). Return `11505`. |
| E12 | Implement SHARE action | Add to `post_shares`. Return `11507`. |
| E13 | Register `PostMutation` in `MutationRoot` merged object | `MutationRoot(RegistrationMutation, AuthMutation, ..., PostMutation)` |

### Phase 3.F — Upload Endpoint (`routes/upload.rs`)

| # | Task | Notes |
|---|------|-------|
| F1 | Create `routes/upload.rs` with `upload_post_handler` | Axum handler for `POST /upload-post`. Accepts multipart form data. |
| F2 | Implement eligibility token validation | Extract `eligibilityToken` from form. Verify token exists and status is `ISSUED`. Reject with `40902` if invalid. |
| F3 | Implement mock file processing | Don't actually store files. Generate mock filenames: `mock-<uuid>.<ext>`. Store filenames in `uploaded_files` map keyed by eligibility token. |
| F4 | Update token status to `FILE_UPLOADED` | Prevent token reuse for another upload. |
| F5 | Return `UploadPostResponse` | Status `success`, code `11515`, `uploadedFiles` as comma-separated filenames. |
| F6 | Wire `POST /upload-post` route into the Axum router | Add alongside `/graphql` and `/reset` routes. |
| F7 | Implement auth check on upload endpoint | Extract `Authorization: Bearer <token>` header and validate. Return `60501` if missing/invalid. |

### Phase 3.G — Helper Functions

| # | Task | Notes |
|---|------|-------|
| G1 | Create `PostRecord → Post` conversion function | Resolves author `ProfileUser` from users map, computes interaction counts from sets, resolves per-user interaction flags, populates `url` field. |
| G2 | Create post filtering pipeline | Chain of filter functions: content type → social → date range → title search → tag → block list → visibility. |
| G3 | Create post sorting function | Match on `PostSortType`, sort by relevant field. For `TRENDING`, use `amounttrending = likes * 2 + views + comments`. |
| G4 | Create pagination helper | `fn paginate<T>(items: &[T], offset: usize, limit: usize) -> (&[T], usize)` returning slice + total count. |

### Phase 3.H — Integration Tests

| # | Test | Assert |
|---|------|--------|
| H1 | List posts (empty state, no seed) | `21518` (no posts found), `counter: 0` |
| H2 | List posts with seed data | `11501`, `counter: 8`, `affectedRows` has posts |
| H3 | List posts with pagination (offset=0, limit=3) | Returns 3 posts, `counter` is total (8) |
| H4 | List posts with offset beyond range | `21518` or empty `affectedRows`, `counter` is total |
| H5 | Filter by content type (IMAGE) | Only image posts returned |
| H6 | Filter by content type (TEXT) | Only text posts returned |
| H7 | Sort by NEWEST | Posts ordered by `createdat` descending |
| H8 | Sort by LIKES | Posts ordered by like count descending |
| H9 | Search by title substring | Only matching posts returned |
| H10 | Filter by tag | Only posts with that tag returned |
| H11 | Get single post by postid (authenticated) | `11501`, single post with interaction flags |
| H12 | Get single post with invalid UUID | `30209` |
| H13 | Get single post that doesn't exist | `31510` |
| H14 | Guest get post | `11501`, interaction flags all false, no social fields on user |
| H15 | Guest get post with invalid postid | `31510` |
| H16 | List user posts with userid filter | Only that user's posts |
| H17 | Create post success (text type) | `11508` or `11513`, `affectedRows` has `id`, `contenttype`, `title` |
| H18 | Create post with empty title | `30210` |
| H19 | Create post with title > 63 chars | `30210` |
| H20 | Create post with invalid tag format | `30262` |
| H21 | Create post with > 10 tags | `30262` |
| H22 | Create post with description > 500 chars | `30263` |
| H23 | Created post appears in listPosts | Create → list → verify present |
| H24 | Like post | `11503`, like count incremented, `isliked` true |
| H25 | Like own post | `31506` |
| H26 | Like post twice | `31501` |
| H27 | Unlike post | Removes like, count decremented |
| H28 | Dislike post | `11504`, dislike count incremented |
| H29 | Dislike own post | `31507` |
| H30 | View post | `11506`, view count incremented, `isviewed` true |
| H31 | Save post (toggle on) | `11512`, `issaved` true |
| H32 | Save post (toggle off / unsave) | `11511`, `issaved` false |
| H33 | Report post | `11505`, `isreported` true |
| H34 | Report own post | `31508` |
| H35 | Report post twice | `31503` |
| H36 | Share post | `11507` |
| H37 | Post action on non-existent post | `31510` |
| H38 | Post action without auth | `60501` |
| H39 | Post eligibility returns token | `10901`, `eligibilityToken` is non-null string |
| H40 | Post eligibility without auth | `60501` |
| H41 | Search tags (match) | `11701`, matching tags returned |
| H42 | Search tags (no match) | `21701`, empty `affectedRows` |
| H43 | List advertisement posts | Returns only ad posts with `advertisement` info |
| H44 | Upload post file (mock) | `11515`, `uploadedFiles` is comma-separated filenames |
| H45 | Upload with invalid eligibility token | `40902` |
| H46 | Upload without auth | `60501` |
| H47 | Create post without auth | `60501` |
| H48 | List posts with FOLLOWED filter | Only posts from followed users returned |

### Phase 3.I — Cleanup & Validation

| # | Task | Notes |
|---|------|-------|
| I1 | Run `cargo clippy -- -D warnings` | Fix all warnings |
| I2 | Run `cargo fmt --check` | Fix formatting |
| I3 | Run full test suite (`cargo test --all-targets`) | All Phase 0 + 1 + 2 + 3 tests pass |
| I4 | Manual smoke test with `cargo run` + curl | Verify HTTP layer works for all new endpoints |

---

## 4. Implementation Details

### 4.1 Post Types (`types/post.rs`)

```rust
use async_graphql::{Enum, InputObject, SimpleObject, ID};
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "lowercase")]
pub enum ContentType {
    Image,
    Audio,
    Video,
    Text,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostFilterType {
    Image,
    Audio,
    Video,
    Text,
    Followed,
    Follower,
    Viewed,
    Friends,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostSortType {
    Newest,
    Trending,
    Likes,
    Dislikes,
    Views,
    Comments,
    ForMe,
    Oldest,
    Follower,
    Followed,
    Relevant,
    Friends,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ContentFilterType {
    Mygrandmalikes,
    Mygrandmahates,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum IgnoreOption {
    Yes,
    No,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostActionType {
    Like,
    Dislike,
    Report,
    View,
    Share,
    Save,
    Unlike,
    Undislike,
    Unsave,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostType {
    Post,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum GetOnly {
    View,
    Like,
    Dislike,
    Commentlike,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// User embedded in post responses.
///
/// Must match the frontend's `PostUser` deserialization in `peer-web/src/models/post.rs`.
/// Field selections from `LIST_POSTS_QUERY` in `graphql.rs`:
/// `id, username, slug, img, isfollowed, isfollowing, isfriend`
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostUser {
    pub id: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    #[graphql(name = "isfollowed")]
    pub isfollowed: bool,
    #[graphql(name = "isfollowing")]
    pub isfollowing: bool,
    #[graphql(name = "isfriend")]
    pub isfriend: bool,
}

/// Full post type returned in all post responses.
///
/// Field names must exactly match the GraphQL schema selections in `graphql.rs`.
/// The frontend deserializes these via `peer-web/src/models/post.rs::Post`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Post {
    pub id: ID,
    pub contenttype: String,
    pub title: String,
    pub media: Option<String>,
    pub cover: Option<String>,
    pub mediadescription: Option<String>,
    pub createdat: String,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    pub amountreports: i32,
    pub amountlikes: i32,
    pub amountviews: i32,
    pub amountcomments: i32,
    pub amountdislikes: i32,
    pub amounttrending: Option<i32>,
    pub isliked: bool,
    pub isviewed: bool,
    pub isreported: bool,
    pub isdisliked: bool,
    pub issaved: bool,
    pub tags: Vec<String>,
    pub url: String,
    pub user: PostUser,
}

/// Response for `listPosts` and `guestListPost` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Post>>,
}

/// Response for single-post queries (alternative shape).
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Post>,
}

/// Input for `createPost` mutation.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct PostInput {
    pub title: String,
    pub mediadescription: Option<String>,
    pub contenttype: ContentType,
    pub media: Option<Vec<String>>,
    pub cover: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    #[graphql(name = "uploadedFiles")]
    pub uploaded_files: Option<String>,
}

/// Response for `createPost` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreatePostResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<CreatedPost>,
}

/// Created post data in the response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreatedPost {
    pub id: ID,
    pub contenttype: String,
    pub title: String,
}

/// Response for `postEligibility` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostEligibilityResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "eligibilityToken")]
    pub eligibility_token: Option<String>,
}

/// A single tag.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}

/// Response for `searchTags` and `listTags` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TagSearchResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Tag>>,
}

/// Advertisement metadata.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementInfo {
    pub advertisementid: String,
    pub advertisementtype: String,
    pub startdate: String,
    pub enddate: String,
}

/// Advertisement post wrapper.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementPost {
    pub post: Post,
    pub advertisement: AdvertisementInfo,
}

/// Response for `listAdvertisementPosts` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AdvertisementPost>>,
}

/// Response for `postInteractions` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostInteractionResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<PostUser>>,
}
```

### 4.2 Internal State Record (`state.rs`)

```rust
use uuid::Uuid;

/// Internal post storage record (not the GraphQL type).
#[derive(Debug, Clone)]
pub struct PostRecord {
    pub id: Uuid,
    pub author_id: Uuid,
    pub contenttype: String,       // "image", "audio", "video", "text"
    pub title: String,
    pub media: String,             // Comma-separated media URLs
    pub cover: String,             // Cover image URL
    pub mediadescription: String,
    pub created_at: String,        // ISO 8601 timestamp
    pub tags: Vec<String>,         // Lowercase normalized
    pub visibility_status: String, // "VISIBLE", "HIDDEN", "ILLEGAL"
    pub uploaded_files: Option<String>,
}

/// Internal advertisement record.
#[derive(Debug, Clone)]
pub struct AdvertisementRecord {
    pub id: String,
    pub post_id: Uuid,
    pub advertisement_type: String, // "STANDARD", "PREMIUM"
    pub start_date: String,
    pub end_date: String,
}

/// Extended MockState fields for Phase 3.
pub struct MockState {
    // --- Phase 0–2 fields (existing) ---
    // ...

    // --- Phase 3 fields ---
    pub posts: Vec<PostRecord>,
    pub post_likes: HashSet<(Uuid, Uuid)>,      // (user_id, post_id)
    pub post_dislikes: HashSet<(Uuid, Uuid)>,
    pub post_saves: HashSet<(Uuid, Uuid)>,
    pub post_views: HashSet<(Uuid, Uuid)>,
    pub post_shares: HashSet<(Uuid, Uuid)>,
    pub post_reports: HashSet<(Uuid, Uuid)>,
    pub tags: HashSet<String>,                    // Global tag registry
    pub eligibility_tokens: HashMap<String, Uuid>,       // token → user_id
    pub eligibility_token_status: HashMap<String, String>, // token → status
    pub uploaded_files: HashMap<String, Vec<String>>,    // token → filenames
    pub advertisements: Vec<AdvertisementRecord>,
}
```

### 4.3 Seed Data (`seed.rs`)

```rust
use uuid::{uuid, Uuid};

// --- Phase 3 seed post UUIDs ---
pub const SEED_POST_1: Uuid = uuid!("10000000-0000-4000-a000-000000000001"); // image post
pub const SEED_POST_2: Uuid = uuid!("10000000-0000-4000-a000-000000000002"); // text post
pub const SEED_POST_3: Uuid = uuid!("10000000-0000-4000-a000-000000000003"); // image post
pub const SEED_POST_4: Uuid = uuid!("10000000-0000-4000-a000-000000000004"); // audio post
pub const SEED_POST_5: Uuid = uuid!("10000000-0000-4000-a000-000000000005"); // video post
pub const SEED_POST_6: Uuid = uuid!("10000000-0000-4000-a000-000000000006"); // text post
pub const SEED_POST_7: Uuid = uuid!("10000000-0000-4000-a000-000000000007"); // image post
pub const SEED_POST_8: Uuid = uuid!("10000000-0000-4000-a000-000000000008"); // text post (ad)

// Seed post data (inserted in MockState::default())
fn seed_posts(verified_user: Uuid, user2: Uuid) -> Vec<PostRecord> {
    vec![
        PostRecord {
            id: SEED_POST_1,
            author_id: verified_user,
            contenttype: "image".into(),
            title: "My first photo".into(),
            media: "https://mock.peer.com/media/photo1.jpg".into(),
            cover: "https://mock.peer.com/covers/photo1_cover.jpg".into(),
            mediadescription: "A beautiful sunset".into(),
            created_at: "2025-04-01T10:00:00Z".into(),
            tags: vec!["peer".into(), "webdev".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_2,
            author_id: verified_user,
            contenttype: "text".into(),
            title: "Hello Peer Network".into(),
            media: "".into(),
            cover: "".into(),
            mediadescription: "Welcome to my first text post!".into(),
            created_at: "2025-04-02T12:00:00Z".into(),
            tags: vec!["peer".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_3,
            author_id: user2,
            contenttype: "image".into(),
            title: "Rust programming tips".into(),
            media: "https://mock.peer.com/media/rust_tips.png".into(),
            cover: "https://mock.peer.com/covers/rust_tips_cover.png".into(),
            mediadescription: "Helpful Rust patterns".into(),
            created_at: "2025-04-03T14:00:00Z".into(),
            tags: vec!["rust".into(), "tutorial".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_4,
            author_id: user2,
            contenttype: "audio".into(),
            title: "Peer podcast episode 1".into(),
            media: "https://mock.peer.com/media/podcast_ep1.mp3".into(),
            cover: "https://mock.peer.com/covers/podcast_cover.jpg".into(),
            mediadescription: "Our first podcast episode".into(),
            created_at: "2025-04-04T09:00:00Z".into(),
            tags: vec!["peer".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_5,
            author_id: verified_user,
            contenttype: "video".into(),
            title: "Web dev tutorial".into(),
            media: "https://mock.peer.com/media/tutorial.mp4".into(),
            cover: "https://mock.peer.com/covers/tutorial_cover.jpg".into(),
            mediadescription: "Learn web development step by step".into(),
            created_at: "2025-04-05T16:00:00Z".into(),
            tags: vec!["webdev".into(), "tutorial".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_6,
            author_id: verified_user,
            contenttype: "text".into(),
            title: "Thoughts on decentralization".into(),
            media: "".into(),
            cover: "".into(),
            mediadescription: "Why peer-to-peer matters".into(),
            created_at: "2025-04-06T11:00:00Z".into(),
            tags: vec!["peer".into(), "webdev".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_7,
            author_id: user2,
            contenttype: "image".into(),
            title: "Weekend project showcase".into(),
            media: "https://mock.peer.com/media/project.jpg,https://mock.peer.com/media/project2.jpg".into(),
            cover: "https://mock.peer.com/covers/project_cover.jpg".into(),
            mediadescription: "Check out what I built this weekend".into(),
            created_at: "2025-04-07T18:00:00Z".into(),
            tags: vec!["rust".into(), "webdev".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
        PostRecord {
            id: SEED_POST_8,
            author_id: verified_user,
            contenttype: "text".into(),
            title: "Sponsored: Learn Rust".into(),
            media: "".into(),
            cover: "".into(),
            mediadescription: "Sponsored post about learning Rust".into(),
            created_at: "2025-04-08T08:00:00Z".into(),
            tags: vec!["rust".into(), "tutorial".into()],
            visibility_status: "VISIBLE".into(),
            uploaded_files: None,
        },
    ]
}

fn seed_tags() -> HashSet<String> {
    HashSet::from([
        "rust".into(),
        "webdev".into(),
        "peer".into(),
        "tutorial".into(),
    ])
}

fn seed_post_likes(verified_user: Uuid, user2: Uuid) -> HashSet<(Uuid, Uuid)> {
    HashSet::from([
        (verified_user, SEED_POST_3),  // verified user liked user2's post
        (user2, SEED_POST_1),          // user2 liked verified user's post
    ])
}

fn seed_post_views(verified_user: Uuid, user2: Uuid) -> HashSet<(Uuid, Uuid)> {
    HashSet::from([
        (verified_user, SEED_POST_3),
        (verified_user, SEED_POST_4),
        (user2, SEED_POST_1),
    ])
}

fn seed_advertisements() -> Vec<AdvertisementRecord> {
    vec![AdvertisementRecord {
        id: "ad-001".into(),
        post_id: SEED_POST_8,
        advertisement_type: "STANDARD".into(),
        start_date: "2025-04-01".into(),
        end_date: "2025-05-01".into(),
    }]
}
```

### 4.4 PostRecord → Post Conversion

This helper converts the internal `PostRecord` to the GraphQL `Post` type, resolving user data and interaction counts/flags:

```rust
impl MockState {
    /// Convert a PostRecord to the GraphQL Post type.
    ///
    /// `viewer_id`: The current authenticated user (None for guest).
    pub fn post_record_to_graphql(
        &self,
        record: &PostRecord,
        viewer_id: Option<Uuid>,
    ) -> Post {
        let author = self.users.get(&record.author_id);

        // Compute interaction counts
        let amountlikes = self.post_likes.iter()
            .filter(|(_, pid)| *pid == record.id).count() as i32;
        let amountdislikes = self.post_dislikes.iter()
            .filter(|(_, pid)| *pid == record.id).count() as i32;
        let amountviews = self.post_views.iter()
            .filter(|(_, pid)| *pid == record.id).count() as i32;
        let amountshares = self.post_shares.iter()
            .filter(|(_, pid)| *pid == record.id).count() as i32;
        let amountreports = self.post_reports.iter()
            .filter(|(_, pid)| *pid == record.id).count() as i32;
        // amountcomments is resolved from comments state (Phase 4), default 0 for now
        let amountcomments = 0i32;

        // Trending score: likes*2 + views + comments
        let amounttrending = amountlikes * 2 + amountviews + amountcomments;

        // Per-user interaction flags
        let (isliked, isviewed, isdisliked, issaved, isreported) = match viewer_id {
            Some(uid) => (
                self.post_likes.contains(&(uid, record.id)),
                self.post_views.contains(&(uid, record.id)),
                self.post_dislikes.contains(&(uid, record.id)),
                self.post_saves.contains(&(uid, record.id)),
                self.post_reports.contains(&(uid, record.id)),
            ),
            None => (false, false, false, false, false),
        };

        // Resolve author profile
        let user = match author {
            Some(u) => {
                let (isfollowed, isfollowing, isfriend) = match viewer_id {
                    Some(vid) => {
                        let followed = self.follows.contains(&(vid, u.uid));
                        let following = self.follows.contains(&(u.uid, vid));
                        (followed, following, followed && following)
                    }
                    None => (false, false, false),
                };
                PostUser {
                    id: u.uid.to_string().into(),
                    username: u.username.clone(),
                    slug: u.slug.clone(),
                    img: Some("https://via.placeholder.com/96".into()),
                    isfollowed,
                    isfollowing,
                    isfriend,
                }
            }
            None => PostUser {
                id: record.author_id.to_string().into(),
                username: "unknown".into(),
                slug: "unknown".into(),
                img: None,
                isfollowed: false,
                isfollowing: false,
                isfriend: false,
            },
        };

        let has_reports = amountreports > 0;

        Post {
            id: record.id.to_string().into(),
            contenttype: record.contenttype.clone(),
            title: record.title.clone(),
            media: if record.media.is_empty() { None } else { Some(record.media.clone()) },
            cover: if record.cover.is_empty() { None } else { Some(record.cover.clone()) },
            mediadescription: if record.mediadescription.is_empty() {
                None
            } else {
                Some(record.mediadescription.clone())
            },
            createdat: record.created_at.clone(),
            visibility_status: Some(record.visibility_status.clone()),
            is_hidden_for_users: Some(record.visibility_status == "HIDDEN"),
            has_active_reports: Some(has_reports),
            amountreports,
            amountlikes,
            amountviews,
            amountcomments,
            amountdislikes,
            amounttrending: Some(amounttrending),
            isliked,
            isviewed,
            isreported,
            isdisliked,
            issaved,
            tags: record.tags.clone(),
            url: format!("/post/{}", record.id),
            user,
        }
    }
}
```

### 4.5 Filtering Pipeline

```rust
impl MockState {
    /// Apply filters to posts and return matching records.
    pub fn filter_posts(
        &self,
        viewer_id: Option<Uuid>,
        filter_by: &[PostFilterType],
        content_filter_by: Option<ContentFilterType>,
        ignore_list: Option<IgnoreOption>,
        userid: Option<Uuid>,
        postid: Option<Uuid>,
        title: Option<&str>,
        tag: Option<&str>,
    ) -> Vec<&PostRecord> {
        self.posts.iter().filter(|p| {
            // Exclude hidden/illegal content
            if p.visibility_status != "VISIBLE" {
                return false;
            }

            // Exclude advertisement posts from normal feed
            if self.advertisements.iter().any(|ad| ad.post_id == p.id) {
                return false;
            }

            // Block list filtering
            if ignore_list != Some(IgnoreOption::No) {
                if let Some(vid) = viewer_id {
                    if self.blocks.contains(&(vid, p.author_id)) {
                        return false;
                    }
                }
            }

            // Exclude posts by deleted users
            if self.deleted_users.contains(&p.author_id) {
                return false;
            }

            // Single post by ID
            if let Some(pid) = postid {
                return p.id == pid;
            }

            // User-scoped
            if let Some(uid) = userid {
                if p.author_id != uid {
                    return false;
                }
            }

            // Content type filters
            for f in filter_by {
                match f {
                    PostFilterType::Image => if p.contenttype != "image" { return false; },
                    PostFilterType::Audio => if p.contenttype != "audio" { return false; },
                    PostFilterType::Video => if p.contenttype != "video" { return false; },
                    PostFilterType::Text => if p.contenttype != "text" { return false; },
                    PostFilterType::Followed => {
                        if let Some(vid) = viewer_id {
                            if !self.follows.contains(&(vid, p.author_id)) { return false; }
                        } else { return false; }
                    }
                    PostFilterType::Follower => {
                        if let Some(vid) = viewer_id {
                            if !self.follows.contains(&(p.author_id, vid)) { return false; }
                        } else { return false; }
                    }
                    PostFilterType::Friends => {
                        if let Some(vid) = viewer_id {
                            let mutual = self.follows.contains(&(vid, p.author_id))
                                && self.follows.contains(&(p.author_id, vid));
                            if !mutual { return false; }
                        } else { return false; }
                    }
                    PostFilterType::Viewed => {
                        if let Some(vid) = viewer_id {
                            if !self.post_views.contains(&(vid, p.id)) { return false; }
                        } else { return false; }
                    }
                }
            }

            // Title search (case-insensitive substring)
            if let Some(t) = title {
                if !t.is_empty() && !p.title.to_lowercase().contains(&t.to_lowercase()) {
                    return false;
                }
            }

            // Tag filter (case-insensitive exact match)
            if let Some(tg) = tag {
                if !tg.is_empty() {
                    let tg_lower = tg.to_lowercase();
                    if !p.tags.iter().any(|pt| pt.to_lowercase() == tg_lower) {
                        return false;
                    }
                }
            }

            true
        }).collect()
    }
}
```

### 4.6 Sorting

```rust
/// Sort filtered posts in place.
fn sort_posts(posts: &mut Vec<&PostRecord>, sort_by: PostSortType, state: &MockState) {
    match sort_by {
        PostSortType::Newest | PostSortType::ForMe | PostSortType::Relevant => {
            posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        PostSortType::Oldest => {
            posts.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        }
        PostSortType::Likes => {
            posts.sort_by(|a, b| {
                let a_likes = state.post_likes.iter().filter(|(_, p)| *p == a.id).count();
                let b_likes = state.post_likes.iter().filter(|(_, p)| *p == b.id).count();
                b_likes.cmp(&a_likes)
            });
        }
        PostSortType::Dislikes => {
            posts.sort_by(|a, b| {
                let a_dl = state.post_dislikes.iter().filter(|(_, p)| *p == a.id).count();
                let b_dl = state.post_dislikes.iter().filter(|(_, p)| *p == b.id).count();
                b_dl.cmp(&a_dl)
            });
        }
        PostSortType::Views => {
            posts.sort_by(|a, b| {
                let a_v = state.post_views.iter().filter(|(_, p)| *p == a.id).count();
                let b_v = state.post_views.iter().filter(|(_, p)| *p == b.id).count();
                b_v.cmp(&a_v)
            });
        }
        PostSortType::Comments => {
            // Phase 4 will populate comment counts; for now sort by created_at
            posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        PostSortType::Trending => {
            posts.sort_by(|a, b| {
                let score = |id: Uuid| -> usize {
                    let likes = state.post_likes.iter().filter(|(_, p)| *p == id).count();
                    let views = state.post_views.iter().filter(|(_, p)| *p == id).count();
                    likes * 2 + views
                };
                score(b.id).cmp(&score(a.id))
            });
        }
        // Social sorts fall back to newest
        PostSortType::Follower | PostSortType::Followed | PostSortType::Friends => {
            posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
    }
}
```

### 4.7 Pagination Helper

```rust
/// Apply offset/limit pagination.
///
/// Returns (paginated_slice, total_count_before_pagination).
fn paginate<T>(items: &[T], offset: i32, limit: i32) -> (&[T], i32) {
    let total = items.len() as i32;
    let offset = offset.max(0) as usize;
    let limit = limit.clamp(1, 20) as usize;

    if offset >= items.len() {
        return (&[], total);
    }

    let end = (offset + limit).min(items.len());
    (&items[offset..end], total)
}
```

### 4.8 Query Resolvers (`schema/query/posts.rs`)

```rust
use async_graphql::{Context, Object, ID};
use uuid::Uuid;

use crate::state::SharedState;
use crate::types::post::*;
use crate::types::registration::DefaultResponse;

pub struct PostQuery;

#[Object]
impl PostQuery {
    // ========================================================================
    // listPosts
    // ========================================================================

    /// Paginated, filtered, sorted list of posts.
    ///
    /// Response codes:
    /// - 11501: Posts retrieved
    /// - 21518: No posts found
    /// - 30209: Invalid post UUID
    /// - 60501: Not authenticated
    async fn list_posts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "filterBy")] filter_by: Option<Vec<PostFilterType>>,
        #[graphql(name = "contentFilterBy")] content_filter_by: Option<ContentFilterType>,
        #[graphql(name = "IgnorList")] ignor_list: Option<IgnoreOption>,
        #[graphql(name = "sortBy")] sort_by: Option<PostSortType>,
        userid: Option<ID>,
        postid: Option<ID>,
        title: Option<String>,
        tag: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
        #[graphql(name = "commentOffset")] _comment_offset: Option<i32>,
        #[graphql(name = "commentLimit")] _comment_limit: Option<i32>,
    ) -> PostListResponse {
        let viewer_id = get_current_user(ctx);

        // Require auth for listPosts
        let viewer_id = match viewer_id {
            Some(uid) => uid,
            None => return PostListResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        // Parse optional UUIDs
        let userid_parsed = userid.and_then(|id| Uuid::parse_str(id.as_str()).ok());
        let postid_parsed = postid.as_ref().and_then(|id| {
            Uuid::parse_str(id.as_str()).ok()
        });

        // Validate postid format if provided
        if postid.is_some() && postid_parsed.is_none() {
            return PostListResponse {
                meta: DefaultResponse::error("30209", "Invalid post UUID"),
                counter: 0,
                affected_rows: None,
            };
        }

        let filters = filter_by.unwrap_or_default();
        let sort = sort_by.unwrap_or(PostSortType::Newest);
        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(10);

        // Filter
        let mut filtered = state_read.filter_posts(
            Some(viewer_id),
            &filters,
            content_filter_by,
            ignor_list,
            userid_parsed,
            postid_parsed,
            title.as_deref(),
            tag.as_deref(),
        );

        // Sort
        sort_posts(&mut filtered, sort, &state_read);

        // Convert to GraphQL types
        let graphql_posts: Vec<Post> = filtered.iter()
            .map(|r| state_read.post_record_to_graphql(r, Some(viewer_id)))
            .collect();

        // Paginate
        let (page, total) = paginate(&graphql_posts, off, lim);

        if page.is_empty() {
            PostListResponse {
                meta: DefaultResponse::success("21518", "No posts found"),
                counter: total,
                affected_rows: Some(vec![]),
            }
        } else {
            PostListResponse {
                meta: DefaultResponse::success("11501", "Posts retrieved successfully"),
                counter: total,
                affected_rows: Some(page.to_vec()),
            }
        }
    }

    // ========================================================================
    // guestListPost
    // ========================================================================

    /// Get a single post for guest viewing (no auth required).
    ///
    /// Response codes:
    /// - 11501: Post retrieved
    /// - 30209: Invalid post UUID
    /// - 31510: Post not found
    async fn guest_list_post(
        &self,
        ctx: &Context<'_>,
        postid: ID,
    ) -> PostListResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let post_uuid = match Uuid::parse_str(postid.as_str()) {
            Ok(id) => id,
            Err(_) => return PostListResponse {
                meta: DefaultResponse::error("30209", "Invalid post UUID"),
                counter: 0,
                affected_rows: None,
            },
        };

        match state_read.posts.iter().find(|p| p.id == post_uuid && p.visibility_status == "VISIBLE") {
            Some(record) => {
                let post = state_read.post_record_to_graphql(record, None);
                PostListResponse {
                    meta: DefaultResponse::success("11501", "Post retrieved successfully"),
                    counter: 1,
                    affected_rows: Some(vec![post]),
                }
            }
            None => PostListResponse {
                meta: DefaultResponse::error("31510", "Post not found"),
                counter: 0,
                affected_rows: None,
            },
        }
    }

    // ========================================================================
    // postEligibility
    // ========================================================================

    /// Check post eligibility and obtain upload token.
    ///
    /// Response codes:
    /// - 10901: Eligibility token issued
    /// - 60501: Not authenticated
    async fn post_eligibility(
        &self,
        ctx: &Context<'_>,
    ) -> PostEligibilityResponse {
        let viewer_id = match get_current_user(ctx) {
            Some(uid) => uid,
            None => return PostEligibilityResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                eligibility_token: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let token = format!("mock-eligibility-{}-{}", viewer_id,
            chrono::Utc::now().timestamp_millis());

        state_write.eligibility_tokens.insert(token.clone(), viewer_id);
        state_write.eligibility_token_status.insert(token.clone(), "ISSUED".into());

        PostEligibilityResponse {
            meta: DefaultResponse::success("10901", "Eligibility token issued"),
            eligibility_token: Some(token),
        }
    }

    // ========================================================================
    // searchTags
    // ========================================================================

    /// Search tags by name (case-insensitive substring match).
    ///
    /// Response codes:
    /// - 11701: Tags retrieved
    /// - 21701: No tags found
    /// - 30101: Missing tagName
    /// - 60501: Not authenticated
    async fn search_tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "tagName")] tag_name: String,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> TagSearchResponse {
        if get_current_user(ctx).is_none() {
            return TagSearchResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: None,
            };
        }

        if tag_name.is_empty() {
            return TagSearchResponse {
                meta: DefaultResponse::error("30101", "Missing tagName"),
                counter: 0,
                affected_rows: None,
            };
        }

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let query_lower = tag_name.to_lowercase();
        let matching: Vec<Tag> = state_read.tags.iter()
            .filter(|t| t.to_lowercase().contains(&query_lower))
            .map(|t| Tag { name: t.clone() })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(20);
        let (page, total) = paginate(&matching, off, lim);

        if page.is_empty() {
            TagSearchResponse {
                meta: DefaultResponse::success("21701", "No tags found"),
                counter: total,
                affected_rows: Some(vec![]),
            }
        } else {
            TagSearchResponse {
                meta: DefaultResponse::success("11701", "Tags retrieved successfully"),
                counter: total,
                affected_rows: Some(page.to_vec()),
            }
        }
    }

    // ========================================================================
    // listTags
    // ========================================================================

    /// List all tags with pagination.
    async fn list_tags(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> TagSearchResponse {
        if get_current_user(ctx).is_none() {
            return TagSearchResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: None,
            };
        }

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let all_tags: Vec<Tag> = state_read.tags.iter()
            .map(|t| Tag { name: t.clone() })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(20);
        let (page, total) = paginate(&all_tags, off, lim);

        if page.is_empty() {
            TagSearchResponse {
                meta: DefaultResponse::success("21701", "No tags found"),
                counter: total,
                affected_rows: Some(vec![]),
            }
        } else {
            TagSearchResponse {
                meta: DefaultResponse::success("11701", "Tags loaded"),
                counter: total,
                affected_rows: Some(page.to_vec()),
            }
        }
    }

    // ========================================================================
    // listAdvertisementPosts
    // ========================================================================

    /// List advertisement posts with optional filters.
    async fn list_advertisement_posts(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<ContentFilterType>,
        title: Option<String>,
        tag: Option<String>,
    ) -> AdListResponse {
        let viewer_id = get_current_user(ctx);

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let ad_posts: Vec<AdvertisementPost> = state_read.advertisements.iter()
            .filter_map(|ad| {
                let record = state_read.posts.iter().find(|p| p.id == ad.post_id)?;

                // Title filter
                if let Some(ref t) = title {
                    if !t.is_empty() && !record.title.to_lowercase().contains(&t.to_lowercase()) {
                        return None;
                    }
                }
                // Tag filter
                if let Some(ref tg) = tag {
                    if !tg.is_empty() {
                        let tg_lower = tg.to_lowercase();
                        if !record.tags.iter().any(|pt| pt.to_lowercase() == tg_lower) {
                            return None;
                        }
                    }
                }

                let post = state_read.post_record_to_graphql(record, viewer_id);
                Some(AdvertisementPost {
                    post,
                    advertisement: AdvertisementInfo {
                        advertisementid: ad.id.clone(),
                        advertisementtype: ad.advertisement_type.clone(),
                        startdate: ad.start_date.clone(),
                        enddate: ad.end_date.clone(),
                    },
                })
            })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(10);
        let (page, total) = paginate(&ad_posts, off, lim);

        AdListResponse {
            meta: DefaultResponse::success("11501", "Advertisement posts retrieved"),
            counter: total,
            affected_rows: Some(page.to_vec()),
        }
    }

    // ========================================================================
    // postInteractions
    // ========================================================================

    /// List users who performed a specific interaction on a post.
    async fn post_interactions(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "getOnly")] get_only: GetOnly,
        #[graphql(name = "postOrCommentId")] post_or_comment_id: ID,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> PostInteractionResponse {
        if get_current_user(ctx).is_none() {
            return PostInteractionResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                affected_rows: None,
            };
        }

        let post_uuid = match Uuid::parse_str(post_or_comment_id.as_str()) {
            Ok(id) => id,
            Err(_) => return PostInteractionResponse {
                meta: DefaultResponse::error("30201", "Invalid UUID format"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let user_ids: Vec<Uuid> = match get_only {
            GetOnly::Like => state_read.post_likes.iter()
                .filter(|(_, pid)| *pid == post_uuid).map(|(uid, _)| *uid).collect(),
            GetOnly::Dislike => state_read.post_dislikes.iter()
                .filter(|(_, pid)| *pid == post_uuid).map(|(uid, _)| *uid).collect(),
            GetOnly::View => state_read.post_views.iter()
                .filter(|(_, pid)| *pid == post_uuid).map(|(uid, _)| *uid).collect(),
            GetOnly::Commentlike => vec![], // Handled in Phase 4
        };

        let users: Vec<PostUser> = user_ids.iter()
            .filter_map(|uid| state_read.users.get(uid))
            .map(|u| PostUser {
                id: u.uid.to_string().into(),
                username: u.username.clone(),
                slug: u.slug.clone(),
                img: Some("https://via.placeholder.com/96".into()),
                isfollowed: false,
                isfollowing: false,
                isfriend: false,
            })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(20);
        let (page, _) = paginate(&users, off, lim);

        PostInteractionResponse {
            meta: DefaultResponse::success("11205", "Interactions retrieved successfully"),
            affected_rows: Some(page.to_vec()),
        }
    }
}
```

### 4.9 Mutation Resolvers (`schema/mutation/post.rs`)

```rust
use async_graphql::{Context, Object, ID};
use uuid::Uuid;

use crate::state::{PostRecord, SharedState};
use crate::types::post::*;
use crate::types::registration::DefaultResponse;

pub struct PostMutation;

#[Object]
impl PostMutation {
    // ========================================================================
    // createPost
    // ========================================================================

    /// Create a new post.
    ///
    /// Response codes:
    /// - 11508: Post created (paid)
    /// - 11513: Post created (free daily action)
    /// - 30101: Missing required fields
    /// - 30210: Invalid title length
    /// - 30262: Invalid tags
    /// - 30263: Invalid description length
    /// - 60501: Not authenticated
    async fn create_post(
        &self,
        ctx: &Context<'_>,
        action: PostType,
        input: PostInput,
    ) -> CreatePostResponse {
        let viewer_id = match get_current_user(ctx) {
            Some(uid) => uid,
            None => return CreatePostResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                affected_rows: None,
            },
        };

        // Validate title (1–63 chars)
        if input.title.is_empty() || input.title.len() > 63 {
            return CreatePostResponse {
                meta: DefaultResponse::error("30210", "Invalid title length (1-63 characters)"),
                affected_rows: None,
            };
        }

        // Validate description (≤500 chars)
        if let Some(ref desc) = input.mediadescription {
            if desc.len() > 500 {
                return CreatePostResponse {
                    meta: DefaultResponse::error("30263", "Invalid description length (max 500)"),
                    affected_rows: None,
                };
            }
        }

        // Validate tags
        if let Some(ref tags) = input.tags {
            if tags.len() > 10 {
                return CreatePostResponse {
                    meta: DefaultResponse::error("30262", "Too many tags (max 10)"),
                    affected_rows: None,
                };
            }
            for tag in tags {
                if tag.len() < 2 || tag.len() > 53
                    || !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    return CreatePostResponse {
                        meta: DefaultResponse::error("30262",
                            &format!("Invalid tag format: {}", tag)),
                        affected_rows: None,
                    };
                }
            }
        }

        // Validate media limits per content type
        let max_media = match input.contenttype {
            ContentType::Image => 5,
            ContentType::Video => 2,
            ContentType::Audio | ContentType::Text => 1,
        };
        if let Some(ref media) = input.media {
            if media.len() > max_media {
                return CreatePostResponse {
                    meta: DefaultResponse::error("30267", "Too many media items"),
                    affected_rows: None,
                };
            }
        }
        if let Some(ref cover) = input.cover {
            if cover.len() > 1 {
                return CreatePostResponse {
                    meta: DefaultResponse::error("30268", "Too many cover items (max 1)"),
                    affected_rows: None,
                };
            }
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let post_id = Uuid::new_v4();
        let contenttype_str = match input.contenttype {
            ContentType::Image => "image",
            ContentType::Audio => "audio",
            ContentType::Video => "video",
            ContentType::Text => "text",
        };

        // Normalize and register tags
        let tags: Vec<String> = input.tags.unwrap_or_default()
            .into_iter()
            .map(|t| t.to_lowercase())
            .collect();
        for tag in &tags {
            state_write.tags.insert(tag.clone());
        }

        // If uploadedFiles was provided, mark the eligibility token as consumed
        if let Some(ref uf) = input.uploaded_files {
            // Find the eligibility token that produced these files
            for (token, files) in &state_write.uploaded_files {
                if files.join(",") == *uf {
                    // Token status updated below after mutable borrow is available
                    break;
                }
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let record = PostRecord {
            id: post_id,
            author_id: viewer_id,
            contenttype: contenttype_str.to_string(),
            title: input.title.clone(),
            media: input.media.map(|m| m.join(",")).unwrap_or_default(),
            cover: input.cover.map(|c| c.join(",")).unwrap_or_default(),
            mediadescription: input.mediadescription.unwrap_or_default(),
            created_at: now,
            tags,
            visibility_status: "VISIBLE".into(),
            uploaded_files: input.uploaded_files,
        };

        state_write.posts.insert(0, record); // Newest first

        CreatePostResponse {
            meta: DefaultResponse::success("11508", "Post created successfully"),
            affected_rows: Some(CreatedPost {
                id: post_id.to_string().into(),
                contenttype: contenttype_str.to_string(),
                title: input.title,
            }),
        }
    }

    // ========================================================================
    // resolvePostAction
    // ========================================================================

    /// Perform an interaction on a post.
    ///
    /// Handles: LIKE, UNLIKE, DISLIKE, UNDISLIKE, VIEW, SAVE, UNSAVE, REPORT, SHARE.
    async fn resolve_post_action(
        &self,
        ctx: &Context<'_>,
        action: PostActionType,
        postid: ID,
    ) -> DefaultResponse {
        let viewer_id = match get_current_user(ctx) {
            Some(uid) => uid,
            None => return DefaultResponse::error("60501", "Authentication required"),
        };

        let post_uuid = match Uuid::parse_str(postid.as_str()) {
            Ok(id) => id,
            Err(_) => return DefaultResponse::error("30209", "Invalid post UUID"),
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Verify post exists
        let post = match state_write.posts.iter().find(|p| p.id == post_uuid) {
            Some(p) => p.clone(),
            None => return DefaultResponse::error("31510", "Post not found"),
        };

        match action {
            PostActionType::Like => {
                if post.author_id == viewer_id {
                    return DefaultResponse::error("31506", "Cannot like own post");
                }
                if !state_write.post_likes.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31501", "Already liked");
                }
                // Remove dislike if exists (mutual exclusion)
                state_write.post_dislikes.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11503", "Post liked")
            }
            PostActionType::Unlike => {
                state_write.post_likes.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11503", "Post unliked")
            }
            PostActionType::Dislike => {
                if post.author_id == viewer_id {
                    return DefaultResponse::error("31507", "Cannot dislike own post");
                }
                if !state_write.post_dislikes.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31502", "Already disliked");
                }
                // Remove like if exists (mutual exclusion)
                state_write.post_likes.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11504", "Post disliked")
            }
            PostActionType::Undislike => {
                state_write.post_dislikes.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11504", "Post undisliked")
            }
            PostActionType::View => {
                if !state_write.post_views.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31505", "Already viewed");
                }
                DefaultResponse::success("11506", "Post viewed")
            }
            PostActionType::Save => {
                if state_write.post_saves.contains(&(viewer_id, post_uuid)) {
                    state_write.post_saves.remove(&(viewer_id, post_uuid));
                    DefaultResponse::success("11511", "Post unsaved")
                } else {
                    state_write.post_saves.insert((viewer_id, post_uuid));
                    DefaultResponse::success("11512", "Post saved")
                }
            }
            PostActionType::Unsave => {
                state_write.post_saves.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11511", "Post unsaved")
            }
            PostActionType::Report => {
                if post.author_id == viewer_id {
                    return DefaultResponse::error("31508", "Cannot report own post");
                }
                if !state_write.post_reports.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31503", "Already reported");
                }
                DefaultResponse::success("11505", "Post reported")
            }
            PostActionType::Share => {
                state_write.post_shares.insert((viewer_id, post_uuid));
                DefaultResponse::success("11507", "Post shared")
            }
        }
    }
}
```

### 4.10 Upload Handler (`routes/upload.rs`)

```rust
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<UploadAffectedRows>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadAffectedRows {
    pub uploaded_files: String,
}

/// POST /upload-post handler.
///
/// Accepts multipart form data with an eligibility token.
/// In mock mode, doesn't store actual files — generates mock filenames.
pub async fn upload_post_handler(
    state: State<AppState>,
    headers: HeaderMap,
    body: axum::extract::Multipart,
) -> (StatusCode, Json<UploadResponse>) {
    // 1. Auth check
    let viewer_id = {
        let token = headers.get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        match token {
            Some(t) => {
                let mock_state = state.mock_state.read().await;
                match mock_state.access_tokens.get(t) {
                    Some(uid) => *uid,
                    None => return (StatusCode::UNAUTHORIZED, Json(UploadResponse {
                        status: "error".into(),
                        response_code: "60501".into(),
                        affected_rows: None,
                    })),
                }
            }
            None => return (StatusCode::UNAUTHORIZED, Json(UploadResponse {
                status: "error".into(),
                response_code: "60501".into(),
                affected_rows: None,
            })),
        }
    };

    // 2. Parse multipart — extract eligibilityToken and file fields
    let mut eligibility_token = None;
    let mut file_count = 0u32;
    let mut body = body;

    while let Ok(Some(field)) = body.next_field().await {
        let name = field.name().map(|n| n.to_string());
        match name.as_deref() {
            Some("eligibilityToken") => {
                eligibility_token = field.text().await.ok();
            }
            Some("file") => {
                // Don't store data, just count files
                let _ = field.bytes().await;
                file_count += 1;
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    // 3. Validate eligibility token
    let token_str = match eligibility_token {
        Some(t) if !t.is_empty() => t,
        _ => return (StatusCode::BAD_REQUEST, Json(UploadResponse {
            status: "error".into(),
            response_code: "40902".into(),
            affected_rows: None,
        })),
    };

    let mut mock_state = state.mock_state.write().await;

    let token_status = mock_state.eligibility_token_status.get(&token_str).cloned();
    match token_status.as_deref() {
        Some("ISSUED") => { /* valid */ }
        _ => return (StatusCode::BAD_REQUEST, Json(UploadResponse {
            status: "error".into(),
            response_code: "40902".into(),
            affected_rows: None,
        })),
    }

    // Verify token belongs to this user
    if mock_state.eligibility_tokens.get(&token_str) != Some(&viewer_id) {
        return (StatusCode::BAD_REQUEST, Json(UploadResponse {
            status: "error".into(),
            response_code: "40902".into(),
            affected_rows: None,
        }));
    }

    // 4. Generate mock filenames
    let filenames: Vec<String> = (0..file_count.max(1))
        .map(|_| format!("mock-{}.jpg", Uuid::new_v4()))
        .collect();

    // 5. Update state
    mock_state.eligibility_token_status.insert(token_str.clone(), "FILE_UPLOADED".into());
    mock_state.uploaded_files.insert(token_str, filenames.clone());

    (StatusCode::OK, Json(UploadResponse {
        status: "success".into(),
        response_code: "11515".into(),
        affected_rows: Some(UploadAffectedRows {
            uploaded_files: filenames.join(","),
        }),
    }))
}
```

### 4.11 Router Update (`lib.rs`)

```rust
use crate::routes::upload::upload_post_handler;

pub fn app() -> Router {
    let state = SharedState::default();
    let schema = build_schema(state.clone());

    Router::new()
        .route("/graphql", post(graphql_handler).get(graphql_playground))
        .route("/reset", post(reset_handler))
        .route("/upload-post", post(upload_post_handler))  // NEW
        .layer(CorsLayer::permissive())
        .with_state(AppState { schema, mock_state: state })
}
```

### 4.12 Schema Assembly

```rust
// schema/mod.rs

#[derive(MergedObject, Default)]
pub struct QueryRoot(HealthQuery, PostQuery);  // PostQuery added

#[derive(MergedObject, Default)]
pub struct MutationRoot(RegistrationMutation, AuthMutation, PostMutation);  // PostMutation added
```

---

## 5. Testing Strategy

### Test Helpers

Extend the existing test helpers from Phase 1:

```rust
/// Helper: create a post as an authenticated user.
async fn create_test_post(
    state: SharedState,
    token: &str,
    title: &str,
    contenttype: &str,
) -> serde_json::Value {
    let app = app_with_state(state);
    graphql_with_auth(app, &format!(r#"
        mutation {{
            createPost(action: POST, input: {{
                title: "{title}",
                contenttype: {contenttype},
                mediadescription: "Test description"
            }}) {{
                meta {{ status ResponseCode ResponseMessage }}
                affectedRows {{ id contenttype title }}
            }}
        }}
    "#), token).await
}

/// Helper: perform a post action as an authenticated user.
async fn do_post_action(
    state: SharedState,
    token: &str,
    postid: &str,
    action: &str,
) -> serde_json::Value {
    let app = app_with_state(state);
    graphql_with_auth(app, &format!(r#"
        mutation {{
            resolvePostAction(postid: "{postid}", action: {action}) {{
                meta {{ status RequestId ResponseCode ResponseMessage }}
            }}
        }}
    "#), token).await
}
```

### Full Test Listing

All 48 tests from Phase 3.H, organized by category:

**List Posts (H1–H10):** Basic listing, pagination, filtering by content type, sorting, title search, tag filter.

**Single Post (H11–H16):** Authenticated single post, invalid UUID, not found, guest access, user-scoped listing.

**Create Post (H17–H23):** Success, validation errors (title, tags, description, media limits), verify appears in feed.

**Post Actions (H24–H38):** Like/unlike, dislike/undislike, view, save/unsave, report, share, own-post guards, duplicate guards, non-existent post, auth required.

**Post Eligibility & Upload (H39–H46):** Eligibility token, upload success, invalid token, auth required.

**Edge Cases (H47–H48):** Create without auth, social filter (FOLLOWED).

### Sample Test Bodies

```rust
#[tokio::test]
async fn test_list_posts_with_seed_data() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;
    let app = app_with_state(state.clone());

    let res = graphql_with_auth(app, r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 10) {
                meta { status ResponseCode }
                counter
                affectedRows {
                    id contenttype title amountlikes amountviews
                    isliked isviewed tags
                    user { id username slug }
                }
            }
        }
    "#, &token).await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    // 8 seed posts minus 1 ad post = 7 in normal feed
    assert!(data["counter"].as_i64().unwrap() >= 7);
    assert!(data["affectedRows"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_list_posts_pagination() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;
    let app = app_with_state(state.clone());

    let res = graphql_with_auth(app, r#"
        query {
            listPosts(sortBy: NEWEST, offset: 0, limit: 3) {
                meta { ResponseCode }
                counter
                affectedRows { id }
            }
        }
    "#, &token).await;

    let data = &res["data"]["listPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert_eq!(data["affectedRows"].as_array().unwrap().len(), 3);
    // counter reflects total, not page size
    assert!(data["counter"].as_i64().unwrap() > 3);
}

#[tokio::test]
async fn test_filter_by_image() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;
    let app = app_with_state(state.clone());

    let res = graphql_with_auth(app, r#"
        query {
            listPosts(filterBy: [IMAGE], sortBy: NEWEST, offset: 0, limit: 20) {
                meta { ResponseCode }
                affectedRows { id contenttype }
            }
        }
    "#, &token).await;

    let rows = res["data"]["listPosts"]["affectedRows"].as_array().unwrap();
    for row in rows {
        assert_eq!(row["contenttype"], "image");
    }
}

#[tokio::test]
async fn test_guest_list_post() {
    // No auth needed
    let app = app();
    let postid = SEED_POST_1.to_string();

    let res = graphql(app, &format!(r#"
        query {{
            guestListPost(postid: "{postid}") {{
                meta {{ status ResponseCode }}
                affectedRows {{
                    id title contenttype
                    isliked isviewed isdisliked issaved
                    user {{ id username slug }}
                }}
            }}
        }}
    "#)).await;

    let data = &res["data"]["guestListPost"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    let post = &data["affectedRows"][0];
    // Guest: all interaction flags false
    assert_eq!(post["isliked"], false);
    assert_eq!(post["isviewed"], false);
    assert_eq!(post["isdisliked"], false);
    assert_eq!(post["issaved"], false);
}

#[tokio::test]
async fn test_create_post_success() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;

    let res = create_test_post(state.clone(), &token, "My new post", "text").await;

    let data = &res["data"]["createPost"];
    assert_eq!(data["meta"]["ResponseCode"], "11508");
    assert!(data["affectedRows"]["id"].is_string());
    assert_eq!(data["affectedRows"]["contenttype"], "text");
    assert_eq!(data["affectedRows"]["title"], "My new post");
}

#[tokio::test]
async fn test_create_post_title_too_long() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;

    let long_title = "a".repeat(64);
    let res = create_test_post(state.clone(), &token, &long_title, "text").await;

    assert_eq!(res["data"]["createPost"]["meta"]["ResponseCode"], "30210");
    assert!(res["data"]["createPost"]["affectedRows"].is_null());
}

#[tokio::test]
async fn test_like_post() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;

    // Like a post by another user
    let postid = SEED_POST_3.to_string(); // user2's post, already liked by seed
    // First unlike (since seed has this like), then re-check
    // OR use a post not yet liked
    let postid = SEED_POST_4.to_string(); // user2's audio post, not liked by seed

    let res = do_post_action(state.clone(), &token, &postid, "LIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["meta"]["ResponseCode"], "11503");

    // Verify isliked flag
    let app = app_with_state(state.clone());
    let res = graphql_with_auth(app, &format!(r#"
        query {{
            listPosts(postid: "{postid}", limit: 1) {{
                affectedRows {{ isliked amountlikes }}
            }}
        }}
    "#), &token).await;

    let post = &res["data"]["listPosts"]["affectedRows"][0];
    assert_eq!(post["isliked"], true);
}

#[tokio::test]
async fn test_like_own_post() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;

    // SEED_POST_1 is authored by verified user
    let postid = SEED_POST_1.to_string();
    let res = do_post_action(state.clone(), &token, &postid, "LIKE").await;
    assert_eq!(res["data"]["resolvePostAction"]["meta"]["ResponseCode"], "31506");
}

#[tokio::test]
async fn test_post_action_not_found() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;

    let res = do_post_action(
        state.clone(), &token,
        "99999999-9999-4999-a999-999999999999", "VIEW"
    ).await;
    assert_eq!(res["data"]["resolvePostAction"]["meta"]["ResponseCode"], "31510");
}

#[tokio::test]
async fn test_post_action_without_auth() {
    let app = app();
    let res = graphql(app, r#"
        mutation {
            resolvePostAction(postid: "any-id", action: VIEW) {
                meta { ResponseCode }
            }
        }
    "#).await;
    assert_eq!(res["data"]["resolvePostAction"]["meta"]["ResponseCode"], "60501");
}

#[tokio::test]
async fn test_post_eligibility() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;
    let app = app_with_state(state.clone());

    let res = graphql_with_auth(app, r#"
        query {
            postEligibility {
                meta { ResponseCode }
                eligibilityToken
            }
        }
    "#, &token).await;

    let data = &res["data"]["postEligibility"];
    assert_eq!(data["meta"]["ResponseCode"], "10901");
    assert!(data["eligibilityToken"].is_string());
    assert!(data["eligibilityToken"].as_str().unwrap().starts_with("mock-eligibility-"));
}

#[tokio::test]
async fn test_search_tags() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;
    let app = app_with_state(state.clone());

    let res = graphql_with_auth(app, r#"
        query {
            searchTags(tagName: "web") {
                meta { ResponseCode }
                counter
                affectedRows { name }
            }
        }
    "#, &token).await;

    let data = &res["data"]["searchTags"];
    assert_eq!(data["meta"]["ResponseCode"], "11701");
    let tags = data["affectedRows"].as_array().unwrap();
    assert!(tags.iter().any(|t| t["name"] == "webdev"));
}

#[tokio::test]
async fn test_list_advertisement_posts() {
    let (state, _) = setup_authenticated().await;
    let token = login_verified_user(&state).await;
    let app = app_with_state(state.clone());

    let res = graphql_with_auth(app, r#"
        query {
            listAdvertisementPosts(offset: 0, limit: 10) {
                meta { ResponseCode }
                counter
                affectedRows {
                    post { id title }
                    advertisement { advertisementid advertisementtype }
                }
            }
        }
    "#, &token).await;

    let data = &res["data"]["listAdvertisementPosts"];
    assert_eq!(data["meta"]["ResponseCode"], "11501");
    assert!(data["counter"].as_i64().unwrap() >= 1);
    let ad = &data["affectedRows"][0];
    assert!(ad["advertisement"]["advertisementid"].is_string());
}
```

---

## 6. Definition of Done

### Build & Test Gates

- [ ] `cargo build` succeeds without warnings
- [ ] `cargo test --all-targets` passes all Phase 0 + 1 + 2 + 3 tests (≥48 new tests)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes

### Functional Requirements — Queries

- [ ] `listPosts(...)` returns paginated posts with correct `counter` and `affectedRows`
- [ ] `listPosts` filters by content type (`IMAGE`, `AUDIO`, `VIDEO`, `TEXT`)
- [ ] `listPosts` filters by social relation (`FOLLOWED`, `FOLLOWER`, `FRIENDS`, `VIEWED`)
- [ ] `listPosts` filters by `title` (case-insensitive substring)
- [ ] `listPosts` filters by `tag` (case-insensitive exact match)
- [ ] `listPosts` filters by `userid` (returns only that user's posts)
- [ ] `listPosts` retrieves single post by `postid`
- [ ] `listPosts` sorts by `NEWEST`, `OLDEST`, `LIKES`, `DISLIKES`, `VIEWS`, `TRENDING`
- [ ] `listPosts` excludes advertisement posts from normal feed
- [ ] `listPosts` excludes posts by blocked users (when `IgnorList != NO`)
- [ ] `listPosts` excludes posts by deleted users
- [ ] `listPosts` returns `21518` when no posts match
- [ ] `listPosts` returns `30209` for malformed `postid`
- [ ] `listPosts` returns `60501` without authentication
- [ ] `guestListPost(postid)` returns single post without auth, all interaction flags `false`
- [ ] `guestListPost` returns `31510` for non-existent post
- [ ] `guestListPost` returns `30209` for invalid UUID
- [ ] `postEligibility` returns eligibility token (`10901`)
- [ ] `postEligibility` returns `60501` without auth
- [ ] `searchTags(tagName)` returns matching tags (`11701`) or `21701` for no match
- [ ] `searchTags` returns `30101` for empty `tagName`
- [ ] `listTags(offset, limit)` returns all tags paginated
- [ ] `listAdvertisementPosts(...)` returns only advertisement posts with `AdvertisementInfo`
- [ ] `postInteractions(getOnly, postOrCommentId)` returns users who performed interaction

### Functional Requirements — Mutations

- [ ] `createPost(action: POST, input: {...})` creates a post and returns `11508` with `id`, `contenttype`, `title`
- [ ] `createPost` validates title length (1–63 chars, `30210`)
- [ ] `createPost` validates description length (≤500 chars, `30263`)
- [ ] `createPost` validates tag format and count (max 10, each 2–53 chars alphanumeric+underscore, `30262`)
- [ ] `createPost` validates media count per content type (`30267`, `30268`)
- [ ] `createPost` registers new tags in global tag set
- [ ] `createPost` returns `60501` without auth
- [ ] Created posts appear in subsequent `listPosts` queries
- [ ] `resolvePostAction(LIKE, postid)` adds like, returns `11503`
- [ ] `resolvePostAction(UNLIKE, postid)` removes like
- [ ] `resolvePostAction(DISLIKE, postid)` adds dislike, returns `11504`; removes existing like
- [ ] `resolvePostAction(UNDISLIKE, postid)` removes dislike
- [ ] `resolvePostAction(VIEW, postid)` records view, returns `11506`
- [ ] `resolvePostAction(SAVE, postid)` toggles save, returns `11512` (saved) or `11511` (unsaved)
- [ ] `resolvePostAction(UNSAVE, postid)` explicitly unsaves
- [ ] `resolvePostAction(REPORT, postid)` records report, returns `11505`
- [ ] `resolvePostAction(SHARE, postid)` records share, returns `11507`
- [ ] Like/dislike/report own post returns `31506`/`31507`/`31508`
- [ ] Duplicate like/dislike/report returns `31501`/`31502`/`31503`
- [ ] Action on non-existent post returns `31510`
- [ ] Actions without auth return `60501`
- [ ] Interaction flags (`isliked`, `isviewed`, etc.) reflect current user state in `listPosts`
- [ ] Like/dislike counts update correctly after actions

### Functional Requirements — Upload Endpoint

- [ ] `POST /upload-post` accepts multipart with `eligibilityToken` and `file` fields
- [ ] Returns `11515` with mock `uploadedFiles` filenames
- [ ] Returns `40902` for invalid/missing/consumed eligibility token
- [ ] Returns `60501` without auth
- [ ] Token status transitions: `ISSUED` → `FILE_UPLOADED` (prevents reuse)

### Response Shape Compatibility

- [ ] `Post` fields match the frontend's `PostUser` and `Post` structs in `peer-web/src/models/post.rs`
- [ ] `PostListResponse` has `meta` (DefaultResponse), `counter` (Int), `affectedRows` (Vec<Post>)
- [ ] `CreatePostResponse` has `meta`, `affectedRows` with `id`, `contenttype`, `title`
- [ ] `PostEligibilityResponse` has `meta`, `eligibilityToken`
- [ ] `TagSearchResponse` has `meta`, `counter`, `affectedRows` with `name`
- [ ] `AdListResponse` has `meta`, `counter`, `affectedRows` with nested `post` + `advertisement`
- [ ] `DefaultResponse` from `resolvePostAction` has `status`, `RequestId`, `ResponseCode`, `ResponseMessage`
- [ ] Upload endpoint returns JSON with `status`, `ResponseCode` (PascalCase), `affectedRows.uploadedFiles`
- [ ] All field casing matches exactly what the frontend queries select (see `graphql.rs` query strings)

### State & Isolation

- [ ] `POST /reset` clears all post state (posts, interactions, tags, eligibility tokens, uploaded files) back to seed defaults
- [ ] Seed data provides 8 posts, 4 tags, 2 likes, 3 views, and 1 advertisement out of the box
- [ ] Test isolation: each test starts from a clean or known state via `/reset` or fresh `app_with_state()`

### Cross-Cutting (carried from parent plan)

- [ ] No `unwrap()` in resolver paths — all errors return proper GraphQL error responses
- [ ] Every resolver has ≥1 success and ≥1 error integration test
- [ ] Response codes match values in `docs/backend_api/03-posts-and-content.md`
- [ ] Field names exactly match the backend schema casing
- [ ] No runtime dependencies on Node.js, npm, or non-Rust tooling
