# Phase 4: Social (Comments, Chat)

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** [Phase 3 — Posts & Content](./mock-backend-rust-rewrite.md#5-phase-3--posts--content) (requires posts for comments; requires users for chat)
> **Goal:** Add all comment and chat queries/mutations so the Leptos view-post comment section and chat page work end-to-end against the mock.
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

**From `src/api/comments.rs` and `src/api/graphql.rs`:**

| Operation | GraphQL SDL | Frontend file | Auth? |
|-----------|-------------|---------------|-------|
| `listComments(postid, commentOffset, commentLimit)` | `query ListComments(...)` | `comments.rs` → `ListComments` server fn | Yes (optional — token passed if available) |
| `listChildComments(parent, offset, limit)` | `query ListChildComments(...)` | `comments.rs` → `ListChildComments` server fn | Yes (optional) |
| `createComment(action, postid, content, parentid?)` | `mutation CreateComment(...)` | `comments.rs` → `CreateComment` server fn | Yes |
| `likeComment(commentid)` | `mutation LikeComment(...)` | `comments.rs` → `LikeComment` server fn | Yes |
| `likeComment(commentid)` (toggle — second call removes the like; production peergamma has no `unlikeComment` resolver, frontend `UnlikeComment` server fn aliases this mutation) | `mutation UnlikeComment(...)` | `comments.rs` → `UnlikeComment` server fn | Yes |

**From `src/api/chat.rs` and `src/api/graphql.rs`:**

| Operation | GraphQL SDL | Frontend file | Auth? |
|-----------|-------------|---------------|-------|
| `listChats(limit, offset)` | `query ListChats(...)` | `chat.rs` → `ListChats` server fn | Yes |
| `sendChatMessage(chatid, content)` | `mutation SendChatMessage(...)` | `chat.rs` → `SendChatMessage` server fn | Yes |
| `createChat(input: { name, recipients, image })` | `mutation CreateChat(...)` | `chat.rs` → `CreateChat` server fn | Yes |

**From `docs/backend_api/04-comments.md` (additional operations in the schema but not yet wired in frontend):**

| Operation | Purpose | Auth? |
|-----------|---------|-------|
| `reportComment(commentid)` | Report a comment for moderation | Yes |

### Target State

After this phase, the mock backend will support:
- Full paginated comment listing (top-level and child/reply)
- Comment creation (top-level and replies) with token cost / daily free logic
- Comment like/unlike toggle
- Comment reporting
- Post `amountcomments` correctly computed from comment state
- Chat listing with participants and messages
- Chat creation (private 1:1 and group)
- Message sending with timestamp ordering
- 5 new GraphQL query resolvers, 5 new mutation resolvers

### New file tree additions

```
packages/mock_backend/src/
├── schema/
│   ├── query/
│   │   ├── comments.rs    # NEW: listComments, listChildComments
│   │   └── chat.rs        # NEW: listChats
│   └── mutation/
│       ├── comment.rs      # NEW: createComment, likeComment, unlikeComment, reportComment
│       └── chat.rs         # NEW: createChat, sendChatMessage
├── types/
│   ├── comment.rs          # NEW: Comment, CommentUser, CommentListResponse, CreateCommentResponse, CommentType enum
│   └── chat.rs             # NEW: Chat, ChatMessage, ChatParticipant, ListChatsResponse, SendMessageResponse, CreateChatResponse
└── state.rs                # MODIFIED: new fields for comments, comment likes, chats, chat messages
```

---

## 2. Prerequisites

### Phase 3 Completion

- [x] Posts exist in `MockState` (comments reference `postid`)
- [x] `PostRecord` type is available with `id` (UUID) field
- [x] Post interaction count computation exists (so `amountcomments` can be updated)
- [x] `post_record_to_graphql()` helper exists and can be extended for comment counts

### Phase 1 & 2 Completion

- [x] Auth middleware from Phase 1 is working (`require_auth()`, `get_current_user()`)
- [x] User profiles exist in `MockState` (comments embed `CommentUser`, chats embed `ChatParticipant`)
- [x] Follow relationships exist (comments show `isfollowed`/`isfollowing` on `CommentUser`)
- [x] `DefaultResponse` type is available from Phase 0

### Phase 5 Integration Point

- [x] Comment creation costs tokens (1.0 per comment) unless daily free action is used
- [x] For Phase 4, token deduction can be **stubbed** (always succeed, or skip balance check) with a `TODO` for Phase 5 integration
- [x] Daily free action tracking will be fully implemented in Phase 5; Phase 4 returns `11608` for the first 4 comments per day, `11605` thereafter

### API Reference

All response codes and field names come from:
- `docs/backend_api/04-comments.md`
- `src/models/comment.rs` (frontend deserialization types)
- `src/models/chat.rs` (frontend deserialization types)
- `src/api/graphql.rs` (exact GraphQL field selections)

---

## 3. Task Breakdown

### Phase 4.A — Comment Types (`types/comment.rs`)

| # | Task | Notes |
|---|------|-------|
| A1 | Create `types/comment.rs` with `CommentType` enum | `COMMENT` (single variant, required by `createComment`) |
| A2 | Define `CommentUser` struct (SimpleObject) | `id`, `username`, `slug`, `img`, `isfollowed`, `isfollowing` — must match frontend's `CommentUser` deserialization |
| A3 | Define `Comment` struct (SimpleObject) | `commentid`, `userid`, `postid`, `parentid`, `content`, `createdat`, `amountlikes`, `amountreplies`, `isliked`, `user: CommentUser` — matches frontend's `Comment` struct |
| A4 | Define `CommentListResponse` struct | `meta: DefaultResponse`, `counter: i32`, `affectedRows: Option<Vec<Comment>>` — used by `listComments` and `listChildComments` |
| A5 | Define `CreateCommentResponse` struct | `meta: DefaultResponse`, `counter: i32`, `affectedRows: Option<Vec<Comment>>` — used by `createComment` (returns the created comment) |
| A6 | Export new types from `types/mod.rs` | Add `pub mod comment;` |

### Phase 4.B — Chat Types (`types/chat.rs`)

| # | Task | Notes |
|---|------|-------|
| B1 | Create `types/chat.rs` with `ChatParticipant` struct | `userid`, `img`, `username`, `slug`, `hasaccess` — matches frontend's `ChatParticipant` |
| B2 | Define `ChatMessage` struct (SimpleObject) | `id`, `senderid`, `chatid`, `content`, `createdat` — matches frontend's `ChatMessage` |
| B3 | Define `Chat` struct (SimpleObject) | `id`, `name`, `image`, `createdat`, `updatedat`, `chatmessages: Vec<ChatMessage>`, `chatparticipants: Vec<ChatParticipant>` — matches frontend's `Chat` |
| B4 | Define `ListChatsResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<Vec<Chat>>` |
| B5 | Define `SendMessageResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<ChatMessage>` |
| B6 | Define `CreateChatResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<CreateChatResult>` |
| B7 | Define `CreateChatResult` struct | `chatid: String` |
| B8 | Export new types from `types/mod.rs` | Add `pub mod chat;` |

### Phase 4.C — State Extensions (`state.rs`, `seed.rs`)

| # | Task | Notes |
|---|------|-------|
| C1 | Add `CommentRecord` struct to state | `id: Uuid`, `author_id: Uuid`, `post_id: Uuid`, `parent_id: Option<Uuid>`, `content: String`, `created_at: String`, `visibility_status: String` |
| C2 | Add `comments: Vec<CommentRecord>` to `MockState` | Ordered by creation time |
| C3 | Add `comment_likes: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, comment_id)` — tracks who liked which comment |
| C4 | Add `comment_reports: HashSet<(Uuid, Uuid)>` to `MockState` | `(user_id, comment_id)` — tracks who reported which comment |
| C5 | Add `daily_comment_count: HashMap<(Uuid, String), u32>` to `MockState` | `(user_id, date_string)` → count of comments created that day, for daily free action tracking |
| C6 | Add `ChatRecord` struct to state | `id: Uuid`, `name: Option<String>`, `image: Option<String>`, `created_at: String`, `updated_at: String`, `participant_ids: Vec<Uuid>` |
| C7 | Add `ChatMessageRecord` struct to state | `id: Uuid`, `sender_id: Uuid`, `chat_id: Uuid`, `content: String`, `created_at: String` |
| C8 | Add `chats: Vec<ChatRecord>` to `MockState` | |
| C9 | Add `chat_messages: Vec<ChatMessageRecord>` to `MockState` | Ordered by creation time |
| C10 | Update `MockState::default()` and `MockState::reset()` to initialize/clear all new fields | Preserve seed data only |

### Phase 4.D — Seed Data (`seed.rs`)

| # | Task | Notes |
|---|------|-------|
| D1 | Add 6 seed comments with deterministic UUIDs | 4 top-level comments on various seed posts, 2 replies to top-level comments |
| D2 | Associate seed comments with existing seed users | Comments authored by `SEED_USER_VERIFIED`, `SEED_USER_2`, and other Phase 2 seed users |
| D3 | Add pre-populated comment like data | 2 comment likes on seed comments |
| D4 | Add 2 seed chats with deterministic UUIDs | 1 private (1:1) chat, 1 group chat |
| D5 | Add 5 seed chat messages | 3 messages in the private chat, 2 messages in the group chat |
| D6 | Associate seed chats with existing seed users as participants | Private chat between user1 and user2; group chat with user1, user2, user3 |

### Phase 4.E — Comment Query Resolvers (`schema/query/comments.rs`)

| # | Task | Notes |
|---|------|-------|
| E1 | Create `schema/query/comments.rs` with `CommentQuery` struct | Uses `#[Object]` |
| E2 | Implement `list_comments(postid, contentFilterBy?, commentOffset?, commentLimit?)` resolver | Fetch top-level comments (`parent_id == None`) for a post. Pagination with `commentOffset`/`commentLimit` (default 10, max 20). Returns `CommentListResponse`. |
| E3 | Implement `list_child_comments(parent, offset?, limit?)` resolver | Fetch replies to a specific top-level comment. No content filtering applied. Pagination with `offset`/`limit`. Returns `CommentListResponse` (reusing the same type with code `11607`). |
| E4 | Implement per-user interaction state on comments | Set `isliked` based on current user's comment likes |
| E5 | Implement `amountreplies` computation | Count child comments for each top-level comment |
| E6 | Implement `amountlikes` computation | Count likes for each comment from `comment_likes` set |
| E7 | Register `CommentQuery` in `QueryRoot` merged object | |

### Phase 4.F — Comment Mutation Resolvers (`schema/mutation/comment.rs`)

| # | Task | Notes |
|---|------|-------|
| F1 | Create `schema/mutation/comment.rs` with `CommentMutation` struct | Uses `#[Object]` |
| F2 | Implement `create_comment(action, postid, content, parentid?)` resolver | Requires auth. Validate `action == COMMENT`. Validate content (1–200 chars). Validate post exists. If `parentid` set, validate parent exists, is top-level, and belongs to same post. Generate UUID, create `CommentRecord`, increment post comment count. Return `CreateCommentResponse`. |
| F3 | Implement daily free action logic for `createComment` | Track `daily_comment_count`. First 4 per day → `11608` (free). After 4 → `11605` (paid, stub deduction). |
| F4 | Implement `like_comment(commentid)` resolver | Requires auth. Validate comment exists (`31601`). Prevent self-like (`31606`). Prevent duplicate (`31604`). Add to `comment_likes`. Return `DefaultResponse` with code `11603`. |
| F5 | Implement `unlike_comment(commentid)` resolver | Requires auth. Validate comment exists. Remove from `comment_likes`. Return `DefaultResponse`. Mirror the `likeComment` response shape (the frontend deserializes both into `LikeCommentResponse`). |
| F6 | Implement `report_comment(commentid)` resolver | Requires auth. Validate comment exists (`31601`). Prevent self-report (`31607`). Prevent duplicate (`31605`). Add to `comment_reports`. Return `DefaultResponse` with code `11604`. |
| F7 | Register `CommentMutation` in `MutationRoot` merged object | |

### Phase 4.G — Chat Query Resolvers (`schema/query/chat.rs`)

| # | Task | Notes |
|---|------|-------|
| G1 | Create `schema/query/chat.rs` with `ChatQuery` struct | Uses `#[Object]` |
| G2 | Implement `list_chats(limit?, offset?)` resolver | Requires auth. Return chats where current user is a participant. Each chat includes `chatmessages` (all messages in the chat, ordered by `created_at`) and `chatparticipants` (all participants with profile info). Sort chats by `updated_at` descending (most recently active first). Pagination with `offset`/`limit`. Return `ListChatsResponse`. |
| G3 | Implement participant resolution | For each chat, resolve `ChatParticipant` from user profiles in state: `userid`, `img`, `username`, `slug`, `hasaccess: true`. |
| G4 | Register `ChatQuery` in `QueryRoot` merged object | |

### Phase 4.H — Chat Mutation Resolvers (`schema/mutation/chat.rs`)

| # | Task | Notes |
|---|------|-------|
| H1 | Create `schema/mutation/chat.rs` with `ChatMutation` struct | Uses `#[Object]` |
| H2 | Implement `create_chat(input: { name, recipients, image? })` resolver | Requires auth. Validate at least 1 recipient. Create `ChatRecord` with generated UUID. Add current user + recipients as participants. For private chats: check if a 1:1 chat already exists between the two users and return existing chat ID if so. Return `CreateChatResponse` with `chatid`. |
| H3 | Implement `send_chat_message(chatid, content)` resolver | Requires auth. Validate chat exists. Validate current user is a participant. Validate content (1–500 chars). Generate UUID for message. Create `ChatMessageRecord`. Update chat's `updated_at` timestamp. Return `SendMessageResponse` with the created message. |
| H4 | Register `ChatMutation` in `MutationRoot` merged object | |

### Phase 4.I — Cross-Cutting: Update Post Comment Counts

| # | Task | Notes |
|---|------|-------|
| I1 | Update `post_record_to_graphql()` in Phase 3 code | Replace `amountcomments = 0` placeholder with actual count from `self.comments.iter().filter(\|c\| c.post_id == record.id).count()` |
| I2 | Update trending score computation | Include `amountcomments` in trending formula: `likes * 2 + views + comments` |
| I3 | Update `Comments` sort in `sort_posts()` | Sort by actual comment count instead of `created_at` fallback |

### Phase 4.J — Helper Functions

| # | Task | Notes |
|---|------|-------|
| J1 | Create `CommentRecord → Comment` conversion function | Resolves author `CommentUser` from users map, computes `amountlikes` from `comment_likes`, computes `amountreplies` from child comment count, sets `isliked` per viewer |
| J2 | Create comment filtering function | Exclude comments from deleted users, exclude comments with `visibility_status != "VISIBLE"` |
| J3 | Create `ChatRecord → Chat` conversion function | Resolves participants from users map, includes all messages for the chat, ordered by `created_at` |
| J4 | Create today's date helper | `fn today_string() -> String` returns `YYYY-MM-DD` for daily free action tracking |

### Phase 4.K — Integration Tests

See [Section 5: Testing Strategy](#5-testing-strategy) for the full test matrix.

### Phase 4.L — Cleanup & Validation

| # | Task | Notes |
|---|------|-------|
| L1 | Run `cargo clippy -- -D warnings` | Fix all warnings |
| L2 | Run `cargo fmt --check` | Fix formatting |
| L3 | Run full test suite (`cargo test --all-targets`) | All Phase 0 + 1 + 2 + 3 + 4 tests pass |
| L4 | Manual smoke test with `cargo run` + curl | Verify HTTP layer works for all new endpoints |

---

## 4. Implementation Details

### 4.1 Comment Types (`types/comment.rs`)

```rust
use async_graphql::{Enum, SimpleObject, ID};
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Comment action type (only COMMENT is used).
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum CommentType {
    Comment,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Simplified user embedded in comment responses.
///
/// Must match the frontend's `CommentUser` deserialization in `src/models/comment.rs`.
/// Field selections from `LIST_COMMENTS_QUERY` in `graphql.rs`:
/// `id, username, slug, img, isfollowed, isfollowing`
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CommentUser {
    pub id: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    #[graphql(name = "isfollowed")]
    pub isfollowed: bool,
    #[graphql(name = "isfollowing")]
    pub isfollowing: bool,
}

/// Comment data returned in all comment responses.
///
/// Field names must exactly match the GraphQL schema selections in `graphql.rs`.
/// The frontend deserializes these via `src/models/comment.rs::Comment`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Comment {
    pub commentid: ID,
    pub userid: ID,
    pub postid: ID,
    pub parentid: Option<ID>,
    pub content: String,
    pub createdat: String,
    pub amountlikes: i32,
    pub amountreplies: i32,
    pub isliked: bool,
    pub user: CommentUser,
}

/// Response for `listComments` and `listChildComments` queries.
///
/// Used for both top-level and child comment listing.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CommentListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Comment>>,
}

/// Response for `createComment` mutation.
///
/// Returns the created comment in `affectedRows`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateCommentResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Comment>>,
}
```

### 4.2 Chat Types (`types/chat.rs`)

```rust
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// GraphQL Objects
// ============================================================================

/// A participant in a chat room.
///
/// Must match the frontend's `ChatParticipant` in `src/models/chat.rs`.
/// Field selections from `LIST_CHATS_QUERY`:
/// `userid, img, username, slug, hasaccess`
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ChatParticipant {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: Option<String>,
    pub hasaccess: Option<bool>,
}

/// A single chat message.
///
/// Must match the frontend's `ChatMessage` in `src/models/chat.rs`.
/// Field selections from `LIST_CHATS_QUERY` and `SEND_CHAT_MESSAGE_MUTATION`:
/// `id, senderid, chatid, content, createdat`
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub senderid: String,
    pub chatid: String,
    pub content: String,
    pub createdat: String,
}

/// A chat room (private or group).
///
/// Must match the frontend's `Chat` in `src/models/chat.rs`.
/// Field selections from `LIST_CHATS_QUERY`:
/// `id, image, name, createdat, updatedat, chatmessages { ... }, chatparticipants { ... }`
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Chat {
    pub id: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub createdat: String,
    pub updatedat: String,
    pub chatmessages: Vec<ChatMessage>,
    pub chatparticipants: Vec<ChatParticipant>,
}

/// Response for `listChats` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListChatsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Chat>>,
}

/// Response for `sendChatMessage` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct SendMessageResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ChatMessage>,
}

/// Response for `createChat` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateChatResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<CreateChatResult>,
}

/// Created chat result containing the new chat ID.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateChatResult {
    pub chatid: String,
}
```

### 4.3 Internal State Records (`state.rs`)

```rust
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Internal comment storage record (not the GraphQL type).
#[derive(Debug, Clone)]
pub struct CommentRecord {
    pub id: Uuid,
    pub author_id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,       // None = top-level, Some = reply
    pub content: String,
    pub created_at: String,            // ISO 8601 timestamp
    pub visibility_status: String,     // "VISIBLE", "HIDDEN", "ILLEGAL"
}

/// Internal chat storage record.
#[derive(Debug, Clone)]
pub struct ChatRecord {
    pub id: Uuid,
    pub name: Option<String>,          // None for private 1:1 chats
    pub image: Option<String>,         // None for private chats
    pub created_at: String,
    pub updated_at: String,            // Updated on each new message
    pub participant_ids: Vec<Uuid>,    // All participants including creator
}

/// Internal chat message storage record.
#[derive(Debug, Clone)]
pub struct ChatMessageRecord {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub chat_id: Uuid,
    pub content: String,
    pub created_at: String,
}

/// Extended MockState fields for Phase 4.
pub struct MockState {
    // --- Phase 0–3 fields (existing) ---
    // ...

    // --- Phase 4: Comments ---
    pub comments: Vec<CommentRecord>,
    pub comment_likes: HashSet<(Uuid, Uuid)>,          // (user_id, comment_id)
    pub comment_reports: HashSet<(Uuid, Uuid)>,         // (user_id, comment_id)
    pub daily_comment_count: HashMap<(Uuid, String), u32>, // (user_id, "YYYY-MM-DD") → count

    // --- Phase 4: Chat ---
    pub chats: Vec<ChatRecord>,
    pub chat_messages: Vec<ChatMessageRecord>,
}
```

### 4.4 Seed Data (`seed.rs`)

```rust
use uuid::{uuid, Uuid};

// --- Phase 4 seed comment UUIDs ---
pub const SEED_COMMENT_1: Uuid = uuid!("20000000-0000-4000-a000-000000000001"); // top-level on post 1
pub const SEED_COMMENT_2: Uuid = uuid!("20000000-0000-4000-a000-000000000002"); // top-level on post 1
pub const SEED_COMMENT_3: Uuid = uuid!("20000000-0000-4000-a000-000000000003"); // top-level on post 3
pub const SEED_COMMENT_4: Uuid = uuid!("20000000-0000-4000-a000-000000000004"); // top-level on post 5
pub const SEED_COMMENT_5: Uuid = uuid!("20000000-0000-4000-a000-000000000005"); // reply to comment 1
pub const SEED_COMMENT_6: Uuid = uuid!("20000000-0000-4000-a000-000000000006"); // reply to comment 3

// --- Phase 4 seed chat UUIDs ---
pub const SEED_CHAT_PRIVATE: Uuid = uuid!("30000000-0000-4000-a000-000000000001"); // 1:1 chat
pub const SEED_CHAT_GROUP: Uuid = uuid!("30000000-0000-4000-a000-000000000002");   // group chat

// --- Phase 4 seed chat message UUIDs ---
pub const SEED_MSG_1: Uuid = uuid!("40000000-0000-4000-a000-000000000001");
pub const SEED_MSG_2: Uuid = uuid!("40000000-0000-4000-a000-000000000002");
pub const SEED_MSG_3: Uuid = uuid!("40000000-0000-4000-a000-000000000003");
pub const SEED_MSG_4: Uuid = uuid!("40000000-0000-4000-a000-000000000004");
pub const SEED_MSG_5: Uuid = uuid!("40000000-0000-4000-a000-000000000005");

fn seed_comments(verified_user: Uuid, user2: Uuid) -> Vec<CommentRecord> {
    vec![
        // Top-level comment on post 1 by user2
        CommentRecord {
            id: SEED_COMMENT_1,
            author_id: user2,
            post_id: SEED_POST_1,
            parent_id: None,
            content: "Great photo! Love the colors.".into(),
            created_at: "2025-04-01T12:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Top-level comment on post 1 by verified_user
        CommentRecord {
            id: SEED_COMMENT_2,
            author_id: verified_user,
            post_id: SEED_POST_1,
            parent_id: None,
            content: "Thanks for the kind words!".into(),
            created_at: "2025-04-01T13:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Top-level comment on post 3 by verified_user
        CommentRecord {
            id: SEED_COMMENT_3,
            author_id: verified_user,
            post_id: SEED_POST_3,
            parent_id: None,
            content: "These Rust tips are really helpful.".into(),
            created_at: "2025-04-03T15:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Top-level comment on post 5 by user2
        CommentRecord {
            id: SEED_COMMENT_4,
            author_id: user2,
            post_id: SEED_POST_5,
            parent_id: None,
            content: "Amazing tutorial, well explained!".into(),
            created_at: "2025-04-05T17:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Reply to comment 1 by verified_user
        CommentRecord {
            id: SEED_COMMENT_5,
            author_id: verified_user,
            post_id: SEED_POST_1,
            parent_id: Some(SEED_COMMENT_1),
            content: "Glad you liked it!".into(),
            created_at: "2025-04-01T14:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
        // Reply to comment 3 by user2
        CommentRecord {
            id: SEED_COMMENT_6,
            author_id: user2,
            post_id: SEED_POST_3,
            parent_id: Some(SEED_COMMENT_3),
            content: "You're welcome, check out part 2 as well!".into(),
            created_at: "2025-04-03T16:00:00Z".into(),
            visibility_status: "VISIBLE".into(),
        },
    ]
}

fn seed_comment_likes(verified_user: Uuid, user2: Uuid) -> HashSet<(Uuid, Uuid)> {
    HashSet::from([
        (verified_user, SEED_COMMENT_4), // verified user liked user2's comment
        (user2, SEED_COMMENT_3),          // user2 liked verified user's comment
    ])
}

fn seed_chats(verified_user: Uuid, user2: Uuid, user3: Uuid) -> Vec<ChatRecord> {
    vec![
        // Private 1:1 chat between verified_user and user2
        ChatRecord {
            id: SEED_CHAT_PRIVATE,
            name: None,
            image: None,
            created_at: "2025-04-01T10:00:00Z".into(),
            updated_at: "2025-04-02T14:30:00Z".into(),
            participant_ids: vec![verified_user, user2],
        },
        // Group chat with 3 participants
        ChatRecord {
            id: SEED_CHAT_GROUP,
            name: Some("Rust Developers".into()),
            image: None,
            created_at: "2025-04-03T09:00:00Z".into(),
            updated_at: "2025-04-04T11:00:00Z".into(),
            participant_ids: vec![verified_user, user2, user3],
        },
    ]
}

fn seed_chat_messages(verified_user: Uuid, user2: Uuid) -> Vec<ChatMessageRecord> {
    vec![
        // Private chat messages
        ChatMessageRecord {
            id: SEED_MSG_1,
            sender_id: verified_user,
            chat_id: SEED_CHAT_PRIVATE,
            content: "Hey, how's the project going?".into(),
            created_at: "2025-04-01T10:05:00Z".into(),
        },
        ChatMessageRecord {
            id: SEED_MSG_2,
            sender_id: user2,
            chat_id: SEED_CHAT_PRIVATE,
            content: "Going well! Just deployed the new feature.".into(),
            created_at: "2025-04-02T14:00:00Z".into(),
        },
        ChatMessageRecord {
            id: SEED_MSG_3,
            sender_id: verified_user,
            chat_id: SEED_CHAT_PRIVATE,
            content: "Awesome, I'll check it out.".into(),
            created_at: "2025-04-02T14:30:00Z".into(),
        },
        // Group chat messages
        ChatMessageRecord {
            id: SEED_MSG_4,
            sender_id: verified_user,
            chat_id: SEED_CHAT_GROUP,
            content: "Welcome to the Rust devs group!".into(),
            created_at: "2025-04-03T09:05:00Z".into(),
        },
        ChatMessageRecord {
            id: SEED_MSG_5,
            sender_id: user2,
            chat_id: SEED_CHAT_GROUP,
            content: "Thanks for the invite. What are we working on first?".into(),
            created_at: "2025-04-04T11:00:00Z".into(),
        },
    ]
}
```

### 4.5 CommentRecord → Comment Conversion

```rust
impl MockState {
    /// Convert a CommentRecord to the GraphQL Comment type.
    ///
    /// `viewer_id`: The current authenticated user (None for unauthenticated).
    pub fn comment_record_to_graphql(
        &self,
        record: &CommentRecord,
        viewer_id: Option<Uuid>,
    ) -> Comment {
        let author = self.users.get(&record.author_id);

        // Compute interaction counts
        let amountlikes = self.comment_likes.iter()
            .filter(|(_, cid)| *cid == record.id).count() as i32;

        // Count child comments (replies to this comment)
        let amountreplies = self.comments.iter()
            .filter(|c| c.parent_id == Some(record.id) && c.visibility_status == "VISIBLE")
            .count() as i32;

        // Per-user interaction flags
        let isliked = match viewer_id {
            Some(uid) => self.comment_likes.contains(&(uid, record.id)),
            None => false,
        };

        // Resolve author profile
        let user = match author {
            Some(u) => {
                let (isfollowed, isfollowing) = match viewer_id {
                    Some(vid) => (
                        self.follows.contains(&(vid, u.uid)),
                        self.follows.contains(&(u.uid, vid)),
                    ),
                    None => (false, false),
                };
                CommentUser {
                    id: u.uid.to_string().into(),
                    username: u.username.clone(),
                    slug: u.slug.clone(),
                    img: Some("https://via.placeholder.com/96".into()),
                    isfollowed,
                    isfollowing,
                }
            }
            None => CommentUser {
                id: record.author_id.to_string().into(),
                username: "unknown".into(),
                slug: "unknown".into(),
                img: None,
                isfollowed: false,
                isfollowing: false,
            },
        };

        Comment {
            commentid: record.id.to_string().into(),
            userid: record.author_id.to_string().into(),
            postid: record.post_id.to_string().into(),
            parentid: record.parent_id.map(|p| p.to_string().into()),
            content: record.content.clone(),
            createdat: record.created_at.clone(),
            amountlikes,
            amountreplies,
            isliked,
            user,
        }
    }
}
```

### 4.6 ChatRecord → Chat Conversion

```rust
impl MockState {
    /// Convert a ChatRecord to the GraphQL Chat type.
    pub fn chat_record_to_graphql(&self, record: &ChatRecord) -> Chat {
        // Resolve participants
        let chatparticipants: Vec<ChatParticipant> = record.participant_ids.iter()
            .filter_map(|uid| {
                self.users.get(uid).map(|u| ChatParticipant {
                    userid: u.uid.to_string(),
                    img: Some("https://via.placeholder.com/96".into()),
                    username: u.username.clone(),
                    slug: Some(u.slug.clone()),
                    hasaccess: Some(true),
                })
            })
            .collect();

        // Collect messages for this chat, ordered by created_at
        let chatmessages: Vec<ChatMessage> = self.chat_messages.iter()
            .filter(|m| m.chat_id == record.id)
            .map(|m| ChatMessage {
                id: m.id.to_string(),
                senderid: m.sender_id.to_string(),
                chatid: m.chat_id.to_string(),
                content: m.content.clone(),
                createdat: m.created_at.clone(),
            })
            .collect();

        Chat {
            id: record.id.to_string(),
            name: record.name.clone(),
            image: record.image.clone(),
            createdat: record.created_at.clone(),
            updatedat: record.updated_at.clone(),
            chatmessages,
            chatparticipants,
        }
    }
}
```

### 4.7 Comment Query Resolvers (`schema/query/comments.rs`)

```rust
use async_graphql::{Context, Object, ID};
use uuid::Uuid;

use crate::state::SharedState;
use crate::types::comment::*;
use crate::types::registration::DefaultResponse;

pub struct CommentQuery;

#[Object]
impl CommentQuery {
    /// Fetch top-level comments for a post.
    ///
    /// Response codes:
    /// - 11601: Comments retrieved
    /// - 21601: No comments found
    /// - 30209: Invalid post UUID
    /// - 30215: Invalid comment offset
    /// - 30216: Invalid comment limit
    /// - 60501: Not authenticated
    async fn list_comments(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<String>,
        #[graphql(name = "commentOffset")] comment_offset: Option<i32>,
        #[graphql(name = "commentLimit")] comment_limit: Option<i32>,
    ) -> CommentListResponse {
        // Auth: optional (pass viewer_id if available)
        let viewer_id = ctx.data_opt::<Uuid>().copied();

        let state = ctx.data_unchecked::<SharedState>().read().await;

        // Validate post UUID
        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return CommentListResponse {
                meta: DefaultResponse::error("30209", "Invalid post UUID"),
                counter: 0,
                affected_rows: None,
            },
        };

        // Validate post exists
        if !state.posts.iter().any(|p| p.id == post_uuid) {
            return CommentListResponse {
                meta: DefaultResponse::error("30209", "Post not found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let offset = comment_offset.unwrap_or(0).max(0) as usize;
        let limit = comment_limit.unwrap_or(10).clamp(1, 20) as usize;

        // Filter top-level comments for this post
        let top_level: Vec<&CommentRecord> = state.comments.iter()
            .filter(|c| {
                c.post_id == post_uuid
                    && c.parent_id.is_none()
                    && c.visibility_status == "VISIBLE"
                    && !state.deleted_users.contains(&c.author_id)
            })
            .collect();

        let total = top_level.len();

        if total == 0 {
            return CommentListResponse {
                meta: DefaultResponse::success("21601", "No comments found"),
                counter: 0,
                affected_rows: None,
            };
        }

        // Paginate
        let end = (offset + limit).min(total);
        let page = if offset < total { &top_level[offset..end] } else { &[] };

        let comments: Vec<Comment> = page.iter()
            .map(|r| state.comment_record_to_graphql(r, viewer_id))
            .collect();

        CommentListResponse {
            meta: DefaultResponse::success("11601", "Comments retrieved"),
            counter: total as i32,
            affected_rows: Some(comments),
        }
    }

    /// Fetch replies to a top-level comment.
    ///
    /// Response codes:
    /// - 11607: Child comments retrieved
    /// - 21606: No child comments found
    /// - 30209: Invalid parent UUID
    /// - 60501: Not authenticated
    async fn list_child_comments(
        &self,
        ctx: &Context<'_>,
        parent: ID,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> CommentListResponse {
        let viewer_id = ctx.data_opt::<Uuid>().copied();
        let state = ctx.data_unchecked::<SharedState>().read().await;

        let parent_uuid = match parent.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return CommentListResponse {
                meta: DefaultResponse::error("30209", "Invalid parent UUID"),
                counter: 0,
                affected_rows: None,
            },
        };

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(10).clamp(1, 20) as usize;

        // Filter child comments (no content filtering on child comments per API spec)
        let children: Vec<&CommentRecord> = state.comments.iter()
            .filter(|c| {
                c.parent_id == Some(parent_uuid)
                    && c.visibility_status == "VISIBLE"
                    && !state.deleted_users.contains(&c.author_id)
            })
            .collect();

        let total = children.len();

        if total == 0 {
            return CommentListResponse {
                meta: DefaultResponse::success("21606", "No child comments found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let end = (off + lim).min(total);
        let page = if off < total { &children[off..end] } else { &[] };

        let comments: Vec<Comment> = page.iter()
            .map(|r| state.comment_record_to_graphql(r, viewer_id))
            .collect();

        CommentListResponse {
            meta: DefaultResponse::success("11607", "Child comments retrieved"),
            counter: total as i32,
            affected_rows: Some(comments),
        }
    }
}
```

### 4.8 Comment Mutation Resolvers (`schema/mutation/comment.rs`)

```rust
use async_graphql::{Context, Object, ID};
use chrono::Utc;
use uuid::Uuid;

use crate::schema::require_auth;
use crate::state::SharedState;
use crate::types::comment::*;
use crate::types::registration::DefaultResponse;

/// Daily free comment limit.
const DAILY_FREE_COMMENTS: u32 = 4;

pub struct CommentMutation;

#[Object]
impl CommentMutation {
    /// Create a new comment or reply on a post.
    ///
    /// Response codes:
    /// - 11605: Comment created (paid)
    /// - 11608: Comment created (free daily action)
    /// - 30101: Missing required fields
    /// - 30265: Missing content or postid
    /// - 30209: Invalid post UUID
    /// - 31602: Post not found
    /// - 31603: Invalid parent comment UUID
    /// - 41604: Parent must be top-level comment
    /// - 60501: Not authenticated
    async fn create_comment(
        &self,
        ctx: &Context<'_>,
        action: CommentType,
        postid: ID,
        content: String,
        parentid: Option<ID>,
    ) -> CreateCommentResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return CreateCommentResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                counter: 0,
                affected_rows: None,
            },
        };

        // Validate content length (1–200 chars)
        if content.is_empty() || content.len() > 200 {
            return CreateCommentResponse {
                meta: DefaultResponse::error("30265", "Content must be 1-200 characters"),
                counter: 0,
                affected_rows: None,
            };
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Validate post UUID
        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return CreateCommentResponse {
                meta: DefaultResponse::error("30209", "Invalid post UUID"),
                counter: 0,
                affected_rows: None,
            },
        };

        // Validate post exists
        if !state.posts.iter().any(|p| p.id == post_uuid) {
            return CreateCommentResponse {
                meta: DefaultResponse::error("31602", "Post not found"),
                counter: 0,
                affected_rows: None,
            };
        }

        // Validate parent comment (if reply)
        let parent_uuid = match &parentid {
            Some(pid) => {
                let uuid = match pid.to_string().parse::<Uuid>() {
                    Ok(u) => u,
                    Err(_) => return CreateCommentResponse {
                        meta: DefaultResponse::error("31603", "Invalid parent comment UUID"),
                        counter: 0,
                        affected_rows: None,
                    },
                };
                // Parent must exist
                let parent = match state.comments.iter().find(|c| c.id == uuid) {
                    Some(c) => c,
                    None => return CreateCommentResponse {
                        meta: DefaultResponse::error("31603", "Parent comment not found"),
                        counter: 0,
                        affected_rows: None,
                    },
                };
                // Parent must be top-level (no nested replies beyond 1 level)
                if parent.parent_id.is_some() {
                    return CreateCommentResponse {
                        meta: DefaultResponse::error("41604", "Parent must be a top-level comment"),
                        counter: 0,
                        affected_rows: None,
                    };
                }
                // Parent must belong to same post
                if parent.post_id != post_uuid {
                    return CreateCommentResponse {
                        meta: DefaultResponse::error("31603", "Parent comment belongs to different post"),
                        counter: 0,
                        affected_rows: None,
                    };
                }
                Some(uuid)
            }
            None => None,
        };

        // Daily free action tracking
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let key = (user_id, today.clone());
        let count = state.daily_comment_count.get(&key).copied().unwrap_or(0);
        let is_free = count < DAILY_FREE_COMMENTS;

        // TODO: Phase 5 — deduct tokens if !is_free and balance insufficient → return 51301

        // Create the comment
        let comment_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        let record = CommentRecord {
            id: comment_id,
            author_id: user_id,
            post_id: post_uuid,
            parent_id: parent_uuid,
            content: content.clone(),
            created_at: now.clone(),
            visibility_status: "VISIBLE".into(),
        };

        state.comments.push(record);

        // Update daily count
        *state.daily_comment_count.entry(key).or_insert(0) += 1;

        // Build response
        let comment = state.comment_record_to_graphql(
            state.comments.last().unwrap(),
            Some(user_id),
        );

        let code = if is_free { "11608" } else { "11605" };
        let message = if is_free { "Comment created (free daily action)" } else { "Comment created" };

        CreateCommentResponse {
            meta: DefaultResponse::success(code, message),
            counter: 1,
            affected_rows: Some(vec![comment]),
        }
    }

    /// Like a comment.
    ///
    /// Response codes:
    /// - 11603: Comment liked
    /// - 30201: Invalid comment UUID
    /// - 31601: Comment not found
    /// - 31604: Already liked
    /// - 31606: Cannot like own comment
    /// - 60501: Not authenticated
    async fn like_comment(
        &self,
        ctx: &Context<'_>,
        commentid: ID,
    ) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let comment_uuid = match commentid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return DefaultResponse::error("30201", "Invalid comment UUID"),
        };

        // Find the comment
        let comment = match state.comments.iter().find(|c| c.id == comment_uuid) {
            Some(c) => c,
            None => return DefaultResponse::error("31601", "Comment not found"),
        };

        // Cannot like own comment
        if comment.author_id == user_id {
            return DefaultResponse::error("31606", "Cannot like own comment");
        }

        // Check duplicate
        if state.comment_likes.contains(&(user_id, comment_uuid)) {
            return DefaultResponse::error("31604", "Already liked");
        }

        state.comment_likes.insert((user_id, comment_uuid));

        DefaultResponse::success("11603", "Comment liked")
    }

    /// Unlike a comment.
    ///
    /// Response codes:
    /// - 11603: Comment unliked
    /// - 30201: Invalid comment UUID
    /// - 31601: Comment not found
    /// - 60501: Not authenticated
    async fn unlike_comment(
        &self,
        ctx: &Context<'_>,
        commentid: ID,
    ) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let comment_uuid = match commentid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return DefaultResponse::error("30201", "Invalid comment UUID"),
        };

        // Find the comment
        if !state.comments.iter().any(|c| c.id == comment_uuid) {
            return DefaultResponse::error("31601", "Comment not found");
        }

        state.comment_likes.remove(&(user_id, comment_uuid));

        DefaultResponse::success("11603", "Comment unliked")
    }

    /// Report a comment for moderation.
    ///
    /// Response codes:
    /// - 11604: Comment reported
    /// - 30201: Invalid comment UUID
    /// - 31601: Comment not found
    /// - 31605: Already reported
    /// - 31607: Cannot report own comment
    /// - 60501: Not authenticated
    async fn report_comment(
        &self,
        ctx: &Context<'_>,
        commentid: ID,
    ) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let comment_uuid = match commentid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return DefaultResponse::error("30201", "Invalid comment UUID"),
        };

        let comment = match state.comments.iter().find(|c| c.id == comment_uuid) {
            Some(c) => c,
            None => return DefaultResponse::error("31601", "Comment not found"),
        };

        if comment.author_id == user_id {
            return DefaultResponse::error("31607", "Cannot report own comment");
        }

        if state.comment_reports.contains(&(user_id, comment_uuid)) {
            return DefaultResponse::error("31605", "Already reported");
        }

        state.comment_reports.insert((user_id, comment_uuid));

        DefaultResponse::success("11604", "Comment reported")
    }
}
```

### 4.9 Chat Query Resolvers (`schema/query/chat.rs`)

```rust
use async_graphql::{Context, Object};

use crate::schema::require_auth;
use crate::state::SharedState;
use crate::types::chat::*;
use crate::types::registration::DefaultResponse;

pub struct ChatQuery;

#[Object]
impl ChatQuery {
    /// List chats for the authenticated user.
    ///
    /// Returns chats sorted by most recently active (updatedat descending).
    /// Each chat includes all messages and participants.
    ///
    /// Response codes:
    /// - 11801: Chats retrieved
    /// - 21801: No chats found
    /// - 60501: Not authenticated
    async fn list_chats(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> ListChatsResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return ListChatsResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 50) as usize;

        // Filter chats where user is a participant, sorted by updatedat descending
        let mut user_chats: Vec<&ChatRecord> = state.chats.iter()
            .filter(|c| c.participant_ids.contains(&user_id))
            .collect();

        user_chats.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        if user_chats.is_empty() {
            return ListChatsResponse {
                meta: DefaultResponse::success("21801", "No chats found"),
                affected_rows: None,
            };
        }

        // Paginate
        let end = (off + lim).min(user_chats.len());
        let page = if off < user_chats.len() { &user_chats[off..end] } else { &[] };

        let chats: Vec<Chat> = page.iter()
            .map(|r| state.chat_record_to_graphql(r))
            .collect();

        ListChatsResponse {
            meta: DefaultResponse::success("11801", "Chats retrieved"),
            affected_rows: Some(chats),
        }
    }
}
```

### 4.10 Chat Mutation Resolvers (`schema/mutation/chat.rs`)

```rust
use async_graphql::{Context, InputObject, Object, ID};
use chrono::Utc;
use uuid::Uuid;

use crate::schema::require_auth;
use crate::state::SharedState;
use crate::types::chat::*;
use crate::types::registration::DefaultResponse;

/// Input for createChat mutation.
#[derive(InputObject, Clone, Debug)]
pub struct CreateChatInput {
    pub name: String,
    pub recipients: Vec<String>,
    pub image: Option<String>,
}

pub struct ChatMutation;

#[Object]
impl ChatMutation {
    /// Create a new chat (private 1:1 or group).
    ///
    /// For private chats (1 recipient): returns existing chat if already exists.
    /// For group chats (2+ recipients): always creates new chat.
    ///
    /// Response codes:
    /// - 11802: Chat created
    /// - 11803: Existing chat returned (private, already exists)
    /// - 30301: Missing recipients
    /// - 30302: Invalid recipient UUID
    /// - 60501: Not authenticated
    async fn create_chat(
        &self,
        ctx: &Context<'_>,
        input: CreateChatInput,
    ) -> CreateChatResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return CreateChatResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        if input.recipients.is_empty() {
            return CreateChatResponse {
                meta: DefaultResponse::error("30301", "At least one recipient is required"),
                affected_rows: None,
            };
        }

        // Parse recipient UUIDs
        let mut recipient_ids: Vec<Uuid> = Vec::new();
        for r in &input.recipients {
            match r.parse::<Uuid>() {
                Ok(u) => recipient_ids.push(u),
                Err(_) => return CreateChatResponse {
                    meta: DefaultResponse::error("30302", "Invalid recipient UUID"),
                    affected_rows: None,
                },
            }
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // For private chats (1 recipient): check if chat already exists
        if recipient_ids.len() == 1 {
            let other = recipient_ids[0];
            if let Some(existing) = state.chats.iter().find(|c| {
                c.name.is_none()
                    && c.participant_ids.len() == 2
                    && c.participant_ids.contains(&user_id)
                    && c.participant_ids.contains(&other)
            }) {
                return CreateChatResponse {
                    meta: DefaultResponse::success("11803", "Chat already exists"),
                    affected_rows: Some(CreateChatResult {
                        chatid: existing.id.to_string(),
                    }),
                };
            }
        }

        // Create new chat
        let chat_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        let mut participant_ids_all = vec![user_id];
        participant_ids_all.extend(recipient_ids);

        // Private chats have no name/image; group chats use the provided values
        let is_group = participant_ids_all.len() > 2;
        let name = if is_group { Some(input.name) } else { None };
        let image = if is_group { input.image } else { None };

        let record = ChatRecord {
            id: chat_id,
            name,
            image,
            created_at: now.clone(),
            updated_at: now,
            participant_ids: participant_ids_all,
        };

        state.chats.push(record);

        CreateChatResponse {
            meta: DefaultResponse::success("11802", "Chat created"),
            affected_rows: Some(CreateChatResult {
                chatid: chat_id.to_string(),
            }),
        }
    }

    /// Send a message to a chat.
    ///
    /// Response codes:
    /// - 11804: Message sent
    /// - 30303: Invalid chat UUID
    /// - 30304: Chat not found
    /// - 30305: Not a participant
    /// - 30306: Message too long (max 500)
    /// - 30307: Message cannot be empty
    /// - 60501: Not authenticated
    async fn send_chat_message(
        &self,
        ctx: &Context<'_>,
        chatid: ID,
        content: String,
    ) -> SendMessageResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return SendMessageResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        // Validate content
        if content.is_empty() {
            return SendMessageResponse {
                meta: DefaultResponse::error("30307", "Message cannot be empty"),
                affected_rows: None,
            };
        }
        if content.len() > 500 {
            return SendMessageResponse {
                meta: DefaultResponse::error("30306", "Message must be 500 characters or fewer"),
                affected_rows: None,
            };
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Parse chat UUID
        let chat_uuid = match chatid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return SendMessageResponse {
                meta: DefaultResponse::error("30303", "Invalid chat UUID"),
                affected_rows: None,
            },
        };

        // Find chat
        let chat = match state.chats.iter_mut().find(|c| c.id == chat_uuid) {
            Some(c) => c,
            None => return SendMessageResponse {
                meta: DefaultResponse::error("30304", "Chat not found"),
                affected_rows: None,
            },
        };

        // Check participant
        if !chat.participant_ids.contains(&user_id) {
            return SendMessageResponse {
                meta: DefaultResponse::error("30305", "Not a participant in this chat"),
                affected_rows: None,
            };
        }

        // Update chat timestamp
        let now = Utc::now().to_rfc3339();
        chat.updated_at = now.clone();

        // Create message
        let msg_id = Uuid::new_v4();
        let message_record = ChatMessageRecord {
            id: msg_id,
            sender_id: user_id,
            chat_id: chat_uuid,
            content: content.clone(),
            created_at: now.clone(),
        };

        state.chat_messages.push(message_record);

        let response_message = ChatMessage {
            id: msg_id.to_string(),
            senderid: user_id.to_string(),
            chatid: chat_uuid.to_string(),
            content,
            createdat: now,
        };

        SendMessageResponse {
            meta: DefaultResponse::success("11804", "Message sent"),
            affected_rows: Some(response_message),
        }
    }
}
```

### 4.11 Schema Assembly Updates

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
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    RegistrationMutation,   // Phase 0
    AuthMutation,           // Phase 1
    ProfileMutation,        // Phase 2
    PostMutation,           // Phase 3
    CommentMutation,        // Phase 4
    ChatMutation,           // Phase 4
);
```

### 4.12 Update Post Comment Counts (Cross-Cutting)

Update `post_record_to_graphql()` in the existing Phase 3 code to compute comment counts from state:

```rust
// In post_record_to_graphql(), replace:
//   let amountcomments = 0i32;
// With:
let amountcomments = self.comments.iter()
    .filter(|c| c.post_id == record.id && c.visibility_status == "VISIBLE")
    .count() as i32;
```

Also update the trending score and `Comments` sort in `sort_posts()`:

```rust
// In sort_posts(), update the Comments branch:
PostSortType::Comments => {
    posts.sort_by(|a, b| {
        let a_c = state.comments.iter().filter(|c| c.post_id == a.id && c.visibility_status == "VISIBLE").count();
        let b_c = state.comments.iter().filter(|c| c.post_id == b.id && c.visibility_status == "VISIBLE").count();
        b_c.cmp(&a_c)
    });
}
```

---

## 5. Testing Strategy

### Comment Tests (≥18)

| # | Test | Assert |
|---|------|--------|
| 1 | List comments on post with seed data | `11601`, `counter >= 2`, `affectedRows` contains seed comments |
| 2 | List comments on post with no comments | `21601`, `counter: 0`, `affectedRows: null` |
| 3 | List comments with pagination (offset=0, limit=1) | Returns 1 comment, `counter` is total |
| 4 | List comments with offset beyond range | `21601` or empty list |
| 5 | List comments with invalid post UUID | `30209` |
| 6 | List child comments for parent with replies | `11607`, returns reply comments |
| 7 | List child comments with no replies | `21606`, `counter: 0` |
| 8 | List child comments with invalid parent UUID | `30209` |
| 9 | Create top-level comment | `11608` (free) or `11605` (paid), returned comment has correct fields |
| 10 | Create reply comment | Success, `parentid` set in response |
| 11 | Create comment with empty content | `30265` |
| 12 | Create comment with content > 200 chars | `30265` |
| 13 | Create comment on non-existent post | `31602` |
| 14 | Create reply to non-existent parent | `31603` |
| 15 | Create reply to a reply (nested > 1 level) | `41604` |
| 16 | Create reply to comment on different post | `31603` |
| 17 | Create comment without auth | `60501` |
| 18 | Created comment appears in listComments | Create → list → verify present |
| 19 | Like comment | `11603`, `amountlikes` incremented, `isliked` true |
| 20 | Like own comment | `31606` |
| 21 | Like comment twice (duplicate) | `31604` |
| 22 | Unlike comment | `11603`, `amountlikes` decremented, `isliked` false |
| 23 | Like non-existent comment | `31601` |
| 24 | Like comment without auth | `60501` |
| 25 | Report comment | `11604` |
| 26 | Report own comment | `31607` |
| 27 | Report comment twice (duplicate) | `31605` |
| 28 | Daily free action (first 4 comments → `11608`) | Create 4 comments → all `11608` |
| 29 | Daily paid action (5th comment → `11605`) | Create 5th comment → `11605` |

### Chat Tests (≥12)

| # | Test | Assert |
|---|------|--------|
| 30 | List chats with seed data | Success, returns chats with participants and messages |
| 31 | List chats for user with no chats | `21801` |
| 32 | List chats without auth | `60501` |
| 33 | Chat includes correct participants | Verify `chatparticipants` has correct users |
| 34 | Chat includes messages ordered by time | Verify `chatmessages` ordered by `createdat` |
| 35 | Private chat shows no name/image | `name: null`, `image: null` |
| 36 | Group chat shows name | `name: "Rust Developers"` |
| 37 | Create private chat | `11802`, returns `chatid` |
| 38 | Create private chat that already exists | `11803`, returns existing `chatid` |
| 39 | Create group chat | `11802`, returns `chatid` |
| 40 | Create chat without recipients | `30301` |
| 41 | Create chat without auth | `60501` |
| 42 | Send message to chat | `11804`, returned message has correct fields |
| 43 | Send message to non-existent chat | `30304` |
| 44 | Send message to chat user is not in | `30305` |
| 45 | Send empty message | `30307` |
| 46 | Send message > 500 chars | `30306` |
| 47 | Send message without auth | `60501` |
| 48 | Send message updates chat's updatedat | Send → list → verify `updatedat` changed |
| 49 | Sent message appears in listChats | Send → list → verify message in `chatmessages` |

### Cross-Cutting Tests (≥3)

| # | Test | Assert |
|---|------|--------|
| 50 | Post amountcomments reflects actual comment count | Create comment → get post → verify `amountcomments` incremented |
| 51 | Reset clears all comment and chat state | POST /reset → list comments empty, list chats empty |
| 52 | All Phase 0–3 tests still pass | Regression check |

---

## 6. Definition of Done

### Phase 4 gate

- [x] `listComments` returns paginated top-level comments for a post with correct `CommentUser` embedding
- [x] `listChildComments` returns replies to a top-level comment
- [x] `createComment` creates both top-level and reply comments with proper nesting validation
- [x] `likeComment` / `unlikeComment` toggle likes with duplicate and self-like prevention
- [x] `reportComment` works with duplicate and self-report prevention
- [x] Daily free action logic: first 4 comments per day return `11608`, subsequent return `11605`
- [x] `listChats` returns chats with participants and messages, sorted by most recent activity
- [x] `createChat` creates private (1:1) and group chats, with deduplication for private chats
- [x] `sendChatMessage` appends messages and updates chat timestamp
- [x] Post `amountcomments` field computed from actual comment count
- [x] `POST /reset` clears all comment and chat state back to seed data
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes
- [x] ≥52 integration tests pass (29 comments + 20 chat + 3 cross-cutting)
- [x] All Phase 0–3 tests still pass (regression)

### Response code summary

| Code | Meaning | Operation |
|------|---------|-----------|
| `11601` | Comments retrieved | `listComments` |
| `11603` | Comment liked / unliked | `likeComment` / `unlikeComment` |
| `11604` | Comment reported | `reportComment` |
| `11605` | Comment created (paid) | `createComment` |
| `11607` | Child comments retrieved | `listChildComments` |
| `11608` | Comment created (free daily) | `createComment` |
| `11801` | Chats retrieved | `listChats` |
| `11802` | Chat created | `createChat` |
| `11803` | Existing chat returned | `createChat` (private dedup) |
| `11804` | Message sent | `sendChatMessage` |
| `21601` | No comments found | `listComments` |
| `21606` | No child comments | `listChildComments` |
| `21801` | No chats found | `listChats` |
| `30201` | Invalid comment UUID | `likeComment` / `unlikeComment` / `reportComment` |
| `30209` | Invalid post/parent UUID | `listComments` / `createComment` |
| `30265` | Invalid comment content | `createComment` |
| `30301` | Missing recipients | `createChat` |
| `30302` | Invalid recipient UUID | `createChat` |
| `30303` | Invalid chat UUID | `sendChatMessage` |
| `30304` | Chat not found | `sendChatMessage` |
| `30305` | Not a participant | `sendChatMessage` |
| `30306` | Message too long | `sendChatMessage` |
| `30307` | Message empty | `sendChatMessage` |
| `31601` | Comment not found | `likeComment` / `unlikeComment` / `reportComment` |
| `31602` | Post not found | `createComment` |
| `31603` | Parent comment invalid | `createComment` |
| `31604` | Already liked | `likeComment` |
| `31605` | Already reported | `reportComment` |
| `31606` | Cannot like own comment | `likeComment` |
| `31607` | Cannot report own comment | `reportComment` |
| `41604` | Parent must be top-level | `createComment` |
| `51301` | Insufficient balance | `createComment` (Phase 5) |
| `60501` | Not authenticated | All auth-required operations |
