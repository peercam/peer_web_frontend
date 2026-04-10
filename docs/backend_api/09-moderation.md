# Content Moderation API

## Overview

The Moderation domain provides tools for moderators to review, hide, restore, or mark content as illegal. Content enters the moderation queue when users report posts, comments, or other users. Only users with the `MODERATOR` role (bitmask 256) can access these operations.

**Authentication**: Required. **Role**: `MODERATOR` (256).

**Schema**: `moderator_schema.graphql` (extends the base schema for moderators).

---

## Queries

### `moderationStats`

Get an overview of moderation ticket counts by status.

```graphql
query {
  moderationStats: ModerationStatsResponse!
}
```

#### Response: `ModerationStatsResponse`

```graphql
type ModerationStatsResponse {
  status: String!
  ResponseCode: String @deprecated  # use meta.ResponseCode
  meta: DefaultResponse!
  affectedRows: ModerationStats
}

type ModerationStats {
  AmountAwaitingReview: Int!     # Tickets pending review
  AmountHidden: Int!             # Content currently hidden
  AmountRestored: Int!           # Content restored by moderator
  AmountIllegal: Int!            # Content marked as illegal
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `12101` | Stats retrieved |
| `40301` | Error fetching stats |
| `40302` | Database error |
| `62101` | Not authorized (not moderator) |
| `60501` | Not authenticated |

---

### `moderationItems`

List moderation tickets with optional status and content type filters.

```graphql
query {
  moderationItems(
    status: ModerationStatus
    contentType: ModerationContentType
    offset: Int
    limit: Int
  ): ModerationItemListResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | `ModerationStatus` | No | Filter by ticket status |
| `contentType` | `ModerationContentType` | No | Filter by content type |
| `offset` | `Int` | No | Pagination offset (min: 0) |
| `limit` | `Int` | No | Results per page (min: 1, max: 20) |

```graphql
enum ModerationStatus {
  waiting_for_review       # Pending moderator action
  hidden                   # Hidden from users by moderator
  restored                 # Restored (report dismissed)
  illegal                  # Marked as illegal content
}

enum ModerationContentType {
  post                     # Reported post
  comment                  # Reported comment
  user                     # Reported user
}
```

#### Response: `ModerationItemListResponse`

```graphql
type ModerationItemListResponse {
  status: String!
  ResponseCode: String @deprecated  # use meta.ResponseCode
  meta: DefaultResponse!
  affectedRows: [ModerationItem!]!
}

type ModerationItem {
  moderationTicketId: ID!              # Ticket UUID
  targetContentId: ID!                 # UUID of reported content
  targettype: String!                  # "post", "comment", or "user"
  reportscount: Int!                   # Number of reports for this content
  status: String!                      # Current moderation status
  createdat: String!
  targetcontent: TargetContent!        # The reported content itself
  reporters: [BasicUserInfo!]!         # Users who reported
  moderatedBy: BasicUserInfo           # Moderator who acted (null if pending)
}

type TargetContent {
  post: Post                           # Non-null if targettype is "post"
  comment: Comment                     # Non-null if targettype is "comment"
  user: BasicUserInfo                  # Non-null if targettype is "user"
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `12102` | Items retrieved |
| `30101` | Invalid parameters |
| `40301` | Error fetching items |
| `62101` | Not authorized |
| `60501` | Not authenticated |

---

## Mutations

### `performModeration`

Take action on a moderation ticket: hide, restore, or mark as illegal.

```graphql
mutation {
  performModeration(
    moderationTicketId: ID!
    moderationAction: ModerationStatus!
  ): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `moderationTicketId` | `ID!` | Yes | Moderation ticket UUID |
| `moderationAction` | `ModerationStatus!` | Yes | Action to take |

#### Allowed Actions

| Action | Effect |
|--------|--------|
| `hidden` | Hides the content from all users |
| `restored` | Dismisses the report; content visible again |
| `illegal` | Marks content as illegal; completely hidden |

> **Warning**: `waiting_for_review` is the initial state and should not be used as a moderation action, but the backend does **not** currently enforce this restriction.

#### Validation
- Ticket must exist
- Ticket must be in a state that allows the transition
- Prevents duplicate moderation actions
- Moderator UUID is recorded on the ticket

#### Side Effects

**When hiding content** (`hidden`):
- Sets `visibility_status` to `hidden` on the target content
- Resets `reports` count to `0` on the target's info table
- Visibility depends on user content-filter preference (see table below):
  - `MYGRANDMALIKES` (strict): completely hidden
  - `MYGRANDMAHATES` (relaxed): placeholder shown
- `isHiddenForUsers` becomes `true` on the content

**When marking illegal** (`illegal`):
- Sets `visibility_status` to `illegal` on the target content
- Resets `reports` count to `0` on the target's info table
- Content is completely hidden (not even placeholder shown)
- Filtered out by `IllegalContentFilterSpec`
- For posts: media files are moved to an illegal folder

**When restoring** (`restored`):
- Sets `visibility_status` back to `normal`
- Resets `reports` count to `0` on the target's info table
- Content is fully visible again
- Prevents future reports on already-restored content (code `32104`)

#### Response Codes

| Code | Description |
|------|-------------|
| `12103` | Moderation action performed |
| `22103` | Ticket not found |
| `30101` | Missing required fields |
| `32101` | Invalid moderation action |
| `32103` | Ticket already processed |
| `42101` | Moderation action failed |
| `40301` | General error |
| `62101` | Not authorized (not moderator) |
| `60501` | Not authenticated |

---

## Content Visibility States

```graphql
enum ContentVisibilityStatus {
  NORMAL       # Fully visible to all users
  HIDDEN       # Hidden by moderator; placeholder shown based on content filter preference
  ILLEGAL      # Marked illegal; completely invisible
}
```

### Visibility Behavior by Content Filter Preference

| Visibility Status | `MYGRANDMALIKES` (strict) | `MYGRANDMAHATES` (relaxed) |
|-------------------|---------------------------|----------------------------|
| `NORMAL` | Visible | Visible |
| `HIDDEN` | Completely hidden | Placeholder shown |
| `ILLEGAL` | Completely hidden | Completely hidden |

---

## Moderation Flow

1. **User reports content** → `reportUser`, `resolvePostAction(REPORT)`, or `reportComment`
2. **Moderation ticket created** → status: `waiting_for_review`
3. **Moderator reviews** → queries `moderationItems(status: waiting_for_review)`
4. **Moderator acts** → calls `performModeration` with appropriate action
5. **Content visibility updated** → immediately reflected in all queries

### Report Deduplication
- Reports use content hashes to prevent duplicate moderation tickets
- Multiple reports for the same content increment `reportscount` on the existing ticket
- Each reporter is tracked in the `reporters` list
