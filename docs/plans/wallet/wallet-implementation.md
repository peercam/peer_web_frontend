# Wallet Implementation Plan

**Feature:** Wallet  
**Priority:** #8 (after Chat)  
**Status:** ✅ Implemented (tests pending)  
**Created:** 2026-04-12  
**Updated:** 2026-04-14

---

## Overview

Implement the wallet page for the Leptos frontend. The wallet provides users with their token balance, P2P transfer capabilities, and a comprehensive transaction history with fee breakdowns in the Peer Network economy.

### Goals

1. Full parity with legacy `wallet.php` user experience
2. Display current token balance with real-time refresh
3. P2P token transfer modal with recipient search
4. Transaction history with infinite scroll
5. Transaction detail view with fee breakdown
6. Shop order delivery details (for shop purchases)
7. Secure transfer validation (amount, recipient, message)
8. Responsive layout for mobile/desktop

---

## Scope

### In Scope

- [x] Wallet page (`/wallet` route)
- [x] Balance display with animated logo
- [x] Reload balance button
- [x] Token transfer button → transfer modal
- [x] Transaction history list with infinite scroll
- [x] Transaction categories (P2P, Mint, Like, Dislike, Shop, etc.)
- [x] Transaction detail expansion (click to expand)
- [x] Fee breakdown display (burn, peer, inviter)
- [x] Transfer modal:
  - [x] Friend list display
  - [x] User search by username (with debounced input)
  - [x] Amount input with validation
  - [x] Fee calculation preview
  - [x] Message input (500 char max, no URLs)
  - [x] Summary/confirmation screen
  - [x] Success/error feedback (with timed auto-close)
- [ ] Shop purchase order details (delivery info) — model + query exist, UI not wired up
- [x] Protected route (authentication required via `AuthGuard`)
- [x] Loading states and skeletons
- [x] Error states

### Out of Scope (Future Work)

- Shop order delivery detail view (model exists, needs UI wiring)
- Thousand-separator formatting for balance display
- Win logs / Payment logs detail views
- Today's interactions summary
- Tokenomics info modal
- Token minting details
- Wallet analytics/charts
- Export transaction history

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `wallet.php` | Page template with layout structure |
| `js/wallet.js` | Transaction history, transfer modal, API calls |
| `js/peerShopProducts.js` | Shop product definitions (for order details) |
| `css/wallet.css` | Wallet page styles |
| `template-parts/wallet/walletTokenTransfer.php` | Transfer dropdown container |
| `template-parts/sidebars/widget-profile.php` | Profile widget |
| `template-parts/sidebars/widget-main-menu.php` | Navigation menu |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  HEADER: 💰 Wallet                                         │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│  (empty)     │  ┌─────────────────────┐│                   │
│              │  │ Available balance   ││  - Profile widget │
│              │  │ [LOGO] 12,345       ││  - Main menu      │
│              │  │                     ││  - New post btn   │
│              │  │ [Transfer] [Reload] ││  - Version        │
│              │  └─────────────────────┘│                   │
│              │                         │                   │
│              │  ┌─────────────────────┐│                   │
│              │  │ Transactions        ││                   │
│              │  │ ─────────────────── ││                   │
│              │  │ [Tx 1] +200  date   ││                   │
│              │  │ [Tx 2] -50   date   ││                   │
│              │  │ [Tx 3] +15   date   ││                   │
│              │  │ ...infinite scroll  ││                   │
│              │  └─────────────────────┘│                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### Transaction Item Layout

```
┌────────────────────────────────────────────────────────────┐
│  [Icon/Avatar]  Transfer from @Username #123456            │
│                 📝 "Thank you for your help..."            │
│                                                            │
│                 10 Jun 2025, 04:20         +2000 [logo]    │
└────────────────────────────────────────────────────────────┘

On click/expand:
┌────────────────────────────────────────────────────────────┐
│  Transaction amount                    2000.00000000       │
│  Base amount                           1920.00000000       │
│  ───────────────────────────────────────────               │
│  Fees included                         80.00000000         │
│   • 2% to Peer Bank (platform fee)     40.00000000         │
│   • 1% Burned                          20.00000000         │
│   • 1% to your Inviter                 20.00000000         │
│  ───────────────────────────────────────────               │
│  📝 Message:                                               │
│  "Full message content here..."                            │
└────────────────────────────────────────────────────────────┘
```

### Transfer Modal Flow

```
┌──────────────────────────────────┐
│  [X]           Transfer          │
├──────────────────────────────────┤
│  Your Balance: 12,345            │
│                                  │
│  Recipient username              │
│  [🔍 Search user...]             │
│                                  │
│  ┌────────────────────────────┐  │
│  │ @Friend1 #12345           │  │
│  │ @Friend2 #67890           │  │
│  │ ...friend list            │  │
│  └────────────────────────────┘  │
└──────────────────────────────────┘
           ↓ Select user
┌──────────────────────────────────┐
│  [X]           Transfer          │
├──────────────────────────────────┤
│  Your Balance: 12,345            │
│                                  │
│  Sending to                      │
│  [Avatar] @Username #1234 [✏️]   │
│                                  │
│  Enter amount                    │
│  [____________] min: 0.00000001  │
│                                  │
│  Transfer fee: 4.04              │
│   └─ (breakdown on expand)       │
│  Total amount: 104.04            │
│                                  │
│  Add a message (optional) 0/500  │
│  [________________________]      │
│  Letters, numbers, emojis. No links│
│                                  │
│            [Continue]            │
└──────────────────────────────────┘
           ↓ Continue
┌──────────────────────────────────┐
│  [X]           Summary           │
├──────────────────────────────────┤
│  Remaining balance: 12,240.96    │
│                                  │
│  Sending to                      │
│  [Avatar] @Username #1234        │
│                                  │
│  Amount: 100.00                  │
│  Fee: 4.04                       │
│  Total: 104.04                   │
│                                  │
│  Message:                        │
│  "Thanks for the coffee! ☕"     │
│                                  │
│  [Back]       [Submit transfer]  │
└──────────────────────────────────┘
           ↓ Submit
┌──────────────────────────────────┐
│       ✅ Completed               │
│                                  │
│  Your transfer was sent          │
│  successfully.                   │
│                                  │
│            [OK]                  │
└──────────────────────────────────┘
```

### Key Features

1. **Balance Display**
   - Current token balance with animated Peer logo
   - Manual refresh button
   - Real-time balance update after transfer

2. **Transaction History**
   - Infinite scroll with IntersectionObserver
   - 20 transactions per batch
   - Duplicate prevention with `seenTxIds` Set
   - Categories: P2P_TRANSFER, TOKEN_MINT, LIKE, DISLIKE, COMMENT, POST_CREATE, AD_PINNED, SHOP_PURCHASE, FEE
   - Expandable detail view

3. **Transaction Categories**
   | Category | Icon | Direction |
   |----------|------|-----------|
   | P2P_TRANSFER | User avatar | In/Out |
   | TOKEN_MINT | ⛏️ Daily Mint | In |
   | LIKE | ❤️ Extra Like | Out |
   | DISLIKE | 👎 Dislike | Out |
   | COMMENT | 💬 Extra Comment | Out |
   | POST_CREATE | 📷 Extra Post | Out |
   | AD_PINNED | 📌 Pinned Promo | Out |
   | SHOP_PURCHASE | 🛒 Peer Shop | Out |

4. **Fee Structure**
   - 2% Platform fee (to Peer Bank)
   - 1% Burn (removed from supply)
   - 1% Inviter fee (if user has inviter)
   - Total: 4% (or 3% if no inviter)

5. **Transfer Validation**
   - Minimum: 0.000001 tokens
   - Maximum decimals: 8
   - Recipient must exist
   - Cannot transfer to self
   - Cannot transfer to system accounts
   - Message max: 500 characters
   - No URLs in message
   - Sufficient balance (amount + fees)

6. **Shop Order Details**
   - Lazy-loaded on expand
   - Delivery info: name, email, address
   - Item specs: size, product name

---

## Backend API Reference

### `balance` Query

```graphql
query {
  balance {
    meta {
      status
      ResponseCode
      ResponseMessage
    }
    currentliquidity
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11204` | Balance retrieved |
| `41201` | Failed to fetch balance |
| `60501` | Not authenticated |

---

### `transactionHistory` Query

```graphql
query TransactionHistory($offset: Int, $limit: Int) {
  transactionHistory(offset: $offset, limit: $limit) {
    meta {
      status
      RequestId
      ResponseCode
      ResponseMessage
    }
    affectedRows {
      transactionId
      operationid
      transactionCategory
      transactiontype
      tokenamount
      netTokenAmount
      message
      createdat
      sender {
        userid
        img
        username
        slug
        biography
        visibilityStatus
        hasActiveReports
        isHiddenForUsers
        updatedat
      }
      recipient {
        userid
        img
        username
        slug
        biography
        visibilityStatus
        hasActiveReports
        isHiddenForUsers
        updatedat
      }
      fees {
        total
        burn
        peer
        inviter
      }
    }
  }
}
```

#### Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `offset` | `Int` | No | Pagination offset (default: 0) |
| `limit` | `Int` | No | Results per page (default/max: 20) |

#### Response Codes

| Code | Description |
|------|-------------|
| `11215` | Transactions retrieved |
| `21209` | No transactions found |
| `30203` | Invalid offset |
| `30204` | Invalid limit |
| `60501` | Not authenticated |

---

### `listFriends` Query

```graphql
query ListFriends {
  listFriends {
    status
    counter
    ResponseCode
    affectedRows {
      userid
      img
      username
      slug
    }
  }
}
```

---

### `searchUser` Query

```graphql
query SearchUser($username: String) {
  searchUser(username: $username) {
    affectedRows {
      id
      username
      slug
      img
    }
  }
}
```

---

### `resolveTransferV2` Mutation

```graphql
mutation ResolveTransferV2(
  $recipient: ID!
  $numberoftokens: Decimal!
  $message: String
) {
  resolveTransferV2(
    recipient: $recipient
    numberoftokens: $numberoftokens
    message: $message
  ) {
    meta {
      status
      RequestId
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

#### Parameters

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `recipient` | `ID!` | Yes | Valid user UUID |
| `numberoftokens` | `Decimal!` | Yes | Min: 0.000001, max 8 decimals |
| `message` | `String` | No | Max 500 chars, no URLs |

#### Response Codes

| Code | Description |
|------|-------------|
| `11211` | Transfer successful |
| `30201` | Invalid recipient UUID |
| `30264` | Invalid token amount |
| `30270` | Message too long |
| `30271` | Invalid message (URLs/control chars) |
| `31007` | Recipient not found |
| `31202` | Cannot transfer to self |
| `31203` | Cannot transfer to fees account |
| `41229` | Transaction failed |
| `51301` | Insufficient balance |
| `60501` | Not authenticated |

---

### `shopOrderDetails` Query

```graphql
query ShopOrderDetails($transactionId: String!) {
  shopOrderDetails(transactionId: $transactionId) {
    affectedRows {
      shopOrderId
      shopItemId
      shopItemSpecs {
        size
      }
      deliveryDetails {
        name
        email
        addressline1
        addressline2
        city
        zipcode
        country
      }
    }
  }
}
```

---

## Implementation Plan

### Phase 1: Models & API Layer

#### 1.1 Transaction Models (`src/models/transaction.rs`)

```rust
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::models::DefaultResponse;

/// Transaction category from backend.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
    #[serde(other)]
    Unknown,
}

/// Basic user info embedded in transactions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionUser {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub visibility_status: Option<String>,
}

/// Fee breakdown for a transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionFees {
    pub total: Decimal,
    pub burn: Decimal,
    pub peer: Decimal,
    pub inviter: Option<Decimal>,
}

/// Transaction history item.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub transaction_id: String,
    pub operationid: String,
    pub transaction_category: Option<TransactionCategory>,
    pub transactiontype: String,
    pub tokenamount: String,
    pub net_token_amount: String,
    pub message: Option<String>,
    pub createdat: String,
    pub sender: TransactionUser,
    pub recipient: TransactionUser,
    pub fees: Option<TransactionFees>,
}

/// Transaction history response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHistoryResponse {
    pub meta: DefaultResponse,
    pub affected_rows: Option<Vec<Transaction>>,
}

impl TransactionHistoryResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
    
    pub fn transactions(&self) -> Vec<Transaction> {
        self.affected_rows.clone().unwrap_or_default()
    }
}

/// Balance response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub meta: DefaultResponse,
    pub currentliquidity: Option<Decimal>,
}

impl BalanceResponse {
    pub fn balance(&self) -> Decimal {
        self.currentliquidity.unwrap_or_default()
    }
}

/// Transfer response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponse {
    pub meta: DefaultResponse,
}

impl TransferResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}
```

#### 1.2 Wallet API (`src/api/wallet.rs`)

```rust
use leptos::prelude::*;
use rust_decimal::Decimal;

use crate::models::transaction::{
    BalanceResponse, Transaction, TransactionHistoryResponse, TransferResponse,
};
use crate::models::user::BasicUser;

/// Fetch current user's token balance.
#[server(GetBalance, "/api")]
pub async fn get_balance() -> Result<Decimal, ServerFnError> {
    use crate::api::graphql::{query, BalanceData, BALANCE_QUERY};
    use crate::utils::auth::get_auth_token;

    let token = get_auth_token().await?;
    let data: BalanceData = query(BALANCE_QUERY, (), Some(&token)).await?;
    
    Ok(data.balance.balance())
}

/// Fetch transaction history with pagination.
#[server(GetTransactionHistory, "/api")]
pub async fn get_transaction_history(
    offset: i32,
    limit: i32,
) -> Result<Vec<Transaction>, ServerFnError> {
    use crate::api::graphql::{query, TransactionHistoryData, TRANSACTION_HISTORY_QUERY};
    use crate::utils::auth::get_auth_token;

    let token = get_auth_token().await?;
    let vars = TransactionHistoryVars { offset, limit };
    let data: TransactionHistoryData = query(TRANSACTION_HISTORY_QUERY, vars, Some(&token)).await?;
    
    Ok(data.transaction_history.transactions())
}

/// List friends for transfer recipient selection.
#[server(ListFriends, "/api")]
pub async fn list_friends() -> Result<Vec<BasicUser>, ServerFnError> {
    use crate::api::graphql::{query, FriendsData, LIST_FRIENDS_QUERY};
    use crate::utils::auth::get_auth_token;

    let token = get_auth_token().await?;
    let data: FriendsData = query(LIST_FRIENDS_QUERY, (), Some(&token)).await?;
    
    Ok(data.list_friends.affected_rows.unwrap_or_default())
}

/// Search for users by username.
#[server(SearchUser, "/api")]
pub async fn search_user(username: String) -> Result<Vec<BasicUser>, ServerFnError> {
    use crate::api::graphql::{query, SearchUserData, SEARCH_USER_QUERY};
    use crate::utils::auth::get_auth_token;

    let token = get_auth_token().await?;
    let vars = SearchUserVars { username };
    let data: SearchUserData = query(SEARCH_USER_QUERY, vars, Some(&token)).await?;
    
    Ok(data.search_user.affected_rows.unwrap_or_default())
}

/// Transfer tokens to another user.
#[server(TransferTokens, "/api")]
pub async fn transfer_tokens(
    recipient_id: String,
    amount: Decimal,
    message: Option<String>,
) -> Result<TransferResponse, ServerFnError> {
    use crate::api::graphql::{mutate, TransferData, TRANSFER_MUTATION};
    use crate::utils::auth::get_auth_token;

    // Validate message
    if let Some(ref msg) = message {
        if msg.len() > 500 {
            return Err(ServerFnError::new("Message exceeds 500 characters"));
        }
        // Check for URLs
        let url_pattern = regex::Regex::new(r"(://|www\.)").unwrap();
        if url_pattern.is_match(msg) {
            return Err(ServerFnError::new("URLs are not allowed in messages"));
        }
    }

    let token = get_auth_token().await?;
    let vars = TransferVars {
        recipient: recipient_id,
        numberoftokens: amount,
        message: message.unwrap_or_default(),
    };
    
    let data: TransferData = mutate(TRANSFER_MUTATION, vars, Some(&token)).await?;
    
    Ok(data.resolve_transfer_v2)
}
```

#### 1.3 GraphQL Queries (`src/api/graphql.rs` additions)

```rust
pub const BALANCE_QUERY: &str = r#"
    query Balance {
        balance {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            currentliquidity
        }
    }
"#;

pub const TRANSACTION_HISTORY_QUERY: &str = r#"
    query TransactionHistory($offset: Int, $limit: Int) {
        transactionHistory(offset: $offset, limit: $limit) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
            affectedRows {
                transactionId
                operationid
                transactionCategory
                transactiontype
                tokenamount
                netTokenAmount
                message
                createdat
                sender {
                    userid
                    img
                    username
                    slug
                    visibilityStatus
                }
                recipient {
                    userid
                    img
                    username
                    slug
                    visibilityStatus
                }
                fees {
                    total
                    burn
                    peer
                    inviter
                }
            }
        }
    }
"#;

pub const LIST_FRIENDS_QUERY: &str = r#"
    query ListFriends {
        listFriends {
            status
            counter
            ResponseCode
            affectedRows {
                userid
                img
                username
                slug
            }
        }
    }
"#;

pub const SEARCH_USER_QUERY: &str = r#"
    query SearchUser($username: String) {
        searchUser(username: $username) {
            affectedRows {
                id
                username
                slug
                img
            }
        }
    }
"#;

pub const TRANSFER_MUTATION: &str = r#"
    mutation ResolveTransferV2(
        $recipient: ID!
        $numberoftokens: Decimal!
        $message: String!
    ) {
        resolveTransferV2(
            recipient: $recipient
            numberoftokens: $numberoftokens
            message: $message
        ) {
            meta {
                status
                RequestId
                ResponseCode
                ResponseMessage
            }
        }
    }
"#;

pub const SHOP_ORDER_DETAILS_QUERY: &str = r#"
    query ShopOrderDetails($transactionId: String!) {
        shopOrderDetails(transactionId: $transactionId) {
            affectedRows {
                shopOrderId
                shopItemId
                shopItemSpecs {
                    size
                }
                deliveryDetails {
                    name
                    email
                    addressline1
                    addressline2
                    city
                    zipcode
                    country
                }
            }
        }
    }
"#;
```

### Phase 2: Wallet Page

#### 2.1 Route Setup (`src/app.rs`)

```rust
// Add protected route in app router
<ProtectedRoute path="/wallet" view=WalletPage/>
```

#### 2.2 Wallet Page (`src/pages/wallet.rs`)

```rust
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::wallet::get_balance;
use crate::components::wallet::{
    BalanceHeader, TransactionHistory, TransferModal,
};
use crate::state::auth::use_auth_context;

/// Wallet page - token balance and transaction history.
#[component]
pub fn WalletPage() -> impl IntoView {
    let auth = use_auth_context();
    let show_transfer_modal = RwSignal::new(false);
    
    // Balance resource with manual refresh trigger
    let refresh_trigger = RwSignal::new(0u32);
    let balance = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { get_balance().await.unwrap_or_default() }
    );
    
    let refresh_balance = move |_| {
        refresh_trigger.update(|n| *n += 1);
    };

    view! {
        <Title text="Wallet - Peer Network"/>
        
        <div class="site_layout">
            <header class="site-header header-profile">
                <h1 id="h1">
                    <i class="peer-icon peer-icon-wallet-filled"/>
                    " Wallet"
                </h1>
            </header>
            
            <aside class="left-sidebar left-sidebar-wallet">
                <div class="inner-scroll">
                    // Empty left sidebar
                </div>
            </aside>
            
            <main id="main" class="site-main site-main-wallet">
                <div class="wallet_main">
                    <Suspense fallback=move || view! { <BalanceSkeleton/> }>
                        {move || balance.get().map(|bal| view! {
                            <BalanceHeader
                                balance=bal
                                on_transfer=move |_| show_transfer_modal.set(true)
                                on_reload=refresh_balance
                            />
                        })}
                    </Suspense>
                    
                    <TransactionHistory/>
                </div>
            </main>
            
            <aside class="right-sidebar right-sidebar-wallet">
                <div class="inner-scroll">
                    <ProfileWidget/>
                    <MainMenu/>
                    <NewPostButton/>
                    <VersionWidget/>
                </div>
            </aside>
        </div>
        
        // Transfer modal
        <Show when=move || show_transfer_modal.get()>
            <TransferModal
                on_close=move |_| show_transfer_modal.set(false)
                on_success=move |_| {
                    show_transfer_modal.set(false);
                    refresh_balance(());
                }
            />
        </Show>
    }
}

#[component]
fn BalanceSkeleton() -> impl IntoView {
    view! {
        <div class="wallet_header">
            <h2 class="wallet_heading xl_font_size">"Available balance"</h2>
            <div class="balance_transfer">
                <div class="wallet_balance">
                    <img src="/svg/logo_sw.svg" alt="peer token" class="logo"/>
                    <span class="bold xxxl_font_size skeleton-text">"----"</span>
                </div>
            </div>
        </div>
    }
}
```

### Phase 3: Wallet Components

#### 3.1 Balance Header (`src/components/wallet/balance_header.rs`)

```rust
use leptos::prelude::*;
use rust_decimal::Decimal;

/// Balance header with transfer and reload buttons.
#[component]
pub fn BalanceHeader(
    balance: Decimal,
    on_transfer: impl Fn(()) + 'static,
    on_reload: impl Fn(()) + 'static,
) -> impl IntoView {
    view! {
        <div class="wallet_header">
            <h2 class="wallet_heading xl_font_size">"Available balance"</h2>
            <div class="balance_transfer">
                <div class="wallet_balance">
                    <img src="/svg/logo_sw.svg" alt="peer token" class="logo"/>
                    <span id="token" class="bold xxxl_font_size">
                        {format_balance(balance)}
                    </span>
                </div>
                
                <div class="wallet_transfer">
                    <a
                        href="#"
                        id="openTransferDropdown"
                        class="md_font_size"
                        on:click=move |e| {
                            e.prevent_default();
                            on_transfer(());
                        }
                    >
                        <span>"Transfer "<em>"to user"</em></span>
                        <i class="peer-icon peer-icon-arrow-right"/>
                    </a>
                </div>
                
                <div class="wallet_reload">
                    <a
                        id="reloadTransactions"
                        href="#"
                        class="md_font_size"
                        on:click=move |e| {
                            e.prevent_default();
                            on_reload(());
                        }
                    >
                        "Reload "
                        <i class="peer-icon peer-icon-refresh-alt"/>
                    </a>
                </div>
            </div>
        </div>
    }
}

fn format_balance(balance: Decimal) -> String {
    // Format with thousand separators and appropriate decimals
    let s = balance.to_string();
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}
```

#### 3.2 Transaction History (`src/components/wallet/transaction_history.rs`)

```rust
use leptos::prelude::*;
use std::collections::HashSet;

use crate::api::wallet::get_transaction_history;
use crate::models::transaction::Transaction;

const LIMIT: i32 = 20;

/// Transaction history with infinite scroll.
#[component]
pub fn TransactionHistory() -> impl IntoView {
    let transactions = RwSignal::new(Vec::<Transaction>::new());
    let offset = RwSignal::new(0i32);
    let has_more = RwSignal::new(true);
    let loading = RwSignal::new(false);
    let seen_ids = RwSignal::new(HashSet::<String>::new());
    
    // Load more transactions
    let load_more = move || {
        if loading.get() || !has_more.get() {
            return;
        }
        
        loading.set(true);
        
        spawn_local(async move {
            match get_transaction_history(offset.get(), LIMIT).await {
                Ok(new_txs) => {
                    let count = new_txs.len() as i32;
                    
                    // Filter duplicates
                    let mut ids = seen_ids.get();
                    let unique_txs: Vec<_> = new_txs
                        .into_iter()
                        .filter(|tx| ids.insert(tx.transaction_id.clone()))
                        .collect();
                    seen_ids.set(ids);
                    
                    transactions.update(|txs| txs.extend(unique_txs));
                    offset.update(|o| *o += count);
                    
                    if count < LIMIT {
                        has_more.set(false);
                    }
                }
                Err(e) => {
                    leptos::logging::error!("Failed to load transactions: {}", e);
                }
            }
            loading.set(false);
        });
    };
    
    // Initial load
    Effect::new(move |_| {
        load_more();
    });
    
    // Setup intersection observer
    let sentinel_ref = NodeRef::<html::Div>::new();
    
    Effect::new(move |_| {
        if let Some(sentinel) = sentinel_ref.get() {
            use wasm_bindgen::prelude::*;
            use web_sys::IntersectionObserver;
            
            let callback = Closure::wrap(Box::new(move |entries: js_sys::Array| {
                for entry in entries.iter() {
                    let entry: web_sys::IntersectionObserverEntry = entry.unchecked_into();
                    if entry.is_intersecting() {
                        load_more();
                    }
                }
            }) as Box<dyn Fn(js_sys::Array)>);
            
            let mut options = web_sys::IntersectionObserverInit::new();
            options.root_margin("100% 0px 100% 0px");
            options.threshold(&JsValue::from_f64(0.01));
            
            if let Ok(observer) = IntersectionObserver::new_with_options(
                callback.as_ref().unchecked_ref(),
                &options,
            ) {
                observer.observe(&sentinel);
                callback.forget();
            }
        }
    });

    view! {
        <div class="wallet_transactions">
            <h3 class="transaction_heading xl_font_size">"Transactions"</h3>
            <div id="history-container" class="transaction_lists">
                <For
                    each=move || transactions.get()
                    key=|tx| tx.transaction_id.clone()
                    children=move |tx| view! { <TransactionItem tx=tx/> }
                />
                
                // Sentinel for infinite scroll
                <div
                    id="history-sentinel"
                    node_ref=sentinel_ref
                    style="height: 20px; width: 100%"
                />
                
                // Loading indicator
                <Show when=move || loading.get()>
                    <div class="transaction-loading">
                        "Loading..."
                    </div>
                </Show>
            </div>
        </div>
    }
}
```

#### 3.3 Transaction Item (`src/components/wallet/transaction_item.rs`)

```rust
use leptos::prelude::*;

use crate::models::transaction::{Transaction, TransactionCategory};

/// Single transaction item with expandable details.
#[component]
pub fn TransactionItem(tx: Transaction) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let tx_clone = tx.clone();
    
    // Determine direction and styling
    let amount: f64 = tx.tokenamount.parse().unwrap_or(0.0);
    let is_incoming = amount >= 0.0;
    let direction_class = if is_incoming { "trans_in" } else { "trans_out" };
    
    // Format display amount
    let display_amount = if is_incoming {
        let net: f64 = tx.net_token_amount.parse().unwrap_or(0.0);
        format!("+{}", format_amount(net))
    } else {
        format_amount(amount)
    };
    
    // Get category info
    let (title, icon) = get_category_display(&tx);

    view! {
        <div
            class=format!("tarnsaction_item {}", direction_class)
            class:open=move || expanded.get()
        >
            <div
                class="transaction_record"
                on:click=move |_| expanded.update(|e| *e = !*e)
            >
                <div class="transaction_info">
                    <div class="transaction_media">
                        {icon}
                    </div>
                    <div class="transaction_content">
                        <div class="tinfo md_font_size">
                            <span class="title bold">{title}</span>
                            {transfer_user_info(&tx_clone)}
                        </div>
                        {short_message(&tx_clone)}
                    </div>
                </div>
                <div class="transaction_date md_font_size txt-color-gray">
                    {format_date(&tx.createdat)}
                </div>
                <div class="transaction_price xl_font_size bold">
                    {display_amount}
                </div>
            </div>
            
            <Show when=move || expanded.get()>
                <TransactionDetail tx=tx.clone()/>
            </Show>
        </div>
    }
}

#[component]
fn TransactionDetail(tx: Transaction) -> impl IntoView {
    view! {
        <div class="transaction_detail">
            <div class="transaction_detail_inner">
                <div class="price_detail_row md_font_size">
                    <span class="price_label txt-color-gray">"Transaction amount"</span>
                    <span class="price bold">{format_amount_str(&tx.tokenamount)}</span>
                </div>
                <div class="price_detail_row md_font_size">
                    <span class="price_label txt-color-gray">"Base amount"</span>
                    <span class="price bold">{format_amount_str(&tx.net_token_amount)}</span>
                </div>
                
                {tx.fees.as_ref().map(|fees| view! {
                    <div class="price_detail_row md_font_size">
                        <span class="price_label txt-color-gray">"Fees included"</span>
                        <span class="price bold">{format_decimal(fees.total)}</span>
                    </div>
                    <div class="price_detail_row">
                        <span class="price_label txt-color-gray">"2% to Peer Bank (platform fee)"</span>
                        <span class="price txt-color-gray">{format_decimal(fees.peer)}</span>
                    </div>
                    <div class="price_detail_row">
                        <span class="price_label txt-color-gray">"1% Burned (removed from supply)"</span>
                        <span class="price txt-color-gray">{format_decimal(fees.burn)}</span>
                    </div>
                    {fees.inviter.map(|inv| view! {
                        <div class="price_detail_row">
                            <span class="price_label txt-color-gray">"1% to your Inviter"</span>
                            <span class="price txt-color-gray">{format_decimal(inv)}</span>
                        </div>
                    })}
                })}
                
                {tx.message.as_ref().filter(|m| !m.is_empty()).map(|msg| view! {
                    <div class="message_row">
                        <span class="message_label md_font_size txt-color-gray">
                            <i class="peer-icon peer-icon-message"/>
                            " Message:"
                        </span>
                        <span class="message_body">{msg.clone()}</span>
                    </div>
                })}
            </div>
        </div>
    }
}

fn get_category_display(tx: &Transaction) -> (String, View) {
    match tx.transaction_category {
        Some(TransactionCategory::P2pTransfer) => {
            let is_incoming = tx.tokenamount.parse::<f64>().unwrap_or(0.0) >= 0.0;
            let title = if is_incoming { "Received from" } else { "Transfer to" };
            let user = if is_incoming { &tx.sender } else { &tx.recipient };
            let avatar = user.img.as_ref()
                .map(|img| format!("/media/{}", img.replace("media/", "")))
                .unwrap_or_else(|| "/svg/noname.svg".to_string());
            
            (title.to_string(), view! {
                <span class="wrap_img">
                    <img class="userimg" src=avatar onerror="this.src='/svg/noname.svg'"/>
                </span>
            }.into_any())
        }
        Some(TransactionCategory::TokenMint) => {
            ("Daily Mint".to_string(), view! {
                <i class="peer-icon peer-icon-daily-mint"/>
            }.into_any())
        }
        Some(TransactionCategory::Like) => {
            ("Extra Like".to_string(), view! {
                <i class="peer-icon peer-icon-like-fill red-text"/>
            }.into_any())
        }
        Some(TransactionCategory::Dislike) => {
            ("Dislike".to_string(), view! {
                <i class="peer-icon peer-icon-dislike-fill red-text"/>
            }.into_any())
        }
        Some(TransactionCategory::Comment) => {
            ("Extra comment".to_string(), view! {
                <i class="peer-icon peer-icon-comment-fill"/>
            }.into_any())
        }
        Some(TransactionCategory::PostCreate) => {
            ("Extra post".to_string(), view! {
                <i class="peer-icon peer-icon-camera-fill"/>
            }.into_any())
        }
        Some(TransactionCategory::AdPinned) => {
            ("Pinned post promo".to_string(), view! {
                <i class="peer-icon peer-icon-pinpost"/>
            }.into_any())
        }
        Some(TransactionCategory::ShopPurchase) => {
            ("Peer Shop".to_string(), view! {
                <i class="peer-icon peer-icon-shop"/>
            }.into_any())
        }
        _ => {
            ("Transaction".to_string(), view! {
                <i class="peer-icon peer-icon-wallet"/>
            }.into_any())
        }
    }
}

fn transfer_user_info(tx: &Transaction) -> View {
    if !matches!(tx.transaction_category, Some(TransactionCategory::P2pTransfer)) {
        return view! {}.into_any();
    }
    
    let is_incoming = tx.tokenamount.parse::<f64>().unwrap_or(0.0) >= 0.0;
    let user = if is_incoming { &tx.sender } else { &tx.recipient };
    
    view! {
        <span class="user_name bold italic">"@"{&user.username}</span>
        " "
        <span class="user_slug txt-color-gray">"#"{&user.slug}</span>
    }.into_any()
}

fn short_message(tx: &Transaction) -> View {
    let Some(msg) = tx.message.as_ref().filter(|m| !m.is_empty()) else {
        return view! {}.into_any();
    };
    
    if !matches!(tx.transaction_category, Some(TransactionCategory::P2pTransfer)) {
        return view! {}.into_any();
    }
    
    let short = if msg.len() > 50 {
        format!("{}...", &msg[..50])
    } else {
        msg.clone()
    };
    
    view! {
        <div class="message txt-color-gray">
            <i class="peer-icon peer-icon-message"/>
            {short}
        </div>
    }.into_any()
}
```

### Phase 4: Transfer Modal

#### 4.1 Transfer Modal (`src/components/wallet/transfer_modal.rs`)

```rust
use leptos::prelude::*;
use rust_decimal::Decimal;

use crate::api::wallet::{get_balance, list_friends, search_user, transfer_tokens};
use crate::models::user::BasicUser;

/// Multi-step transfer modal.
#[component]
pub fn TransferModal(
    on_close: impl Fn(()) + Clone + 'static,
    on_success: impl Fn(()) + Clone + 'static,
) -> impl IntoView {
    // Current step: 1 = select user, 2 = enter amount, 3 = confirm
    let step = RwSignal::new(1u8);
    let selected_user = RwSignal::new(Option::<BasicUser>::None);
    let amount = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let balance = RwSignal::new(Decimal::ZERO);
    let submitting = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    
    // Load balance
    spawn_local(async move {
        if let Ok(bal) = get_balance().await {
            balance.set(bal);
        }
    });
    
    let on_close_clone = on_close.clone();
    let on_success_clone = on_success.clone();
    
    // Handle transfer submission
    let submit_transfer = move |_| {
        let Some(user) = selected_user.get() else { return };
        let amt = amount.get();
        let msg = message.get();
        
        let amt_decimal = match amt.parse::<Decimal>() {
            Ok(d) => d,
            Err(_) => {
                error.set(Some("Invalid amount".to_string()));
                return;
            }
        };
        
        submitting.set(true);
        error.set(None);
        
        let on_success = on_success_clone.clone();
        
        spawn_local(async move {
            let user_id = user.id.clone();
            let msg_opt = if msg.is_empty() { None } else { Some(msg) };
            
            match transfer_tokens(user_id, amt_decimal, msg_opt).await {
                Ok(resp) if resp.is_success() => {
                    on_success(());
                }
                Ok(_) => {
                    error.set(Some("Transfer failed. Please try again.".to_string()));
                    submitting.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Error: {}", e)));
                    submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="transfer-backdrop" on:click=move |_| on_close_clone(())/>
        
        <div class="transfer-dropdown" id="transferDropdown">
            <div class="transfer-form-screen">
                <div class="transfer-header">
                    <h2 class="xl_font_size">
                        {move || match step.get() {
                            3 => "Summary",
                            _ => "Transfer",
                        }}
                    </h2>
                    <button class="close-transfer" on:click=move |_| on_close(())>
                        "×"
                    </button>
                </div>
                
                <div class="balance-header">
                    <span class="md_font_size txt-color-gray bal_label">
                        {move || if step.get() == 3 { "Remaining balance" } else { "Your Balance" }}
                    </span>
                    <span class="xl_font_size bold tbalance">
                        {move || {
                            if step.get() == 3 {
                                let amt: Decimal = amount.get().parse().unwrap_or_default();
                                let total = calculate_total_with_fees(amt);
                                format_decimal(balance.get() - total)
                            } else {
                                format_decimal(balance.get())
                            }
                        }}
                    </span>
                </div>
                
                // Step 1: Select user
                <Show when=move || step.get() == 1>
                    <UserSelector
                        on_select=move |user| {
                            selected_user.set(Some(user));
                            step.set(2);
                        }
                    />
                </Show>
                
                // Step 2: Enter amount and message
                <Show when=move || step.get() == 2>
                    <AmountForm
                        user=selected_user.get().unwrap()
                        amount=amount
                        message=message
                        balance=balance.get()
                        on_back=move |_| step.set(1)
                        on_continue=move |_| step.set(3)
                    />
                </Show>
                
                // Step 3: Confirmation
                <Show when=move || step.get() == 3>
                    <ConfirmTransfer
                        user=selected_user.get().unwrap()
                        amount=amount.get()
                        message=message.get()
                        error=error
                        submitting=submitting.get()
                        on_back=move |_| step.set(2)
                        on_submit=submit_transfer
                    />
                </Show>
            </div>
        </div>
    }
}

fn calculate_total_with_fees(amount: Decimal) -> Decimal {
    // 4% fee (or 3% if no inviter - simplified to 4%)
    let fee_rate = Decimal::new(4, 2); // 0.04
    amount + (amount * fee_rate)
}

fn format_decimal(d: Decimal) -> String {
    let s = d.to_string();
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
```

### Phase 5: Styles

#### 5.1 Wallet Styles (`style/wallet.scss`)

Port the existing CSS from `css/wallet.css` with SCSS improvements:
- Variables for colors and spacing
- Nesting for cleaner selectors
- Mixins for common patterns

---

## Testing Plan

### Unit Tests

1. **Amount validation**
   - Minimum amount (0.000001)
   - Maximum decimals (8)
   - Negative amounts
   - Non-numeric input

2. **Fee calculation**
   - With inviter (4%)
   - Without inviter (3%)
   - Edge cases (very small amounts)

3. **Message validation**
   - Under 500 chars
   - Exactly 500 chars
   - Over 500 chars
   - Contains URL (should fail)
   - Emoji handling

### Integration Tests

1. **Balance display**
   - Fetch and display balance
   - Refresh balance

2. **Transaction history**
   - Initial load
   - Infinite scroll
   - Category rendering
   - Expandable details

3. **Transfer flow**
   - Friend list display
   - User search
   - Amount entry + fee display
   - Confirmation + submission
   - Success state
   - Error handling

### E2E Tests

1. Full transfer flow (mock API)
2. Transaction history scroll
3. Modal open/close
4. Error recovery

---

## File Structure

```
src/
├── api/
│   ├── graphql.rs           # GraphQL queries (BALANCE_QUERY, TRANSACTION_HISTORY_QUERY,
│   │                        #   TRANSFER_MUTATION, SHOP_ORDER_DETAILS_QUERY)
│   └── wallet.rs            # Wallet API server functions
├── components/
│   └── wallet/
│       ├── mod.rs               # Module exports
│       ├── balance_header.rs    # Balance display + skeleton
│       ├── transaction_history.rs # Infinite scroll list
│       ├── transaction_item.rs  # Single tx + detail expansion
│       └── transfer_modal.rs    # Multi-step modal (UserSelector,
│                                #   AmountForm, ConfirmTransfer,
│                                #   SuccessScreen all inline)
├── models/
│   └── transaction.rs      # Transaction types + utility fns
├── pages/
│   └── wallet.rs           # Wallet page + WalletHeader,
│                            #   NewPostButton, MobileFooter
└── style/
    └── wallet.scss         # Wallet styles (1092 lines)
```

> **Note:** The plan originally proposed separate files for `user_selector.rs`,
> `amount_form.rs`, and `confirm_transfer.rs`, but these were implemented as
> private sub-components within `transfer_modal.rs` for cohesion.

---

## Migration Checklist

- [x] Create transaction models
- [x] Implement wallet API module
- [x] Add GraphQL queries/mutations
- [x] Create wallet page component
- [x] Create balance header component
- [x] Create transaction history with infinite scroll
- [x] Create transaction item with expand/collapse
- [x] Create transfer modal (user selection)
- [x] Create transfer modal (amount/message form)
- [x] Create transfer modal (confirmation)
- [x] Implement fee calculation
- [x] Add message validation
- [x] Port wallet CSS to SCSS
- [x] Add loading states
- [x] Add error handling
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Write E2E tests
- [ ] Manual QA testing

---

## Dependencies

- rust_decimal (token amount precision)
- web-sys (IntersectionObserver)
- gloo-timers (debounced search, success screen auto-close)
- chrono (transaction date formatting)

> **Note:** The plan originally proposed using `regex` for URL detection, but
> the implementation uses simple substring matching against known patterns
> (`://`, `www.`, `.com`, `.net`, `.org`, `.io`) instead.

---

## Estimates

| Phase | Effort |
|-------|--------|
| Phase 1: Models & API | 4 hours |
| Phase 2: Wallet Page | 2 hours |
| Phase 3: Components | 6 hours |
| Phase 4: Transfer Modal | 6 hours |
| Phase 5: Styles | 3 hours |
| Testing | 4 hours |
| **Total** | **~25 hours** |

---

## Notes

1. **Decimal precision**: Use `rust_decimal` crate for token amounts to match backend precision (8 decimal places). ✅ Done.

2. **Fee visibility**: Show fee breakdown proactively before transfer to avoid user surprise. ✅ Done — expandable fee section in AmountForm.

3. **Real-time validation**: Validate amount and message as user types, not just on submit. ✅ Partially — validation runs on "Continue" click, not on every keystroke.

4. **Optimistic UI**: Consider optimistic updates for balance after transfer (with rollback on error). ❌ Not implemented — balance is re-fetched via `refresh_trigger` after success.

5. **Shop orders**: Lazy-load delivery details only when expanding shop purchase transactions. ❌ Not implemented — model and query exist but no UI wiring.

## Known Issues

1. **Unused `balance` prop** — `AmountForm` accepts a `balance: Decimal` parameter that is never used in the component body (compiler warning). Should either be removed or used for a "max available" indicator.

2. **`format_balance` passthrough** — `format_balance()` just delegates to `format_decimal()` without adding thousand separators as originally intended.

3. **Route protection** — Uses `<Route>` + `<AuthGuard>` wrapper instead of `<ProtectedRoute>`. This works but the page component briefly renders before redirect for unauthenticated users.
