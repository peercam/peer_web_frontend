# Users & Profiles API

## Overview

The Users & Profiles domain covers user discovery, profile viewing, social relationships (follow/block), referrals, and user preferences. All operations in this section require authentication unless noted otherwise.

**Authentication**: Required (`Authorization: Bearer <accessToken>`)

---

## Queries

### `listUsersV2`

Search and list users with optional filters and pagination.

```graphql
query {
  listUsersV2(
    contentFilterBy: ContentFilterType
    userid: ID
    username: String
    offset: Int
    limit: Int
  ): UserListResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity level |
| `userid` | `ID` | No | Search by specific user UUID |
| `username` | `String` | No | Search by username (partial match) |
| `offset` | `Int` | No | Pagination offset (min: 0, default: 0) |
| `limit` | `Int` | No | Results per page (min: 1, max: 20, default: 10) |

#### Response: `UserListResponse`

```graphql
type UserListResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [User]
}

type User {
  id: ID
  username: String
  status: Int
  slug: Int                                    # Unique 5-digit numeric identifier
  img: String                                  # Avatar URL
  biography: String
  visibilityStatus: ContentVisibilityStatus!   # NORMAL | HIDDEN | ILLEGAL
  isHiddenForUsers: Boolean!                   # Whether content is replaced with placeholder
  hasActiveReports: Boolean!
  createdat: Date
  updatedat: Date
  # Admin-only fields (extended via admin_schema):
  situation: String
  email: String
  verified: Int
  roles_mask: Int
  ip: String
  liquidity: Decimal
}
```

#### Content Filtering

Results are filtered based on the `contentFilterBy` parameter and global content filtering specifications:

| Spec | Effect |
|------|--------|
| `IllegalContentFilterSpec` | Hides content marked as illegal |
| `SystemUserSpec` | Excludes system accounts (role bitmask 1, 2, 4) |
| `DeletedUserSpec` | Excludes soft-deleted users (status 6) |
| `PeerShopSpec` | Excludes shop account |
| `UserIsBlockedByMeSpec` | Excludes users blocked by current user |
| `CurrentUserIsBlockedUserSpec` | Excludes users who blocked current user |

#### Content Filtering Severity

```graphql
enum ContentFilterType {
  MYGRANDMALIKES     # Stricter: hides all flagged content
  MYGRANDMAHATES     # Less strict: shows placeholders for flagged content
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11001` | Users retrieved successfully |
| `21001` | No users found |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `60501` | Not authenticated |

---

### `getProfile`

Fetch a user's full profile with social statistics.

```graphql
query {
  getProfile(
    userid: ID
    contentFilterBy: ContentFilterType
    postLimit: Int @deprecated  # Use listPosts with userid instead
  ): ProfileInfo!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `userid` | `ID` | No | Target user UUID (defaults to current user) |
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity |

#### Response: `ProfileInfo`

```graphql
type ProfileInfo {
  meta: DefaultResponse!
  affectedRows: Profile
}

type Profile {
  id: ID
  username: String
  status: Int
  slug: Int
  img: String
  biography: String
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
  hasActiveReports: Boolean!
  iFollowThisUser: Boolean!             # Whether current user follows this profile
  thisUserFollowsMe: Boolean!           # Whether this profile follows current user
  isreported: Boolean                    # Whether current user has reported this user
  amountposts: Int
  amounttrending: Int
  amountfollowed: Int                    # Number of users this profile follows
  amountfollower: Int                    # Number of followers
  amountfriends: Int                     # Mutual follow count
  amountblocked: Int
  amountreports: Int
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11008` | Profile loaded successfully |
| `21001` | User not found |
| `30201` | Invalid user UUID |
| `60501` | Not authenticated |

---

### `getUserInfo`

Get the authenticated user's own account information and preferences.

```graphql
query {
  getUserInfo: UserInfoResponse!
}
```

#### Response: `UserInfoResponse`

```graphql
type UserInfoResponse {
  meta: DefaultResponse!
  affectedRows: UserInfo
}

type UserInfo {
  userid: ID!
  liquidity: Decimal!               # Current token balance
  amountposts: Int!
  amountreports: Int!
  amountblocked: Int!
  amountfollower: Int!
  amountfollowed: Int!
  amountfriends: Int!
  invited: ID!                       # UUID of inviter (if any)
  updatedat: Date
  userPreferences: UserPreferences
}

type UserPreferences {
  contentFilteringSeverityLevel: ContentFilterType
  onboardingsWereShown: [OnboardingType!]!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11009` | User data prepared successfully |
| `60501` | Not authenticated |

---

### `listFollowRelations`

List a user's followers and following relationships.

```graphql
query {
  listFollowRelations(
    contentFilterBy: ContentFilterType
    userid: ID
    offset: Int
    limit: Int
  ): FollowRelationsResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity |
| `userid` | `ID` | No | Target user UUID (defaults to current user) |
| `offset` | `Int` | No | Pagination offset (min: 0) |
| `limit` | `Int` | No | Results per page (min: 1, max: 20) |

#### Response: `FollowRelationsResponse`

```graphql
type FollowRelationsResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: FollowRelations
}

type FollowRelations {
  followers: [ProfileUser!]
  following: [ProfileUser!]
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11101` | Follow data loaded |
| `21102` | No relations found |
| `60501` | Not authenticated |

---

### `listFriends`

List mutual follow relationships (both users follow each other).

```graphql
query {
  listFriends(
    contentFilterBy: ContentFilterType
    userid: ID
    offset: Int
    limit: Int
  ): UserFriendsResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `contentFilterBy` | `ContentFilterType` | No | Content filtering severity |
| `userid` | `ID` | No | Target user UUID (defaults to current user) |
| `offset` | `Int` | No | Pagination offset |
| `limit` | `Int` | No | Results per page (max: 20) |

#### Response: `UserFriendsResponse`

```graphql
type UserFriendsResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [BasicUserInfo]
}

type BasicUserInfo {
  userid: ID
  img: String
  username: String
  slug: Int
  biography: String
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
  hasActiveReports: Boolean!
  updatedat: Date!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11102` | Friends data loaded |
| `21101` | No friends found |
| `60501` | Not authenticated |

---

### `listBlockedUsers`

List users blocked by the current user and users who blocked the current user.

```graphql
query {
  listBlockedUsers(
    contentFilterBy: ContentFilterType
    offset: Int
    limit: Int
  ): BlockedUsersResponse!
}
```

#### Response: `BlockedUsersResponse`

```graphql
type BlockedUsersResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: BlockedUsers
}

type BlockedUsers {
  iBlocked: [BlockedUser!]          # Users I have blocked
  blockedBy: [BlockedUser!]         # Users who blocked me
}

type BlockedUser {
  userid: String
  img: String
  username: String
  slug: Int
  hasActiveReports: Boolean!
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11107` | Blocked list loaded |
| `21103` | No blocked users |
| `60501` | Not authenticated |

---

### `getReferralInfo`

Get the current user's referral UUID and shareable link.

```graphql
query {
  getReferralInfo: ReferralInfoResponse!
}
```

#### Response: `ReferralInfoResponse`

```graphql
type ReferralInfoResponse {
  meta: DefaultResponse!
  referralUuid: ID           # User's referral UUID
  referralLink: String       # Full shareable referral URL
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11011` | Referral info loaded |
| `21002` | No referral info found (new referral generated) |
| `60501` | Not authenticated |

---

### `referralList`

List users who signed up using the current user's referral, and who invited the current user.

```graphql
query {
  referralList(offset: Int, limit: Int): ReferralListResponse!
}
```

#### Response: `ReferralListResponse`

```graphql
type ReferralListResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: ReferralUsers!
}

type ReferralUsers {
  invitedBy: ProfileUser           # User who invited the current user
  iInvited: [ProfileUser!]!       # Users invited by the current user
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11011` | Referral list loaded |
| `21003` | No referral data |
| `60501` | Not authenticated |

---

## Mutations

### `updateUserPreferences`

Update the user's content filtering and onboarding preferences.

```graphql
mutation {
  updateUserPreferences(
    userPreferences: UserPreferencesInput
  ): UserPreferencesResponse!
}
```

#### Input: `UserPreferencesInput`

```graphql
input UserPreferencesInput {
  contentFilteringSeverityLevel: ContentFilterType
  shownOnboardings: [OnboardingType!]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `contentFilteringSeverityLevel` | `ContentFilterType` | `MYGRANDMALIKES` or `MYGRANDMAHATES` |
| `shownOnboardings` | `[OnboardingType!]` | Onboardings the user has seen. Valid: `INTROONBOARDING` |

#### Response: `UserPreferencesResponse`

```graphql
type UserPreferencesResponse {
  meta: DefaultResponse!
  affectedRows: UserPreferences
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11014` | Preferences updated |
| `60501` | Not authenticated |

---

### `updateUsername`

Change the authenticated user's username. Requires current password.

```graphql
mutation {
  updateUsername(username: String!, password: String!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `username` | `String!` | Yes | 3–23 chars, `^[a-zA-Z0-9_-]+$` |
| `password` | `String!` | Yes | Current account password |

#### Response Codes

| Code | Description |
|------|-------------|
| `11007` | Username changed |
| `30202` | Invalid username format |
| `30101` | Missing fields |
| `60501` | Not authenticated |

---

### `updateEmail`

Change the authenticated user's email. Requires current password.

```graphql
mutation {
  updateEmail(email: String!, password: String!): DefaultResponse!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11006` | Email updated |
| `30103` | Invalid email format |
| `60501` | Not authenticated |

---

### `updatePassword`

Change the authenticated user's password. Requires current password.

```graphql
mutation {
  updatePassword(password: String!, expassword: String!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `password` | `String!` | Yes | New password (8–128 chars, upper+lower+digit) |
| `expassword` | `String!` | Yes | Current password |

#### Response Codes

| Code | Description |
|------|-------------|
| `11005` | Password updated |
| `30101` | Missing fields |
| `60501` | Not authenticated |

---

### `updateBio`

Update the authenticated user's biography.

```graphql
mutation {
  updateBio(biography: String!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `biography` | `String!` | Yes | 1–5,000 characters |

#### Response Codes

| Code | Description |
|------|-------------|
| `11003` | Bio updated |
| `30101` | Missing fields |
| `60501` | Not authenticated |

---

### `updateProfileImage`

Update the authenticated user's avatar image (base64-encoded).

```graphql
mutation {
  updateProfileImage(img: String!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `img` | `String!` | Yes | Base64-encoded image data (max 5 MB) |

#### Response Codes

| Code | Description |
|------|-------------|
| `11004` | Profile picture updated |
| `30101` | Missing fields |
| `60501` | Not authenticated |

---

### `toggleUserFollowStatus`

Follow or unfollow a user. Toggles the follow state.

```graphql
mutation {
  toggleUserFollowStatus(userid: ID!): FollowStatusResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `userid` | `ID!` | Yes |

#### Response: `FollowStatusResponse`

```graphql
type FollowStatusResponse {
  meta: DefaultResponse!
  isfollowing: Boolean         # New follow state after toggle
}
```

#### Side Effects
- Updates follower/following counts for both users
- Updates friend count if relationship becomes mutual

#### Response Codes

| Code | Description |
|------|-------------|
| `11104` | Now following user |
| `11103` | Unfollowed user |
| `30201` | Invalid UUID |
| `60501` | Not authenticated |

---

### `toggleBlockUserStatus`

Block or unblock a user. Toggles the block state.

```graphql
mutation {
  toggleBlockUserStatus(userid: ID!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `userid` | `ID!` | Yes |

#### Side Effects
- When blocking: removes any existing follow relationships in both directions
- Updates blocked user counts

#### Response Codes

| Code | Description |
|------|-------------|
| `11105` | User blocked |
| `11106` | User unblocked |
| `30201` | Invalid UUID |
| `60501` | Not authenticated |

---

### `reportUser`

Report a user for moderation review.

```graphql
mutation {
  reportUser(userid: ID!): DefaultResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `userid` | `ID!` | Yes |

#### Side Effects
- Creates a moderation ticket with content hash
- Increments user's report count
- Cannot report yourself
- Cannot duplicate an existing report

#### Response Codes

| Code | Description |
|------|-------------|
| `11012` | User reported |
| `30201` | Invalid UUID |
| `31007` | User not found |
| `31008` | Already reported (duplicate) |
| `31009` | Cannot report yourself |
| `32102` | Already moderated |
| `32104` | Cannot report restored content |
| `32201` | Interaction not allowed (e.g. shop account) |
| `60501` | Not authenticated |

---

### `verifyReferralString`

Validate a referral string/UUID (guest-accessible, in guest schema).

```graphql
mutation {
  verifyReferralString(referralString: String!): ReferralResponse!
}
```

#### Response: `ReferralResponse`

```graphql
type ReferralResponse {
  meta: DefaultResponse!
  affectedRows: ReferralInfo
}

type ReferralInfo {
  uid: String
  username: String
  slug: String
  img: String
}
```

---

## Shared Types

### `ProfileUser`

Used across responses to represent a user in social contexts:

```graphql
type ProfileUser {
  id: ID!
  username: String
  slug: Int
  img: String
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
  hasActiveReports: Boolean!
  iFollowThisUser: Boolean!
  thisUserFollowsMe: Boolean!
  isreported: Boolean
  isfriend: Boolean
}
```

### `ContentVisibilityStatus`

```graphql
enum ContentVisibilityStatus {
  NORMAL       # Visible to all users
  HIDDEN       # Hidden via moderation (placeholder shown)
  ILLEGAL      # Marked illegal (completely hidden)
}
```
