# Comments API

## Overview

The Comments domain handles creating, listing, liking, and reporting comments on posts. Comments support a single level of nesting (replies to top-level comments). Comment interactions are free; creating a comment costs tokens (or uses a daily free action).

**Authentication**: Required for all operations.

---

## Queries

### `listComments`

Fetch top-level comments for a specific post.

```graphql
query {
  listComments(
    postid: ID!
    contentFilterBy: ContentFilterType
    commentOffset: Int
    commentLimit: Int
  ): CommentListResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `postid` | `ID!` | Yes | Post UUID to fetch comments for |
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity |
| `commentOffset` | `Int` | No | Pagination offset (min: 0) |
| `commentLimit` | `Int` | No | Results per page (min: 1, max: 20, default: 10) |

#### Response: `CommentListResponse`

```graphql
type CommentListResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [Comment!]
}
```

#### Content Filtering
- `HiddenContentFilterSpec` — applies based on user's filtering preference
- `DeletedUserSpec` — excludes comments from deleted users
- `SystemUserSpec` — excludes comments from system accounts

#### Response Codes

| Code | Description |
|------|-------------|
| `11601` | Comments retrieved |
| `21601` | No comments found |
| `30209` | Invalid post UUID |
| `30215` | Invalid comment offset |
| `30216` | Invalid comment limit |
| `60501` | Not authenticated |

---

### `listChildComments`

Fetch replies to a top-level comment.

```graphql
query {
  listChildComments(
    parent: ID!
    offset: Int
    limit: Int
  ): CommentResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `parent` | `ID!` | Yes | Parent comment UUID |
| `offset` | `Int` | No | Pagination offset |
| `limit` | `Int` | No | Results per page (max: 20) |

#### Response: `CommentResponse`

```graphql
type CommentResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [Comment]
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11607` | Child comments retrieved |
| `21606` | No child comments found |
| `30209` | Invalid parent UUID |
| `60501` | Not authenticated |

> **Note**: Unlike `listComments`, no content filtering specs are applied to child comments.

---

## Mutations

### `createComment`

Create a new comment or reply on a post. Costs tokens unless the user has free daily comments available.

```graphql
mutation {
  createComment(
    action: CommentType!
    postid: ID!
    parentid: ID
    content: String!
  ): CommentResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | `CommentType!` | Yes | Must be `COMMENT` |
| `postid` | `ID!` | Yes | Post UUID to comment on |
| `parentid` | `ID` | No | Parent comment UUID (for replies) |
| `content` | `String!` | Yes | Comment text |

```graphql
enum CommentType {
  COMMENT
}
```

#### Content Constraints

| Field | Min Length | Max Length |
|-------|-----------|-----------|
| `content` | 1 | 200 |

#### Nesting Rules
- Comments without a `parentid` are top-level comments
- Comments with a `parentid` are replies
- Replies can only be made to top-level comments (no nested replies beyond 1 level)
- The parent comment must exist and belong to the same post

#### Token Cost
- **Price**: 1.0 token per comment
- **Daily free**: 4 free comments per day
- If free action used, response code `11608` instead of `11605`

#### Side Effects
- Creates comment record with generated UUID
- Creates associated `comment_info` record (likes: 0, reports: 0)
- Increments post's comment count
- Increments parent comment's reply count (if reply)
- Deducts tokens from wallet (unless free)
- Logs gems for the action

#### Response Codes

| Code | Description |
|------|-------------|
| `11605` | Comment created (paid) |
| `11608` | Comment created (free daily action) |
| `30101` | Missing required fields |
| `30265` | Missing content or postid |
| `30209` | Invalid post UUID |
| `31602` | Post not found |
| `31603` | Invalid parent comment UUID |
| `41604` | Parent must be top-level comment |
| `41602` | Database insert failed |
| `51301` | Insufficient token balance |
| `60501` | Not authenticated |

---

### `likeComment`

Like a comment. Free action (no token cost).

```graphql
mutation {
  likeComment(commentid: ID!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `commentid` | `ID!` | Yes |

#### Validation
- Cannot like your own comment
- Prevents duplicate likes
- Content filtering specs applied (hidden/illegal content cannot be liked)

#### Side Effects
- Increments comment's like count
- Creates user activity record
- Interaction permission check against content filtering

#### Response Codes

| Code | Description |
|------|-------------|
| `11603` | Comment liked |
| `30201` | Invalid comment UUID |
| `31601` | Comment not found |
| `31604` | Already liked |
| `31606` | Cannot like own comment |
| `31608` | Interaction not allowed (content filtering) |
| `60501` | Not authenticated |

---

### `reportComment`

Report a comment for moderation review. Free action.

```graphql
mutation {
  reportComment(commentid: ID!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `commentid` | `ID!` | Yes |

#### Validation
- Cannot report your own comment
- Prevents duplicate reports
- Content filtering checks (moderation state)
- Cannot report already-moderated or restored content

#### Side Effects
- Creates moderation ticket with content hash
- Increments comment's report count
- Increments active reports count

#### Response Codes

| Code | Description |
|------|-------------|
| `11604` | Comment reported |
| `30201` | Invalid comment UUID |
| `31601` | Comment not found |
| `31605` | Already reported |
| `31607` | Cannot report own comment |
| `32102` | Already moderated |
| `32104` | Cannot report restored content |
| `41601` | Database error |
| `41603` | Transaction error |
| `60501` | Not authenticated |

---

## Comment Type

The full `Comment` type returned in all comment responses:

```graphql
type Comment {
  commentid: ID!
  userid: ID!
  postid: ID!
  parentid: ID                              # Null for top-level comments
  content: String!
  createdat: Date!
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
  hasActiveReports: Boolean!
  amountlikes: Int!
  amountreplies: Int!                       # Number of child replies
  amountreports: Int!
  isreported: Boolean!                      # Current user has reported
  isliked: Boolean!                         # Current user has liked
  user: ProfileUser!                        # Comment author
}
```

---

## Comment Info (Internal Reference)

Internally, each comment has a `comment_info` record tracking:
- `likes` — total like count
- `reports` — total report count
- `comments` — reply count

These are exposed as `amountlikes`, `amountreports`, and `amountreplies` on the `Comment` type.
