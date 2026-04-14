# Peer Shop Implementation Plan

**Feature:** Peer Shop (View & Purchase Shop Products)  
**Priority:** #13a (first of the remaining "Not Started" pages)  
**Status:** 🚧 In Progress (Phases 1–4 complete, Phases 5–6 remaining)  
**Created:** 2026-04-14  
**Last Updated:** 2026-04-14  
**Reviewed:** 2026-04-14 (code review complete — see Known Issues)

---

## Overview

Implement the Peer Shop page for the Leptos frontend. This page displays the Peer Shop account's profile and products (posts) with integrated e-commerce capabilities. Users can browse shop products, view product details (sizes, pricing), and complete purchases using Peer Tokens through a multi-step checkout flow with delivery information.

The Peer Shop is a **specialised View Profile page** — it renders the `@Peer_Shop` account's profile header and post/product feed, augmented with product pricing badges, an FAQ popup, and a full checkout overlay for purchasing physical merchandise.

### Goals

1. Full parity with legacy `viewPeerShop.php` + `js/peerShopProducts.js` user experience
2. Shop profile header with gradient background, avatar, username, bio, and follow/info buttons
3. Product feed (shop account's posts) with price badges and content type indicators
4. Product checkout overlay: multi-step form (product preview → size selection → delivery form → order review → payment)
5. FAQ popup with shop policies (order processing, delivery, returns, delivery area, support)
6. `performShopOrder` GraphQL mutation for placing orders with token payment
7. Firebase Firestore integration for product metadata (sizes, stock, additional product data)
8. Authenticated-only page (requires login)
9. Responsive layout matching the 3-column View Profile pattern
10. Order success/failure feedback with navigation to wallet

---

## Scope

### In Scope

- [x] Peer Shop page (`/shop` route) — decided on dedicated route
- [x] Auth guard (redirect to `/login` if unauthenticated)
- [x] **Shop Profile Header:**
  - [x] Gradient background (blue → purple radial gradient)
  - [x] Avatar display (shop account image)
  - [x] Username and slug display
  - [x] Biography/description
  - [x] Follow button (reuse existing `toggleUserFollowStatus`)
  - [x] Info/FAQ button → opens FAQ popup
- [ ] **Product Feed:**
  - [x] Fetch shop account's posts via `list_user_posts(username: "Peer_Shop")`
  - [x] Product price badge on each post card (token amount + logo)
  - [ ] Content type indicator (image/video/audio/text icons)
  - [ ] Post card click → View Post overlay (reuse existing)
  - [ ] Infinite scroll pagination (currently fetches 40 posts)
  - [ ] Content type filters (left sidebar — static/non-functional)
  - [ ] Sort controls (newest, oldest, etc.)
- [x] **Buy Button:**
  - [x] "Buy" button on shop product cards
  - [x] Opens Checkout Popup
- [x] **Checkout Popup (multi-step overlay):**
  - [x] **Step 1 — Product & Delivery Form:**
    - [x] Product preview (images, title, description, price)
    - [x] Size selection (radio buttons, out-of-stock disabled) — UI ready, awaiting Firebase data
    - [x] Delivery information section (collapsible "1–3 working days, only within Germany")
    - [x] Delivery form: full name, email, address line 1, address line 2 (optional), city, ZIP code
    - [x] Field validation (name ≥2 chars, valid email, address ≥5 chars, city ≥2 chars, 5-digit ZIP)
    - [x] "Next" button → validates and advances to Step 2
  - [x] **Step 2 — Order Review & Payment:**
    - [x] Product summary with selected size
    - [x] Delivery information review (name, email, address, city, ZIP, country)
    - [x] "Paying to" indicator (`@Peer_Shop`)
    - [x] Amount breakdown: item price, fees (collapsible fee breakdown: Peer Bank, Burns, Inviter)
    - [x] "Back" button → returns to Step 1
    - [x] "Pay" button → executes `performShopOrder` mutation
  - [x] **Success/Failure states:**
    - [x] Order success modal → "View Wallet" action
    - [x] Order failure modal → "Try again" action
    - [x] Loading state during payment processing
- [x] **FAQ Popup:**
  - [x] Order Processing section
  - [x] Delivery Information & Time section (highlighted)
  - [x] Return Policy section
  - [x] Delivery Area section
  - [x] Need Help? section (email link)
  - [x] Close button
- [ ] **API Layer:**
  - [x] `perform_shop_order` server function (`performShopOrder` mutation)
  - [x] `get_shop_order_details` server function (`shopOrderDetails` query)
  - [ ] Firebase Firestore product data fetching (client-side, `#[cfg(feature = "hydrate")]` gated)
- [x] **SCSS Styling:**
  - [x] Shop profile header gradient
  - [x] Product price badge
  - [x] Checkout popup (multi-step form, delivery fields, amount breakdown)
  - [x] FAQ popup
  - [ ] Responsive layout (basic breakpoints in place, needs polish)
- [x] **Right sidebar:**
  - [x] Profile widget
  - [x] Main menu
  - [x] New post button
  - [x] Version widget

### Out of Scope (Future Work)

- Shop admin panel (product management — Firestore console for now)
- Product search/filter by price range
- Order history page (can be viewed in Wallet transaction history)
- Product reviews/ratings
- Multiple delivery countries (currently Germany only)
- Product inventory management UI
- Shopping cart (single-item checkout only)
- Payment with non-token methods (crypto, fiat)
- Push notifications for order status updates

---

## Legacy Implementation Analysis

### Files

| File | Purpose | Lines |
|------|---------|-------|
| `viewPeerShop.php` | Page shell: 3-column layout, profile header, post container, checkout/FAQ popup containers, View Post overlay | ~140 |
| `js/peerShopProducts.js` | Firebase product loading, FAQ popup rendering, full checkout flow (multi-step form, validation, `performShopOrder` mutation, success/failure handling) | 846 |
| `js/viewprofile.js` | View Profile data loading, follow actions (reused by Peer Shop) | ~300 |
| `js/posts.js` | Post card rendering with shop product price badge | ~500 |
| `js/load_posts.js` | Infinite scroll, ad interleaving, filter integration | ~200 |
| `css/viewPeerShop.css` | Shop-specific styles: gradient header, product price badge, checkout popup, Buy button | 733 |
| `css/profile.css` | Base profile styles (shared) | ~400 |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  HEADER: 🏠 Profile                                        │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│  (filters)   │  ┌─────────────────┐    │                   │
│              │  │ SHOP PROFILE    │    │  - Profile widget │
│  - Content   │  │ HEADER          │    │  - Main menu      │
│    Filters   │  │ (gradient bg)   │    │  - New post btn   │
│  - Sort      │  │ Avatar │ Info   │    │  - Version        │
│    Options   │  │ [Follow] [Info] │    │                   │
│              │  └─────────────────┘    │                   │
│              │                         │                   │
│              │  ┌─────────────────┐    │                   │
│              │  │ PRODUCT GRID    │    │                   │
│              │  │ Post cards with │    │                   │
│              │  │ price badges    │    │                   │
│              │  │ (infinite scroll│    │                   │
│              │  │  with filters)  │    │                   │
│              │  └─────────────────┘    │                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### Shop Profile Header (gradient)

```
┌──────────────────────────────────────────────────────────┐
│  ▓▓▓▓▓▓▓▓▓▓ Radial Gradient (#00BEFF → #553CFF) ▓▓▓▓▓▓ │
│  ┌─────────┐                                             │
│  │         │   Username                                  │
│  │  Avatar │   #12445 (slug)                             │
│  │         │                                             │
│  │         │   Bio description                           │
│  └─────────┘                                             │
│                                                          │
│  [Follow]  [Info ⓘ]                                      │
└──────────────────────────────────────────────────────────┘
```

### Product Card (post with price badge)

```
┌───────────────────────────────┐
│  ┌───────────────────────────┐│
│  │                           ││
│  │   Product Image/Media     ││
│  │                           ││
│  │              ┌────────┐   ││
│  │              │ 💰 250 │   ││  ← Price badge (token amount + logo)
│  │              └────────┘   ││
│  └───────────────────────────┘│
│  Product Title                │
│  Short description...         │
│  ♡ 12  💬 3  👁 45            │
└───────────────────────────────┘
```

### Checkout Popup — Step 1

```
┌─────────────────────────────────────┐
│  Shipping                     ✕     │
├─────────────────────────────────────┤
│  ┌────┐  Product Title              │
│  │ img│  Description text...        │
│  └────┘  Price  250 💰              │
├─────────────────────────────────────┤
│  Select size                        │
│  [ S ] [ M ] [̶ ̶L̶ ̶] [ XL ]          │  ← out-of-stock crossed out
├─────────────────────────────────────┤
│  ▼ Delivery information             │
│  1-3 working days, only in Germany  │
│  (collapsible detail text)          │  
├─────────────────────────────────────┤
│  ┌─────────────────────────────────┐│
│  │ Full name                       ││
│  │ Email address                   ││
│  │ Address line 1                  ││
│  │ Address line 2 (optional)       ││
│  │ [ City        ] [ ZIP   ]      ││
│  └─────────────────────────────────┘│
├─────────────────────────────────────┤
│  [← Back]                  [Next →] │
└─────────────────────────────────────┘
```

### Checkout Popup — Step 2

```
┌─────────────────────────────────────┐
│  Shipping                     ✕     │
├─────────────────────────────────────┤
│  ┌────┐  Product Title              │
│  │ img│  Description text...        │
│  └────┘  Price 250 💰  Size: M     │
├─────────────────────────────────────┤
│  Delivery information               │
│  Name:      Max Mustermann          │
│  E-mail:    max@example.de          │
│  Address 1: Musterstr. 42           │
│  City:      Berlin                  │
│  ZIP:       10115                   │
│  Country:   Germany                 │
├─────────────────────────────────────┤
│  Paying to  @Peer_Shop #12445      │
├─────────────────────────────────────┤
│  Total amount              250.00   │
│    Item Price              250.00   │
│  ▼ Fees included            12.50   │
│    Peer Bank                 5.00   │
│    Burns                     5.00   │
│    Inviter                   2.50   │
├─────────────────────────────────────┤
│  [← Back]                  [Pay →]  │
└─────────────────────────────────────┘
```

### Key Behaviours

1. **Shop Detection**
   - Legacy: `viewPeerShop.php` sets `data-peershop=true` on the `#profile` container; `viewprofile.js` checks this flag to conditionally render shop-specific UI (price badges, buy button)
   - New: Either a dedicated `/shop` route OR detect the Peer Shop account when navigating to its profile via `/profile/:slug`

2. **Product Data (Firebase Firestore)**
   - Products are loaded from Firestore `shop` collection on page load
   - Each product is keyed by Post ID and contains: `sizes` (object mapping size→stock), `one_size_stock` (for single-size items), and additional metadata
   - Shop products are enriched when post cards render: if `peerShopProducts[post.id]` exists, the post is a product
   - This is **client-side only** — Firestore SDK runs in browser

3. **Checkout Flow**
   - Triggered by "Buy" button on a product post in the View Post overlay
   - Step 1: Product preview + size selection (from Firebase) + delivery form with validation
   - Step 2: Review all details + fee breakdown + "Pay" button
   - Pay → calls `performShopOrder` GraphQL mutation
   - Success → success modal → navigate to wallet
   - Failure → error modal → retry

4. **Fee Breakdown Calculation** (client-side, matching `getCommissionBreakdown()` from legacy)
   - Uses the same commission logic as the Wallet transfer_modal
   - Fees: Peer Bank cut, Burns cut, Inviter cut
   - Total = Item price (fees are included, not added on top)

5. **FAQ Popup**
   - Accessed via "Info" button on the shop profile header
   - Static content: 5 sections (Order Processing, Delivery Info, Return Policy, Delivery Area, Need Help)
   - Modal overlay with close button

---

## Architecture

### New Files

| File | Purpose | Est. Lines |
|------|---------|------------|
| `src/pages/peer_shop.rs` | Peer Shop page component (3-column layout, profile header, product feed) | ~200 |
| `src/components/shop/mod.rs` | Shop module barrel | ~20 |
| `src/components/shop/checkout_popup.rs` | Multi-step checkout overlay (product preview, size selection, delivery form, review, payment) | ~500 |
| `src/components/shop/faq_popup.rs` | FAQ modal with shop policies | ~120 |
| `src/components/shop/product_price_badge.rs` | Price badge component for post cards | ~30 |
| `src/components/shop/shop_profile_header.rs` | Shop-specific gradient profile header with Info button | ~80 |
| `src/api/shop.rs` | Server functions: `perform_shop_order`, `get_shop_order_details` | ~120 |
| `src/models/shop.rs` | Shop-related types: `ShopOrderDetails`, `ShopOrderInput`, `ShopProduct` | ~80 |
| `style/peer-shop.scss` | Shop-specific styles (gradient header, price badge, checkout popup, FAQ) | ~400 |

### Modified Files

| File | Change |
|------|--------|
| `src/app.rs` | Add `/shop` route |
| `src/pages/mod.rs` | Add `pub mod peer_shop;` + re-export |
| `src/components/mod.rs` | Add `pub mod shop;` |
| `src/api/mod.rs` | Add `pub mod shop;` |
| `src/models/mod.rs` | Add `pub mod shop;` |
| `style/main.scss` | Import `peer-shop.scss` |
| `src/api/graphql.rs` | Add `PERFORM_SHOP_ORDER_MUTATION`, `SHOP_ORDER_DETAILS_QUERY`, response types |
| `src/components/posts/post_card.rs` | Conditionally render price badge when `is_shop_product` flag is set |

### Component Hierarchy

```
PeerShopPage
├── AuthGuard
│   ├── PageHeader (title: "Profile")
│   ├── LeftSidebar
│   │   ├── FiltersSidebar (content type filters)
│   │   └── SortFilter
│   ├── Main
│   │   ├── ShopProfileHeader (gradient, avatar, bio, [Follow], [Info])
│   │   └── ProductFeed
│   │       └── PostCard (with ProductPriceBadge)
│   │           └── ViewPostOverlay (on click)
│   │               └── BuyButton → CheckoutPopup
│   ├── RightSidebar (ProfileWidget, MainMenu, NewPostBtn, VersionWidget)
│   ├── FaqPopup (opened by Info button)
│   └── CheckoutPopup
│       ├── Step1: ProductPreview + SizeSelector + DeliveryForm
│       └── Step2: OrderReview + FeeBreakdown + PayButton
└── MobileFooter
```

---

## Data Flow

### Product Loading

```
1. Page mounts → fetch shop profile via get_profile(SHOP_USER_ID)
2. Page mounts → fetch shop posts via list_user_posts(SHOP_USER_ID, ...)
3. Client-side → load Firebase Firestore `shop` collection
4. For each post, check if Firestore has product data → enrich with sizes/stock/price
5. Render product cards with price badges
```

### Checkout Flow

```
1. User clicks "Buy" on product post in View Post overlay
2. CheckoutPopup opens → Step 1 visible
3. User selects size (if applicable) + fills delivery form
4. User clicks "Next" → form validation runs
5. If valid → Step 2 renders with review data
6. User clicks "Pay" → loading state
7. perform_shop_order server fn called:
   - GraphQL mutation: performShopOrder(tokenAmount, shopItemId, orderDetails)
   - On success: response code 12201
   - On failure: error message
8. Success → success modal → "View Wallet" button
9. Failure → error modal → "Try again" button
```

### Firebase Integration (Client-Side Only)

```rust
// In checkout_popup.rs (or a shared hook)
#[cfg(feature = "hydrate")]
mod firebase {
    use wasm_bindgen::prelude::*;
    // Use web_sys or gloo to call Firestore REST API
    // OR use the Firebase JS SDK via wasm-bindgen interop
}
```

**Options for Firebase product data:**
1. **REST API** (preferred for WASM): Call Firestore REST endpoint to fetch `shop` collection documents. No JS SDK dependency.
2. **JS interop**: Bind to already-loaded Firebase JS SDK via `wasm-bindgen`. More complex but matches legacy exactly.
3. **Server-side proxy**: Add a server function that fetches from Firestore on the backend (keeps credentials server-side). Most secure, but adds latency.

**Recommendation:** Option 1 (Firestore REST API) for initial implementation, with product data cached in a Leptos `Resource`. This avoids JS SDK dependencies while keeping product data fresh.

---

## API Layer

### New GraphQL Operations

#### `performShopOrder` Mutation

```graphql
mutation PerformShopOrder(
  $tokenAmount: Decimal!
  $shopItemId: String!
  $orderDetails: ShopOrderDetailsInput!
) {
  performShopOrder(
    tokenAmount: $tokenAmount
    shopItemId: $shopItemId
    orderDetails: $orderDetails
  ) {
    status
    RequestId
    ResponseCode
    ResponseMessage
  }
}
```

#### `shopOrderDetails` Query

```graphql
query ShopOrderDetails($transactionId: String!) {
  shopOrderDetails(transactionId: $transactionId) {
    meta { status ResponseCode ResponseMessage }
    affectedRows {
      shopOrderId
      shopItemId
      shopItemSpecs { size }
      deliveryDetails {
        name email addressline1 addressline2
        city zipcode country
      }
      createdat
    }
  }
}
```

### Server Functions

```rust
// src/api/shop.rs

#[server(PerformShopOrder, "/api")]
pub async fn perform_shop_order(
    token_amount: String,      // Decimal as string
    shop_item_id: String,
    name: String,
    email: String,
    address_line_1: String,
    address_line_2: Option<String>,
    city: String,
    zipcode: String,
    size: Option<String>,
) -> Result<(), ServerFnError> { ... }

#[server(GetShopOrderDetails, "/api")]
pub async fn get_shop_order_details(
    transaction_id: String,
) -> Result<ShopOrderDetails, ServerFnError> { ... }
```

### Models

```rust
// src/models/shop.rs

/// Product data from Firebase Firestore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopProduct {
    pub post_id: String,
    pub sizes: Option<HashMap<String, u32>>,      // size → stock count
    pub one_size_stock: Option<u32>,               // for single-size items
}

/// Shop order details from the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopOrderDetails {
    pub shop_order_id: String,
    pub shop_item_id: String,
    pub shop_item_specs: Option<ShopItemSpecs>,
    pub delivery_details: ShopOrderDeliveryDetails,
    pub createdat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItemSpecs {
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOrderDeliveryDetails {
    pub name: String,
    pub email: String,
    pub addressline1: String,
    pub addressline2: Option<String>,
    pub city: String,
    pub zipcode: String,
    pub country: String,
}
```

---

## Validation Rules

### Delivery Form (Client + Server)

| Field | Constraint | Error Message |
|-------|-----------|---------------|
| Full name | ≥ 2 characters | "Name is required" |
| Email | Valid email regex | "Enter a valid email" |
| Address line 1 | ≥ 5 characters | "Address is required" |
| Address line 2 | Optional (no validation) | — |
| City | ≥ 2 characters | "City is required" |
| ZIP code | Exactly 5 digits | "Enter valid ZIP code" |
| Size | Required if product has sizes | "Please select a size" |

Server-side validation in `perform_shop_order` must re-validate all fields before calling the GraphQL mutation (Defense in Depth — OWASP).

---

## Fee Breakdown Logic

Reuse the same commission calculation as the Wallet transfer modal:

```rust
/// Calculate the fee breakdown for a shop purchase.
pub fn get_commission_breakdown(total: f64) -> CommissionBreakdown {
    // Same logic as transfer_modal fee calculation
    // Fees are INCLUDED in the item price, not added on top
    // Breakdown: Peer Bank %, Burns %, Inviter %
}
```

This should be extracted into a shared utility (e.g., `src/utils/fees.rs`) if not already shared with the Wallet components.

---

## Styling Notes

### Shop Profile Header Gradient

```scss
.view-peer-shop .profile_header {
  background: #0069FF;
  background-image: radial-gradient(circle at center, #00BEFF 0%, #553CFF 100%);
  border-radius: 70px;
  padding: 50px;
}
```

### Product Price Badge

```scss
.product_price {
  border-radius: 100px;
  background: var(--dark-inactive, #323232);
  display: flex;
  height: 72px;
  padding: 0 40px;
  justify-content: center;
  align-items: center;
  gap: 10px;
  
  &::after {
    content: "";
    background-image: url(/svg/logo_sw.svg);
    background-size: cover;
    width: 40px;
    height: 40px;
  }
}
```

### Checkout Popup

Key layout: fixed overlay with centered popup, scrollable form area, sticky action bar at bottom.

---

## Implementation Order

### Phase 1: Core Page & Profile Header ✅ Complete

1. ✅ Created `src/pages/peer_shop.rs` with 3-column layout
2. ✅ Registered route in `app.rs` and `pages/mod.rs`
3. ✅ Created `ShopProfileHeader` component (gradient, avatar, bio, buttons)
4. ✅ Fetching shop profile via `get_profile(Some("Peer_Shop"), None)` — uses username constant
5. ✅ SCSS for gradient header
6. ✅ Follow button wired up

### Phase 2: Product Feed & Price Badges ✅ Mostly Complete

1. ✅ Fetch shop posts via `list_user_posts`
2. ✅ Created `ProductPriceBadge` component
3. ✅ Render post cards with price badges
4. ⬜ Infinite scroll pagination (currently fetches 40 posts)
5. ⬜ Left sidebar filters (static/non-functional — UI present but not wired)

### Phase 3: FAQ Popup ✅ Complete

1. ✅ Created `FaqPopup` component with 5 sections
2. ✅ Wired to Info button on ShopProfileHeader
3. ✅ Styled the modal overlay

### Phase 4: Checkout Flow ✅ Complete

1. ✅ Created `CheckoutPopup` component with step 1/step 2 views
2. ✅ Implemented delivery form with validation (client + server-side)
3. ✅ Implemented size selection UI (awaiting Firebase data for real sizes)
4. ✅ Implemented order review (step 2)
5. ✅ Implemented fee breakdown calculation (reuses `calculate_fees` from models/transaction.rs)
6. ✅ Created `src/api/shop.rs` with `perform_shop_order` server function
7. ✅ Added `PERFORM_SHOP_ORDER_MUTATION` and response types to `graphql.rs`
8. ✅ Created `src/models/shop.rs`
9. ✅ Wired payment flow (loading → success/failure → modal feedback)
10. ✅ SCSS for checkout overlay

### Phase 5: Firebase Product Integration ⬜ Not Started

1. ⬜ Implement Firestore product data fetching (`#[cfg(feature = "hydrate")]` gated)
2. ⬜ Cache product data in a Leptos `Resource` or signal (keyed on `shop` collection, loaded once at page mount)
3. ⬜ Enrich post cards with product metadata (sizes, stock)
4. ⬜ Disable "Buy" for out-of-stock products
5. ⬜ Update `token_amount` handling to support decimal prices from Firebase (currently `u32`)

**Note:** Currently using `DEFAULT_PRODUCT_PRICE = 500` fallback for all products. Size selection UI is rendered but has no real data. This phase is required for full feature parity.

### Phase 6: Polish & Testing ⬜ Not Started

1. ⬜ Responsive layout polish (basic breakpoints in SCSS, needs mobile testing)
2. ⬜ Error states (network errors, insufficient balance, product unavailable)
3. ✅ Loading skeletons (implemented in peer_shop.rs)
4. ⬜ E2E test: browse shop → select product → checkout → mock payment
5. ⬜ View Post overlay integration (Buy button currently on cards, not in overlay)
6. ⬜ Content type indicators on product cards
7. ⬜ Mock backend support for `performShopOrder`
8. ⬜ Fix hardcoded shop slug `#12445` in checkout review (pass from profile data — see Known Issues #1)
9. ⬜ Add `meta` field to `SHOP_ORDER_DETAILS_QUERY` for error handling (see Known Issues #3)
10. ⬜ Add `autocomplete` attributes to delivery form inputs (see Known Issues #5)
11. ⬜ Right sidebar: replace hardcoded nav with ProfileWidget, MainMenu, NewPostBtn, VersionWidget
12. ⬜ Compiler warning cleanup (26 warnings, mostly unused imports)

---

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| View Profile page | ✅ Done | Reused profile header pattern, post feed pattern |
| View Post overlay | 🟡 ~95% Done | Buy button currently on cards; overlay integration deferred to Phase 6 |
| Wallet API | ✅ Implemented | Fee calculation reused from `models/transaction.rs::calculate_fees()` |
| PostCard component | ✅ Used | Price badge rendered as overlay on PostCard |
| `performShopOrder` mock backend | ❌ Not Started | Not blocking frontend dev (can test against real backend) |
| Firebase SDK / Firestore REST | ❌ Not in codebase | Phase 5 — product metadata (sizes, stock, prices) |

---

## Open Questions

1. **Dedicated route vs. profile slug detection?** ✅ Decided
   - **Decision:** Option A — dedicated `/shop` route
   - Implemented as `StaticSegment("shop")` in `app.rs`

2. **Firebase product data approach?** ⬜ Deferred to Phase 5
   - Firestore REST API vs. JS SDK interop vs. server-side proxy
   - **Recommendation:** Firestore REST API for initial impl, upgrade to server-side proxy later for security
   - Currently using `DEFAULT_PRODUCT_PRICE = 500` as fallback

3. **Shop account username source?** ✅ Decided
   - **Decision:** Hardcoded constant `SHOP_ACCOUNT_USERNAME = "Peer_Shop"` in `peer_shop.rs`
   - Profile fetched via `get_profile(Some("Peer_Shop"), None)` using username lookup

4. **Fee calculation sharing?** ✅ Decided
   - **Decision:** Reusing `calculate_fees()` from `src/models/transaction.rs` directly
   - No extraction needed — function is already in the shared models module

---

## Definition of Done

- [x] `/shop` route renders the Peer Shop page
- [x] Auth guard redirects unauthenticated users to `/login`
- [x] Shop profile header displays with gradient background, avatar, username, bio
- [x] Follow button works (toggles follow state)
- [x] Info button opens FAQ popup with 5 sections
- [x] Product feed displays shop account's posts with price badges
- [ ] Post card click opens View Post overlay
- [ ] "Buy" button in View Post overlay opens Checkout Popup (currently on cards directly)
- [x] Checkout Step 1: product preview, size selection, delivery form with validation
- [x] Checkout Step 2: order review with fee breakdown, "Pay" button
- [x] `performShopOrder` server function calls GraphQL mutation correctly
- [x] Order success → success modal → navigate to wallet
- [x] Order failure → error modal → retry
- [ ] Firebase product data loads sizes/stock and disables out-of-stock options
- [ ] Responsive layout works on mobile/tablet/desktop (basic breakpoints in place)
- [x] Page builds clean on `cargo check` (default features) — 0 errors, warnings only
- [ ] Page builds clean on `--features ssr` (pre-existing base64/multipart errors, not shop-related)
- [ ] No compiler warnings (26 warnings, mostly unused imports — cleanup in Phase 6)
- [x] SCSS imported in `main.scss`
- [ ] Convergence tracker updated: Peer Shop → 🚧 In Progress

---

## Known Issues (from code review 2026-04-14)

Issues identified during review of the Phases 1–4 implementation. All are non-blocking and tracked for Phase 5/6.

### 1. Hardcoded Peer Shop slug in checkout review step

**File:** `src/components/shop/checkout_popup.rs` ("Paying to" section)  
**Issue:** The slug `#12445` is hardcoded in the review step. If the shop account's slug changes, this will be stale.  
**Fix:** Pass the profile's `slug` field into `CheckoutPopup` as a prop.

### 2. `token_amount` is integer-only

**File:** `src/components/shop/checkout_popup.rs`  
**Issue:** `price_display` is `format!("{}", product_price)` where `product_price` is `u32`. Once Firebase provides real prices (which may be decimal), this will truncate.  
**Fix:** Change `product_price` to `Decimal` or `f64` when Firebase integration lands in Phase 5.

### 3. `shopOrderDetails` query missing `meta` field

**File:** `src/api/graphql.rs` (`SHOP_ORDER_DETAILS_QUERY`)  
**Issue:** The query only requests `affectedRows` but not `meta { status ResponseCode ResponseMessage }`. Error responses from the backend won't have status metadata.  
**Fix:** Add `meta { status ResponseCode ResponseMessage }` to the query and update the response type.

### 4. Legacy GraphQL injection vs. new parameterised approach ✅

**Legacy:** `js/peerShopProducts.js` builds GraphQL via string interpolation (injection risk).  
**New:** `src/api/graphql.rs` uses parameterised variables correctly. **No action needed — this is an improvement.**

### 5. Missing `autocomplete` attributes on delivery form

**File:** `src/components/shop/checkout_popup.rs`  
**Issue:** Form inputs lack `autocomplete` attributes (`name`, `email`, `address-line1`, `postal-code`, etc.). Browser autofill won't work optimally.  
**Fix:** Add appropriate `autocomplete` attributes to each input.

### 6. Simplified right sidebar

**File:** `src/pages/peer_shop.rs` (`ShopRightSidebar`)  
**Issue:** Plan specifies ProfileWidget + MainMenu + NewPostBtn + VersionWidget, but implementation is a simple hardcoded nav.  
**Fix:** Replace with the shared sidebar components once they are extracted/available.
