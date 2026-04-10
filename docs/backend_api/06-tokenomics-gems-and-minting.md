# Tokenomics, Gems & Minting API

## Overview

The Tokenomics domain covers the Peer platform's economic system: action pricing, daily free action allowances, gems (earned through post interactions), token minting (converting gems to tokens), and the overall tokenomics configuration.

**Authentication**: Required for all operations. Admin-only operations are noted.

---

## Queries

### `getActionPrices`

Get the current token prices for all paid actions.

```graphql
query {
  getActionPrices: GetActionPricesResponse!
}
```

#### Response: `GetActionPricesResponse`

```graphql
type GetActionPricesResponse {
  meta: DefaultResponse!
  affectedRows: ActionPriceResult!
}

type ActionPriceResult {
  postPrice: Float!          # 20.0 tokens
  likePrice: Float!          # 3.0 tokens
  dislikePrice: Float!       # 3.0 tokens
  commentPrice: Float!       # 1.0 tokens
}
```

#### Current Action Token Prices

| Action | Token Cost |
|--------|-----------|
| Create Post | 20.0 |
| Like | 3.0 |
| Dislike | 3.0 |
| Comment | 1.0 |
| Basic Advertisement | 50.0 |
| Pinned Advertisement | 200.0 |

#### Response Codes

| Code | Description |
|------|-------------|
| `11304` | Prices fetched |
| `41301` | Failed to fetch |
| `60501` | Not authenticated |

---

### `getTokenomics`

Get comprehensive tokenomics data: action prices, gems returns, and minting data.

```graphql
query {
  getTokenomics: TokenomicsResponse!
}
```

#### Response: `TokenomicsResponse`

```graphql
type TokenomicsResponse {
  meta: DefaultResponse!
  actionTokenPrices: ActionPriceResult!
  actionGemsReturns: ActionGemsReturns!
  mintingData: MintingData!
}

type ActionGemsReturns {
  viewGemsReturn: Float!       # 0.25 gems per view
  likeGemsReturn: Float!       # 5.0 gems per like
  dislikeGemsReturn: Float!    # -3.0 gems per dislike
  commentGemsReturn: Float!    # 2.0 gems per comment
}

type MintingData {
  tokensMintedYesterday: Float!
}
```

#### Gems Returns

When other users interact with your posts, you earn gems:

| Interaction | Gems Earned |
|-------------|-------------|
| View | +0.25 |
| Like | +5.0 |
| Dislike | -3.0 |
| Comment | +2.0 |

#### Minting
- **Daily minting budget**: 5,000 tokens/day
- Tokens are distributed proportionally based on each user's gem share

#### Response Codes

| Code | Description |
|------|-------------|
| `11212` | Tokenomics data retrieved |
| `60501` | Not authenticated |

---

### `getDailyFreeStatus`

Get the current user's daily free action usage and availability.

```graphql
query {
  getDailyFreeStatus: GetDailyResponse!
}
```

#### Response: `GetDailyResponse`

```graphql
type GetDailyResponse {
  meta: DefaultResponse!
  affectedRows: [DailyFreeResponse]
}

type DailyFreeResponse {
  name: String!          # Action name (e.g., "post", "like", "comment", "dislike")
  used: Int!             # How many free actions used today
  available: Int!        # How many free actions remaining
}
```

#### Daily Free Action Limits

| Action | Daily Free Allowance |
|--------|---------------------|
| Post | 1 |
| Like | 3 |
| Comment | 4 |
| Dislike | 0 |

When a free action is available, performing that action costs 0 tokens and uses one of the daily allowance. Resets daily.

#### Response Codes

| Code | Description |
|------|-------------|
| `11303` | Daily free status loaded |
| `40301` | Error fetching status |
| `60501` | Not authenticated |

---

## Admin-Only Operations

The following operations require the `ADMIN` role (bitmask 16) or `SUPER_ADMIN` role (bitmask 262144).

### `gemster` (Query)

Get uncollected gems statistics.

```graphql
query {
  gemster: GemsterResponse!
}
```

#### Response: `GemsterResponse`

```graphql
type GemsterResponse {
  meta: DefaultResponse!
  affectedRows: DailyGemStatusData
}

type DailyGemStatusData {
  d0: Decimal       # Today's gems
  d1: Decimal       # Yesterday's gems
  d2: Decimal       # 2 days ago
  d3: Decimal
  d4: Decimal
  d5: Decimal
  d6: Decimal
  d7: Decimal
  w0: Decimal       # This week total
  m0: Decimal       # This month total
  y0: Decimal       # This year total
}
```

---

### `dailygemstatus` (Query)

Get daily gem status overview (available on bridge schema as well).

```graphql
query {
  dailygemstatus: DailyGemStatusResponse!
}
```

#### Response: `DailyGemStatusResponse`

Same `DailyGemStatusData` format as `gemster`.

---

### `dailygemsresults` (Query)

Get per-user gems breakdown for a specific day.

```graphql
query {
  dailygemsresults(day: DayFilterType!): DailyGemsResultsResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `day` | `DayFilterType!` | Yes |

#### `DayFilterType` Enum

| Value | Meaning |
|-------|---------|
| `D0` | Today |
| `D1` | Yesterday |
| `D2` | 2 days ago |
| `D3` | 3 days ago |
| `D4` | 4 days ago |
| `D5` | 5 days ago |
| `D6` | 6 days ago |
| `D7` | 7 days ago |
| `W0` | This week |
| `M0` | This month |
| `Y0` | This year |

#### Response: `DailyGemsResultsResponse`

```graphql
type DailyGemsResultsResponse {
  meta: DefaultResponse!
  affectedRows: DailyGemsResultsData
}

type DailyGemsResultsData {
  data: [DailyGemsResultsUserData]
  totalGems: Decimal
}

type DailyGemsResultsUserData {
  userid: ID
  pkey: ID            # Solana public key
  gems: Decimal       # Gems earned by this user
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11207` | Gems data loaded |
| `21206` | No gems for given day |
| `30223` | Invalid day parameter |
| `60501` | Not authenticated |

---

### `globalwins` (Mutation)

Convert all pending post interactions into gems. This is the gem generation step.

```graphql
mutation {
  globalwins: DefaultResponse!
}
```

#### Gems Generation Mapping

For each user's posts, interactions are converted to gems using these factors:
- `user_post_views` × `viewGemsReturn` (0.25)
- `user_post_likes` × `likeGemsReturn` (5.0)
- `user_post_dislikes` × `dislikeGemsReturn` (-3.0)
- `user_post_comments` × `commentGemsReturn` (2.0)

#### Response Codes

| Code | Description |
|------|-------------|
| `11206` | Interactions converted to gems |
| `21205` | No interactions to convert |
| `41215` | Conversion failed |
| `60501` | Not authenticated |

---

### `distributeTokensForGems` (Mutation)

Mint and distribute tokens from gems for a specific date. This is the token distribution step.

```graphql
mutation {
  distributeTokensForGems(date: String!): GemstersResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `date` | `String!` | Yes | YYYY-MM-DD format, cannot be future date |

#### Distribution Logic
1. Calculate total gems for the given date
2. Calculate `gemsintoken = DAILY_NUMBER_TOKEN (5000) / totalGems`
3. For each user: `tokens = user_gems × gemsintoken`
4. Credit tokens to each user's wallet from the mint account

#### Validation
- Date must be valid YYYY-MM-DD
- Cannot mint for future dates
- Cannot duplicate minting for the same date

#### Response: `GemstersResponse`

```graphql
type GemstersResponse {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: GemstersData
}

type GemstersData {
  winStatus: WinStatus
  userStatus: [GemstersUserStatus]
}

type WinStatus {
  totalGems: Decimal
  gemsintoken: Decimal        # Token value per gem
  bestatigung: Decimal        # Confirmation value
}

type GemstersUserStatus {
  userid: ID
  gems: Decimal
  tokens: Decimal             # Tokens distributed
  percentage: Decimal         # User's gem share percentage
  details: [GemstersUserStatusDetails]
}

type GemstersUserStatusDetails {
  gemid: ID
  userid: ID
  postid: ID
  fromid: ID
  gems: Decimal
  numbers: Decimal
  whereby: Decimal
  createdat: Date
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11208` | Tokens distributed |
| `21206` | No gems for date |
| `30105` | Invalid date format |
| `31204` | Already minted for date |
| `40301` | Distribution failed |
| `60501` | Not authenticated |

---

### `gemsters` (Mutation)

Mint and distribute tokens from gems for a relative day offset. Same distribution logic as `distributeTokensForGems` but accepts a `DayFilterType` enum instead of a date string.

```graphql
mutation {
  gemsters(day: DayFilterType!): GemstersResponse!
}
```

#### Parameters

| Field | Type | Required |
|-------|------|----------|
| `day` | `DayFilterType!` | Yes |

See [`DayFilterType` Enum](#dayfiltertype-enum) for valid values.

Response format and codes are the same as [`distributeTokensForGems`](#distributetokensforgems-mutation).

---

### `getMintAccount` (Query)

Get the mint account balance and details. Admin-only.

```graphql
query {
  getMintAccount: MintAccountResponse!
}
```

#### Response: `MintAccountResponse`

```graphql
type MintAccountResponse {
  meta: DefaultResponse!
  mintAccount: MintAccount!
}

type MintAccount {
  accountid: ID!
  initialBalance: Decimal!
  currentBalance: Decimal!
  updatedat: String!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `0` | Account retrieved |
| `40401` | Account not found |
| `40301` | Fetch error |
| `60501` | Not authenticated |

---

### `alphaMint` (Mutation)

Distribute alpha tokens to pre-mapped users from a JSON file. One-time admin operation.

```graphql
mutation {
  alphaMint: DefaultResponse!
}
```

#### Logic
1. Creates/loads a temporary `alpha_mint@peerapp.de` account
2. Reads `runtime-data/Alpha_tokens_to_Peer_tokens.json`
3. Maps `peer_username` / `peer_app_slug` → `alpha_user_tokens`
4. Validates all users exist
5. Transfers tokens to each user
6. Skips duplicate transfers (same user + amount)
7. Cleans up temp account after completion

#### Response Codes

| Code | Description |
|------|-------------|
| `200` | Alpha mint complete |
| `404` | Some users not found (error message lists missing usernames) |
| `41020` | Mint operation failed |
| `60501` | Not authenticated |

---

## Bridge Schema Operations

The Web3 Bridge schema (`bridge_schema.graphql`) provides read-only access to gem data for the `WEB3_BRIDGE_USER` role (bitmask 8):

```graphql
type Query {
  hello: HelloResponse!
  dailygemstatus: DailyGemStatusResponse!
  dailygemsresults(day: DayFilterType!): DailyGemsResultsResponse!
}
```

These are the same queries as the admin gem queries, accessible to bridge users for cross-system data synchronization.
