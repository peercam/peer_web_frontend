# Advertisements API

## Overview

The Advertisements domain allows users to promote their posts through two plans: **Basic** (time-based with configurable duration) and **Pinned** (featured placement). Advertisement creation costs tokens and is processed through the wallet payment system.

**Authentication**: Required for all operations.

---

## Queries

### `listAdvertisementPosts`

Fetch posts that are currently being advertised, with their advertisement metadata.

```graphql
query {
  listAdvertisementPosts(
    filterBy: [ContentType!]
    contentFilterBy: ContentFilterType
    userid: ID
    postid: ID
    title: String
    tag: String
    offset: Int
    limit: Int
    commentOffset: Int
    commentLimit: Int
  ): ListAdvertisementPostsResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `filterBy` | `[ContentType!]` | No | Filter by content type (`image`, `audio`, `video`, `text`) |
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity |
| `userid` | `ID` | No | Filter by advertiser UUID |
| `postid` | `ID` | No | Filter by post UUID |
| `title` | `String` | No | Search by post title |
| `tag` | `String` | No | Filter by tag |
| `offset` | `Int` | No | Pagination offset (min: 0) |
| `limit` | `Int` | No | Max results (min: 1, max: 20) |
| `commentOffset` | `Int` | No | Offset for embedded comments |
| `commentLimit` | `Int` | No | Limit for embedded comments |

#### Response: `ListAdvertisementPostsResponse`

```graphql
type ListAdvertisementPostsResponse {
  status: String!                          # Deprecated: use meta.status
  ResponseCode: String                     # Deprecated: use meta.ResponseCode
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [AdvertisementPost!]
}

type AdvertisementPost {
  post: Post!                              # Full post data
  advertisement: AdvCreator!               # Advertisement metadata
}

type AdvCreator {
  advertisementid: ID!
  postid: ID!
  advertisementtype: AdvertisementType!    # PINNED or BASIC
  startdate: Date!
  enddate: Date!
  createdat: Date!
  user: ProfileUser!                       # Advertiser user info
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `12002` | Advertisements fetched |
| `22002` | No advertisements found |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `60501` | Not authenticated |

---

### `advertisementHistory`

Fetch the user's advertisement history with aggregated statistics.

```graphql
query {
  advertisementHistory(
    filter: AdvertisementHistoryFilter
    sort: AdvertisementSort
    offset: Int
    limit: Int
  ): ListedAdvertisementData!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `filter` | `AdvertisementHistoryFilter` | No | Filter criteria |
| `sort` | `AdvertisementSort` | No | Sort order |
| `offset` | `Int` | No | Pagination offset |
| `limit` | `Int` | No | Max results (max: 20) |

#### Input: `AdvertisementHistoryFilter`

```graphql
input AdvertisementHistoryFilter {
  from: Date                          # Start date
  to: Date                            # End date
  type: AdvertisementType             # PINNED or BASIC
  advertisementId: ID                 # Specific ad UUID
  postId: ID                          # Specific post UUID
  userId: ID                          # Specific user UUID
}
```

#### Sort Options

```graphql
enum AdvertisementSort {
  NEWEST              # Most recent first
  OLDEST              # Oldest first
  BIGGEST_COST        # Highest cost first
  SMALLEST_COST       # Lowest cost first
}
```

#### Response: `ListedAdvertisementData`

```graphql
type ListedAdvertisementData {
  status: String!                          # Deprecated: use meta.status
  ResponseCode: String                     # Deprecated: use meta.ResponseCode
  meta: DefaultResponse!
  affectedRows: AdvertisementHistoryResult
}

type AdvertisementHistoryResult {
  stats: TotalAdvertisementHistoryStats     # Aggregated totals
  advertisements: [Advertisement]            # Individual ad records
}

type TotalAdvertisementHistoryStats {
  tokenSpent: Float!
  euroSpent: Float!
  amountAds: Int!
  gemsEarned: Float!
  amountLikes: Int!
  amountViews: Int!
  amountComments: Int!
  amountDislikes: Int!
  amountReports: Int!
}

type Advertisement {
  id: ID!
  createdAt: Date!
  type: AdvertisementType!            # PINNED or BASIC
  timeframeStart: Date!
  timeframeEnd: Date!
  totalTokenCost: Float!
  totalEuroCost: Float!
  gemsEarned: Float!
  amountLikes: Int!
  amountViews: Int!
  amountComments: Int!
  amountDislikes: Int!
  amountReports: Int!
  user: ProfileUser!
  post: Post!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `12002` | History fetched |
| `22002` | No advertisements found |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `40301` | General error |
| `60501` | Not authenticated |

---

## Mutations

### `advertisePostBasic`

Create a basic (time-based) advertisement for a post.

```graphql
mutation {
  advertisePostBasic(
    postid: ID!
    startday: Date!
    durationInDays: AdDuration!
    advertisePlan: AdvertisementBasicPlan!
  ): ListAdvertisementData!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `postid` | `ID!` | Yes | Post UUID to advertise |
| `startday` | `Date!` | Yes | Advertisement start date (YYYY-MM-DD) |
| `durationInDays` | `AdDuration!` | Yes | Duration of advertisement |
| `advertisePlan` | `AdvertisementBasicPlan!` | Yes | Must be `BASIC` |

```graphql
enum AdDuration {
  ONE_DAY         # 1 day
  TWO_DAYS        # 2 days
  THREE_DAYS      # 3 days
  FOUR_DAYS       # 4 days
  FIVE_DAYS       # 5 days
  SIX_DAYS        # 6 days
  SEVEN_DAYS      # 7 days
}

enum AdvertisementBasicPlan {
  BASIC
}
```

#### Pricing
- **Base cost**: 50.0 tokens per day
- **Total cost**: `50.0 × durationInDays`
- **Discount**: 20% off if advertisement duration ≥ 24 hours already active

#### Validation
- Post must exist and belong to the current user
- Post must pass content filtering (not illegal/hidden)
- Post must not already have an active advertisement of the same type
- User must have sufficient token balance

#### Response: `ListAdvertisementData`

```graphql
type ListAdvertisementData {
  status: String!                          # Deprecated: use meta.status
  ResponseCode: String                     # Deprecated: use meta.ResponseCode
  meta: DefaultResponse!
  affectedRows: [AdvertisementRow]
}

type AdvertisementRow {
  id: ID!
  createdAt: Date!
  type: AdvertisementType!        # BASIC
  timeframeStart: Date!
  timeframeEnd: Date!
  totalTokenCost: Float!
  totalEuroCost: Float!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `12001` | Advertisement created |
| `30209` | Invalid post UUID |
| `31510` | Post not found |
| `32005` | Invalid advertisement plan |
| `32006` | Post already has active ad |
| `32007` | Invalid start date |
| `32008` | Invalid duration |
| `32020` | Post content filtered |
| `42004` | Advertisement creation failed |
| `42005` | Database error |
| `51301` | Insufficient token balance |
| `60501` | Not authenticated |

---

### `advertisePostPinned`

Create a pinned (featured) advertisement for a post.

```graphql
mutation {
  advertisePostPinned(
    postid: ID!
    advertisePlan: AdvertisementPinnedPlan!
  ): ListAdvertisementData!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `postid` | `ID!` | Yes | Post UUID to advertise |
| `advertisePlan` | `AdvertisementPinnedPlan!` | Yes | Must be `PINNED` |

```graphql
enum AdvertisementPinnedPlan {
  PINNED
}
```

#### Pricing
- **Cost**: 200.0 tokens (flat rate)

#### Validation
Same validation rules as `advertisePostBasic`.

#### Response Codes

Same as `advertisePostBasic`.

---

## Enums

```graphql
enum AdvertisementType {
  PINNED       # Featured/pinned advertisement
  BASIC        # Time-based standard advertisement
}
```

---

## Advertisement Lifecycle

1. **Create**: User calls `advertisePostBasic` or `advertisePostPinned`
2. **Payment**: Tokens deducted from user's wallet via `WalletService::performPayment()`
3. **Active**: Post appears in `listAdvertisementPosts` results
4. **Excluded from feed**: Advertised posts are excluded from normal `listPosts` results via `ExcludeAdvertisementsForNormalFeedSpec`
5. **Gems**: Post interactions during advertisement period earn gems for the post owner
6. **Expiry**: Basic ads expire after their duration; pinned ads are managed separately
7. **History**: All past and current ads visible via `advertisementHistory`
