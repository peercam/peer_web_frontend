# Wallet & Token Transfers API

## Overview

The Wallet & Tokens domain covers querying the user's token balance, peer-to-peer transfers with fee calculations, and viewing transaction/payment history. All token amounts use the `Decimal` scalar for precision.

**Authentication**: Required for all operations.

---

## Queries

### `balance`

Get the current user's token balance.

```graphql
query {
  balance: CurrentLiquidity!
}
```

#### Response: `CurrentLiquidity`

```graphql
type CurrentLiquidity {
  meta: DefaultResponse!
  currentliquidity: Decimal         # Current token balance
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11204` | Balance retrieved |
| `41201` | Failed to fetch balance |
| `60501` | Not authenticated |

---

### `getTransactionHistory`

Fetch the user's transaction history with optional filters.

```graphql
query {
  getTransactionHistory(
    type: TokenMovementFilterType
    direction: DirectionFilterType
    start_date: String
    end_date: String
    limit: Int
    offset: Int
    sort: SortFilterType
  ): TransactionResponse!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | `TokenMovementFilterType` | No | Filter by transaction type |
| `direction` | `DirectionFilterType` | No | Filter by direction |
| `start_date` | `String` | No | Start date (YYYY-MM-DD) |
| `end_date` | `String` | No | End date (YYYY-MM-DD) |
| `limit` | `Int` | No | Results per page (max: 20) |
| `offset` | `Int` | No | Pagination offset |
| `sort` | `SortFilterType` | No | Sort order |

```graphql
enum TokenMovementFilterType {
  TRANSACTION      # P2P transfers
  AIRDROP          # Airdrop distributions
  MINT             # Token minting events
  PAYMENT          # Action payments (like, post, etc.)
  BURN             # Token burn events
}

enum DirectionFilterType {
  INCOME           # Tokens received
  DEDUCTION        # Tokens spent/sent
}

enum SortFilterType {
  NEWEST           # Most recent first
  OLDEST           # Oldest first
}
```

#### Response: `TransactionResponse`

```graphql
type TransactionResponse {
  meta: DefaultResponse!
  affectedRows: [Transaction!]
}

type Transaction {
  transactionid: String!
  operationid: String!
  transactiontype: String!
  senderid: String!
  recipientid: String!
  tokenamount: Decimal!
  transferaction: String!
  message: String!
  createdat: String!
  sender: BasicUserInfo!
  recipient: BasicUserInfo!
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11215` | Transactions retrieved |
| `21209` | No transactions found |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `60501` | Not authenticated |

---

### `transactionHistory`

Enhanced transaction history with fee breakdowns and transaction categories.

```graphql
query {
  transactionHistory(
    type: TokenMovementFilterType
    start_date: String
    end_date: String
    limit: Int
    offset: Int
    sort: SortFilterType
  ): TransactionHistoryResponse!
}
```

#### Parameters

Same as `getTransactionHistory` except no `direction` filter.

#### Response: `TransactionHistoryResponse`

```graphql
type TransactionHistoryResponse {
  meta: DefaultResponse
  affectedRows: [TransactionHistoryItem!]
}

type TransactionHistoryItem {
  operationid: String!                  # Groups related transactions
  transactionId: String!                # Unique transaction ID
  transactionCategory: TransactionCategory
  transactiontype: String!              # CREDIT transaction type
  tokenamount: String!                  # Gross amount (including fees)
  netTokenAmount: String!               # Net amount (CREDIT transaction value)
  message: String
  createdat: String!
  sender: BasicUserInfo!
  recipient: BasicUserInfo!
  fees: TransactionFeeSummary
}

type TransactionFeeSummary {
  total: Decimal           # Sum of all fee transactions
  burn: Decimal            # Amount burned
  peer: Decimal            # Amount to Peer platform
  inviter: Decimal         # Amount to inviter (can be null)
}
```

```graphql
enum TransactionCategory {
  P2P_TRANSFER         # User-to-user transfer
  AD_PINNED            # Pinned advertisement payment
  POST_CREATE          # Post creation payment
  LIKE                 # Like action payment
  DISLIKE              # Dislike action payment
  COMMENT              # Comment action payment
  TOKEN_MINT           # Minted tokens distribution
  SHOP_PURCHASE        # Shop order payment
  INVITER_FEE_EARN     # Inviter fee received
}
```

> **Note:** The PHP backend also defines a `FEE` case on `TransactionCategory` that is not exposed in the GraphQL enum. Fee-only transaction rows will have `transactionCategory: null` in API responses.

---

### `listWinLogs`

Fetch the user's win (earning) logs for a specific day.

```graphql
query {
  listWinLogs(
    day: DayFilterType!
    offset: Int
    limit: Int
  ): UserLogWins!
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `day` | `DayFilterType!` | Yes | Time period to query |
| `offset` | `Int` | No | Pagination offset |
| `limit` | `Int` | No | Max: 20 |

```graphql
enum DayFilterType {
  D0      # Today
  D1      # 1 day ago
  D2      # 2 days ago
  D3      # 3 days ago
  D4      # 4 days ago
  D5      # 5 days ago
  D6      # 6 days ago
  D7      # 7 days ago
  W0      # This week
  M0      # This month
  Y0      # This year
}
```

#### Response: `UserLogWins`

```graphql
type UserLogWins {
  meta: DefaultResponse!
  counter: Int!
  affectedRows: [LogWins]
}

type LogWins {
  from: String
  token: String
  userid: String
  postid: String
  action: String
  numbers: Decimal
  createdat: String
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11203` | Logs found |
| `21202` | No records for date |
| `30101` | Missing day parameter |
| `30105` | Invalid day option |
| `60501` | Not authenticated |

---

### `listPaymentLogs`

Fetch the user's payment (spending) logs for a specific day.

```graphql
query {
  listPaymentLogs(
    day: DayFilterType!
    offset: Int
    limit: Int
  ): UserLogWins!
}
```

Same parameters, response, and codes as `listWinLogs`.

---

### `listTodaysInteractions`

Get a summary of today's interactions on the current user's posts.

```graphql
query {
  listTodaysInteractions: ListTodaysInteractionsResponse!
}
```

#### Response: `ListTodaysInteractionsResponse`

```graphql
type ListTodaysInteractionsResponse {
  meta: DefaultResponse!
  affectedRows: TodaysInteractionsData
}

type TodaysInteractionsData {
  totalInteractions: Decimal
  totalScore: Decimal
  totalDetails: TodaysInteractionsDetailsData
}

type TodaysInteractionsDetailsData {
  views: Decimal
  likes: Decimal
  dislikes: Decimal
  comments: Decimal
  viewsScore: Decimal
  likesScore: Decimal
  dislikesScore: Decimal
  commentsScore: Decimal
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11205` | Interactions retrieved |
| `21204` | No interactions today |
| `41205` | Failed to fetch |
| `60501` | Not authenticated |

---

## Mutations

### `resolveTransferV2`

Transfer tokens to another user. Includes fee calculations and inviter fee distribution.

```graphql
mutation {
  resolveTransferV2(
    recipient: ID!
    numberoftokens: Decimal!
    message: String
  ): TransferTokenResponse!
}
```

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `recipient` | `ID!` | Yes | Valid user UUID |
| `numberoftokens` | `Decimal!` | Yes | Min: 0.000001, max: 8 decimal places |
| `message` | `String` | No | Max: 500 chars, no URLs, no control chars |

#### Validation Rules
- Recipient must be a valid UUID
- Cannot transfer to yourself
- Cannot transfer to system/fees accounts
- Recipient must exist and pass content filtering
- Message cannot contain URLs (pattern: `(:\/\/|www\.)`)
- Message cannot contain control characters
- Sender must have sufficient balance (amount + fees)

#### Fee Structure

Fees are applied on top of the transfer amount:

| Fee | Rate | Recipient |
|-----|------|-----------|
| Burn | 1% | Burned (sent to burn account) |
| Peer | 2% | Platform fee (sent to company account) |
| Inviter | 1% | Sender's inviter (if exists) |

**Total fee**: 4% of transfer amount (3% if no inviter)

Example: Transferring 100 tokens costs the sender 104 tokens total (100 + 4 fees).

#### Response: `TransferTokenResponse`

```graphql
type TransferTokenResponse {
  meta: DefaultResponse!
  affectedRows: TransferToken!
}

type TransferToken {
  tokenSend: Decimal                               # @deprecated — use tokenSendFormatted
  tokensSubstractedFromWallet: Decimal             # @deprecated — use tokensSubstractedFromWalletFormatted
  tokenSendFormatted: String!                      # Tokens sent to recipient
  tokensSubstractedFromWalletFormatted: String!    # Total deducted from sender
  createdat: String
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11211` | Transfer successful |
| `30201` | Invalid recipient UUID |
| `30264` | Invalid token amount (min amount / max decimals) |
| `30270` | Message too long (exceeds max length) |
| `30271` | Invalid message (control characters or URLs) |
| `31007` | Recipient not found or filtered |
| `31202` | Cannot transfer to self |
| `31203` | Cannot transfer to fees account |
| `40301` | General error |
| `41229` | Transaction failed |
| `51301` | Insufficient balance |
| `60501` | Not authenticated |

---

## Action Codes (Internal Reference)

Token transactions use numeric action codes internally:

| Code | Action |
|------|--------|
| 1 | VIEW |
| 2 | LIKE |
| 3 | DISLIKE |
| 4 | COMMENT |
| 5 | POST |
| 6 | POSTINVESTBASIC |
| 7 | POSTINVESTPREMIUM |
| 8 | REPORT |
| 11 | INVITATION |
| 12 | OWNSHARED |
| 13 | OTHERSHARED |
| 14 | DIRECTDEBIT |
| 15 | CREDIT |
| 18 | TRANSFER |
| 30 | FREELIKE |
| 31 | FREECOMMENT |
| 32 | FREEPOST |
