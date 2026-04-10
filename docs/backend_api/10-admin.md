# Admin Operations API

## Overview

Admin-only operations extend the base authenticated schema with additional queries and mutations for platform management. These include extended user search, friendship graphs, leaderboard generation, and admin post comment views.

**Authentication**: Required. **Role**: `ADMIN` (bitmask 16).

**Schema**: `admin_schema.graphql` (extends the base schema).

---

## Extended Types

The admin schema extends the base `User` and `HelloResponse` types with additional fields:

### Extended `User` Type

```graphql
extend type User {
  situation: String        # Additional context field
  email: String            # User's email (not visible to regular users)
  verified: Int            # Verification status (0 = unverified, 1 = verified)
  roles_mask: Int          # Role bitmask
  ip: String               # Last known IP address
  liquidity: Decimal       # Token balance
}
```

### Extended `HelloResponse` Type

```graphql
extend type HelloResponse {
  userroles: Int!          # Current user's role bitmask
  userRoleString: String!  # Human-readable role description
}
```

---

## Queries

### `searchUserAdmin` (deprecated)

Deprecated in favour of `listUsersAdminV2`. Accepts the same parameters in a different order. Will be removed in a future release.

---

### `listUsersAdminV2`

Extended user search with admin-specific filter fields.

> **Note**: `userid` and `username` cannot be used together in the same request.

```graphql
query {
  listUsersAdminV2(
    contentFilterBy: ContentFilterType
    userid: ID
    email: String
    username: String
    status: Int
    verified: Int
    ip: String
    offset: Int
    limit: Int
  ): UserListResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity |
| `userid` | `ID` | No | Search by user UUID |
| `email` | `String` | No | Search by email |
| `username` | `String` | No | Search by username |
| `status` | `Int` | No | Filter by account status |
| `verified` | `Int` | No | Filter by verification status (0/1) |
| `ip` | `String` | No | Filter by IP address |
| `offset` | `Int` | No | Pagination offset |
| `limit` | `Int` | No | Max results (max: 20) |

#### Response

Returns `UserListResponse` with the extended `User` type (includes `email`, `verified`, `roles_mask`, `ip`, `liquidity`).

#### Response Codes

| Code | Description |
|------|-------------|
| `11009` | Users found (user data prepared) |
| `31007` | No users found |
| `30201` | Invalid UUID format for `userid` |
| `30202` | Invalid `username` (length or pattern) |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `30257` | Invalid IP address |
| `31012` | `userid` and `username` cannot be used together |
| `41207` | Unexpected server error |
| `60501` | Not authenticated |

---

### `allfriends`

List all follow relationships across the entire platform.

```graphql
query {
  allfriends(offset: Int, limit: Int): AllUserFriends!
}
```

#### Response: `AllUserFriends`

```graphql
type AllUserFriends {
  meta: DefaultResponse!
  status: String!            # deprecated — use meta.status
  counter: Int!
  ResponseCode: String       # deprecated — use meta.ResponseCode
  affectedRows: [AllUserInfo]
}

type AllUserInfo {
  followerid: ID
  followername: String
  followedid: ID
  followedname: String
}
```

---

### `postcomments`

Admin view of all comments on a specific post (with subcomments).

```graphql
query {
  postcomments(
    postid: ID!
    offset: Int
    limit: Int
  ): PostCommentsResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `postid` | `ID!` | Yes |
| `offset` | `Int` | No |
| `limit` | `Int` | No |

#### Response: `PostCommentsResponse`

```graphql
type PostCommentsResponse {
  meta: DefaultResponse!
  status: String!            # deprecated — use meta.status
  counter: Int!
  ResponseCode: String       # deprecated — use meta.ResponseCode
  affectedRows: [PostCommentsData]
}

type PostCommentsData {
  commentid: ID
  userid: ID
  postid: ID
  parentid: ID
  content: String
  createdat: Date
  amountlikes: Decimal
  isliked: Boolean
  user: BasicUserInfo
  subcomments: [PostSubCommentsData]
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
}

type PostSubCommentsData {
  commentid: ID
  userid: ID
  postid: ID
  parentid: ID
  content: String
  createdat: Date
  amountlikes: Decimal
  amountreplies: Decimal
  isliked: Boolean
  user: BasicUserInfo
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
}
```

---

### `generateLeaderboard`

Generate a leaderboard CSV file for a given time period.

```graphql
query {
  generateLeaderboard(
    leaderboardParams: LeaderboardParamsInput!
  ): LeaderboardResponse!
}
```

#### Input: `LeaderboardParamsInput`

```graphql
input LeaderboardParamsInput {
  start_date: String!              # YYYY-MM-DD
  end_date: String!                # YYYY-MM-DD
  leaderboardUsersCount: Int!      # Top N users (default: 20)
}
```

#### Validation
- `end_date` must be ≥ `start_date`
- Date format: YYYY-MM-DD

#### Output CSV Columns

| Column | Description |
|--------|-------------|
| `uid` | User UUID |
| `username` | Display name |
| `slug` | User slug |
| `comments_on_posts` | Comments received on user's posts |
| `likes_on_posts` | Likes received on user's posts |
| `ppc_points` | Power Points Contest score |
| `likes_given` | Likes the user gave to others |
| `comments_given` | Comments the user made |
| `referrals` | Users referred |
| `total_points` | Aggregate score |

#### Response: `LeaderboardResponse`

```graphql
type LeaderboardResponse {
  meta: DefaultResponse!
  leaderboardResultLink: String    # URL to download the CSV file
}
```

The CSV is saved at:
```
runtime-data/media/other/power_power_contest_leaderboards_data/leaderboard_{startYmd}_{endYmd}_top{N}.csv
```

#### Response Codes

| Code | Description |
|------|-------------|
| `12301` | Leaderboard generated |
| `22201` | No users found for criteria |
| `30301` | Invalid parameters |
| `33002` | Invalid date range |
| `42201` | Generation failed |
| `60501` | Not authenticated |

---

## Admin Gem & Mint Operations

The admin gem and minting operations are documented in the [Tokenomics, Gems & Minting API](06-tokenomics-gems-and-minting.md) document. Key admin operations include:

| Operation | Type | Description |
|-----------|------|-------------|
| `gemster` | Query | Get uncollected gems statistics |
| `dailygemstatus` | Query | Daily gem status overview |
| `dailygemsresults` | Query | Per-user gems breakdown |
| `globalwins` | Mutation | Convert interactions to gems |
| `gemsters` / `distributeTokensForGems` | Mutation | Mint tokens from gems |
| `alphaMint` | Mutation | One-time alpha token distribution |
| `getMintAccount` | Query | Mint account balance |

---

## Role Reference

| Role | Bitmask | Access Level |
|------|---------|-------------|
| `USER` | 0 | Base schema only |
| `SYSTEM_ACCOUNT` | 1 | Internal LP account |
| `COMPANY_ACCOUNT` | 2 | Internal Peer Bank |
| `BURN_ACCOUNT` | 4 | Internal burn operations |
| `WEB3_BRIDGE_USER` | 8 | Bridge schema only |
| `ADMIN` | 16 | Base + Admin schema |
| `PEER_SHOP` | 32 | Internal shop account |
| `MODERATOR` | 256 | Base + Moderator schema |
