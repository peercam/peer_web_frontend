# View Post — Completion Sprint Plan

**Feature:** View Post (#4)  
**Priority:** Next after Dashboard (#3)  
**Status:** � Nearly complete — polish & testing remain  
**Created:** 2026-04-14  
**Updated:** 2026-04-14

---

## Summary

View Post is the **next feature** in the migration priority after Dashboard. The Dashboard post feed is in progress — clicking a post card needs to open View Post. This sprint document audits the existing code against the original planning doc and identifies the **remaining work** to bring View Post to ✅ Implemented.

**Verdict:** ~95% of the Rust component/API code is written. All core functionality is implemented. The remaining gaps are error recovery toasts, mock backend fixtures, E2E tests, responsive polish, and dashboard overlay integration.

---

## Code Audit

### What Exists ✅

| Layer | File | Lines | Status |
|-------|------|-------|--------|
| **Page** | `src/pages/view_post.rs` | 219 | ✅ Complete — guest/auth routing, SEO meta, skeleton, error state, guest CTA |
| **Route** | `src/app.rs` | — | ✅ `/post/:id` and `/p/:id` registered |
| **API — Guest Post** | `src/api/comments.rs` → `guest_get_post` | — | ✅ Server fn wired to `GUEST_POST_QUERY` |
| **API — Auth Post** | `src/api/comments.rs` → `get_post` | — | ✅ Server fn wired to `GET_POST_QUERY` |
| **API — List Comments** | `src/api/comments.rs` → `list_comments` | — | ✅ Paginated, offset/limit, optional auth |
| **API — Child Comments** | `src/api/comments.rs` → `list_child_comments` | — | ✅ Paginated replies |
| **API — Create Comment** | `src/api/comments.rs` → `create_comment` | — | ✅ Auth required, optional parent_id |
| **API — Like/Unlike Comment** | `src/api/comments.rs` → `like_comment`, `unlike_comment` | — | ✅ Optimistic update in component |
| **API — Post Actions** | `src/api/posts.rs` → `post_action` | — | ✅ Like/dislike/save/report/view |
| **API — Follow** | `src/api/profile.rs` → `toggle_follow` | — | ✅ Exists, wired in PostHeader with optimistic update |
| **GraphQL Queries** | `src/api/graphql.rs` | — | ✅ All 7 queries/mutations defined (`GUEST_POST_QUERY`, `GET_POST_QUERY`, `LIST_COMMENTS_QUERY`, `LIST_CHILD_COMMENTS_QUERY`, `CREATE_COMMENT_MUTATION`, `LIKE_COMMENT_MUTATION`, `UNLIKE_COMMENT_MUTATION`) |
| **Models — Post** | `src/models/post.rs` | — | ✅ `Post`, `PostUser`, `ContentType`, `PostActionType` |
| **Models — Comment** | `src/models/comment.rs` | — | ✅ `Comment`, `CommentUser`, `CommentListResponse`, `CreateCommentResponse`, `LikeCommentResponse` |
| **Component — PostMedia** | `src/components/view_post/post_media.rs` | 150 | ✅ Image gallery (slider, dots, thumbnails), video player, audio player, text display |
| **Component — ImageModal** | `src/components/view_post/image_modal.rs` | ~80 | ✅ Fullscreen viewer, nav arrows, Escape key close, counter |
| **Component — PostHeader** | `src/components/view_post/post_header.rs` | ~70 | ✅ Complete — follow/unfollow wired with optimistic update |
| **Component — PostContent** | `src/components/view_post/post_content.rs` | ~65 | ✅ Title, description, tags, timestamp |
| **Component — PostActions** | `src/components/view_post/post_actions.rs` | ~170 | ✅ Like/dislike/save/share/report with optimistic updates |
| **Component — Comments** | `src/components/view_post/comments.rs` | 150+ | ✅ Infinite scroll, CommentItem, CommentInput, ReplyForm, ChildComments, like toggle |
| **Component — ShareModal** | `src/components/view_post/share_modal.rs` | ~120 | ✅ Copy link (clipboard API), WhatsApp, Telegram, overlay close |
| **SCSS** | `style/view-post.scss` | 952 | ✅ Desktop layout, gallery, actions, comments, share modal, responsive |

### What Was Completed ✅ (since initial audit)

| # | Task | File(s) | Notes |
|---|------|---------|-------|
| 1 | **Wire follow/unfollow API in PostHeader** | `components/view_post/post_header.rs` | ✅ `toggle_follow` called via `spawn_local` with optimistic update and revert on failure |
| 2 | **Add `use_mobile_redirect` hook** | `src/hooks/use_mobile_redirect.rs`, `src/hooks/mod.rs` | ✅ Detects Android/iOS via user agent, attempts `peer://post/{id}` deep link |
| 3 | **Call mobile redirect from ViewPostPage** | `src/pages/view_post.rs` | ✅ Hook invoked for guest mobile visitors |
| 4 | **Track post view on load** | `src/pages/view_post.rs` | ✅ Fire-and-forget `post_action(View)` once on mount for authenticated users |
| 5 | **`format_time_ago` — proper relative time** | `components/view_post/post_content.rs` | ✅ Full `chrono`-based implementation: just now, min, h, d, w, fallback to date |

### What Remains 🔲

| # | Task | File(s) | Effort | Notes |
|---|------|---------|--------|-------|
| 6 | **Error recovery for API failures** | `components/view_post/comments.rs`, `post_actions.rs` | M | Show toast on failed like/comment/action, revert optimistic state |
| 7 | **Mock backend: Posts & Comments (Phase 3)** | `packages/mock_backend/` | L | See section below |
| 8 | **E2E tests (Playwright)** | `end2end/` | L | See testing section below |
| 9 | **SCSS responsive polish** | `style/view-post.scss` | M | Test on actual mobile viewports, verify touch interactions |
| 10 | **Dashboard → View Post overlay integration** | `src/components/dashboard/`, `src/pages/dashboard.rs` | M | Clicking a PostCard from the feed should open View Post as a modal overlay (not full-page nav). Needs `show_post_overlay` signal + conditional rendering. Deferred until Dashboard is further along. |

**Legend:** S = Small (< 1 hour), M = Medium (1–3 hours), L = Large (3+ hours)

---

## Task Details

### Tasks 1–5: ✅ Completed

All five quick-win and quality-polish tasks have been implemented:

- **Task 1 (Follow API):** `PostHeader` calls `toggle_follow` with optimistic update and failure revert via `StoredValue` + `spawn_local`.
- **Task 2 (Mobile redirect hook):** `src/hooks/use_mobile_redirect.rs` detects Android/iOS via user agent and attempts `peer://post/{id}` deep link. Registered in `src/hooks/mod.rs`.
- **Task 3 (Mobile redirect call):** `ViewPostPage` invokes `use_mobile_redirect(post_id_signal, is_guest)` for guest mobile visitors.
- **Task 4 (View tracking):** `ViewPostPage` calls `post_action(id, PostActionType::View)` fire-and-forget once on mount for authenticated users via `Effect`.
- **Task 5 (Relative time):** `format_time_ago` uses `chrono::NaiveDateTime` parsing with fallback, returning "just now", "N min ago", "N h ago", "N d ago", "N w ago", or the date portion for older posts.

---

## Mock Backend: Phase 3 — Posts & Comments

The current mock backend (`packages/mock_backend/`) only supports registration. For View Post (and Dashboard) to be testable end-to-end, the mock needs these operations:

### Required Operations

| Operation | Type | Auth | Priority |
|-----------|------|------|----------|
| `guestListPost` | Query | No | **P0** — guest post view |
| `listPosts` (single post by `postid`) | Query | Yes | **P0** — auth post view |
| `resolvePostAction` | Mutation | Yes | P1 — like/dislike/save/view |
| `listComments` | Query | Optional | P1 — comment display |
| `listChildComments` | Query | Optional | P1 — nested replies |
| `createComment` | Mutation | Yes | P2 — comment creation |
| `likeComment` / `unlikeComment` | Mutation | Yes | P2 — comment interaction |

### Fixture Data Needed

```
fixtures/
├── posts/
│   ├── post-image.json       # Image post with multiple media URLs
│   ├── post-video.json       # Video post with cover
│   ├── post-audio.json       # Audio post with cover
│   └── post-text.json        # Text-only post
└── comments/
    ├── comments-page-1.json  # First 10 comments
    ├── comments-page-2.json  # Next 10 comments
    └── child-comments.json   # Replies to a comment
```

### Schema Additions (for Node.js mock)

```graphql
# Add to schema.graphql
type PostUser {
  id: ID!
  username: String!
  slug: String!
  img: String
  isfollowed: Boolean
  isfollowing: Boolean
  isfriend: Boolean
}

type Post {
  id: ID!
  contenttype: String!
  title: String!
  media: String
  cover: String
  mediadescription: String
  createdat: String!
  amountlikes: Int!
  amountviews: Int!
  amountcomments: Int!
  amountdislikes: Int!
  tags: [String]
  isliked: Boolean
  isdisliked: Boolean
  issaved: Boolean
  isreported: Boolean
  isviewed: Boolean
  user: PostUser!
}

type PostListResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [Post!]
}

type CommentUser {
  id: ID!
  username: String!
  slug: String!
  img: String
  isfollowed: Boolean
  isfollowing: Boolean
}

type Comment {
  commentid: ID!
  userid: ID!
  postid: ID!
  parentid: ID
  content: String!
  createdat: String!
  amountlikes: Int!
  amountreplies: Int!
  isliked: Boolean
  user: CommentUser!
}

type CommentListResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [Comment!]
}

type PostActionResponse {
  meta: DefaultResponse!
}

type LikeCommentResponse {
  status: String!
  ResponseCode: String!
  ResponseMessage: String
}

extend type Query {
  guestListPost(postid: ID!): PostListResponse!
  listPosts(postid: ID, limit: Int, offset: Int, filterBy: [String], sortBy: String): PostListResponse!
  listComments(postid: ID!, commentOffset: Int, commentLimit: Int): CommentListResponse!
  listChildComments(parent: ID!, offset: Int, limit: Int): CommentListResponse!
}

extend type Mutation {
  resolvePostAction(postid: ID!, action: String!): PostActionResponse!
  createComment(action: String!, postid: ID!, content: String!, parentid: ID): CommentListResponse!
  likeComment(commentid: ID!): LikeCommentResponse!
}
```

> **Note:** If the Rust mock backend rewrite (Phase 0) is complete by then, implement Phase 3 in Rust instead. See [mock-backend-rust-rewrite.md](../mock-backend/mock-backend-rust-rewrite.md).

---

## Testing Plan

### Unit Tests

| Test | Location | Status |
|------|----------|--------|
| Comment model deserialization (nested fields, defaults) | `src/models/comment.rs` | 🔲 |
| Post model deserialization (all content types) | `src/models/post.rs` | 🔲 |
| `format_time_ago` (various timestamps) | `src/components/view_post/post_content.rs` | 🔲 |
| `parse_media_urls` (JSON array, CSV, single URL) | `src/components/view_post/post_media.rs` | 🔲 |
| Email masking (`mask_email`) — borrow from forgot_password tests | — | 🔲 |

### E2E Tests (Playwright)

| Scenario | Auth | Assertions |
|----------|------|------------|
| Guest views post at `/post/{id}` | No | Post media visible, title/description shown, "View-only mode" banner, "Sign Up" CTA |
| Guest cannot interact | No | Like/dislike/save buttons disabled or hidden, no comment input |
| Guest sees comments (read-only) | No | Comments list loads, no reply buttons |
| Auth user views post | Yes | Full post data, interaction buttons enabled, comment input visible |
| Auth user likes/unlikes post | Yes | Like count changes, icon state toggles |
| Auth user dislikes post | Yes | Dislike count changes, like auto-removes if active |
| Auth user saves post | Yes | Save button toggles state |
| Auth user creates comment | Yes | Comment appears at top of list, count increments |
| Auth user likes comment | Yes | Comment like count changes |
| Auth user views replies | Yes | "Show N replies..." toggleable, child comments load |
| Auth user replies to comment | Yes | Reply form appears, new reply confirmed |
| Share — copy link | Any | Clipboard contains correct URL, "Copied!" feedback |
| Share — WhatsApp/Telegram | Any | Link opens correct share URL |
| Image gallery navigation | Any | Arrows work, dots/thumbnails update, fullscreen modal opens |
| Video post | Any | Video element present, cover image shown, plays |
| Audio post | Any | Audio element present, cover shown |
| Post not found | Any | Error state shown, "Go to Dashboard" link |
| Short URL `/p/{id}` | Any | Same as `/post/{id}` |

---

## Sprint Sequence

**Recommended order for remaining implementation:**

```
Phase A — Quick wins (Tasks 1–5)          ✅ DONE

Phase B — Quality polish (Task 6)         ~2 hours
  6. Add error recovery (toasts, optimistic revert)

Phase C — Mock backend (Task 7)           ~4 hours
  7. Add posts/comments to mock backend
     (P0 operations first, then P1/P2)

Phase D — Testing (Task 8)                ~4 hours
  8. Write E2E tests against mock backend

Phase E — Integration (Tasks 9–10)        Deferred
  9. SCSS responsive polish (after visual QA)
  10. Dashboard overlay integration (after Dashboard completes)
```

**Remaining estimated effort:** Phases B–D ≈ 10 hours of focused work.

---

## Dependencies

| Dependency | Status | Blocker? |
|------------|--------|----------|
| `Post`, `PostUser` models | ✅ Exists | No |
| `Comment`, `CommentUser` models | ✅ Exists | No |
| Auth context (`use_auth`) | ✅ Exists | No |
| GraphQL client (`query`, `mutate`) | ✅ Exists | No |
| `toggle_follow` API | ✅ Exists, wired | No |
| `post_action` API | ✅ Exists, wired | No |
| Mock backend posts/comments | 🔲 Needed | **Yes** — blocks E2E tests |
| Dashboard PostCard click handler | 🔲 In progress | No — View Post works standalone at `/post/:id` |
| `chrono` crate (for `format_time_ago`) | ✅ In use | No |

---

## Acceptance Criteria

Before marking View Post as ✅ Implemented in the convergence tracker:

- [x] Guest can view any post at `/post/{id}` without auth
- [x] All 4 content types render correctly (image, video, audio, text)
- [x] Image gallery: navigation, dots, thumbnails, fullscreen modal
- [x] Auth user can like, dislike, save, report
- [x] Auth user can follow/unfollow the post author
- [x] Comments load with pagination ("Load more")
- [x] Auth user can create comments and replies
- [x] Auth user can like/unlike comments
- [x] Share modal: copy link, WhatsApp, Telegram
- [x] SEO meta tags render in SSR (og:title, og:image, etc.)
- [x] Mobile deep-link redirect for Android/iOS guests
- [x] Post view tracked on load for auth users
- [ ] Toast notifications on API errors
- [x] Proper relative timestamps ("2h ago", "3d ago")
- [ ] E2E tests passing against mock backend
- [ ] No compiler warnings

---

## Changelog

### 2026-04-14 (update)
- Audited implementation against plan: tasks 1–5 confirmed complete
- Updated status from ~85% to ~95% complete
- PostHeader follow API wiring: ✅ done with optimistic update
- Mobile deep-link hook: ✅ created and invoked from ViewPostPage
- Post view tracking: ✅ fire-and-forget on authenticated load
- `format_time_ago`: ✅ full chrono-based relative time implementation
- Remaining: 5 tasks (error toasts, mock backend, E2E tests, responsive polish, dashboard overlay)

### 2026-04-14
- Created completion sprint document
- Audited all existing code (page, 8 components, 7 API functions, 7 GraphQL ops, 2 models, 952 lines SCSS)
- Identified 10 remaining tasks (1 TODO in code, mobile hook, view tracking, time formatting, error recovery, mock backend, tests, responsive polish, dashboard overlay)
- Estimated ~85% code complete, ~13 hours remaining for Phases A–D
- Documented mock backend Phase 3 schema additions for posts/comments
- Defined 17 E2E test scenarios
