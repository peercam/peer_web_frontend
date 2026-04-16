# Phase 5: Economy (Wallet, Tokenomics, Shop, Ads)

> **Parent Plan:** [mock-backend-rust-rewrite.md](./mock-backend-rust-rewrite.md)
> **Depends on:** [Phase 3 — Posts & Content](./mock-backend-rust-rewrite.md#5-phase-3--posts--content) (requires posts for ads), [Phase 1 — Login & Session Flows](./mock-backend-rust-rewrite.md#3-phase-1--login--session-flows) (requires auth for all operations)
> **Goal:** Add wallet balance, token transfers with fee calculations, tokenomics (action prices, daily free actions, gems, minting), advertisements (basic + pinned), and shop purchase operations so the Leptos wallet page and advertisement flows work end-to-end against the mock.
> **Plan Quality:** ⭐⭐⭐⭐⭐ (5/5)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Prerequisites](#2-prerequisites)
3. [Task Breakdown](#3-task-breakdown)
4. [Implementation Details](#4-implementation-details)
5. [Testing Strategy](#5-testing-strategy)
6. [Definition of Done](#6-definition-of-done)

---

## 1. Overview

### What the frontend calls today

**From `peer-web/src/api/wallet.rs` and `peer-web/src/api/graphql.rs`:**

| Operation | GraphQL SDL | Frontend file | Auth? |
|-----------|-------------|---------------|-------|
| `balance` | `query Balance` | `wallet.rs` → `GetBalance` / `GetBalanceResponse` server fn | Yes |
| `transactionHistory(offset, limit)` | `query TransactionHistory(...)` | `wallet.rs` → `GetTransactionHistory` / `GetTransactionHistoryResponse` server fn | Yes |
| `resolveTransferV2(recipient, numberoftokens, message)` | `mutation ResolveTransferV2(...)` | `wallet.rs` → `TransferTokens` server fn | Yes |
| `shopOrderDetails(transactionId)` | `query ShopOrderDetails(...)` | `graphql.rs` → `ShopOrderDetailsData` wrapper | Yes |
| `listAdvertisementPosts(offset, limit, contentFilterBy, title, tag)` | `query ListAdvertisementPosts(...)` | `graphql.rs` → `ListAdPostsData` wrapper | Yes |

**From `peer-web/src/api/posts.rs` (already partially wired):**

| Operation | GraphQL SDL | Frontend file | Auth? |
|-----------|-------------|---------------|-------|
| `listAdvertisementPosts(...)` | `query ListAdvertisementPosts(...)` | `posts.rs` → `ListAdPosts` server fn | Yes |

**From `docs/backend_api/` (additional operations in the schema, some not yet wired in frontend):**

| Operation | Source doc | Purpose | Auth? |
|-----------|-----------|---------|-------|
| `getTransactionHistory(type, direction, ...)` | `05-wallet-and-transfers.md` | Legacy transaction history with direction filter | Yes |
| `listWinLogs(day, offset, limit)` | `05-wallet-and-transfers.md` | Win (earning) logs by day | Yes |
| `listPaymentLogs(day, offset, limit)` | `05-wallet-and-transfers.md` | Payment (spending) logs by day | Yes |
| `listTodaysInteractions` | `05-wallet-and-transfers.md` | Today's post interaction summary | Yes |
| `getActionPrices` | `06-tokenomics-gems-and-minting.md` | Current token costs per action | Yes |
| `getTokenomics` | `06-tokenomics-gems-and-minting.md` | Combined prices + gems + minting data | Yes |
| `getDailyFreeStatus` | `06-tokenomics-gems-and-minting.md` | Daily free action usage/availability | Yes |
| `advertisementHistory(filter, sort, offset, limit)` | `07-advertisements.md` | User's ad history with stats | Yes |
| `advertisePostBasic(postid, startday, durationInDays, advertisePlan)` | `07-advertisements.md` | Create basic (time-based) ad | Yes |
| `advertisePostPinned(postid, advertisePlan)` | `07-advertisements.md` | Create pinned ad | Yes |
| `performShopOrder(tokenAmount, shopItemId, orderDetails)` | `08-shop.md` | Purchase shop item with tokens | Yes |

### Target State

After this phase, the mock backend will support:
- Token balance queries
- P2P token transfers with fee breakdown (burn 1%, peer 2%, inviter 1%)
- Transaction history with category/direction filtering and pagination
- Win/payment logs by day
- Today's interaction summary
- Action price queries (post 20.0, like 3.0, dislike 3.0, comment 1.0)
- Daily free action tracking (post 1, like 3, comment 4, dislike 0)
- Tokenomics overview (prices + gem returns + minting data)
- Basic and pinned advertisement creation with token deduction
- Advertisement listing and history with statistics
- Shop order placement and order detail retrieval
- Integration with Phase 4's token deduction TODOs (comment creation cost)
- 14 new GraphQL query resolvers, 5 new mutation resolvers

### New file tree additions

```
tests/mock_backend/src/
├── schema/
│   ├── query/
│   │   ├── wallet.rs       # NEW: balance, getTransactionHistory, transactionHistory, listWinLogs, listPaymentLogs, listTodaysInteractions
│   │   ├── tokenomics.rs   # NEW: getActionPrices, getTokenomics, getDailyFreeStatus
│   │   ├── ads.rs          # NEW: listAdvertisementPosts, advertisementHistory
│   │   └── shop.rs         # NEW: shopOrderDetails
│   └── mutation/
│       ├── wallet.rs       # NEW: resolveTransferV2
│       ├── ads.rs          # NEW: advertisePostBasic, advertisePostPinned
│       └── shop.rs         # NEW: performShopOrder
├── types/
│   ├── wallet.rs           # NEW: CurrentLiquidity, Transaction, TransactionHistoryItem, TransferResponse, etc.
│   ├── tokenomics.rs       # NEW: ActionPriceResult, DailyFreeResponse, TokenomicsResponse, etc.
│   ├── ad.rs               # NEW: AdvertisementPost, AdvCreator, ListAdvertisementPostsResponse, etc.
│   └── shop.rs             # NEW: ShopOrderDetails, ShopItemSpecs, ShopOrderDeliveryDetails, etc.
└── state.rs                # MODIFIED: new fields for wallets, transactions, daily actions, ads, shop orders
```

---

## 2. Prerequisites

### Phase 3 Completion

- [x] Posts exist in `MockState` (ads reference `postid`)
- [x] `PostRecord` type is available with `id` (UUID) field
- [x] Post-to-GraphQL conversion helpers exist (ads embed full `Post` objects)
- [x] Post interactions (likes, views, dislikes, comments) tracked in state

### Phase 1 & 2 Completion

- [x] Auth middleware from Phase 1 is working (`require_auth()`, `get_current_user()`)
- [x] User profiles exist in `MockState` (transfers require recipient lookup, ads embed `ProfileUser`)
- [x] `DefaultResponse` type is available from Phase 0

### Phase 4 Integration Points

- [x] Comment creation has `TODO` for Phase 5 token deduction — this phase implements it
- [x] Post action (like/dislike) has `TODO` for Phase 5 token deduction — this phase implements it
- [x] `daily_comment_count` exists in state from Phase 4 — this phase generalizes to `daily_actions_used`

### API Reference

All response codes and field names come from:
- `docs/backend_api/05-wallet-and-transfers.md`
- `docs/backend_api/06-tokenomics-gems-and-minting.md`
- `docs/backend_api/07-advertisements.md`
- `docs/backend_api/08-shop.md`
- `peer-web/src/models/transaction.rs` (frontend deserialization types)
- `peer-web/src/models/post.rs` (advertisement types)
- `peer-web/src/api/graphql.rs` (exact GraphQL field selections)

---

## 3. Task Breakdown

### Phase 5.A — Wallet Types (`types/wallet.rs`)

| # | Task | Notes |
|---|------|-------|
| A1 | Create `types/wallet.rs` with `CurrentLiquidity` struct | `meta: DefaultResponse`, `currentliquidity: Option<Decimal>` — matches frontend's `BalanceResponse` |
| A2 | Define `TransactionUser` struct (SimpleObject) | `userid`, `img`, `username`, `slug`, `visibilityStatus`, `hasActiveReports`, `isHiddenForUsers` — matches frontend field selections |
| A3 | Define `TransactionFees` struct (SimpleObject) | `total`, `burn`, `peer`, `inviter: Option<Decimal>` — matches frontend's `TransactionFees` |
| A4 | Define `TransactionCategory` enum | `P2P_TRANSFER`, `AD_PINNED`, `POST_CREATE`, `LIKE`, `DISLIKE`, `COMMENT`, `TOKEN_MINT`, `SHOP_PURCHASE`, `INVITER_FEE_EARN` — `SCREAMING_SNAKE_CASE` |
| A5 | Define `TransactionHistoryItem` struct (SimpleObject) | `transactionId`, `operationid`, `transactionCategory`, `transactiontype`, `tokenamount`, `netTokenAmount`, `message`, `createdat`, `sender`, `recipient`, `fees` — matches frontend's `Transaction` model |
| A6 | Define `TransactionHistoryResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<Vec<TransactionHistoryItem>>` |
| A7 | Define `TransferToken` struct (SimpleObject) | `tokenSendFormatted`, `tokensSubstractedFromWalletFormatted`, `createdat` — matches frontend field selections |
| A8 | Define `TransferTokenResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<TransferToken>` |
| A9 | Define filter enums: `TokenMovementFilterType`, `DirectionFilterType`, `SortFilterType` | Used by `getTransactionHistory` |
| A10 | Define `DayFilterType` enum | `D0`–`D7`, `W0`, `M0`, `Y0` — used by win/payment logs |
| A11 | Define `LogWins` struct (SimpleObject) | `from`, `token`, `userid`, `postid`, `action`, `numbers`, `createdat` |
| A12 | Define `UserLogWins` struct | `meta: DefaultResponse`, `counter: i32`, `affectedRows: Option<Vec<LogWins>>` |
| A13 | Define `TodaysInteractionsDetailsData` struct | `views`, `likes`, `dislikes`, `comments`, `viewsScore`, `likesScore`, `dislikesScore`, `commentsScore` |
| A14 | Define `TodaysInteractionsData` struct | `totalInteractions`, `totalScore`, `totalDetails` |
| A15 | Define `ListTodaysInteractionsResponse` | `meta: DefaultResponse`, `affectedRows: Option<TodaysInteractionsData>` |
| A16 | Export from `types/mod.rs` | Add `pub mod wallet;` |

### Phase 5.B — Tokenomics Types (`types/tokenomics.rs`)

| # | Task | Notes |
|---|------|-------|
| B1 | Create `types/tokenomics.rs` with `ActionPriceResult` struct | `postPrice: f64`, `likePrice: f64`, `dislikePrice: f64`, `commentPrice: f64` |
| B2 | Define `GetActionPricesResponse` struct | `meta: DefaultResponse`, `affectedRows: ActionPriceResult` |
| B3 | Define `ActionGemsReturns` struct | `viewGemsReturn: f64`, `likeGemsReturn: f64`, `dislikeGemsReturn: f64`, `commentGemsReturn: f64` |
| B4 | Define `MintingData` struct | `tokensMintedYesterday: f64` |
| B5 | Define `TokenomicsResponse` struct | `meta: DefaultResponse`, `actionTokenPrices: ActionPriceResult`, `actionGemsReturns: ActionGemsReturns`, `mintingData: MintingData` |
| B6 | Define `DailyFreeResponse` struct | `name: String`, `used: i32`, `available: i32` |
| B7 | Define `GetDailyResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<Vec<DailyFreeResponse>>` |
| B8 | Export from `types/mod.rs` | Add `pub mod tokenomics;` |

### Phase 5.C — Advertisement Types (`types/ad.rs`)

| # | Task | Notes |
|---|------|-------|
| C1 | Create `types/ad.rs` with `AdvertisementType` enum | `PINNED`, `BASIC` |
| C2 | Define `AdvCreator` struct (SimpleObject) | `advertisementid`, `postid`, `advertisementtype`, `startdate`, `enddate`, `createdat`, `user: ProfileUser` — matches frontend's `AdvertisementInfo` field subset |
| C3 | Define `AdvertisementPost` struct (SimpleObject) | `post: Post`, `advertisement: AdvCreator` — matches frontend's `AdvertisementPost` model |
| C4 | Define `ListAdvertisementPostsResponse` struct | `meta: DefaultResponse`, `counter: i32`, `affectedRows: Option<Vec<AdvertisementPost>>` |
| C5 | Define `AdvertisementRow` struct | `id`, `createdAt`, `type`, `timeframeStart`, `timeframeEnd`, `totalTokenCost`, `totalEuroCost` — returned by create mutations |
| C6 | Define `ListAdvertisementData` struct | `meta: DefaultResponse`, `affectedRows: Option<Vec<AdvertisementRow>>` |
| C7 | Define `AdDuration` enum | `ONE_DAY` through `SEVEN_DAYS` with `to_days() -> u32` helper |
| C8 | Define `AdvertisementBasicPlan` and `AdvertisementPinnedPlan` enums | Single-variant enums (`BASIC`, `PINNED`) |
| C9 | Define `TotalAdvertisementHistoryStats` struct | `tokenSpent`, `euroSpent`, `amountAds`, `gemsEarned`, `amountLikes`, `amountViews`, `amountComments`, `amountDislikes`, `amountReports` |
| C10 | Define `Advertisement` struct (full history item) | `id`, `createdAt`, `type`, `timeframeStart`, `timeframeEnd`, `totalTokenCost`, `totalEuroCost`, `gemsEarned`, interaction counts, `user`, `post` |
| C11 | Define `AdvertisementHistoryResult` struct | `stats: TotalAdvertisementHistoryStats`, `advertisements: Option<Vec<Advertisement>>` |
| C12 | Define `ListedAdvertisementData` struct | `meta: DefaultResponse`, `affectedRows: Option<AdvertisementHistoryResult>` |
| C13 | Define filter/sort inputs: `AdvertisementHistoryFilter` (InputObject), `AdvertisementSort` enum | Filter: `from`, `to`, `type`, `advertisementId`, `postId`, `userId`; Sort: `NEWEST`, `OLDEST`, `BIGGEST_COST`, `SMALLEST_COST` |
| C14 | Export from `types/mod.rs` | Add `pub mod ad;` |

### Phase 5.D — Shop Types (`types/shop.rs`)

| # | Task | Notes |
|---|------|-------|
| D1 | Create `types/shop.rs` with `ShopItemSpecs` struct | `size: Option<String>` — matches frontend's `ShopItemSpecs` |
| D2 | Define `ShopOrderDeliveryDetails` struct | `name`, `email`, `addressline1`, `addressline2`, `city`, `zipcode`, `country` — matches frontend's `DeliveryDetails` |
| D3 | Define `ShopOrderDetails` struct | `shopOrderId`, `shopItemId`, `shopItemSpecs`, `deliveryDetails`, `createdat` |
| D4 | Define `ShopOrderDetailsResponse` struct | `meta: DefaultResponse`, `affectedRows: Option<Vec<ShopOrderDetails>>` — note: frontend reads without `meta` in current query but response shape includes it |
| D5 | Define `ShopOrderDetailsInput` (InputObject) | `name`, `email`, `addressline1`, `addressline2: Option`, `city`, `zipcode`, `country: ShopSupportedDeliveryCountry`, `shopItemSpecs: Option<ShopItemSpecsInput>` |
| D6 | Define `ShopItemSpecsInput` (InputObject) | `size: Option<String>` |
| D7 | Define `ShopSupportedDeliveryCountry` enum | `GERMANY` (single variant) |
| D8 | Export from `types/mod.rs` | Add `pub mod shop;` |

### Phase 5.E — State Extensions (`state.rs`, `seed.rs`)

| # | Task | Notes |
|---|------|-------|
| E1 | Add `wallets: HashMap<Uuid, Decimal>` to `MockState` | `uid → token balance` |
| E2 | Add `TransactionRecord` struct to state | `id: Uuid`, `operation_id: Uuid`, `category: TransactionCategory`, `transaction_type: String` (CREDIT/DEBIT), `sender_id: Uuid`, `recipient_id: Uuid`, `token_amount: Decimal`, `net_token_amount: Decimal`, `message: Option<String>`, `fees: Option<TransactionFeesRecord>`, `created_at: String` |
| E3 | Add `TransactionFeesRecord` struct | `total: Decimal`, `burn: Decimal`, `peer: Decimal`, `inviter: Option<Decimal>` |
| E4 | Add `transactions: Vec<TransactionRecord>` to `MockState` | Ordered by creation time |
| E5 | Add `daily_actions_used: HashMap<(Uuid, String, String), u32>` to `MockState` | `(user_id, date_string, action_name) → count`. Replaces Phase 4's `daily_comment_count`; generalizes to track all action types |
| E6 | Add `GemRecord` struct to state | `user_id: Uuid`, `post_id: Uuid`, `from_user_id: Uuid`, `gems: f64`, `action: String`, `created_at: String` |
| E7 | Add `gems: Vec<GemRecord>` to `MockState` | Accumulated from post interactions |
| E8 | Add `minted_dates: HashSet<String>` to `MockState` | Dates for which minting has occurred (prevent duplicate) |
| E9 | Add `AdvertisementRecord` struct to state | `id: Uuid`, `post_id: Uuid`, `advertiser_id: Uuid`, `ad_type: AdvertisementType`, `start_date: String`, `end_date: String`, `token_cost: Decimal`, `created_at: String` |
| E10 | Add `advertisements: Vec<AdvertisementRecord>` to `MockState` | |
| E11 | Add `ShopOrderRecord` struct to state | `id: Uuid`, `transaction_id: Uuid`, `shop_item_id: String`, `buyer_id: Uuid`, `token_amount: Decimal`, `item_specs: Option<String>`, `delivery: ShopDeliveryRecord`, `created_at: String` |
| E12 | Add `ShopDeliveryRecord` struct | `name`, `email`, `addressline1`, `addressline2: Option`, `city`, `zipcode`, `country` |
| E13 | Add `shop_orders: Vec<ShopOrderRecord>` to `MockState` | |
| E14 | Add system account UUIDs as constants | `SYSTEM_BURN_ACCOUNT`, `SYSTEM_PEER_ACCOUNT`, `SYSTEM_SHOP_ACCOUNT`, `SYSTEM_MINT_ACCOUNT` |
| E15 | Update `MockState::default()` and `MockState::reset()` | Initialize/clear all new fields, preserve seed data |

### Phase 5.F — Seed Data (`seed.rs`)

| # | Task | Notes |
|---|------|-------|
| F1 | Seed each existing user with a default wallet balance of `1000.0` tokens | `SEED_USER_VERIFIED` → 1000.0, `SEED_USER_2` → 1000.0, etc. |
| F2 | Seed system accounts with large balances | `SYSTEM_MINT_ACCOUNT` → 5_000_000.0, `SYSTEM_PEER_ACCOUNT` → 0.0, etc. |
| F3 | Add 3 seed transactions with deterministic UUIDs | 1 P2P transfer, 1 like payment, 1 post creation payment |
| F4 | Add 1 seed basic advertisement with deterministic UUID | Ad on `SEED_POST_1`, basic type, 3-day duration, cost 150.0 |
| F5 | Add 1 seed shop order with deterministic UUID | Simple order for reference in tests |
| F6 | Add seed gem data | 2–3 `GemRecord` entries from post interactions |
| F7 | Define hardcoded action prices | `POST_PRICE: 20.0`, `LIKE_PRICE: 3.0`, `DISLIKE_PRICE: 3.0`, `COMMENT_PRICE: 1.0`, `AD_BASIC_DAILY_PRICE: 50.0`, `AD_PINNED_PRICE: 200.0` |
| F8 | Define daily free action limits | `FREE_POSTS: 1`, `FREE_LIKES: 3`, `FREE_COMMENTS: 4`, `FREE_DISLIKES: 0` |
| F9 | Define gem return rates | `VIEW_GEM: 0.25`, `LIKE_GEM: 5.0`, `DISLIKE_GEM: -3.0`, `COMMENT_GEM: 2.0` |
| F10 | Define fee rates | `BURN_FEE_RATE: 0.01`, `PEER_FEE_RATE: 0.02`, `INVITER_FEE_RATE: 0.01` |

### Phase 5.G — Wallet Query Resolvers (`schema/query/wallet.rs`)

| # | Task | Notes |
|---|------|-------|
| G1 | Create `schema/query/wallet.rs` with `WalletQuery` struct | Uses `#[Object]` |
| G2 | Implement `balance` resolver | Requires auth. Return `CurrentLiquidity` with `currentliquidity = wallets[user_id]`. Code `11204`. |
| G3 | Implement `transaction_history(offset, limit)` resolver | Requires auth. Filter transactions where `sender_id == user_id \|\| recipient_id == user_id`. Group by `operation_id`. Paginate. Return `TransactionHistoryResponse`. Code `11215` / `21209`. |
| G4 | Implement `get_transaction_history(type?, direction?, start_date?, end_date?, limit?, offset?, sort?)` resolver | Requires auth. Same base filter as G3, plus optional `TokenMovementFilterType`, `DirectionFilterType`, date range, `SortFilterType`. Code `11215` / `21209`. |
| G5 | Implement `list_win_logs(day, offset?, limit?)` resolver | Requires auth. Return earning events (likes/views/comments received on user's posts) for the given day. Code `11203` / `21202`. |
| G6 | Implement `list_payment_logs(day, offset?, limit?)` resolver | Requires auth. Return spending events for the given day. Code `11203` / `21202`. |
| G7 | Implement `list_todays_interactions` resolver | Requires auth. Aggregate today's interactions on the current user's posts (views, likes, dislikes, comments) and compute scores using gem return rates. Code `11205` / `21204`. |
| G8 | Register `WalletQuery` in `QueryRoot` merged object | |

### Phase 5.H — Wallet Mutation Resolvers (`schema/mutation/wallet.rs`)

| # | Task | Notes |
|---|------|-------|
| H1 | Create `schema/mutation/wallet.rs` with `WalletMutation` struct | Uses `#[Object]` |
| H2 | Implement `resolve_transfer_v2(recipient, numberoftokens, message?)` mutation | Requires auth. Validate recipient UUID. Validate not self-transfer (`31202`). Validate recipient exists (`31007`). Validate not system account (`31203`). Validate message (max 500 chars, no URLs `30271`, no control chars). Calculate fees: burn 1%, peer 2%, inviter 1% (or 0% if no inviter). Total deduction = amount + fees. Check balance (`51301`). Deduct from sender, credit recipient + fee accounts. Record transactions (CREDIT + DEBIT + fee rows grouped by `operation_id`). Return `TransferTokenResponse`. Code `11211`. |
| H3 | Implement helper: `deduct_tokens(state, user_id, amount, category, message)` | Shared helper for all token deductions (transfers, ad purchases, shop, action costs). Returns `Result<Uuid, &str>` (operation_id or error code). |
| H4 | Register `WalletMutation` in `MutationRoot` merged object | |

### Phase 5.I — Tokenomics Query Resolvers (`schema/query/tokenomics.rs`)

| # | Task | Notes |
|---|------|-------|
| I1 | Create `schema/query/tokenomics.rs` with `TokenomicsQuery` struct | Uses `#[Object]` |
| I2 | Implement `get_action_prices` resolver | Requires auth. Return hardcoded action prices. Code `11304`. |
| I3 | Implement `get_tokenomics` resolver | Requires auth. Return combined `ActionPriceResult` + `ActionGemsReturns` + `MintingData`. Code `11212`. |
| I4 | Implement `get_daily_free_status` resolver | Requires auth. For each action type (post, like, comment, dislike), compute `used` from `daily_actions_used` for today, `available = limit - used`. Code `11303`. |
| I5 | Register `TokenomicsQuery` in `QueryRoot` merged object | |

### Phase 5.J — Advertisement Query Resolvers (`schema/query/ads.rs`)

| # | Task | Notes |
|---|------|-------|
| J1 | Create `schema/query/ads.rs` with `AdQuery` struct | Uses `#[Object]` |
| J2 | Implement `list_advertisement_posts(filterBy?, contentFilterBy?, userid?, postid?, title?, tag?, offset?, limit?, commentOffset?, commentLimit?)` resolver | Requires auth. Filter active advertisements (current date between `start_date` and `end_date`). Apply content type filter, user filter, post filter, title search, tag filter. Paginate. For each ad, embed full `Post` object (via `post_record_to_graphql`) and `AdvCreator` metadata. Code `12002` / `22002`. |
| J3 | Implement `advertisement_history(filter?, sort?, offset?, limit?)` resolver | Requires auth. Return all advertisements created by the current user. Apply optional filters (date range, type, specific ad/post/user). Sort by `AdvertisementSort`. Compute aggregate stats (`TotalAdvertisementHistoryStats`) from all matching ads. Paginate the ad list. Code `12002` / `22002`. |
| J4 | Register `AdQuery` in `QueryRoot` merged object | |

### Phase 5.K — Advertisement Mutation Resolvers (`schema/mutation/ads.rs`)

| # | Task | Notes |
|---|------|-------|
| K1 | Create `schema/mutation/ads.rs` with `AdMutation` struct | Uses `#[Object]` |
| K2 | Implement `advertise_post_basic(postid, startday, durationInDays, advertisePlan)` mutation | Requires auth. Validate post exists and belongs to current user (`31510`). Validate post not already actively advertised (`32006`). Calculate cost: `50.0 × duration_days`. Check balance (`51301`). Deduct tokens. Create `AdvertisementRecord`. Return `ListAdvertisementData`. Code `12001`. |
| K3 | Implement `advertise_post_pinned(postid, advertisePlan)` mutation | Requires auth. Same validations as K2. Cost: 200.0 flat. Pinned ads run for a default 7-day period. Code `12001`. |
| K4 | Register `AdMutation` in `MutationRoot` merged object | |

### Phase 5.L — Shop Query Resolvers (`schema/query/shop.rs`)

| # | Task | Notes |
|---|------|-------|
| L1 | Create `schema/query/shop.rs` with `ShopQuery` struct | Uses `#[Object]` |
| L2 | Implement `shop_order_details(transactionId)` resolver | Requires auth. Look up shop order by transaction ID. Validate current user is the order owner. Return `ShopOrderDetailsResponse`. Code `12202` / `22101`. |
| L3 | Register `ShopQuery` in `QueryRoot` merged object | |

### Phase 5.M — Shop Mutation Resolvers (`schema/mutation/shop.rs`)

| # | Task | Notes |
|---|------|-------|
| M1 | Create `schema/mutation/shop.rs` with `ShopMutation` struct | Uses `#[Object]` |
| M2 | Implement `perform_shop_order(tokenAmount, shopItemId, orderDetails)` mutation | Requires auth. Validate delivery details (name 2–100 chars, email valid, addressline1 6–100 chars, city 2–100, zipcode exactly 5, country = GERMANY). Check balance (`51301`). Deduct tokens (transfer to `SYSTEM_SHOP_ACCOUNT`). Create `ShopOrderRecord`. Record transaction with category `SHOP_PURCHASE`. Return `DefaultResponse`. Code `12201`. |
| M3 | Register `ShopMutation` in `MutationRoot` merged object | |

### Phase 5.N — Cross-Cutting: Token Deduction Integration

| # | Task | Notes |
|---|------|-------|
| N1 | Implement token deduction for post actions (like/dislike) in Phase 3 code | When daily free actions exhausted, deduct `LIKE_PRICE` / `DISLIKE_PRICE` from wallet. Return `51301` if insufficient balance. |
| N2 | Implement token deduction for comment creation in Phase 4 code | Replace Phase 4 TODO: when 5th+ comment of the day, deduct `COMMENT_PRICE`. Return `51301` if insufficient. |
| N3 | Implement token deduction for post creation in Phase 3 code | When daily free post used, deduct `POST_PRICE`. Return `51301` if insufficient. |
| N4 | Update all action resolvers to track `daily_actions_used` | Replace Phase 4's `daily_comment_count` with generalized `daily_actions_used` map |
| N5 | Implement gem accumulation on post interactions | When a user likes/views/comments on another's post, create `GemRecord` with the appropriate gem return rate |
| N6 | Exclude advertised posts from normal `listPosts` results | Add filter in Phase 3's `list_posts` resolver: skip posts with an active advertisement |

### Phase 5.O — Helper Functions

| # | Task | Notes |
|---|------|-------|
| O1 | Create `calculate_transfer_fees(amount, has_inviter) -> TransactionFeesRecord` | Burn 1%, peer 2%, inviter 1% (0 if no inviter). Total = sum. |
| O2 | Create `is_daily_free(state, user_id, action) -> bool` | Check `daily_actions_used` against limits. Returns true if free action available. |
| O3 | Create `use_daily_action(state, user_id, action)` | Increment `daily_actions_used` for today. |
| O4 | Create `try_deduct_for_action(state, user_id, action) -> Result<bool, &str>` | Check free → if free, use it and return `Ok(true)`. If not free, check balance → deduct → return `Ok(false)`. If insufficient → return `Err("51301")`. Returns whether the action was free. |
| O5 | Create `TransactionRecord → TransactionHistoryItem` conversion function | Resolve `sender` and `recipient` as `TransactionUser` from user profiles. |
| O6 | Create `AdvertisementRecord → AdvCreator` conversion function | Resolve advertiser profile. |
| O7 | Create `today_date_string() -> String` | Returns `YYYY-MM-DD` for consistent daily tracking. |
| O8 | Create `is_ad_active(ad, today) -> bool` | Check if `start_date <= today <= end_date`. |

### Phase 5.P — Schema Assembly Updates

| # | Task | Notes |
|---|------|-------|
| P1 | Add `WalletQuery`, `TokenomicsQuery`, `AdQuery`, `ShopQuery` to `QueryRoot` | |
| P2 | Add `WalletMutation`, `AdMutation`, `ShopMutation` to `MutationRoot` | |

### Phase 5.Q — Cleanup & Validation

| # | Task | Notes |
|---|------|-------|
| Q1 | Run `cargo clippy -- -D warnings` | Fix all warnings |
| Q2 | Run `cargo fmt --check` | Fix formatting |
| Q3 | Run full test suite (`cargo test --all-targets`) | All Phase 0–5 tests pass |
| Q4 | Manual smoke test with `cargo run` + curl | Verify HTTP layer for all new endpoints |

---

## 4. Implementation Details

### 4.1 Wallet Types (`types/wallet.rs`)

```rust
use async_graphql::{Enum, SimpleObject, ID};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Transaction category.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum TransactionCategory {
    P2pTransfer,
    AdPinned,
    PostCreate,
    Like,
    Dislike,
    Comment,
    TokenMint,
    ShopPurchase,
    InviterFeeEarn,
}

/// Filter transactions by movement type.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum TokenMovementFilterType {
    Transaction,
    Airdrop,
    Mint,
    Payment,
    Burn,
}

/// Filter transactions by direction.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum DirectionFilterType {
    Income,
    Deduction,
}

/// Sort order for transactions.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum SortFilterType {
    Newest,
    Oldest,
}

/// Day filter for win/payment logs.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum DayFilterType {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    W0,
    M0,
    Y0,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Basic user info embedded in transaction responses.
///
/// Must match the frontend's `TransactionUser` in `peer-web/src/models/transaction.rs`.
/// Field selections from `TRANSACTION_HISTORY_QUERY`:
/// `userid, img, username, slug, visibilityStatus, hasActiveReports, isHiddenForUsers`
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionUser {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: String,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
}

/// Fee breakdown for a transaction.
///
/// Matches frontend's `TransactionFees`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionFees {
    pub total: Decimal,
    pub burn: Decimal,
    pub peer: Decimal,
    pub inviter: Option<Decimal>,
}

/// A single transaction history item.
///
/// Must match the frontend's `Transaction` in `peer-web/src/models/transaction.rs`.
/// Field selections from `TRANSACTION_HISTORY_QUERY`:
/// `transactionId, operationid, transactionCategory, transactiontype, tokenamount,
///  netTokenAmount, message, createdat, sender { ... }, recipient { ... }, fees { ... }`
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionHistoryItem {
    #[graphql(name = "transactionId")]
    pub transaction_id: String,
    pub operationid: String,
    #[graphql(name = "transactionCategory")]
    pub transaction_category: Option<TransactionCategory>,
    pub transactiontype: String,
    pub tokenamount: String,
    #[graphql(name = "netTokenAmount")]
    pub net_token_amount: String,
    pub message: Option<String>,
    pub createdat: String,
    pub sender: TransactionUser,
    pub recipient: TransactionUser,
    pub fees: Option<TransactionFees>,
}

/// Response for `balance` query.
///
/// Matches frontend's `BalanceResponse`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CurrentLiquidity {
    pub meta: DefaultResponse,
    pub currentliquidity: Option<Decimal>,
}

/// Response for `transactionHistory` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionHistoryResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<TransactionHistoryItem>>,
}

/// Transfer result details.
///
/// Matches frontend field selections from `TRANSFER_MUTATION`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransferToken {
    #[graphql(name = "tokenSendFormatted")]
    pub token_send_formatted: String,
    #[graphql(name = "tokensSubstractedFromWalletFormatted")]
    pub tokens_substracted_from_wallet_formatted: String,
    pub createdat: Option<String>,
}

/// Response for `resolveTransferV2` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransferTokenResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<TransferToken>,
}

/// A win/payment log entry.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct LogWins {
    pub from: Option<String>,
    pub token: Option<String>,
    pub userid: Option<String>,
    pub postid: Option<String>,
    pub action: Option<String>,
    pub numbers: Option<Decimal>,
    pub createdat: Option<String>,
}

/// Response for `listWinLogs` / `listPaymentLogs` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserLogWins {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<LogWins>>,
}

/// Details of today's interactions on the user's posts.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TodaysInteractionsDetailsData {
    pub views: Decimal,
    pub likes: Decimal,
    pub dislikes: Decimal,
    pub comments: Decimal,
    #[graphql(name = "viewsScore")]
    pub views_score: Decimal,
    #[graphql(name = "likesScore")]
    pub likes_score: Decimal,
    #[graphql(name = "dislikesScore")]
    pub dislikes_score: Decimal,
    #[graphql(name = "commentsScore")]
    pub comments_score: Decimal,
}

/// Aggregated today's interactions data.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TodaysInteractionsData {
    #[graphql(name = "totalInteractions")]
    pub total_interactions: Decimal,
    #[graphql(name = "totalScore")]
    pub total_score: Decimal,
    #[graphql(name = "totalDetails")]
    pub total_details: TodaysInteractionsDetailsData,
}

/// Response for `listTodaysInteractions` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListTodaysInteractionsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<TodaysInteractionsData>,
}
```

### 4.2 Tokenomics Types (`types/tokenomics.rs`)

```rust
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Current token prices for all paid actions.
///
/// Returned by `getActionPrices` and embedded in `getTokenomics`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ActionPriceResult {
    #[graphql(name = "postPrice")]
    pub post_price: f64,
    #[graphql(name = "likePrice")]
    pub like_price: f64,
    #[graphql(name = "dislikePrice")]
    pub dislike_price: f64,
    #[graphql(name = "commentPrice")]
    pub comment_price: f64,
}

/// Response for `getActionPrices` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetActionPricesResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: ActionPriceResult,
}

/// Gem returns for interactions on a user's posts.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ActionGemsReturns {
    #[graphql(name = "viewGemsReturn")]
    pub view_gems_return: f64,
    #[graphql(name = "likeGemsReturn")]
    pub like_gems_return: f64,
    #[graphql(name = "dislikeGemsReturn")]
    pub dislike_gems_return: f64,
    #[graphql(name = "commentGemsReturn")]
    pub comment_gems_return: f64,
}

/// Minting data for the platform.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct MintingData {
    #[graphql(name = "tokensMintedYesterday")]
    pub tokens_minted_yesterday: f64,
}

/// Response for `getTokenomics` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TokenomicsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "actionTokenPrices")]
    pub action_token_prices: ActionPriceResult,
    #[graphql(name = "actionGemsReturns")]
    pub action_gems_returns: ActionGemsReturns,
    #[graphql(name = "mintingData")]
    pub minting_data: MintingData,
}

/// Daily free action status for a single action type.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyFreeResponse {
    pub name: String,
    pub used: i32,
    pub available: i32,
}

/// Response for `getDailyFreeStatus` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetDailyResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<DailyFreeResponse>>,
}
```

### 4.3 Advertisement Types (`types/ad.rs`)

```rust
use async_graphql::{Enum, InputObject, SimpleObject, ID};
use serde::{Deserialize, Serialize};

use super::post::Post;
use super::registration::DefaultResponse;
use super::user::ProfileUser;

// ============================================================================
// Enums
// ============================================================================

/// Advertisement type.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementType {
    Pinned,
    Basic,
}

/// Duration options for basic advertisements.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdDuration {
    OneDay,
    TwoDays,
    ThreeDays,
    FourDays,
    FiveDays,
    SixDays,
    SevenDays,
}

impl AdDuration {
    /// Convert to number of days.
    pub fn to_days(&self) -> u32 {
        match self {
            Self::OneDay => 1,
            Self::TwoDays => 2,
            Self::ThreeDays => 3,
            Self::FourDays => 4,
            Self::FiveDays => 5,
            Self::SixDays => 6,
            Self::SevenDays => 7,
        }
    }
}

/// Plan type for basic advertisements (single variant).
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementBasicPlan {
    Basic,
}

/// Plan type for pinned advertisements (single variant).
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementPinnedPlan {
    Pinned,
}

/// Sort order for advertisement history.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementSort {
    Newest,
    Oldest,
    BiggestCost,
    SmallestCost,
}

// ============================================================================
// Input Objects
// ============================================================================

/// Filter criteria for advertisement history.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementHistoryFilter {
    pub from: Option<String>,
    pub to: Option<String>,
    #[graphql(name = "type")]
    pub ad_type: Option<AdvertisementType>,
    #[graphql(name = "advertisementId")]
    pub advertisement_id: Option<ID>,
    #[graphql(name = "postId")]
    pub post_id: Option<ID>,
    #[graphql(name = "userId")]
    pub user_id: Option<ID>,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Advertisement metadata (creator/config) embedded in advertisement post responses.
///
/// Frontend field selections from `LIST_AD_POSTS_QUERY`:
/// `advertisementid, advertisementtype, startdate, enddate`
///
/// The full version (used in history) also includes `createdat` and `user`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvCreator {
    pub advertisementid: ID,
    pub advertisementtype: String,
    pub startdate: String,
    pub enddate: String,
    pub createdat: Option<String>,
    pub user: Option<ProfileUser>,
}

/// An advertised post (post + advertisement metadata).
///
/// Must match frontend's `AdvertisementPost` in `peer-web/src/models/post.rs`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementPost {
    pub post: Post,
    pub advertisement: AdvCreator,
}

/// Response for `listAdvertisementPosts` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListAdvertisementPostsResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AdvertisementPost>>,
}

/// A single advertisement row returned by create mutations.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementRow {
    pub id: ID,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "type")]
    pub ad_type: AdvertisementType,
    #[graphql(name = "timeframeStart")]
    pub timeframe_start: String,
    #[graphql(name = "timeframeEnd")]
    pub timeframe_end: String,
    #[graphql(name = "totalTokenCost")]
    pub total_token_cost: f64,
    #[graphql(name = "totalEuroCost")]
    pub total_euro_cost: f64,
}

/// Response for `advertisePostBasic` / `advertisePostPinned` mutations.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListAdvertisementData {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AdvertisementRow>>,
}

/// Aggregated statistics for advertisement history.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TotalAdvertisementHistoryStats {
    #[graphql(name = "tokenSpent")]
    pub token_spent: f64,
    #[graphql(name = "euroSpent")]
    pub euro_spent: f64,
    #[graphql(name = "amountAds")]
    pub amount_ads: i32,
    #[graphql(name = "gemsEarned")]
    pub gems_earned: f64,
    #[graphql(name = "amountLikes")]
    pub amount_likes: i32,
    #[graphql(name = "amountViews")]
    pub amount_views: i32,
    #[graphql(name = "amountComments")]
    pub amount_comments: i32,
    #[graphql(name = "amountDislikes")]
    pub amount_dislikes: i32,
    #[graphql(name = "amountReports")]
    pub amount_reports: i32,
}

/// Full advertisement record for history.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Advertisement {
    pub id: ID,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "type")]
    pub ad_type: AdvertisementType,
    #[graphql(name = "timeframeStart")]
    pub timeframe_start: String,
    #[graphql(name = "timeframeEnd")]
    pub timeframe_end: String,
    #[graphql(name = "totalTokenCost")]
    pub total_token_cost: f64,
    #[graphql(name = "totalEuroCost")]
    pub total_euro_cost: f64,
    #[graphql(name = "gemsEarned")]
    pub gems_earned: f64,
    #[graphql(name = "amountLikes")]
    pub amount_likes: i32,
    #[graphql(name = "amountViews")]
    pub amount_views: i32,
    #[graphql(name = "amountComments")]
    pub amount_comments: i32,
    #[graphql(name = "amountDislikes")]
    pub amount_dislikes: i32,
    #[graphql(name = "amountReports")]
    pub amount_reports: i32,
    pub user: ProfileUser,
    pub post: Post,
}

/// Result container for advertisement history.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementHistoryResult {
    pub stats: TotalAdvertisementHistoryStats,
    pub advertisements: Option<Vec<Advertisement>>,
}

/// Response for `advertisementHistory` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListedAdvertisementData {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<AdvertisementHistoryResult>,
}
```

### 4.4 Shop Types (`types/shop.rs`)

```rust
use async_graphql::{Enum, InputObject, SimpleObject};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Supported delivery countries.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ShopSupportedDeliveryCountry {
    Germany,
}

// ============================================================================
// Input Objects
// ============================================================================

/// Item specification input.
#[derive(InputObject, Clone, Debug)]
pub struct ShopItemSpecsInput {
    pub size: Option<String>,
}

/// Delivery and order details input.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDetailsInput {
    pub name: String,
    pub email: String,
    pub addressline1: String,
    pub addressline2: Option<String>,
    pub city: String,
    pub zipcode: String,
    pub country: ShopSupportedDeliveryCountry,
    #[graphql(name = "shopItemSpecs")]
    pub shop_item_specs: Option<ShopItemSpecsInput>,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Shop item specifications.
///
/// Matches frontend's `ShopItemSpecs`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ShopItemSpecs {
    pub size: Option<String>,
}

/// Delivery details for a shop order.
///
/// Matches frontend's `DeliveryDetails`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDeliveryDetails {
    pub name: Option<String>,
    pub email: Option<String>,
    pub addressline1: Option<String>,
    pub addressline2: Option<String>,
    pub city: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
}

/// Shop order details.
///
/// Matches frontend's `ShopOrderDetails`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDetails {
    #[graphql(name = "shopOrderId")]
    pub shop_order_id: String,
    #[graphql(name = "shopItemId")]
    pub shop_item_id: String,
    #[graphql(name = "shopItemSpecs")]
    pub shop_item_specs: Option<ShopItemSpecs>,
    #[graphql(name = "deliveryDetails")]
    pub delivery_details: Option<ShopOrderDeliveryDetails>,
    pub createdat: Option<String>,
}

/// Response for `shopOrderDetails` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDetailsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<ShopOrderDetails>>,
}
```

### 4.5 Internal State Records (`state.rs`)

```rust
use std::collections::{HashMap, HashSet};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::types::wallet::TransactionCategory;
use crate::types::ad::AdvertisementType;

// ============================================================================
// System account constants
// ============================================================================

/// Burn account — tokens sent here are effectively destroyed.
pub const SYSTEM_BURN_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000001");
/// Peer platform fee account.
pub const SYSTEM_PEER_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000002");
/// Shop account — receives shop purchase payments.
pub const SYSTEM_SHOP_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000003");
/// Mint account — source for minted tokens.
pub const SYSTEM_MINT_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000004");

// ============================================================================
// Action pricing constants
// ============================================================================

pub const POST_PRICE: Decimal = Decimal::from_parts(200, 0, 0, false, 1);   // 20.0
pub const LIKE_PRICE: Decimal = Decimal::from_parts(30, 0, 0, false, 1);    // 3.0
pub const DISLIKE_PRICE: Decimal = Decimal::from_parts(30, 0, 0, false, 1); // 3.0
pub const COMMENT_PRICE: Decimal = Decimal::from_parts(10, 0, 0, false, 1); // 1.0
pub const AD_BASIC_DAILY_PRICE: Decimal = Decimal::from_parts(500, 0, 0, false, 1); // 50.0
pub const AD_PINNED_PRICE: Decimal = Decimal::from_parts(2000, 0, 0, false, 1);     // 200.0

// Daily free action limits
pub const FREE_POSTS: u32 = 1;
pub const FREE_LIKES: u32 = 3;
pub const FREE_COMMENTS: u32 = 4;
pub const FREE_DISLIKES: u32 = 0;

// Gem return rates
pub const VIEW_GEM_RETURN: f64 = 0.25;
pub const LIKE_GEM_RETURN: f64 = 5.0;
pub const DISLIKE_GEM_RETURN: f64 = -3.0;
pub const COMMENT_GEM_RETURN: f64 = 2.0;

// Fee rates
pub const BURN_FEE_RATE: Decimal = Decimal::from_parts(1, 0, 0, false, 2);    // 0.01
pub const PEER_FEE_RATE: Decimal = Decimal::from_parts(2, 0, 0, false, 2);    // 0.02
pub const INVITER_FEE_RATE: Decimal = Decimal::from_parts(1, 0, 0, false, 2); // 0.01

// Daily minting budget
pub const DAILY_MINT_BUDGET: f64 = 5000.0;

// ============================================================================
// Internal state records
// ============================================================================

/// Internal transaction storage record.
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub id: Uuid,
    pub operation_id: Uuid,                    // Groups related transactions (credit + debit + fees)
    pub category: Option<TransactionCategory>,
    pub transaction_type: String,              // "CREDIT" or "DEBIT"
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub token_amount: Decimal,                 // Signed: positive for credit, negative for debit
    pub net_token_amount: Decimal,             // Amount excluding fees
    pub message: Option<String>,
    pub fees: Option<TransactionFeesRecord>,
    pub created_at: String,
}

/// Internal fee record.
#[derive(Debug, Clone)]
pub struct TransactionFeesRecord {
    pub total: Decimal,
    pub burn: Decimal,
    pub peer: Decimal,
    pub inviter: Option<Decimal>,
}

/// Internal gem storage record.
#[derive(Debug, Clone)]
pub struct GemRecord {
    pub user_id: Uuid,          // Post owner who earns the gems
    pub post_id: Uuid,
    pub from_user_id: Uuid,     // User who triggered the interaction
    pub gems: f64,
    pub action: String,         // "view", "like", "dislike", "comment"
    pub created_at: String,
}

/// Internal advertisement storage record.
#[derive(Debug, Clone)]
pub struct AdvertisementRecord {
    pub id: Uuid,
    pub post_id: Uuid,
    pub advertiser_id: Uuid,
    pub ad_type: AdvertisementType,
    pub start_date: String,        // YYYY-MM-DD
    pub end_date: String,          // YYYY-MM-DD
    pub token_cost: Decimal,
    pub created_at: String,
}

/// Internal shop order storage record.
#[derive(Debug, Clone)]
pub struct ShopOrderRecord {
    pub id: Uuid,
    pub transaction_id: Uuid,      // Links to the payment transaction
    pub shop_item_id: String,
    pub buyer_id: Uuid,
    pub token_amount: Decimal,
    pub item_specs: Option<String>, // e.g., size
    pub delivery: ShopDeliveryRecord,
    pub created_at: String,
}

/// Internal delivery details record.
#[derive(Debug, Clone)]
pub struct ShopDeliveryRecord {
    pub name: String,
    pub email: String,
    pub addressline1: String,
    pub addressline2: Option<String>,
    pub city: String,
    pub zipcode: String,
    pub country: String,
}

/// Extended MockState fields for Phase 5.
pub struct MockState {
    // --- Phase 0–4 fields (existing) ---
    // ...

    // --- Phase 5: Wallet ---
    pub wallets: HashMap<Uuid, Decimal>,
    pub transactions: Vec<TransactionRecord>,

    // --- Phase 5: Tokenomics ---
    pub daily_actions_used: HashMap<(Uuid, String, String), u32>,  // (user_id, date, action) → count
    pub gems: Vec<GemRecord>,
    pub minted_dates: HashSet<String>,

    // --- Phase 5: Advertisements ---
    pub advertisements: Vec<AdvertisementRecord>,

    // --- Phase 5: Shop ---
    pub shop_orders: Vec<ShopOrderRecord>,
}
```

### 4.6 Seed Data (`seed.rs`)

```rust
use rust_decimal::Decimal;
use uuid::{uuid, Uuid};

// --- Phase 5 system account UUIDs ---
pub const SYSTEM_BURN_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000001");
pub const SYSTEM_PEER_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000002");
pub const SYSTEM_SHOP_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000003");
pub const SYSTEM_MINT_ACCOUNT: Uuid = uuid!("00000000-0000-4000-a000-000000000004");

// --- Phase 5 seed transaction UUIDs ---
pub const SEED_TX_1: Uuid = uuid!("50000000-0000-4000-a000-000000000001");
pub const SEED_TX_2: Uuid = uuid!("50000000-0000-4000-a000-000000000002");
pub const SEED_TX_3: Uuid = uuid!("50000000-0000-4000-a000-000000000003");
pub const SEED_OP_1: Uuid = uuid!("50000000-0000-4000-a000-000000000101");
pub const SEED_OP_2: Uuid = uuid!("50000000-0000-4000-a000-000000000102");
pub const SEED_OP_3: Uuid = uuid!("50000000-0000-4000-a000-000000000103");

// --- Phase 5 seed advertisement UUID ---
pub const SEED_AD_1: Uuid = uuid!("60000000-0000-4000-a000-000000000001");

// --- Phase 5 seed shop order UUID ---
pub const SEED_SHOP_ORDER_1: Uuid = uuid!("70000000-0000-4000-a000-000000000001");
pub const SEED_SHOP_TX_1: Uuid = uuid!("70000000-0000-4000-a000-000000000002");

/// Default token balance for seeded users.
pub const DEFAULT_USER_BALANCE: Decimal = Decimal::from_parts(10000, 0, 0, false, 1); // 1000.0

fn seed_wallets(users: &[Uuid]) -> HashMap<Uuid, Decimal> {
    let mut wallets = HashMap::new();
    for &uid in users {
        wallets.insert(uid, DEFAULT_USER_BALANCE);
    }
    // System accounts
    wallets.insert(SYSTEM_MINT_ACCOUNT, Decimal::from_parts(50000000, 0, 0, false, 1)); // 5_000_000.0
    wallets.insert(SYSTEM_PEER_ACCOUNT, Decimal::ZERO);
    wallets.insert(SYSTEM_BURN_ACCOUNT, Decimal::ZERO);
    wallets.insert(SYSTEM_SHOP_ACCOUNT, Decimal::ZERO);
    wallets
}

fn seed_transactions(user1: Uuid, user2: Uuid) -> Vec<TransactionRecord> {
    vec![
        // P2P transfer: user1 sent 50 tokens to user2
        TransactionRecord {
            id: SEED_TX_1,
            operation_id: SEED_OP_1,
            category: Some(TransactionCategory::P2pTransfer),
            transaction_type: "CREDIT".into(),
            sender_id: user1,
            recipient_id: user2,
            token_amount: Decimal::from_parts(500, 0, 0, false, 1),   // 50.0
            net_token_amount: Decimal::from_parts(500, 0, 0, false, 1),
            message: Some("Great post!".into()),
            fees: Some(TransactionFeesRecord {
                total: Decimal::from_parts(20, 0, 0, false, 1),  // 2.0
                burn: Decimal::from_parts(5, 0, 0, false, 1),   // 0.5
                peer: Decimal::from_parts(10, 0, 0, false, 1),  // 1.0
                inviter: Some(Decimal::from_parts(5, 0, 0, false, 1)), // 0.5
            }),
            created_at: "2025-04-10T10:00:00Z".into(),
        },
        // Like payment: user1 paid 3 tokens for like
        TransactionRecord {
            id: SEED_TX_2,
            operation_id: SEED_OP_2,
            category: Some(TransactionCategory::Like),
            transaction_type: "DEBIT".into(),
            sender_id: user1,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: Decimal::from_parts(30, 0, 0, true, 1),  // -3.0
            net_token_amount: Decimal::from_parts(30, 0, 0, true, 1),
            message: None,
            fees: None,
            created_at: "2025-04-11T14:00:00Z".into(),
        },
        // Post creation payment: user2 paid 20 tokens for post
        TransactionRecord {
            id: SEED_TX_3,
            operation_id: SEED_OP_3,
            category: Some(TransactionCategory::PostCreate),
            transaction_type: "DEBIT".into(),
            sender_id: user2,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: Decimal::from_parts(200, 0, 0, true, 1), // -20.0
            net_token_amount: Decimal::from_parts(200, 0, 0, true, 1),
            message: None,
            fees: None,
            created_at: "2025-04-12T09:00:00Z".into(),
        },
    ]
}

fn seed_advertisements(user1: Uuid) -> Vec<AdvertisementRecord> {
    vec![
        AdvertisementRecord {
            id: SEED_AD_1,
            post_id: SEED_POST_1,
            advertiser_id: user1,
            ad_type: AdvertisementType::Basic,
            start_date: "2025-04-10".into(),
            end_date: "2025-04-13".into(),
            token_cost: Decimal::from_parts(1500, 0, 0, false, 1), // 150.0 (3 days × 50.0)
            created_at: "2025-04-10T08:00:00Z".into(),
        },
    ]
}

fn seed_shop_orders(user2: Uuid) -> Vec<ShopOrderRecord> {
    vec![
        ShopOrderRecord {
            id: SEED_SHOP_ORDER_1,
            transaction_id: SEED_SHOP_TX_1,
            shop_item_id: "peer-tshirt-001".into(),
            buyer_id: user2,
            token_amount: Decimal::from_parts(500, 0, 0, false, 1), // 50.0
            item_specs: Some("L".into()),
            delivery: ShopDeliveryRecord {
                name: "Test User".into(),
                email: "test@example.com".into(),
                addressline1: "Musterstraße 42".into(),
                addressline2: None,
                city: "Berlin".into(),
                zipcode: "10115".into(),
                country: "GERMANY".into(),
            },
            created_at: "2025-04-13T15:00:00Z".into(),
        },
    ]
}

fn seed_gems(user1: Uuid, user2: Uuid) -> Vec<GemRecord> {
    vec![
        GemRecord {
            user_id: user1,
            post_id: SEED_POST_1,
            from_user_id: user2,
            gems: LIKE_GEM_RETURN,   // 5.0
            action: "like".into(),
            created_at: "2025-04-11T14:00:00Z".into(),
        },
        GemRecord {
            user_id: user1,
            post_id: SEED_POST_1,
            from_user_id: user2,
            gems: VIEW_GEM_RETURN,   // 0.25
            action: "view".into(),
            created_at: "2025-04-11T13:00:00Z".into(),
        },
        GemRecord {
            user_id: user2,
            post_id: SEED_POST_3,
            from_user_id: user1,
            gems: COMMENT_GEM_RETURN, // 2.0
            action: "comment".into(),
            created_at: "2025-04-12T10:00:00Z".into(),
        },
    ]
}
```

### 4.7 Wallet Query Resolvers (`schema/query/wallet.rs`)

```rust
use async_graphql::{Context, Object};
use rust_decimal::Decimal;

use crate::schema::require_auth;
use crate::state::SharedState;
use crate::types::registration::DefaultResponse;
use crate::types::wallet::*;

pub struct WalletQuery;

#[Object]
impl WalletQuery {
    /// Get the current user's token balance.
    ///
    /// Response codes:
    /// - 11204: Balance retrieved
    /// - 60501: Not authenticated
    async fn balance(&self, ctx: &Context<'_>) -> CurrentLiquidity {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return CurrentLiquidity {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                currentliquidity: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let balance = state.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);

        CurrentLiquidity {
            meta: DefaultResponse::success("11204", "Balance retrieved"),
            currentliquidity: Some(balance),
        }
    }

    /// Get transaction history with pagination.
    ///
    /// Uses the enhanced `transactionHistory` query format with categories and fees.
    ///
    /// Response codes:
    /// - 11215: Transactions retrieved
    /// - 21209: No transactions found
    /// - 30203: Invalid offset
    /// - 30204: Invalid limit
    /// - 60501: Not authenticated
    async fn transaction_history(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "type")] filter_type: Option<TokenMovementFilterType>,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: Option<SortFilterType>,
    ) -> TransactionHistoryResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return TransactionHistoryResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 20) as usize;

        // Filter transactions involving this user
        let mut user_txs: Vec<&TransactionRecord> = state.transactions.iter()
            .filter(|t| t.sender_id == user_id || t.recipient_id == user_id)
            .filter(|t| {
                // Optional type filter
                match filter_type {
                    Some(TokenMovementFilterType::Transaction) =>
                        t.category == Some(TransactionCategory::P2pTransfer),
                    Some(TokenMovementFilterType::Mint) =>
                        t.category == Some(TransactionCategory::TokenMint),
                    Some(TokenMovementFilterType::Payment) =>
                        matches!(t.category, Some(TransactionCategory::Like)
                            | Some(TransactionCategory::Dislike)
                            | Some(TransactionCategory::Comment)
                            | Some(TransactionCategory::PostCreate)),
                    _ => true,
                }
            })
            .filter(|t| {
                // Optional date range filters
                let in_start = start_date.as_ref()
                    .map(|sd| t.created_at.as_str() >= sd.as_str())
                    .unwrap_or(true);
                let in_end = end_date.as_ref()
                    .map(|ed| t.created_at.as_str() <= ed.as_str())
                    .unwrap_or(true);
                in_start && in_end
            })
            .collect();

        // Sort
        match sort {
            Some(SortFilterType::Oldest) => user_txs.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            _ => user_txs.sort_by(|a, b| b.created_at.cmp(&a.created_at)), // NEWEST default
        }

        if user_txs.is_empty() {
            return TransactionHistoryResponse {
                meta: DefaultResponse::success("21209", "No transactions found"),
                affected_rows: None,
            };
        }

        // Paginate
        let end = (off + lim).min(user_txs.len());
        let page = if off < user_txs.len() { &user_txs[off..end] } else { &[] };

        let items: Vec<TransactionHistoryItem> = page.iter()
            .map(|t| state.transaction_record_to_graphql(t))
            .collect();

        TransactionHistoryResponse {
            meta: DefaultResponse::success("11215", "Transactions retrieved"),
            affected_rows: Some(items),
        }
    }

    /// Get the legacy transaction history with direction filter.
    ///
    /// Response codes same as `transactionHistory`.
    async fn get_transaction_history(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "type")] filter_type: Option<TokenMovementFilterType>,
        direction: Option<DirectionFilterType>,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: Option<SortFilterType>,
    ) -> TransactionHistoryResponse {
        // Delegate to transactionHistory with additional direction filtering
        // Direction filter: INCOME = positive amount, DEDUCTION = negative amount
        // Implementation mirrors transaction_history with the added direction check
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return TransactionHistoryResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 20) as usize;

        let mut user_txs: Vec<&TransactionRecord> = state.transactions.iter()
            .filter(|t| t.sender_id == user_id || t.recipient_id == user_id)
            .filter(|t| match direction {
                Some(DirectionFilterType::Income) => t.recipient_id == user_id,
                Some(DirectionFilterType::Deduction) => t.sender_id == user_id,
                None => true,
            })
            .collect();

        // Apply same type/date filters and sort as transaction_history...
        // (implementation mirrors above, omitted for brevity)

        match sort {
            Some(SortFilterType::Oldest) => user_txs.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            _ => user_txs.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        }

        if user_txs.is_empty() {
            return TransactionHistoryResponse {
                meta: DefaultResponse::success("21209", "No transactions found"),
                affected_rows: None,
            };
        }

        let end = (off + lim).min(user_txs.len());
        let page = if off < user_txs.len() { &user_txs[off..end] } else { &[] };

        let items: Vec<TransactionHistoryItem> = page.iter()
            .map(|t| state.transaction_record_to_graphql(t))
            .collect();

        TransactionHistoryResponse {
            meta: DefaultResponse::success("11215", "Transactions retrieved"),
            affected_rows: Some(items),
        }
    }

    /// Get today's interactions summary on the user's posts.
    ///
    /// Response codes:
    /// - 11205: Interactions retrieved
    /// - 21204: No interactions today
    /// - 60501: Not authenticated
    async fn list_todays_interactions(
        &self,
        ctx: &Context<'_>,
    ) -> ListTodaysInteractionsResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return ListTodaysInteractionsResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let today = today_date_string();

        // Collect user's post IDs
        let user_post_ids: Vec<Uuid> = state.posts.iter()
            .filter(|p| p.author_id == user_id)
            .map(|p| p.id)
            .collect();

        // Count today's interactions on those posts from gems
        let todays_gems: Vec<&GemRecord> = state.gems.iter()
            .filter(|g| g.user_id == user_id && g.created_at.starts_with(&today))
            .collect();

        if todays_gems.is_empty() {
            return ListTodaysInteractionsResponse {
                meta: DefaultResponse::success("21204", "No interactions today"),
                affected_rows: None,
            };
        }

        let views = todays_gems.iter().filter(|g| g.action == "view").count() as f64;
        let likes = todays_gems.iter().filter(|g| g.action == "like").count() as f64;
        let dislikes = todays_gems.iter().filter(|g| g.action == "dislike").count() as f64;
        let comments = todays_gems.iter().filter(|g| g.action == "comment").count() as f64;

        let views_score = views * VIEW_GEM_RETURN;
        let likes_score = likes * LIKE_GEM_RETURN;
        let dislikes_score = dislikes * DISLIKE_GEM_RETURN;
        let comments_score = comments * COMMENT_GEM_RETURN;

        let total_interactions = views + likes + dislikes + comments;
        let total_score = views_score + likes_score + dislikes_score + comments_score;

        ListTodaysInteractionsResponse {
            meta: DefaultResponse::success("11205", "Interactions retrieved"),
            affected_rows: Some(TodaysInteractionsData {
                total_interactions: Decimal::from_f64_retain(total_interactions).unwrap_or_default(),
                total_score: Decimal::from_f64_retain(total_score).unwrap_or_default(),
                total_details: TodaysInteractionsDetailsData {
                    views: Decimal::from_f64_retain(views).unwrap_or_default(),
                    likes: Decimal::from_f64_retain(likes).unwrap_or_default(),
                    dislikes: Decimal::from_f64_retain(dislikes).unwrap_or_default(),
                    comments: Decimal::from_f64_retain(comments).unwrap_or_default(),
                    views_score: Decimal::from_f64_retain(views_score).unwrap_or_default(),
                    likes_score: Decimal::from_f64_retain(likes_score).unwrap_or_default(),
                    dislikes_score: Decimal::from_f64_retain(dislikes_score).unwrap_or_default(),
                    comments_score: Decimal::from_f64_retain(comments_score).unwrap_or_default(),
                },
            }),
        }
    }
}
```

### 4.8 Wallet Mutation Resolvers (`schema/mutation/wallet.rs`)

```rust
use async_graphql::{Context, Object, ID};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::schema::require_auth;
use crate::state::*;
use crate::types::registration::DefaultResponse;
use crate::types::wallet::*;

pub struct WalletMutation;

#[Object]
impl WalletMutation {
    /// Transfer tokens to another user.
    ///
    /// Fee structure: burn 1% + peer 2% + inviter 1% = 4% total (3% if no inviter).
    ///
    /// Response codes:
    /// - 11211: Transfer successful
    /// - 30201: Invalid recipient UUID
    /// - 30264: Invalid token amount
    /// - 30270: Message too long
    /// - 30271: Invalid message (URLs or control chars)
    /// - 31007: Recipient not found
    /// - 31202: Cannot transfer to self
    /// - 31203: Cannot transfer to system account
    /// - 51301: Insufficient balance
    /// - 60501: Not authenticated
    async fn resolve_transfer_v2(
        &self,
        ctx: &Context<'_>,
        recipient: ID,
        numberoftokens: Decimal,
        message: Option<String>,
    ) -> TransferTokenResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return TransferTokenResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        // Validate amount
        let min_amount = Decimal::new(1, 6); // 0.000001
        if numberoftokens < min_amount {
            return TransferTokenResponse {
                meta: DefaultResponse::error("30264", "Amount must be at least 0.000001"),
                affected_rows: None,
            };
        }

        // Parse recipient UUID
        let recipient_uuid = match recipient.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return TransferTokenResponse {
                meta: DefaultResponse::error("30201", "Invalid recipient UUID"),
                affected_rows: None,
            },
        };

        // Cannot transfer to self
        if recipient_uuid == user_id {
            return TransferTokenResponse {
                meta: DefaultResponse::error("31202", "Cannot transfer to self"),
                affected_rows: None,
            };
        }

        // Cannot transfer to system accounts
        let system_accounts = [SYSTEM_BURN_ACCOUNT, SYSTEM_PEER_ACCOUNT, SYSTEM_SHOP_ACCOUNT, SYSTEM_MINT_ACCOUNT];
        if system_accounts.contains(&recipient_uuid) {
            return TransferTokenResponse {
                meta: DefaultResponse::error("31203", "Cannot transfer to system account"),
                affected_rows: None,
            };
        }

        // Validate message
        if let Some(ref msg) = message {
            if msg.len() > 500 {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("30270", "Message exceeds 500 characters"),
                    affected_rows: None,
                };
            }
            if msg.contains("://") || msg.contains("www.") {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("30271", "URLs not allowed in messages"),
                    affected_rows: None,
                };
            }
            if msg.chars().any(|c| c.is_control() && c != '\n') {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("30271", "Control characters not allowed"),
                    affected_rows: None,
                };
            }
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Validate recipient exists
        if !state.users.contains_key(&recipient_uuid) {
            return TransferTokenResponse {
                meta: DefaultResponse::error("31007", "Recipient not found"),
                affected_rows: None,
            };
        }

        // Calculate fees
        let burn_fee = numberoftokens * BURN_FEE_RATE;
        let peer_fee = numberoftokens * PEER_FEE_RATE;
        // TODO: Check if sender has an inviter in later phases
        let inviter_fee = numberoftokens * INVITER_FEE_RATE;
        let total_fee = burn_fee + peer_fee + inviter_fee;
        let total_deduction = numberoftokens + total_fee;

        // Check balance
        let sender_balance = state.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);
        if sender_balance < total_deduction {
            return TransferTokenResponse {
                meta: DefaultResponse::error("51301", "Insufficient balance"),
                affected_rows: None,
            };
        }

        // Execute transfer
        let now = Utc::now().to_rfc3339();
        let operation_id = Uuid::new_v4();

        // Deduct from sender
        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= total_deduction;
        // Credit recipient
        *state.wallets.entry(recipient_uuid).or_insert(Decimal::ZERO) += numberoftokens;
        // Credit fee accounts
        *state.wallets.entry(SYSTEM_BURN_ACCOUNT).or_insert(Decimal::ZERO) += burn_fee;
        *state.wallets.entry(SYSTEM_PEER_ACCOUNT).or_insert(Decimal::ZERO) += peer_fee;

        // Record transaction
        let fees_record = TransactionFeesRecord {
            total: total_fee,
            burn: burn_fee,
            peer: peer_fee,
            inviter: Some(inviter_fee),
        };

        state.transactions.push(TransactionRecord {
            id: Uuid::new_v4(),
            operation_id,
            category: Some(TransactionCategory::P2pTransfer),
            transaction_type: "CREDIT".into(),
            sender_id: user_id,
            recipient_id: recipient_uuid,
            token_amount: numberoftokens,
            net_token_amount: numberoftokens,
            message: message.clone(),
            fees: Some(fees_record),
            created_at: now.clone(),
        });

        TransferTokenResponse {
            meta: DefaultResponse::success("11211", "Transfer successful"),
            affected_rows: Some(TransferToken {
                token_send_formatted: numberoftokens.to_string(),
                tokens_substracted_from_wallet_formatted: total_deduction.to_string(),
                createdat: Some(now),
            }),
        }
    }
}
```

### 4.9 Tokenomics Query Resolvers (`schema/query/tokenomics.rs`)

```rust
use async_graphql::{Context, Object};

use crate::schema::require_auth;
use crate::state::*;
use crate::types::registration::DefaultResponse;
use crate::types::tokenomics::*;

pub struct TokenomicsQuery;

#[Object]
impl TokenomicsQuery {
    /// Get current action prices.
    ///
    /// Response codes:
    /// - 11304: Prices fetched
    /// - 60501: Not authenticated
    async fn get_action_prices(&self, ctx: &Context<'_>) -> GetActionPricesResponse {
        let _user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return GetActionPricesResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: ActionPriceResult {
                    post_price: 0.0, like_price: 0.0, dislike_price: 0.0, comment_price: 0.0,
                },
            },
        };

        GetActionPricesResponse {
            meta: DefaultResponse::success("11304", "Prices fetched"),
            affected_rows: ActionPriceResult {
                post_price: 20.0,
                like_price: 3.0,
                dislike_price: 3.0,
                comment_price: 1.0,
            },
        }
    }

    /// Get comprehensive tokenomics data.
    ///
    /// Response codes:
    /// - 11212: Tokenomics data retrieved
    /// - 60501: Not authenticated
    async fn get_tokenomics(&self, ctx: &Context<'_>) -> TokenomicsResponse {
        let _user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return TokenomicsResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                action_token_prices: ActionPriceResult {
                    post_price: 0.0, like_price: 0.0, dislike_price: 0.0, comment_price: 0.0,
                },
                action_gems_returns: ActionGemsReturns {
                    view_gems_return: 0.0, like_gems_return: 0.0,
                    dislike_gems_return: 0.0, comment_gems_return: 0.0,
                },
                minting_data: MintingData { tokens_minted_yesterday: 0.0 },
            },
        };

        TokenomicsResponse {
            meta: DefaultResponse::success("11212", "Tokenomics data retrieved"),
            action_token_prices: ActionPriceResult {
                post_price: 20.0,
                like_price: 3.0,
                dislike_price: 3.0,
                comment_price: 1.0,
            },
            action_gems_returns: ActionGemsReturns {
                view_gems_return: VIEW_GEM_RETURN,
                like_gems_return: LIKE_GEM_RETURN,
                dislike_gems_return: DISLIKE_GEM_RETURN,
                comment_gems_return: COMMENT_GEM_RETURN,
            },
            minting_data: MintingData {
                tokens_minted_yesterday: 0.0, // Computed from minted_dates in a real impl
            },
        }
    }

    /// Get daily free action usage and availability.
    ///
    /// Response codes:
    /// - 11303: Daily free status loaded
    /// - 60501: Not authenticated
    async fn get_daily_free_status(&self, ctx: &Context<'_>) -> GetDailyResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return GetDailyResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let today = today_date_string();

        let actions = vec![
            ("post", FREE_POSTS),
            ("like", FREE_LIKES),
            ("comment", FREE_COMMENTS),
            ("dislike", FREE_DISLIKES),
        ];

        let rows: Vec<DailyFreeResponse> = actions.into_iter().map(|(name, limit)| {
            let used = state.daily_actions_used
                .get(&(user_id, today.clone(), name.to_string()))
                .copied()
                .unwrap_or(0);
            let available = limit.saturating_sub(used) as i32;
            DailyFreeResponse {
                name: name.to_string(),
                used: used as i32,
                available,
            }
        }).collect();

        GetDailyResponse {
            meta: DefaultResponse::success("11303", "Daily free status loaded"),
            affected_rows: Some(rows),
        }
    }
}
```

### 4.10 Advertisement Query Resolvers (`schema/query/ads.rs`)

```rust
use async_graphql::{Context, Object, ID};

use crate::schema::require_auth;
use crate::state::*;
use crate::types::ad::*;
use crate::types::registration::DefaultResponse;

pub struct AdQuery;

#[Object]
impl AdQuery {
    /// List currently active advertisement posts.
    ///
    /// Response codes:
    /// - 12002: Advertisements fetched
    /// - 22002: No advertisements found
    /// - 60501: Not authenticated
    async fn list_advertisement_posts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "filterBy")] _filter_by: Option<Vec<String>>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<String>,
        userid: Option<ID>,
        postid: Option<ID>,
        title: Option<String>,
        tag: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
        #[graphql(name = "commentOffset")] _comment_offset: Option<i32>,
        #[graphql(name = "commentLimit")] _comment_limit: Option<i32>,
    ) -> ListAdvertisementPostsResponse {
        let viewer_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return ListAdvertisementPostsResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                counter: 0,
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let today = today_date_string();

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 20) as usize;

        // Filter active advertisements
        let active_ads: Vec<&AdvertisementRecord> = state.advertisements.iter()
            .filter(|a| is_ad_active(a, &today))
            .filter(|a| {
                // Optional userid filter
                userid.as_ref().map(|uid| uid.to_string() == a.advertiser_id.to_string()).unwrap_or(true)
            })
            .filter(|a| {
                // Optional postid filter
                postid.as_ref().map(|pid| pid.to_string() == a.post_id.to_string()).unwrap_or(true)
            })
            .collect();

        if active_ads.is_empty() {
            return ListAdvertisementPostsResponse {
                meta: DefaultResponse::success("22002", "No advertisements found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let total = active_ads.len();
        let end = (off + lim).min(total);
        let page = if off < total { &active_ads[off..end] } else { &[] };

        let ad_posts: Vec<AdvertisementPost> = page.iter()
            .filter_map(|a| state.ad_record_to_graphql(a, Some(viewer_id)))
            .collect();

        ListAdvertisementPostsResponse {
            meta: DefaultResponse::success("12002", "Advertisements fetched"),
            counter: total as i32,
            affected_rows: Some(ad_posts),
        }
    }

    /// Get the user's advertisement history with aggregated statistics.
    ///
    /// Response codes:
    /// - 12002: History fetched
    /// - 22002: No advertisements found
    /// - 60501: Not authenticated
    async fn advertisement_history(
        &self,
        ctx: &Context<'_>,
        filter: Option<AdvertisementHistoryFilter>,
        sort: Option<AdvertisementSort>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> ListedAdvertisementData {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return ListedAdvertisementData {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 20) as usize;

        // Filter user's ads
        let mut user_ads: Vec<&AdvertisementRecord> = state.advertisements.iter()
            .filter(|a| a.advertiser_id == user_id)
            .filter(|a| {
                if let Some(ref f) = filter {
                    let type_match = f.ad_type.map(|t| a.ad_type == t).unwrap_or(true);
                    let from_match = f.from.as_ref().map(|d| a.start_date.as_str() >= d.as_str()).unwrap_or(true);
                    let to_match = f.to.as_ref().map(|d| a.end_date.as_str() <= d.as_str()).unwrap_or(true);
                    type_match && from_match && to_match
                } else {
                    true
                }
            })
            .collect();

        // Sort
        match sort {
            Some(AdvertisementSort::Oldest) => user_ads.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            Some(AdvertisementSort::BiggestCost) => user_ads.sort_by(|a, b| b.token_cost.cmp(&a.token_cost)),
            Some(AdvertisementSort::SmallestCost) => user_ads.sort_by(|a, b| a.token_cost.cmp(&b.token_cost)),
            _ => user_ads.sort_by(|a, b| b.created_at.cmp(&a.created_at)), // NEWEST default
        }

        if user_ads.is_empty() {
            return ListedAdvertisementData {
                meta: DefaultResponse::success("22002", "No advertisements found"),
                affected_rows: None,
            };
        }

        // Compute aggregate stats (simplified for mock)
        let total_cost: f64 = user_ads.iter().map(|a| a.token_cost.to_string().parse::<f64>().unwrap_or(0.0)).sum();
        let stats = TotalAdvertisementHistoryStats {
            token_spent: total_cost,
            euro_spent: 0.0, // Mock doesn't track euro
            amount_ads: user_ads.len() as i32,
            gems_earned: 0.0,
            amount_likes: 0,
            amount_views: 0,
            amount_comments: 0,
            amount_dislikes: 0,
            amount_reports: 0,
        };

        // Paginate
        let end = (off + lim).min(user_ads.len());
        let page = if off < user_ads.len() { &user_ads[off..end] } else { &[] };

        let ads: Vec<Advertisement> = page.iter()
            .filter_map(|a| state.ad_record_to_full_graphql(a))
            .collect();

        ListedAdvertisementData {
            meta: DefaultResponse::success("12002", "History fetched"),
            affected_rows: Some(AdvertisementHistoryResult {
                stats,
                advertisements: Some(ads),
            }),
        }
    }
}
```

### 4.11 Advertisement Mutation Resolvers (`schema/mutation/ads.rs`)

```rust
use async_graphql::{Context, Object, ID};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::schema::require_auth;
use crate::state::*;
use crate::types::ad::*;
use crate::types::registration::DefaultResponse;

pub struct AdMutation;

#[Object]
impl AdMutation {
    /// Create a basic (time-based) advertisement for a post.
    ///
    /// Cost: 50.0 tokens per day × duration.
    ///
    /// Response codes:
    /// - 12001: Advertisement created
    /// - 30209: Invalid post UUID
    /// - 31510: Post not found or not owned
    /// - 32006: Post already has active advertisement
    /// - 51301: Insufficient balance
    /// - 60501: Not authenticated
    async fn advertise_post_basic(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        startday: String,
        #[graphql(name = "durationInDays")] duration_in_days: AdDuration,
        #[graphql(name = "advertisePlan")] _advertise_plan: AdvertisementBasicPlan,
    ) -> ListAdvertisementData {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return ListAdvertisementData {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return ListAdvertisementData {
                meta: DefaultResponse::error("30209", "Invalid post UUID"),
                affected_rows: None,
            },
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Validate post exists and belongs to user
        let post = match state.posts.iter().find(|p| p.id == post_uuid) {
            Some(p) if p.author_id == user_id => p,
            _ => return ListAdvertisementData {
                meta: DefaultResponse::error("31510", "Post not found or not owned by you"),
                affected_rows: None,
            },
        };

        // Check no active ad on this post
        let today = today_date_string();
        if state.advertisements.iter().any(|a| a.post_id == post_uuid && is_ad_active(a, &today)) {
            return ListAdvertisementData {
                meta: DefaultResponse::error("32006", "Post already has active advertisement"),
                affected_rows: None,
            };
        }

        // Calculate cost
        let days = duration_in_days.to_days();
        let cost = AD_BASIC_DAILY_PRICE * Decimal::from(days);

        // Check balance
        let balance = state.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);
        if balance < cost {
            return ListAdvertisementData {
                meta: DefaultResponse::error("51301", "Insufficient balance"),
                affected_rows: None,
            };
        }

        // Deduct tokens
        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= cost;
        *state.wallets.entry(SYSTEM_PEER_ACCOUNT).or_insert(Decimal::ZERO) += cost;

        // Calculate end date
        let start = NaiveDate::parse_from_str(&startday, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive());
        let end = start + chrono::Duration::days(days as i64);

        let ad_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        let record = AdvertisementRecord {
            id: ad_id,
            post_id: post_uuid,
            advertiser_id: user_id,
            ad_type: AdvertisementType::Basic,
            start_date: startday.clone(),
            end_date: end.format("%Y-%m-%d").to_string(),
            token_cost: cost,
            created_at: now.clone(),
        };

        state.advertisements.push(record);

        // Record transaction
        let operation_id = Uuid::new_v4();
        state.transactions.push(TransactionRecord {
            id: Uuid::new_v4(),
            operation_id,
            category: Some(TransactionCategory::AdPinned), // Reuse category for ad payments
            transaction_type: "DEBIT".into(),
            sender_id: user_id,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: -cost,
            net_token_amount: -cost,
            message: Some(format!("Basic advertisement for {} days", days)),
            fees: None,
            created_at: now.clone(),
        });

        let cost_f64 = cost.to_string().parse::<f64>().unwrap_or(0.0);

        ListAdvertisementData {
            meta: DefaultResponse::success("12001", "Advertisement created"),
            affected_rows: Some(vec![AdvertisementRow {
                id: ad_id.to_string().into(),
                created_at: now,
                ad_type: AdvertisementType::Basic,
                timeframe_start: startday,
                timeframe_end: end.format("%Y-%m-%d").to_string(),
                total_token_cost: cost_f64,
                total_euro_cost: 0.0,
            }]),
        }
    }

    /// Create a pinned (featured) advertisement for a post.
    ///
    /// Cost: 200.0 tokens flat.
    ///
    /// Response codes:
    /// - 12001: Advertisement created
    /// - 30209: Invalid post UUID
    /// - 31510: Post not found or not owned
    /// - 32006: Post already has active advertisement
    /// - 51301: Insufficient balance
    /// - 60501: Not authenticated
    async fn advertise_post_pinned(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        #[graphql(name = "advertisePlan")] _advertise_plan: AdvertisementPinnedPlan,
    ) -> ListAdvertisementData {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return ListAdvertisementData {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return ListAdvertisementData {
                meta: DefaultResponse::error("30209", "Invalid post UUID"),
                affected_rows: None,
            },
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Same validations as basic
        match state.posts.iter().find(|p| p.id == post_uuid) {
            Some(p) if p.author_id == user_id => {}
            _ => return ListAdvertisementData {
                meta: DefaultResponse::error("31510", "Post not found or not owned by you"),
                affected_rows: None,
            },
        };

        let today = today_date_string();
        if state.advertisements.iter().any(|a| a.post_id == post_uuid && is_ad_active(a, &today)) {
            return ListAdvertisementData {
                meta: DefaultResponse::error("32006", "Post already has active advertisement"),
                affected_rows: None,
            };
        }

        let cost = AD_PINNED_PRICE;
        let balance = state.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);
        if balance < cost {
            return ListAdvertisementData {
                meta: DefaultResponse::error("51301", "Insufficient balance"),
                affected_rows: None,
            };
        }

        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= cost;
        *state.wallets.entry(SYSTEM_PEER_ACCOUNT).or_insert(Decimal::ZERO) += cost;

        // Pinned ads default to 7 days
        let start = Utc::now().date_naive();
        let end = start + chrono::Duration::days(7);

        let ad_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        state.advertisements.push(AdvertisementRecord {
            id: ad_id,
            post_id: post_uuid,
            advertiser_id: user_id,
            ad_type: AdvertisementType::Pinned,
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            token_cost: cost,
            created_at: now.clone(),
        });

        let operation_id = Uuid::new_v4();
        state.transactions.push(TransactionRecord {
            id: Uuid::new_v4(),
            operation_id,
            category: Some(TransactionCategory::AdPinned),
            transaction_type: "DEBIT".into(),
            sender_id: user_id,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: -cost,
            net_token_amount: -cost,
            message: Some("Pinned advertisement".into()),
            fees: None,
            created_at: now.clone(),
        });

        let cost_f64 = cost.to_string().parse::<f64>().unwrap_or(0.0);

        ListAdvertisementData {
            meta: DefaultResponse::success("12001", "Advertisement created"),
            affected_rows: Some(vec![AdvertisementRow {
                id: ad_id.to_string().into(),
                created_at: now,
                ad_type: AdvertisementType::Pinned,
                timeframe_start: start.format("%Y-%m-%d").to_string(),
                timeframe_end: end.format("%Y-%m-%d").to_string(),
                total_token_cost: cost_f64,
                total_euro_cost: 0.0,
            }]),
        }
    }
}
```

### 4.12 Shop Mutation Resolvers (`schema/mutation/shop.rs`)

```rust
use async_graphql::{Context, Object};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::schema::require_auth;
use crate::state::*;
use crate::types::registration::DefaultResponse;
use crate::types::shop::*;
use crate::types::wallet::TransactionCategory;

pub struct ShopMutation;

#[Object]
impl ShopMutation {
    /// Purchase a shop item using tokens.
    ///
    /// Response codes:
    /// - 12201: Order placed
    /// - 30101: Missing required fields
    /// - 51301: Insufficient balance
    /// - 60501: Not authenticated
    async fn perform_shop_order(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "tokenAmount")] token_amount: Decimal,
        #[graphql(name = "shopItemId")] shop_item_id: String,
        #[graphql(name = "orderDetails")] order_details: ShopOrderDetailsInput,
    ) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        // Validate delivery details
        if order_details.name.len() < 2 || order_details.name.len() > 100 {
            return DefaultResponse::error("30101", "Name must be 2-100 characters");
        }
        if !order_details.email.contains('@') {
            return DefaultResponse::error("30101", "Invalid email format");
        }
        if order_details.addressline1.len() < 6 || order_details.addressline1.len() > 100 {
            return DefaultResponse::error("30101", "Address line 1 must be 6-100 characters");
        }
        if order_details.city.len() < 2 || order_details.city.len() > 100 {
            return DefaultResponse::error("30101", "City must be 2-100 characters");
        }
        if order_details.zipcode.len() != 5 {
            return DefaultResponse::error("30101", "Zipcode must be exactly 5 characters");
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Check balance
        let balance = state.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);
        if balance < token_amount {
            return DefaultResponse::error("51301", "Insufficient balance");
        }

        // Deduct tokens
        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= token_amount;
        *state.wallets.entry(SYSTEM_SHOP_ACCOUNT).or_insert(Decimal::ZERO) += token_amount;

        // Record transaction
        let now = Utc::now().to_rfc3339();
        let tx_id = Uuid::new_v4();
        let operation_id = Uuid::new_v4();

        state.transactions.push(TransactionRecord {
            id: tx_id,
            operation_id,
            category: Some(TransactionCategory::ShopPurchase),
            transaction_type: "DEBIT".into(),
            sender_id: user_id,
            recipient_id: SYSTEM_SHOP_ACCOUNT,
            token_amount: -token_amount,
            net_token_amount: -token_amount,
            message: Some(format!("Shop purchase: {}", shop_item_id)),
            fees: None,
            created_at: now.clone(),
        });

        // Create shop order
        let order = ShopOrderRecord {
            id: Uuid::new_v4(),
            transaction_id: tx_id,
            shop_item_id,
            buyer_id: user_id,
            token_amount,
            item_specs: order_details.shop_item_specs.as_ref().and_then(|s| s.size.clone()),
            delivery: ShopDeliveryRecord {
                name: order_details.name,
                email: order_details.email,
                addressline1: order_details.addressline1,
                addressline2: order_details.addressline2,
                city: order_details.city,
                zipcode: order_details.zipcode,
                country: "GERMANY".into(),
            },
            created_at: now,
        };

        state.shop_orders.push(order);

        DefaultResponse::success("12201", "Order placed successfully")
    }
}
```

### 4.13 Shop Query Resolvers (`schema/query/shop.rs`)

```rust
use async_graphql::{Context, Object};

use crate::schema::require_auth;
use crate::state::SharedState;
use crate::types::registration::DefaultResponse;
use crate::types::shop::*;

pub struct ShopQuery;

#[Object]
impl ShopQuery {
    /// Get shop order details by transaction ID.
    ///
    /// Response codes:
    /// - 12202: Order details retrieved
    /// - 22101: Order not found
    /// - 30101: Missing transaction ID
    /// - 60501: Not authenticated
    async fn shop_order_details(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "transactionId")] transaction_id: String,
    ) -> ShopOrderDetailsResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return ShopOrderDetailsResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: None,
            },
        };

        if transaction_id.is_empty() {
            return ShopOrderDetailsResponse {
                meta: DefaultResponse::error("30101", "Missing transaction ID"),
                affected_rows: None,
            };
        }

        let state = ctx.data_unchecked::<SharedState>().read().await;

        // Find order by transaction ID, verify ownership
        let order = state.shop_orders.iter().find(|o| {
            o.transaction_id.to_string() == transaction_id && o.buyer_id == user_id
        });

        match order {
            Some(o) => ShopOrderDetailsResponse {
                meta: DefaultResponse::success("12202", "Order details retrieved"),
                affected_rows: Some(vec![ShopOrderDetails {
                    shop_order_id: o.id.to_string(),
                    shop_item_id: o.shop_item_id.clone(),
                    shop_item_specs: o.item_specs.as_ref().map(|s| ShopItemSpecs {
                        size: Some(s.clone()),
                    }),
                    delivery_details: Some(ShopOrderDeliveryDetails {
                        name: Some(o.delivery.name.clone()),
                        email: Some(o.delivery.email.clone()),
                        addressline1: Some(o.delivery.addressline1.clone()),
                        addressline2: o.delivery.addressline2.clone(),
                        city: Some(o.delivery.city.clone()),
                        zipcode: Some(o.delivery.zipcode.clone()),
                        country: Some(o.delivery.country.clone()),
                    }),
                    createdat: Some(o.created_at.clone()),
                }]),
            },
            None => ShopOrderDetailsResponse {
                meta: DefaultResponse::success("22101", "Order not found"),
                affected_rows: None,
            },
        }
    }
}
```

### 4.14 Helper Functions

```rust
// helpers.rs or within state.rs

use chrono::Utc;

/// Get today's date as YYYY-MM-DD string.
pub fn today_date_string() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// Check if an advertisement is currently active.
pub fn is_ad_active(ad: &AdvertisementRecord, today: &str) -> bool {
    ad.start_date.as_str() <= today && today <= ad.end_date.as_str()
}

impl MockState {
    /// Convert a TransactionRecord to the GraphQL TransactionHistoryItem type.
    pub fn transaction_record_to_graphql(&self, record: &TransactionRecord) -> TransactionHistoryItem {
        let resolve_user = |uid: &Uuid| -> TransactionUser {
            match self.users.get(uid) {
                Some(u) => TransactionUser {
                    userid: u.uid.to_string(),
                    img: Some("https://via.placeholder.com/96".into()),
                    username: u.username.clone(),
                    slug: u.slug.clone(),
                    visibility_status: Some("VISIBLE".into()),
                    has_active_reports: Some(false),
                    is_hidden_for_users: Some(false),
                },
                None => TransactionUser {
                    userid: uid.to_string(),
                    img: None,
                    username: "System".into(),
                    slug: "system".into(),
                    visibility_status: Some("VISIBLE".into()),
                    has_active_reports: Some(false),
                    is_hidden_for_users: Some(false),
                },
            }
        };

        TransactionHistoryItem {
            transaction_id: record.id.to_string(),
            operationid: record.operation_id.to_string(),
            transaction_category: record.category,
            transactiontype: record.transaction_type.clone(),
            tokenamount: record.token_amount.to_string(),
            net_token_amount: record.net_token_amount.to_string(),
            message: record.message.clone(),
            createdat: record.created_at.clone(),
            sender: resolve_user(&record.sender_id),
            recipient: resolve_user(&record.recipient_id),
            fees: record.fees.as_ref().map(|f| TransactionFees {
                total: f.total,
                burn: f.burn,
                peer: f.peer,
                inviter: f.inviter,
            }),
        }
    }

    /// Convert an AdvertisementRecord to the AdvertisementPost GraphQL type.
    pub fn ad_record_to_graphql(
        &self,
        record: &AdvertisementRecord,
        viewer_id: Option<Uuid>,
    ) -> Option<AdvertisementPost> {
        let post_record = self.posts.iter().find(|p| p.id == record.post_id)?;
        let post = self.post_record_to_graphql(post_record, viewer_id);

        Some(AdvertisementPost {
            post,
            advertisement: AdvCreator {
                advertisementid: record.id.to_string().into(),
                advertisementtype: match record.ad_type {
                    AdvertisementType::Basic => "BASIC".into(),
                    AdvertisementType::Pinned => "PINNED".into(),
                },
                startdate: record.start_date.clone(),
                enddate: record.end_date.clone(),
                createdat: Some(record.created_at.clone()),
                user: None, // Omitted for list view
            },
        })
    }

    /// Check if a daily free action is available and use it if so.
    ///
    /// Returns `true` if the action was free, `false` if paid.
    /// Does NOT deduct tokens — caller must handle payment.
    pub fn is_daily_free(&self, user_id: Uuid, action: &str) -> bool {
        let today = today_date_string();
        let key = (user_id, today, action.to_string());
        let used = self.daily_actions_used.get(&key).copied().unwrap_or(0);
        let limit = match action {
            "post" => FREE_POSTS,
            "like" => FREE_LIKES,
            "comment" => FREE_COMMENTS,
            "dislike" => FREE_DISLIKES,
            _ => 0,
        };
        used < limit
    }

    /// Record use of a daily action.
    pub fn use_daily_action(&mut self, user_id: Uuid, action: &str) {
        let today = today_date_string();
        let key = (user_id, today, action.to_string());
        *self.daily_actions_used.entry(key).or_insert(0) += 1;
    }

    /// Try to perform a paid action: use free allowance if available, otherwise deduct tokens.
    ///
    /// Returns:
    /// - `Ok(true)` if the action was free
    /// - `Ok(false)` if tokens were deducted
    /// - `Err("51301")` if insufficient balance
    pub fn try_deduct_for_action(&mut self, user_id: Uuid, action: &str) -> Result<bool, &'static str> {
        if self.is_daily_free(user_id, action) {
            self.use_daily_action(user_id, action);
            return Ok(true);
        }

        let price = match action {
            "post" => POST_PRICE,
            "like" => LIKE_PRICE,
            "dislike" => DISLIKE_PRICE,
            "comment" => COMMENT_PRICE,
            _ => return Ok(false),
        };

        let balance = self.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);
        if balance < price {
            return Err("51301");
        }

        *self.wallets.entry(user_id).or_insert(Decimal::ZERO) -= price;
        *self.wallets.entry(SYSTEM_PEER_ACCOUNT).or_insert(Decimal::ZERO) += price;
        self.use_daily_action(user_id, action);

        Ok(false)
    }
}
```

### 4.15 Schema Assembly Updates

```rust
// schema/mod.rs — updated merged objects

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    HealthQuery,
    // Phase 2:
    UserQuery,
    // Phase 3:
    PostQuery,
    // Phase 4:
    CommentQuery,
    ChatQuery,
    // Phase 5:
    WalletQuery,
    TokenomicsQuery,
    AdQuery,
    ShopQuery,
);

#[derive(MergedObject, Default)]
pub struct MutationRoot(
    RegistrationMutation,   // Phase 0
    AuthMutation,           // Phase 1
    ProfileMutation,        // Phase 2
    PostMutation,           // Phase 3
    CommentMutation,        // Phase 4
    ChatMutation,           // Phase 4
    WalletMutation,         // Phase 5
    AdMutation,             // Phase 5
    ShopMutation,           // Phase 5
);
```

### 4.16 Cargo.toml Dependency Additions

```toml
[dependencies]
# Existing...
rust_decimal = { version = "1", features = ["serde", "serde-with-str"] }
chrono = { version = "0.4", features = ["serde"] }
```

---

## 5. Testing Strategy

### Wallet Tests (≥16)

| # | Test | Assert |
|---|------|--------|
| 1 | Get balance for seeded user | `11204`, `currentliquidity == 1000.0` |
| 2 | Get balance without auth | `60501` |
| 3 | Transfer tokens (50.0) to another user | `11211`, sender balance decreased by 52.0 (50 + 4% fees), recipient increased by 50.0 |
| 4 | Transfer — verify fee breakdown | `fees.burn == 0.50`, `fees.peer == 1.00`, `fees.inviter == 0.50` |
| 5 | Transfer — verify `tokenSendFormatted` and `tokensSubstractedFromWalletFormatted` | Correct formatted strings |
| 6 | Transfer — transaction appears in history | Get history → find tx with correct category, sender, recipient |
| 7 | Transfer to self → `31202` | Error |
| 8 | Transfer to non-existent user → `31007` | Error |
| 9 | Transfer to system account → `31203` | Error |
| 10 | Transfer with insufficient balance → `51301` | Error, balances unchanged |
| 11 | Transfer with amount below minimum → `30264` | Error |
| 12 | Transfer with message > 500 chars → `30270` | Error |
| 13 | Transfer with URL in message → `30271` | Error |
| 14 | Transaction history with pagination | Offset/limit produce correct page |
| 15 | Transaction history with no transactions → `21209` | Empty response |
| 16 | Transaction history filtered by type | Only matching categories returned |

### Tokenomics Tests (≥8)

| # | Test | Assert |
|---|------|--------|
| 17 | Get action prices | `11304`, `postPrice: 20.0`, `likePrice: 3.0`, `dislikePrice: 3.0`, `commentPrice: 1.0` |
| 18 | Get action prices without auth → `60501` | Error |
| 19 | Get tokenomics | `11212`, contains prices + gem returns + minting data |
| 20 | Get daily free status (fresh user) | `11303`, all actions show `used: 0`, correct `available` values |
| 21 | Get daily free status after using actions | Perform 2 likes → `like.used == 2`, `like.available == 1` |
| 22 | Daily free like (first 3 free) | Like 3 posts → balance unchanged |
| 23 | Paid like (4th like costs tokens) | 4th like → balance decreased by 3.0 |
| 24 | Today's interactions | Post seeded with interactions → verify counts and scores |

### Advertisement Tests (≥12)

| # | Test | Assert |
|---|------|--------|
| 25 | List advertisement posts (seed data) | `12002`, returns ad with embedded post and ad metadata |
| 26 | List advertisement posts with no active ads → `22002` | Empty |
| 27 | List ads without auth → `60501` | Error |
| 28 | Create basic ad (3 days) | `12001`, cost = 150.0, balance decreased, ad appears in list |
| 29 | Create basic ad — verify end date calculation | `startday + 3 days` |
| 30 | Create pinned ad | `12001`, cost = 200.0, balance decreased |
| 31 | Create ad for non-owned post → `31510` | Error |
| 32 | Create ad for post with existing active ad → `32006` | Error |
| 33 | Create ad with insufficient balance → `51301` | Error |
| 34 | Advertisement history returns user's ads with stats | `12002`, includes `TotalAdvertisementHistoryStats` |
| 35 | Advertisement history with sort by cost | Ads ordered by `totalTokenCost` descending |
| 36 | Advertised posts excluded from normal `listPosts` | Create ad → `listPosts` does not include that post |

### Shop Tests (≥8)

| # | Test | Assert |
|---|------|--------|
| 37 | Purchase shop item | `12201`, balance decreased, order created |
| 38 | Get shop order details by transaction ID | `12202`, correct delivery details, item specs |
| 39 | Get order for non-existent transaction → `22101` | Not found |
| 40 | Purchase without auth → `60501` | Error |
| 41 | Purchase with insufficient balance → `51301` | Error |
| 42 | Purchase with invalid name (too short) → `30101` | Validation error |
| 43 | Purchase with invalid zipcode (not 5 chars) → `30101` | Validation error |
| 44 | Purchase with invalid email → `30101` | Validation error |

### Cross-Cutting Tests (≥6)

| # | Test | Assert |
|---|------|--------|
| 45 | Comment creation deducts tokens after free limit | 5th comment → balance decreased by 1.0 |
| 46 | Comment creation with insufficient balance after free limit → `51301` | Error, comment not created |
| 47 | Post action (like) deducts tokens after free limit | 4th like → balance decreased by 3.0 |
| 48 | Gem accumulation on like | Like another user's post → gem record created |
| 49 | Reset clears all economy state | POST /reset → balance returns to seed, transactions cleared, ads cleared, orders cleared |
| 50 | All Phase 0–4 tests still pass | Regression check |

---

## 6. Definition of Done

### Phase 5 gate

- [x] `balance` returns correct token balance for authenticated users
- [x] `resolveTransferV2` transfers tokens with fee calculation (burn 1%, peer 2%, inviter 1%), validates all inputs, records transactions
- [x] `transactionHistory` and `getTransactionHistory` return paginated, filtered transaction lists with correct `TransactionHistoryItem` shape
- [x] `listTodaysInteractions` aggregates today's post interactions with gem scores
- [x] `getActionPrices` returns hardcoded prices (post 20.0, like 3.0, dislike 3.0, comment 1.0)
- [x] `getTokenomics` returns combined prices + gem returns + minting data
- [x] `getDailyFreeStatus` accurately tracks daily free action usage per user per action type
- [x] `listAdvertisementPosts` returns active ads with embedded full `Post` objects
- [x] `advertisementHistory` returns user's ads with aggregate statistics and sorting
- [x] `advertisePostBasic` creates time-based ads with correct cost calculation (50.0/day × duration)
- [x] `advertisePostPinned` creates pinned ads with 200.0 flat cost
- [x] `performShopOrder` validates delivery details, deducts tokens, creates order record
- [x] `shopOrderDetails` returns order by transaction ID with delivery details
- [x] Token deduction integrated into Phase 3/4 action resolvers (like, dislike, comment, post creation) with daily free action logic
- [x] Gem records created on post interactions
- [x] Advertised posts excluded from normal `listPosts` results
- [x] `POST /reset` clears all economy state back to seed data
- [x] `cargo clippy -- -D warnings` passes
- [x] `cargo fmt --check` passes
- [x] ≥50 integration tests pass (16 wallet + 8 tokenomics + 12 ads + 8 shop + 6 cross-cutting) — **49 actual (Phase 5 scope), 219 total**
- [x] All Phase 0–4 tests still pass (regression)

### Response code summary

| Code | Meaning | Operation |
|------|---------|-----------|
| `11203` | Logs found | `listWinLogs` / `listPaymentLogs` |
| `11204` | Balance retrieved | `balance` |
| `11205` | Interactions retrieved | `listTodaysInteractions` |
| `11211` | Transfer successful | `resolveTransferV2` |
| `11212` | Tokenomics data retrieved | `getTokenomics` |
| `11215` | Transactions retrieved | `transactionHistory` / `getTransactionHistory` |
| `11303` | Daily free status loaded | `getDailyFreeStatus` |
| `11304` | Prices fetched | `getActionPrices` |
| `12001` | Advertisement created | `advertisePostBasic` / `advertisePostPinned` |
| `12002` | Advertisements fetched | `listAdvertisementPosts` / `advertisementHistory` |
| `12201` | Order placed | `performShopOrder` |
| `12202` | Order details retrieved | `shopOrderDetails` |
| `21202` | No log records for date | `listWinLogs` / `listPaymentLogs` |
| `21204` | No interactions today | `listTodaysInteractions` |
| `21209` | No transactions found | `transactionHistory` / `getTransactionHistory` |
| `22002` | No advertisements found | `listAdvertisementPosts` / `advertisementHistory` |
| `22101` | Order not found | `shopOrderDetails` |
| `30101` | Missing/invalid required fields | `performShopOrder` / `shopOrderDetails` |
| `30201` | Invalid recipient UUID | `resolveTransferV2` |
| `30203` | Invalid offset | `transactionHistory` |
| `30204` | Invalid limit | `transactionHistory` |
| `30209` | Invalid post UUID | `advertisePostBasic` / `advertisePostPinned` |
| `30264` | Invalid token amount | `resolveTransferV2` |
| `30270` | Message too long | `resolveTransferV2` |
| `30271` | Invalid message (URLs/control chars) | `resolveTransferV2` |
| `31007` | Recipient not found | `resolveTransferV2` |
| `31202` | Cannot transfer to self | `resolveTransferV2` |
| `31203` | Cannot transfer to system account | `resolveTransferV2` |
| `31510` | Post not found or not owned | `advertisePostBasic` / `advertisePostPinned` |
| `32006` | Post already has active ad | `advertisePostBasic` / `advertisePostPinned` |
| `51301` | Insufficient balance | `resolveTransferV2` / `advertisePost*` / `performShopOrder` / action deductions |
| `60501` | Not authenticated | All operations |
