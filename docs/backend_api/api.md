# Peer Backend API Documentation

## Overview

The Peer Backend API is a **GraphQL-based API** built with PHP (Slim Framework) that powers the Peer social platform. It provides functionality for user management, content creation (posts/comments), social interactions, wallet/token operations, advertisements, and content moderation.

### Base URL
```
POST /graphql
```

### Technology Stack
- **Framework**: Slim 4 (PHP)
- **API Protocol**: GraphQL
- **Authentication**: JWT (RS256)
- **Database**: PostgreSQL

---

## Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/graphql` | Main GraphQL endpoint for all queries and mutations |
| `GET` | `/health` | Health check endpoint |
| `POST` | `/upload-post` | Multipart file upload for posts with media |

---

## Authentication

### JWT Authentication

The API uses JSON Web Tokens (JWT) with RS256 algorithm for authentication.

#### Headers
```http
Authorization: Bearer <access_token>
Content-Type: application/json
```

#### Token Types
| Token | Purpose | Validity |
|-------|---------|----------|
| Access Token | API requests | Configurable (default: 1 hour) |
| Refresh Token | Obtain new access tokens | Configurable (default: 1 week) |

#### Authentication Flow

1. **Register** a new account
2. **Verify** account (required before login)
3. **Login** to obtain tokens
4. **Use Access Token** for authenticated requests
5. **Refresh** when access token expires

### Role-Based Access Control

Different roles have access to different GraphQL schemas:

| Role | Value | Schema Access |
|------|-------|---------------|
| `USER` | 0 | Base schema |
| `SYSTEM_ACCOUNT` | 1 | LP Account |
| `COMPANY_ACCOUNT` | 2 | Peer Bank Account |
| `BURN_ACCOUNT` | 4 | Token burn operations |
| `WEB3_BRIDGE_USER` | 8 | Bridge schema only |
| `ADMIN` | 16 | Base + Admin schema |
| `PEER_SHOP` | 32 | Shop operations |
| `MODERATOR` | 256 | Base + Moderator schema |

---

## GraphQL Schema

### Queries (Guest - No Auth Required)

```graphql
type Query {
  # Basic hello endpoint with system info
  hello: HelloResponse
  
  # View a single post without authentication
  guestListPost(postid: ID!): PostResponse!
}
```

### Mutations (Guest - No Auth Required)

```graphql
type Mutation {
  # User registration
  register(input: RegistrationInput!): RegisterResponse!
  
  # Account verification
  verifyAccount(userid: ID!): DefaultResponse!
  
  # User login
  login(email: String!, password: String!): AuthPayload!
  
  # Password reset flow
  requestPasswordReset(email: String!): ResetPasswordRequestResponse!
  resetPasswordTokenVerify(token: String!): DefaultResponse!
  resetPassword(token: String!, password: String!): DefaultResponse!
  
  # Token refresh
  refreshToken(refreshToken: String!): AuthPayload!
  
  # Referral verification
  verifyReferralString(referralString: String!): ReferralResponse!
  
  # Contact form
  contactus(name: String!, email: String!, message: String!): ContactusResponse!
}
```

### Queries (Authenticated Users)

```graphql
type Query {
  # System info
  hello: HelloResponse!
  
  # User operations
  listUsersV2(
    contentFilterBy: ContentFilterType, 
    userid: ID, 
    username: String, 
    offset: Int, 
    limit: Int
  ): UserListResponse!
  
  getProfile(
    userid: ID,
    contentFilterBy: ContentFilterType
  ): ProfileInfo!
  
  listFollowRelations(
    contentFilterBy: ContentFilterType, 
    userid: ID, 
    offset: Int, 
    limit: Int
  ): FollowRelationsResponse!
  
  listFriends(
    contentFilterBy: ContentFilterType, 
    userid: ID, 
    offset: Int, 
    limit: Int
  ): UserFriendsResponse!
  
  getUserInfo: UserInfoResponse!
  
  listBlockedUsers(
    contentFilterBy: ContentFilterType,
    offset: Int, 
    limit: Int
  ): BlockedUsersResponse!

  # Post operations
  listPosts(
    filterBy: [PostFilterType!], 
    contentFilterBy: ContentFilterType, 
    IgnorList: IgnoreOption, 
    sortBy: PostSortType, 
    userid: ID, 
    postid: ID, 
    title: String, 
    tag: String, 
    from: Date, 
    to: Date, 
    offset: Int, 
    limit: Int, 
    commentOffset: Int, 
    commentLimit: Int
  ): PostListResponse!
  
  listAdvertisementPosts(
    filterBy: [ContentType!], 
    contentFilterBy: ContentFilterType, 
    userid: ID, 
    postid: ID, 
    title: String, 
    tag: String, 
    offset: Int, 
    limit: Int, 
    commentOffset: Int, 
    commentLimit: Int
  ): ListAdvertisementPostsResponse!
  
  # Comment operations
  listComments(
    postid: ID!, 
    contentFilterBy: ContentFilterType, 
    commentOffset: Int, 
    commentLimit: Int
  ): CommentListResponse!
  
  listChildComments(
    parent: ID!, 
    offset: Int, 
    limit: Int
  ): CommentResponse!
  
  # Tag operations
  listTags(offset: Int, limit: Int): TagSearchResponse!
  searchTags(tagName: String!, offset: Int, limit: Int): TagSearchResponse!
  
  # Wallet & Token operations
  balance: CurrentLiquidity!
  getDailyFreeStatus: GetDailyResponse!
  listWinLogs(day: DayFilterType!, offset: Int, limit: Int): UserLogWins!
  listPaymentLogs(day: DayFilterType!, offset: Int, limit: Int): UserLogWins!
  listTodaysInteractions: ListTodaysInteractionsResponse!
  getActionPrices: GetActionPricesResponse!
  getTokenomics: TokenomicsResponse!
  
  getTransactionHistory(
    type: TokenMovementFilterType, 
    direction: DirectionFilterType, 
    start_date: String, 
    end_date: String, 
    limit: Int, 
    offset: Int, 
    sort: SortFilterType
  ): TransactionResponse!
  
  transactionHistory(
    type: TokenMovementFilterType,
    start_date: String,
    end_date: String,
    limit: Int,
    offset: Int,
    sort: SortFilterType
  ): TransactionHistoryResponse!
  
  # Referral operations
  getReferralInfo: ReferralInfoResponse!
  referralList(offset: Int, limit: Int): ReferralListResponse!
  
  # Post eligibility (for creating posts)
  postEligibility: PostEligibilityResponse!
  
  # Interactions
  postInteractions(
    getOnly: GetOnly!, 
    contentFilterBy: ContentFilterType, 
    postOrCommentId: ID!, 
    offset: Int, 
    limit: Int
  ): PostInteractionResponse
  
  # Advertisement history
  advertisementHistory(
    filter: AdvertisementHistoryFilter, 
    sort: AdvertisementSort, 
    offset: Int, 
    limit: Int
  ): ListedAdvertisementData!
  
  # Shop
  shopOrderDetails(transactionId: String!): ShopOrderDetailsResponse!
}
```

### Mutations (Authenticated Users)

```graphql
type Mutation {
  # User profile management
  updateUserPreferences(userPreferences: UserPreferencesInput): UserPreferencesResponse!
  updateUsername(username: String!, password: String!): DefaultResponse!
  updateEmail(email: String!, password: String!): DefaultResponse!
  updatePassword(password: String!, expassword: String!): DefaultResponse!
  updateBio(biography: String!): DefaultResponse!
  updateProfileImage(img: String!): DefaultResponse!
  deleteAccount(password: String!): DefaultResponse!
  
  # Social interactions
  toggleUserFollowStatus(userid: ID!): FollowStatusResponse!
  toggleBlockUserStatus(userid: ID!): DefaultResponse!
  reportUser(userid: ID!): DefaultResponse!
  
  # Post operations
  createPost(action: PostType!, input: PostInput!): PostResponse!
  resolvePostAction(action: PostActionType!, postid: ID!): DefaultResponse!
  
  # Comment operations
  createComment(
    action: CommentType!, 
    postid: ID!, 
    parentid: ID, 
    content: String!
  ): CommentResponse!
  likeComment(commentid: ID!): DefaultResponse!
  reportComment(commentid: ID!): DefaultResponse!
  
  # Token transfers
  resolveTransferV2(
    recipient: ID!, 
    numberoftokens: Decimal!, 
    message: String
  ): TransferTokenResponse!
  
  # Advertisement
  advertisePostBasic(
    postid: ID!, 
    startday: Date!, 
    durationInDays: AdDuration!, 
    advertisePlan: AdvertisementBasicPlan!
  ): ListAdvertisementData!
  
  advertisePostPinned(
    postid: ID!, 
    advertisePlan: AdvertisementPinnedPlan!
  ): ListAdvertisementData!
  
  # Shop purchases
  performShopOrder(
    tokenAmount: Decimal!, 
    shopItemId: String!, 
    orderDetails: ShopOrderDetailsInput!
  ): DefaultResponse!
}
```

### Admin-Only Queries

```graphql
extend type Query {
  # Extended user search with admin fields
  listUsersAdminV2(
    contentFilterBy: ContentFilterType,
    userid: ID,
    email: String,
    username: String,
    status: Int,
    verified: Int,
    ip: String,
    offset: Int,
    limit: Int
  ): UserListResponse!
  
  # Friend relationships
  allfriends(offset: Int, limit: Int): AllUserFriends!
  
  # Gems system
  gemster: GemsterResponse!
  dailygemstatus: DailyGemStatusResponse!
  dailygemsresults(day: DayFilterType!): DailyGemsResultsResponse!
  
  # Mint operations
  getMintAccount: MintAccountResponse!
  
  # Leaderboard
  generateLeaderboard(leaderboardParams: LeaderboardParamsInput!): LeaderboardResponse!
  
  # Post comments (admin view)
  postcomments(postid: ID!, offset: Int, limit: Int): PostCommentsResponse!
}

extend type Mutation {
  # Gems distribution
  globalwins: DefaultResponse!
  gemsters(day: DayFilterType!): GemstersResponse!
  distributeTokensForGems(date: String!): GemstersResponse!
  
  # Alpha minting
  alphaMint: DefaultResponse!
}
```

### Moderator-Only Operations

```graphql
extend type Query {
  # Moderation dashboard
  moderationStats: ModerationStatsResponse!
  
  moderationItems(
    status: ModerationStatus, 
    contentType: ModerationContentType, 
    offset: Int, 
    limit: Int
  ): ModerationItemListResponse!
}

extend type Mutation {
  performModeration(
    moderationTicketId: ID!, 
    moderationAction: ModerationStatus!
  ): DefaultResponse!
}
```

---

## Input Types

### RegistrationInput
```graphql
input RegistrationInput {
  email: String!
  password: String!
  username: String!
  pkey: String           # Optional public key
  referralUuid: ID       # Optional referral ID
}
```

### PostInput
```graphql
input PostInput {
  title: String!
  mediadescription: String
  contenttype: ContentType!    # image, audio, video, text
  media: [String!]
  cover: [String!]
  tags: [String!]
  uploadedFiles: String        # For multipart uploads
}
```

### UserPreferencesInput
```graphql
input UserPreferencesInput {
  contentFilteringSeverityLevel: ContentFilterType
  shownOnboardings: [OnboardingType!]
}
```

### ShopOrderDetailsInput
```graphql
input ShopOrderDetailsInput {
  name: String!
  email: String!
  addressline1: String!
  addressline2: String
  city: String!
  zipcode: String!
  country: ShopSupportedDeliveryCountry!
  shopItemSpecs: ShopItemSpecsInput
}
```

### AdvertisementHistoryFilter
```graphql
input AdvertisementHistoryFilter {
  from: Date
  to: Date
  type: AdvertisementType
  advertisementId: ID
  postId: ID
  userId: ID
}
```

---

## Enums

### Content & Post Types
```graphql
enum ContentType {
  image
  audio
  video
  text
}

enum PostFilterType {
  IMAGE
  AUDIO
  VIDEO
  TEXT
  FOLLOWED
  FOLLOWER
  VIEWED
}

enum PostSortType {
  NEWEST
  TRENDING
  LIKES
  DISLIKES
  VIEWS
  COMMENTS
  FOR_ME
  OLDEST
  FOLLOWER
  FOLLOWED
  RELEVANT
  FRIENDS
}
```

### Actions & Interactions
```graphql
enum PostActionType {
  LIKE
  DISLIKE
  REPORT
  VIEW
  SHARE
  SAVE
}

enum GetOnly {
  VIEW
  LIKE
  DISLIKE
  COMMENTLIKE
}
```

### Token & Transaction
```graphql
enum TokenMovementFilterType {
  TRANSACTION
  AIRDROP
  MINT
  PAYMENT
  BURN
}

enum DirectionFilterType {
  INCOME
  DEDUCTION
}

enum TransactionCategory {
  P2P_TRANSFER
  AD_PINNED
  POST_CREATE
  LIKE
  DISLIKE
  COMMENT
  TOKEN_MINT
  SHOP_PURCHASE
  INVITER_FEE_EARN
}
```

### Advertisement
```graphql
enum AdvertisementType {
  PINNED
  BASIC
}

enum AdDuration {
  ONE_DAY
  TWO_DAYS
  THREE_DAYS
  FOUR_DAYS
  FIVE_DAYS
  SIX_DAYS
  SEVEN_DAYS
}

enum AdvertisementSort {
  NEWEST
  OLDEST
  BIGGEST_COST
  SMALLEST_COST
}
```

### Content Filtering & Visibility
```graphql
enum ContentFilterType {
  MYGRANDMALIKES    # Stricter content filtering
  MYGRANDMAHATES    # Less strict content filtering
}

enum ContentVisibilityStatus {
  NORMAL
  HIDDEN
  ILLEGAL
}
```

### Moderation
```graphql
enum ModerationStatus {
  waiting_for_review
  hidden
  restored
  illegal
}

enum ModerationContentType {
  post
  comment
  user
}
```

### Time Filters
```graphql
enum DayFilterType {
  D0    # Today
  D1    # 1 day ago
  D2    # 2 days ago
  D3    # 3 days ago
  D4    # 4 days ago
  D5    # 5 days ago
  D6    # 6 days ago
  D7    # 7 days ago
  W0    # This week
  M0    # This month
  Y0    # This year
}

enum SortFilterType {
  NEWEST
  OLDEST
}
```

---

## Response Types

### Standard Response Structure

All responses follow a consistent structure with a `meta` object:

```graphql
type DefaultResponse {
  status: String!           # "success" or "error"
  RequestId: String!        # Unique request identifier
  ResponseCode: String!     # Numeric response code
  ResponseMessage: String!  # Human-readable message
}
```

### Authentication Response
```graphql
type AuthPayload {
  meta: DefaultResponse!
  accessToken: String       # JWT access token
  refreshToken: String      # JWT refresh token
}
```

### User Types
```graphql
type User {
  id: ID
  username: String
  status: Int
  slug: Int
  img: String
  biography: String
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
  hasActiveReports: Boolean!
  createdat: Date
  updatedat: Date
  # Admin-only fields:
  email: String
  verified: Int
  roles_mask: Int
  ip: String
  liquidity: Decimal
}

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

### Post Type
```graphql
type Post {
  id: ID!
  contenttype: String!
  title: String!
  media: String!
  cover: String!
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
  isliked: Boolean!
  isviewed: Boolean!
  isreported: Boolean!
  isdisliked: Boolean!
  issaved: Boolean!
  tags: [String]!
  url: String!
  user: ProfileUser!
  comments: [Comment!]!
}
```

### Comment Type
```graphql
type Comment {
  commentid: ID!
  userid: ID!
  postid: ID!
  parentid: ID
  content: String!
  createdat: Date!
  visibilityStatus: ContentVisibilityStatus!
  isHiddenForUsers: Boolean!
  hasActiveReports: Boolean!
  amountlikes: Int!
  amountreplies: Int!
  amountreports: Int!
  isreported: Boolean!
  isliked: Boolean!
  user: ProfileUser!
}
```

### Transaction Type
```graphql
type TransactionHistoryItem {
  operationid: String!
  transactionId: String!
  transactionCategory: TransactionCategory
  transactiontype: String!
  tokenamount: String!
  netTokenAmount: String!
  message: String
  createdat: String!
  sender: BasicUserInfo!
  recipient: BasicUserInfo!
  fees: TransactionFeeSummary
}

type TransactionFeeSummary {
  total: Decimal
  burn: Decimal
  peer: Decimal
  inviter: Decimal
}
```

---

## Custom Scalars

```graphql
scalar Decimal    # Precise decimal numbers for token amounts
scalar Date       # Date/datetime strings
```

---

## File Upload Endpoint

### POST /upload-post

For uploading posts with media files (images, audio, video).

#### Request Format
- **Content-Type**: `multipart/form-data`
- **Authorization**: `Bearer <access_token>`

#### Form Fields
| Field | Type | Description |
|-------|------|-------------|
| `eligibilityToken` | string | Token obtained from `postEligibility` query |
| `file` | file[] | Media file(s) to upload |

#### Response
```json
{
  "status": "success",
  "ResponseCode": "10101",
  "affectedRows": {
    "uploadedFiles": "file1.jpg,file2.jpg"
  }
}
```

---

## Rate Limiting

The API implements request rate limiting per IP address:
- Configurable rate limit (requests per time window)
- Configurable time window
- File-based storage for rate tracking

When rate limited, requests return HTTP 429.

---

## Security Headers

All responses include security headers:

```http
Content-Security-Policy: default-src 'self'; script-src 'self'; object-src 'none';
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
Referrer-Policy: no-referrer
Permissions-Policy: geolocation=(), microphone=(), camera=()
Cache-Control: no-store, no-cache, must-revalidate, max-age=0
Pragma: no-cache
```

### CORS Headers
```http
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: POST
Access-Control-Allow-Headers: Content-Type, Accept, Authorization
Access-Control-Allow-Credentials: true
```

---

## Error Handling

### Error Response Format
```json
{
  "data": {
    "operationName": {
      "meta": {
        "status": "error",
        "RequestId": "uuid-request-id",
        "ResponseCode": "30301",
        "ResponseMessage": "Human readable error message"
      }
    }
  }
}
```

### Common Response Codes

| Code | Description |
|------|-------------|
| `10101` | Generic success |
| `10401` | Contact us success |
| `10601` | Registration success |
| `10701` | Verification success |
| `10801` | Login success |
| `30301` | Missing/invalid required fields |
| `30601` | Email already registered |
| `40301` | Generic database/server error |
| `60501` | Permission denied |

### HTTP Status Codes

| Code | Description |
|------|-------------|
| `200` | Success |
| `400` | Bad request (invalid JSON, missing query) |
| `401` | Invalid access token |
| `429` | Rate limit exceeded |

---

## Example Requests

### Hello (No Auth)
```graphql
query Hello {
  hello {
    currentuserid
    currentVersion
    wikiLink
    lastMergedPullRequestNumber
    companyAccountId
  }
}
```

### Registration
```graphql
mutation Register {
  register(input: {
    email: "user@example.com",
    password: "securePassword123",
    username: "newuser",
    referralUuid: "optional-referral-id"
  }) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    userid
  }
}
```

### Login
```graphql
mutation Login {
  login(
    email: "user@example.com",
    password: "securePassword123"
  ) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    accessToken
    refreshToken
  }
}
```

### List Posts
```graphql
query ListPosts {
  listPosts(
    sortBy: NEWEST,
    limit: 10,
    offset: 0
  ) {
    meta {
      status
      ResponseCode
      ResponseMessage
    }
    counter
    affectedRows {
      id
      title
      contenttype
      media
      amountlikes
      amountviews
      amountcomments
      user {
        id
        username
        img
      }
      tags
      createdat
    }
  }
}
```

### Create Post
```graphql
mutation CreatePost {
  createPost(
    action: POST,
    input: {
      title: "My First Post",
      contenttype: text,
      mediadescription: "This is a text post"
    }
  ) {
    meta {
      status
      ResponseCode
      ResponseMessage
    }
    affectedRows {
      id
      title
    }
  }
}
```

### Transfer Tokens
```graphql
mutation TransferTokens {
  resolveTransferV2(
    recipient: "recipient-user-id",
    numberoftokens: 100.50,
    message: "Thanks for your help!"
  ) {
    meta {
      status
      ResponseCode
      ResponseMessage
    }
    affectedRows {
      tokenSendFormatted
      tokensSubstractedFromWalletFormatted
      createdat
    }
  }
}
```

---

## Testing

API tests are available via Postman/Newman collections in `tests/postman_collection/`:

| Collection | Purpose |
|------------|---------|
| `001_user_graphql_postman_collection.json` | User operations |
| `002_posts_graphql_postman_collection.json` | Post operations |
| `003_wallet_graphql_postman_collection.json` | Wallet operations |
| `004_advertisements_graphql_postman_collection.json` | Advertisement operations |
| `005_moderation_graphql_postman_collection.json` | Moderation operations |
| `006_shop_graphql_postman_collection.json` | Shop operations |
| `007_mint_graphql_postman_collection.json` | Mint operations |

Run all tests:
```bash
make hot-ci
```

---

## Environment Configuration

Key environment variables affecting the API:

| Variable | Description |
|----------|-------------|
| `APP_ENV` | Environment (DEVELOPMENT/PRODUCTION) |
| `DB_HOST`, `DB_DATABASE`, `DB_USERNAME`, `DB_PASSWORD` | Database connection |
| `TOKEN_EXPIRY` | Access token validity (seconds) |
| `REFRESH_TOKEN_EXPIRY` | Refresh token validity (seconds) |
| `LIMITER_RATE` | Rate limit (requests per window) |
| `LIMITER_TIME` | Rate limit time window (seconds) |
| `MEDIA_SERVER_URL` | URL for media file serving |
