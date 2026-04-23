# Posts & Content API

## Overview

The Posts domain covers content creation (text, image, audio, video), post interactions (like, dislike, view, report, save, share), tag management, and the multipart file upload endpoint. Post creation can be paid (using tokens) or free (daily free actions).

**Authentication**: Required for all operations except `guestListPost`.

---

## Queries

### `listPosts`

Fetch a paginated, filtered, and sorted list of posts.

```graphql
query {
  listPosts(
    filterBy: [PostFilterType!]
    contentFilterBy: ContentFilterType
    IgnorList: IgnoreOption
    sortBy: PostSortBy
    userid: ID
    postid: ID
    title: String
    tag: String
    from: Date
    to: Date
    offset: Int
    limit: Int
    commentOffset: Int
    commentLimit: Int
  ): PostListResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `filterBy` | `[PostFilterType!]` | No | Filter by content type or social relation |
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity |
| `IgnorList` | `IgnoreOption` | No | Whether to apply block list (`YES` / `NO`) |
| `sortBy` | `PostSortBy` | No | Sort order |
| `userid` | `ID` | No | Filter by post author UUID |
| `postid` | `ID` | No | Fetch a specific post by UUID |
| `title` | `String` | No | Search by title (1–63 chars) |
| `tag` | `String` | No | Filter by tag name |
| `from` | `Date` | No | Start date filter (YYYY-MM-DD) |
| `to` | `Date` | No | End date filter (YYYY-MM-DD) |
| `offset` | `Int` | No | Pagination offset (min: 0) |
| `limit` | `Int` | No | Results per page (min: 1, max: 20, default: 10) |
| `commentOffset` | `Int` | No | Offset for embedded comments |
| `commentLimit` | `Int` | No | Limit for embedded comments (max: 20) |

#### Enums

```graphql
enum PostFilterType {
  IMAGE       # Image posts only
  AUDIO       # Audio posts only
  VIDEO       # Video posts only
  TEXT        # Text posts only
  FOLLOWED    # Posts from users I follow
  FOLLOWER    # Posts from my followers
  VIEWED      # Posts I've already viewed
  FRIENDS     # Posts from mutual friends
}

enum PostSortBy {
  NEWEST      # Most recent first
  TRENDING    # By trending score
  LIKES       # Most liked
  DISLIKES    # Most disliked
  VIEWS       # Most viewed
  COMMENTS    # Most commented
  FOR_ME      # Personalized feed
  OLDEST      # Oldest first
  FOLLOWER    # From followers
  FOLLOWED    # From following
  RELEVANT    # Relevance-based
  FRIENDS     # From mutual friends
}

enum IgnoreOption {
  YES         # Apply block list filtering
  NO          # Skip block list filtering
}

enum ContentFilterType {
  MYGRANDMALIKES   # Stricter: hides flagged content
  MYGRANDMAHATES   # Relaxed: shows more content
}
```

#### Response: `PostListResponse`

```graphql
type PostListResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [Post!]
}
```

#### Content Filtering Applied

The following specifications are applied to post queries:
- `IllegalContentFilterSpec` — excludes posts marked illegal
- `SystemUserSpec` — excludes posts by system accounts
- `DeletedUserSpec` — excludes posts by deleted users
- `ExcludeAdvertisementsForNormalFeedSpec` — excludes advertised posts from the normal feed
- `HiddenContentFilterSpec` — applies based on user's content filtering preference

#### Response Codes

| Code | Description |
|------|-------------|
| `11501` | Posts retrieved successfully |
| `21518` | No posts found |
| `30103` | Invalid filterBy or sortBy |
| `30201` | Invalid user UUID |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `30209` | Invalid post UUID |
| `30210` | Invalid title length |
| `30211` | Invalid tag format |
| `30212` | Invalid from date |
| `30213` | Invalid to date |
| `31510` | Post not found (by ID) |
| `60501` | Not authenticated |

---

### `guestListPost`

View a single post without authentication (guest schema).

```graphql
query {
  guestListPost(postid: ID!): PostResponse!
}
```

**No authentication required.**

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `postid` | `ID!` | Yes |

#### Response: `PostResponse`

```graphql
type PostResponse {
  meta: DefaultResponse!
  affectedRows: Post
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11501` | Post retrieved successfully |
| `30209` | Invalid post UUID |
| `31510` | Post not found |

---

### `postEligibility`

Check if the current user can create a post and obtain an eligibility token for file uploads.

```graphql
query {
  postEligibility: PostEligibilityResponse!
}
```

#### Response: `PostEligibilityResponse`

```graphql
type PostEligibilityResponse {
  meta: DefaultResponse!
  eligibilityToken: String!    # Token to use with /upload-post endpoint
}
```

The eligibility token must be included with multipart file uploads. It expires after use and prevents unauthorized uploads.

#### Response Codes

| Code | Description |
|------|-------------|
| `10901` | Eligibility token issued |
| `31512` | Rate limit exceeded (max 5 tokens/hour) |
| `51301` | Insufficient token balance |
| `60501` | Not authenticated |

---

### `postInteractions`

List users who performed a specific interaction on a post or comment.

```graphql
query {
  postInteractions(
    getOnly: GetOnly!
    contentFilterBy: ContentFilterType
    postOrCommentId: ID!
    offset: Int
    limit: Int
  ): PostInteractionResponse
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `getOnly` | `GetOnly!` | Yes | Interaction type to retrieve |
| `contentFilterBy` | `ContentFilterType` | No | Content filtering |
| `postOrCommentId` | `ID!` | Yes | Post or comment UUID |
| `offset` | `Int` | No | Pagination offset |
| `limit` | `Int` | No | Results per page (max: 20) |

```graphql
enum GetOnly {
  VIEW          # Users who viewed
  LIKE          # Users who liked
  DISLIKE       # Users who disliked
  COMMENTLIKE   # Users who liked a comment
}
```

#### Response: `PostInteractionResponse`

```graphql
type PostInteractionResponse {
  meta: DefaultResponse!
  affectedRows: [ProfileUser!]
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11205` | Interactions retrieved successfully |
| `30103` | Invalid getOnly or missing postOrCommentId |
| `30201` | Invalid UUID format |
| `60501` | Not authenticated |

---

### `listTags`

List all tags with pagination.

```graphql
query {
  listTags(offset: Int, limit: Int): TagSearchResponse!
}
```

#### Response: `TagSearchResponse`

```graphql
type TagSearchResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [Tag]
}

type Tag {
  name: String!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11701` | Tags loaded |
| `21701` | No tags found |
| `60501` | Not authenticated |

---

### `searchTags`

Search tags by name (fuzzy match).

```graphql
query {
  searchTags(
    tagName: String!
    offset: Int
    limit: Int
  ): TagSearchResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `tagName` | `String!` | Yes | 2–53 chars, alphanumeric/underscores (`^[a-zA-Z0-9_]+$`) |
| `offset` | `Int` | No | Pagination offset |
| `limit` | `Int` | No | Max: 20 |

#### Response Codes

| Code | Description |
|------|-------------|
| `11701` | Tags retrieved successfully |
| `21701` | No tags found |
| `30101` | Missing tagName |
| `60501` | Not authenticated |

---

## Mutations

### `createPost`

Create a new post. Costs tokens unless the user has daily free posts available.

```graphql
mutation {
  createPost(
    action: PostType!
    input: PostInput!
  ): PostResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `action` | `PostType!` | Yes |
| `input` | `PostInput!` | Yes |

```graphql
enum PostType {
  POST
}

input PostInput {
  title: String!                    # 1–63 characters
  mediadescription: String          # 1–500 characters (optional)
  contenttype: ContentType!         # image, audio, video, text
  media: [String!]                  # Media file paths/URLs
  cover: [String!]                  # Cover image paths/URLs
  tags: [String!]                   # Up to 10 tags, each 2–53 chars
  uploadedFiles: String             # Comma-separated filenames from /upload-post
}

enum ContentType {
  image
  audio
  video
  text
}
```

#### Media Limits Per Content Type

| Content Type | Max Media Items | Max Cover Items |
|-------------|----------------|-----------------|
| `image` | 5 | 1 |
| `audio` | 1 | 1 |
| `video` | 2 | 1 |
| `text` | 1 | 1 |

#### Tag Constraints
- Pattern: `^[a-zA-Z0-9_]+$`
- Length: 2–53 characters per tag
- Maximum: 10 tags per post creation
- Tags are normalized to lowercase

#### Token Cost
- **Price**: 20.0 tokens per post
- **Daily free**: 1 free post per day (when available, no token cost)
- If free action used, response code `11513` instead of `11508`

#### Side Effects
- Creates post record with UUID
- Associates tags (creates new tags if needed)
- Increments user's post count
- Logs gems for the action
- Deducts tokens from wallet (unless free)

#### Response Codes

| Code | Description |
|------|-------------|
| `11508` | Post created (paid) |
| `11513` | Post created (free daily action) |
| `30101` | Missing required fields |
| `30102` | Empty field values |
| `30206` | Invalid contenttype |
| `30210` | Invalid title length |
| `30251` | Invalid media format |
| `30262` | Invalid tags (format/count) |
| `30263` | Invalid mediadescription length |
| `30266` | Mixed media types not allowed |
| `30267` | Too many media items |
| `30268` | Too many cover items |
| `31511` | Temporary file expired |
| `40301` | Unexpected error |
| `40306` | Cover upload failed |
| `51301` | Insufficient token balance |
| `60501` | Not authenticated |

---

### `resolvePostAction`

Perform an interaction on a post (like, dislike, view, report, save, share).

```graphql
mutation {
  resolvePostAction(
    action: PostActionType!
    postid: ID!
  ): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `action` | `PostActionType!` | Yes |
| `postid` | `ID!` | Yes |

```graphql
enum PostActionType {
  LIKE        # Costs 3.0 tokens (or daily free: 3 per day)
  DISLIKE     # Costs 3.0 tokens (no free daily actions)
  REPORT      # Free
  VIEW        # Free
  SHARE       # Free
  SAVE        # Free (toggle)
}
```

#### Action Details

| Action | Token Cost | Daily Free | Notes |
|--------|-----------|------------|-------|
| `LIKE` | 3.0 | 3/day | Cannot like own post. Duplicate prevented. |
| `DISLIKE` | 3.0 | 0/day | Cannot dislike own post. Duplicate prevented. |
| `VIEW` | 0 | — | Tracked per user per post |
| `REPORT` | 0 | — | Cannot report own post. Creates moderation ticket. |
| `SAVE` | 0 | — | Toggles saved state (save/unsave) |
| `SHARE` | 0 | — | Increments share count |

#### Response Codes

| Code | Description |
|------|-------------|
| `11503` | Post liked |
| `11504` | Post disliked |
| `11505` | Post reported |
| `11506` | Post viewed |
| `11507` | Post shared |
| `11512` | Post saved |
| `11511` | Post unsaved |
| `11514` | Daily free like used |
| `30103` | Invalid filter/sort parameter |
| `30105` | Invalid action |
| `30209` | Invalid post UUID |
| `31501` | Already liked |
| `31502` | Already disliked |
| `31503` | Already reported this post |
| `31504` | Already shared this post |
| `31505` | Already viewed this post |
| `31506` | Cannot like own post |
| `31507` | Cannot dislike own post |
| `31508` | Cannot report own post |
| `31509` | Cannot view own post |
| `31510` | Post not found |
| `31513` | Interaction not allowed (content filtered) |
| `31107` | Interaction not allowed (PeerShop user) |
| `32102` | Already moderated |
| `32104` | Cannot report restored content |
| `51301` | Insufficient balance |
| `60501` | Not authenticated |

---

## Post Type

The full `Post` type returned in all post responses:

```graphql
type Post {
  id: ID!
  contenttype: String!                      # "image", "audio", "video", "text"
  title: String!
  media: String!                            # Comma-separated media URLs
  cover: String!                            # Cover image URL
  mediadescription: String!
  createdat: Date!
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
  hasActiveReports: Boolean!
  amountreports: Int!
  amountlikes: Int!
  amountviews: Int!
  amountcomments: Int!
  amountdislikes: Int!
  amounttrending: Int!
  isliked: Boolean!                         # Current user has liked
  isviewed: Boolean!                        # Current user has viewed
  isreported: Boolean!                      # Current user has reported
  isdisliked: Boolean!                      # Current user has disliked
  issaved: Boolean!                         # Current user has saved
  tags: [String]!
  url: String!                              # Shareable post URL
  user: ProfileUser!                        # Post author
  comments: [Comment!]!                     # Embedded comments (limited)
}
```

---

## File Upload Endpoint

### `POST /upload-post`

Multipart file upload for post media (images, audio, video).

**This is a REST endpoint, not a GraphQL operation.**

#### Prerequisites
1. Authenticate via GraphQL
2. Call `postEligibility` query to get an eligibility token
3. Upload files to this endpoint with the token

#### Request

```http
POST /upload-post
Content-Type: multipart/form-data
Authorization: Bearer <accessToken>
```

#### Form Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `eligibilityToken` | `string` | Yes | Token from `postEligibility` query |
| `file` | `file[]` | Yes | Media file(s) to upload |

#### Constraints
- Maximum file size: 500 MB
- Eligibility token is single-use (consumed after upload)
- Token statuses: `FILE_UPLOADED`, `POST_CREATED` (prevents reuse)

#### Response

```json
{
  "status": "success",
  "ResponseCode": "11515",
  "affectedRows": {
    "uploadedFiles": "file1.jpg,file2.jpg"
  }
}
```

The `uploadedFiles` string should be passed in `PostInput.uploadedFiles` when calling `createPost`.

#### Response Codes

| Code | Description |
|------|-------------|
| `11515` | File uploaded successfully |
| `30102` | Missing or invalid fields |
| `30261` | Invalid file |
| `40902` | Invalid eligibility token |
| `41514` | Upload failed |
| `60501` | Not authenticated |

---

## Custom Scalars

```graphql
scalar Date       # Date/datetime strings (formats: YYYY-MM-DD or ISO 8601)
scalar Decimal    # Precise decimal numbers for token amounts
```
