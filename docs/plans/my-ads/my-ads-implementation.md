# My Ads Implementation Plan

**Feature:** My Ads  
**Priority:** #10 (after Settings)  
**Status:** 🟡 In Progress (Phases 1–4 done, Phase 3 partially wired, Phase 5 not started)  
**Created:** 2026-04-14  
**Updated:** 2026-04-14

---

## Overview

Implement the My Ads page for the Leptos frontend — a dedicated dashboard where users view their advertisement history, aggregated campaign statistics, and per-ad performance breakdowns. The page also provides the "Boost Post" flow (accessible from the profile page) that lets users promote posts via the Pinned (`200 tokens`) or Basic (`50 tokens/day × duration`) ad plans.

### Goals

1. Full parity with legacy `myAds.php` + `pinnedPost.js` user experience
2. Aggregated statistics header (earnings, spendings, interactions)
3. Paginated ad listing with infinite scroll, sorted active-first
4. Expandable per-ad detail panel (campaign details, per-ad interactions)
5. Boost Post modal (multi-step: select post → preview → eligibility check → pay)
6. Visibility-aware UI (hidden/illegal post badges)
7. Auth-guarded page
8. Responsive layout matching the three-column site pattern

---

## Scope

### In Scope

- [x] My Ads page (`/my-ads` route)
- [x] Auth guard (redirect to `/login` if unauthenticated)
- [x] **Stats Header:**
  - [x] Earnings section — gems earned (with gem icon)
  - [x] Spendings section — tokens spent (with logo icon)
  - [x] Interactions section — likes, dislikes, comments, views, reports (with icons + dividers)
- [x] **Ad Listing:**
  - [x] Ad card: post thumbnail/placeholder, title, description, timeframe (start → end), status badge (Active / Ended), pinned badge
  - [x] Click-to-expand detail panel per ad: per-ad earnings, interactions, campaign dates, total ad cost
  - [x] Active-first sorting (active ads above ended ads)
  - [x] Infinite scroll (20 per batch, IntersectionObserver)
  - [x] Content type icon overlay on thumbnails (text/image/video/audio)
  - [x] Hidden post badge (eye icon + warning text)
  - [x] Illegal post frame (removed-as-illegal overlay)
- [x] **Empty State:**
  - [x] "You haven't promoted any posts yet" message + CTA link to profile
- [ ] **Boost Post Modal** (opened from Profile page → Ads dropdown → "Boost post"):
  - [x] Step 1: Warning screen (if post is reported/hidden) — promote anyway / cancel
  - [x] Step 2: Post preview with pinned badge
  - [x] Step 3: Eligibility check — balance check, "Pay" or "Go to profile" button
  - [x] `advertisePostPinned` mutation call on confirm
  - [ ] Success / Error feedback ⚠️ Toast created but not used on error path
  - [ ] Wire modal to Profile page "Boost post" button (task 3.2)
- [x] **API Layer:**
  - [x] `advertisementHistory` query (with filter, sort, pagination)
  - [x] `advertisePostPinned` mutation
  - [ ] (Future) `advertisePostBasic` mutation
- [x] Loading states and skeleton screens
- [x] Staggered entry animation (fade-in with translateY offset)

### Out of Scope (Future Work)

- Basic (time-based) ad creation flow — multi-step with date picker + duration selector
- Ad analytics charts / trends
- Ad re-promotion or extension
- Bulk ad management
- Ad moderation / appeal flow
- Real-time ad timer countdown (active ads showing remaining time)

---

## Legacy Implementation Analysis

### Files

| File | Lines | Purpose |
|------|-------|---------|
| `myAds.php` | 144 | Page template: three-column layout, stats header, ad list container, sidebar widgets |
| `js/ads/AdsHistory/myAds.js` | 625 | Full ad history: GraphQL fetch, infinite scroll, ad card rendering, stats population, visibility badges |
| `js/ads/pinnedPost/pinnedPost.js` | 645 | Boost Post modal: post selection, eligibility check, preview, `advertisePostPinned` mutation, pinned badge insertion |
| `css/myAds.css` | 682 | Stagger animations, stats header grid, ad cards, dropdown panels, responsive breakpoints |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  HEADER: Logo + "Dashboard" title                          │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│  (filters)   │  h1 "My Ads"            │                   │
│              │  ┌─────────────────────┐ │  - Daily action   │
│  - Content   │  │  STATS HEADER       │ │  - Main menu      │
│    filter    │  │  Earnings | Spend.  │ │  - New post btn   │
│  - Sort      │  │  | Interactions     │ │  - Version        │
│  - Collapse  │  └─────────────────────┘ │                   │
│  - Back btn  │                         │                   │
│              │  h2 "All advertisements" │                   │
│              │  Total: N               │                   │
│              │  ┌─────────────────────┐ │                   │
│              │  │  AD CARD (active)   │ │                   │
│              │  │  [img] title  dates │ │                   │
│              │  │        desc   badge │ │                   │
│              │  │  ── expand panel ── │ │                   │
│              │  └─────────────────────┘ │                   │
│              │  ┌─────────────────────┐ │                   │
│              │  │  AD CARD (ended)    │ │                   │
│              │  └─────────────────────┘ │                   │
│              │  ... infinite scroll ... │                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### Key Behaviours

1. **Data Fetching** — `advertisementHistory` GraphQL query with `filter: { userId }`, `sort: NEWEST`, `offset`, `limit: 20`. Stats populated from `affectedRows.stats`, ads from `affectedRows.advertisements`.

2. **Active-First Sort** — After fetching, active ads (`enddate > now`) are sorted above ended ads. Within each group, server-side `NEWEST` ordering is preserved.

3. **Infinite Scroll** — IntersectionObserver on a sentinel div at the bottom of the ad list. Loads next batch when visible. Removes sentinel when `offset >= stats.amountAds`.

4. **Ad Card Expand** — Click toggles dropdown panel showing per-ad stats (gems, likes, dislikes, comments, views, reports) and campaign details (start date, end date, total cost).

5. **Visibility Handling** — Posts with `visibilityStatus === 'HIDDEN'` get a hidden badge (eye-close icon). Posts with `visibilityStatus === 'ILLEGAL'` get an illegal overlay frame replacing the title/description. Hidden posts from dropdown show warning text.

6. **Number Formatting** — `formatNumber(num)`: ≥1M → `1.2M`, ≥1K → `1.2K`, else raw.

7. **Stagger Animation** — Page fades in (`opacity: 0 → 1`), then each ad card animates in sequence with 100ms delay per card.

8. **Boost Post Flow** (from `pinnedPost.js`):
   - Profile page "Boost post" button calls `selectCardForBoosting(card)`
   - Checks: not illegal → not already boosted → shows warning if reported/hidden → shows preview → eligibility (balance ≥ 200 tokens) → calls `advertisePostPinned` mutation → inserts pinned badge on success

---

## Leptos Component Architecture

```
my_ads/
├── mod.rs               # Module exports
├── page.rs              # MyAdsPage (top-level, auth guard, context)
├── stats_header.rs      # StatsHeader (earnings, spendings, interactions)
├── ad_list.rs           # AdList (infinite scroll, sentinel observer)
├── ad_card.rs           # AdCard (single ad item with expand panel)
├── boost_modal.rs       # BoostPostModal (multi-step: warn → preview → pay)
└── empty_state.rs       # EmptyState (no ads CTA)
```

### Component Breakdown

#### 1. `MyAdsPage` (`page.rs`)

Top-level page component. Provides `MyAdsContext`, fetches initial data, lays out the three-column structure.

```
Route: /my-ads
Auth: Required (AuthGuard)
Context: MyAdsContext { stats, ads, offset, has_more, is_loading }
```

| Signal | Type | Purpose |
|--------|------|---------|
| `stats` | `RwSignal<Option<AdHistoryStats>>` | Aggregated stats from API |
| `ads` | `RwSignal<Vec<Advertisement>>` | Loaded ad records |
| `offset` | `RwSignal<i32>` | Pagination cursor |
| `has_more` | `RwSignal<bool>` | Whether more pages exist |
| `is_loading` | `RwSignal<bool>` | Loading guard for infinite scroll |

#### 2. `StatsHeader` (`stats_header.rs`)

Displays aggregated campaign statistics.

| Section | Fields | Icon |
|---------|--------|------|
| Earnings | `gems_earned` | `peer-icon-gems.svg` |
| Spendings | `tokens_spent` | `logo_sw.svg` |
| Interactions | likes, dislikes, comments, views, reports | `peer-icon-like`, `peer-icon-dislike`, `peer-icon-comment-alt`, `peer-icon-eye-open`, `peer-icon-warning` |

The header reads from `MyAdsContext::stats`. Renders a skeleton while data is loading.

#### 3. `AdList` (`ad_list.rs`)

Manages the infinite-scroll ad listing.

- Reads `MyAdsContext::ads` signal
- Sorts ads: active first, then ended (preserving creation order within groups)
- Renders `<For each=ads>` → `<AdCard>`
- Inserts sentinel `<div>` observed by `create_effect` + `IntersectionObserver` (via `wasm_bindgen` / `web_sys`)
- Calls `load_more_ads()` when sentinel is visible and `!is_loading && has_more`

#### 4. `AdCard` (`ad_card.rs`)

Single ad item with expandable detail panel.

| Prop | Type | Notes |
|------|------|-------|
| `ad` | `Advertisement` | Individual ad record |

**Collapsed state:**
- Post thumbnail (image from `post.cover` or `post.media`, fallback for text/audio)
- Content type icon badge (image/video/audio/text)
- Pinned badge (if `ad.type == PINNED`)
- Title + description
- Timeframe box (start date/time → end date/time)
- Status badge (`Active` green / `Ended` grey)
- Visibility badges (hidden eye icon, illegal overlay frame)

**Expanded state (toggle on click):**
- Per-ad stats: gems earned, likes, dislikes, comments, views, reports
- Campaign details: start date, end date, total token cost

Uses `RwSignal<bool>` for `expanded` toggle.

#### 5. `BoostPostModal` (`boost_modal.rs`)

Multi-step modal for promoting a post. Opened from the Profile page's "Boost post" button.

| Step | Content | Actions |
|------|---------|---------|
| 1 | Warning (if reported/hidden): "Your post was reported…" | "Promote anyway" / Cancel |
| 2 | Post preview with pinned badge overlay | Next → |
| 3 | Eligibility check: balance check, fee display (200 tokens for Pinned) | Pay / Go to profile (if insufficient balance) |
| 4 | (Success/Error feedback via Toast) | Auto-close |

**State:**
- `current_step: RwSignal<u8>` — steps 1–3
- `post_id: RwSignal<Option<String>>` — selected post to boost
- `balance: RwSignal<Decimal>` — user's token balance (fetched from wallet API)

**Communication:**
- Profile page calls `BoostPostModal` with a `post_id` prop
- On success, inserts pinned badge class on the post card (via callback)

#### 6. `EmptyState` (`empty_state.rs`)

Shown when `ads.is_empty() && !is_loading`.

```html
<div class="empty-state-container">
  <p>"You haven't promoted any posts yet. Start your first promotion to see statistics"</p>
  <a href="/profile" class="button btn-white">"Take me to my posts"</a>
</div>
```

---

## API Layer

### New File: `src/api/ads.rs`

| Server Function | GraphQL Operation | Auth | Purpose |
|-----------------|-------------------|------|---------|
| `get_ad_history(filter, sort, offset, limit)` | `advertisementHistory` query | Yes | Fetch paginated ad history with stats |
| `advertise_post_pinned(post_id)` | `advertisePostPinned` mutation | Yes | Create a pinned advertisement |

### New Models: `src/models/advertisement.rs`

```rust
/// Aggregated stats for all user advertisements.
pub struct AdHistoryStats {
    pub token_spent: f64,
    pub euro_spent: f64,
    pub amount_ads: i32,
    pub gems_earned: f64,
    pub amount_likes: i32,
    pub amount_views: i32,
    pub amount_comments: i32,
    pub amount_dislikes: i32,
    pub amount_reports: i32,
}

/// Individual advertisement record.
pub struct Advertisement {
    pub id: String,
    pub created_at: String,
    pub ad_type: AdvertisementType,  // PINNED or BASIC
    pub timeframe_start: String,
    pub timeframe_end: String,
    pub total_token_cost: f64,
    pub total_euro_cost: f64,
    pub gems_earned: f64,
    pub amount_likes: i32,
    pub amount_views: i32,
    pub amount_comments: i32,
    pub amount_dislikes: i32,
    pub amount_reports: i32,
    pub user: ProfileUser,
    pub post: Post,
}

/// Ad history API response (wraps stats + list).
pub struct AdHistoryResponse {
    pub meta: DefaultResponse,
    pub affected_rows: AdHistoryResult,
}

pub struct AdHistoryResult {
    pub stats: AdHistoryStats,
    pub advertisements: Vec<Advertisement>,
}

pub enum AdvertisementType {
    Pinned,
    Basic,
}
```

### GraphQL Queries (to add to `src/api/graphql.rs`)

```graphql
query AdvertisementHistory($filter: AdvertisementHistoryFilter, $sort: AdvertisementSort, $offset: Int, $limit: Int) {
  advertisementHistory(filter: $filter, sort: $sort, offset: $offset, limit: $limit) {
    status
    ResponseCode
    meta { status ResponseCode ResponseMessage }
    affectedRows {
      stats {
        tokenSpent euroSpent amountAds gemsEarned
        amountLikes amountViews amountComments amountDislikes amountReports
      }
      advertisements {
        id createdAt type timeframeStart timeframeEnd
        totalTokenCost totalEuroCost gemsEarned
        amountLikes amountViews amountComments amountDislikes amountReports
        post { id contenttype title media cover mediadescription visibilityStatus isHiddenForUsers hasActiveReports isreported }
        user { id img }
      }
    }
  }
}
```

```graphql
mutation AdvertisePostPinned($postid: ID!, $advertisePlan: AdvertisementPinnedPlan!) {
  advertisePostPinned(postid: $postid, advertisePlan: $advertisePlan) {
    status
    ResponseCode
    meta { status ResponseCode ResponseMessage }
    affectedRows {
      id createdAt type timeframeStart timeframeEnd totalTokenCost totalEuroCost
    }
  }
}
```

---

## SCSS Styling

### New File: `style/my-ads.scss`

Approximate structure mirroring `css/myAds.css` (682 lines):

| Section | Description |
|---------|-------------|
| `.site-main-my-ads` | Main content area with fade-in transition |
| `.my-ads-header` | Flex row: earnings / spendings / interactions boxes |
| `.header-box` | Stat card with icon + label + value |
| `.interactions-box` | Horizontal stat group with vertical dividers |
| `.my-ads-list` | Ad listing container |
| `.ad-card` | Single ad item with hover/active states |
| `.ad-card.active` | Green status dot + border accent |
| `.ad-card.ended` | Grey status dot + muted styling |
| `.ad-card.pinned` | Pinned badge overlay |
| `.ad-main-info` | Top section: thumbnail + details + timeframe + status |
| `.ad-dropdown` | Expandable detail panel (transform: scaleY) |
| `.ad-dropdown.open` | Panel expanded state |
| `.status-badge` | Pill badge (green=active, grey=ended) |
| `.ad-hidden-badge` | Eye-close icon for hidden post warning |
| `.illegal-ads-post-frame` | Red-tinted illegal content overlay |
| `.empty-state-container` | Centered message + CTA button |
| Animations | `translateY(20px) → 0` stagger on `.ad-card`, 100ms delay per item |
| Responsive | Stack stats vertically below tablet breakpoint |

---

## Implementation Tasks

### Phase 1: Models & API (Estimate: S) — ✅ Done

| # | Task | File(s) | Status | Notes |
|---|------|---------|--------|-------|
| 1.1 | Create `Advertisement` and stats models | `src/models/advertisement.rs`, `src/models/mod.rs` | ✅ | `AdHistoryStats`, `Advertisement`, `AdHistoryResponse`, `AdvertisementType` enum |
| 1.2 | Add `ADVERTISEMENT_HISTORY_QUERY` to graphql module | `src/api/graphql.rs` | ✅ | Query string + response wrapper struct |
| 1.3 | Add `ADVERTISE_POST_PINNED_MUTATION` to graphql module | `src/api/graphql.rs` | ✅ | Mutation string + response wrapper struct |
| 1.4 | Create `src/api/ads.rs` with server fns | `src/api/ads.rs`, `src/api/mod.rs` | ✅ | `get_ad_history()`, `advertise_post_pinned()` |

### Phase 2: Page & Components (Estimate: M) — ✅ Done

| # | Task | File(s) | Status | Notes |
|---|------|---------|--------|-------|
| 2.1 | Create `MyAdsPage` with auth guard and context | `src/pages/my_ads.rs`, `src/pages/mod.rs` | ✅ | Three-column layout with `AuthGuard` |
| 2.2 | Register `/my-ads` route | `src/app.rs` | ✅ | Route entry and import added |
| 2.3 | Create `StatsHeader` component | `src/components/my_ads/stats_header.rs` | ✅ | Skeleton loading, `format_number()` |
| 2.4 | Create `AdCard` component | `src/components/my_ads/ad_card.rs` | ✅ | Collapsed/expanded states, visibility badges, status badge |
| 2.5 | Create `AdList` component with infinite scroll | `src/components/my_ads/ad_list.rs` | ✅ | Active-first sort, IntersectionObserver, sentinel |
| 2.6 | Create `EmptyState` component | `src/components/my_ads/empty_state.rs` | ✅ | No-ads CTA |
| 2.7 | Wire `MyAdsPage` → `StatsHeader` + `AdList`/`EmptyState` | `src/pages/my_ads.rs` | ✅ | Data loading via signals in `AdList` |

### Phase 3: Boost Post Modal (Estimate: M) — 🟡 Partially Done

| # | Task | File(s) | Status | Notes |
|---|------|---------|--------|-------|
| 3.1 | Create `BoostPostModal` component | `src/components/my_ads/boost_modal.rs` | ✅ | Multi-step: warn → preview → eligibility → pay |
| 3.2 | Wire modal from Profile page "Boost post" button | `src/components/profile/actions.rs` | ❌ | `on_boost_posts` callback exists but not connected to modal |
| 3.3 | Add eligibility check (balance fetch) | `boost_modal.rs` | ✅ | Reuses `get_balance()` from wallet API |
| 3.4 | Call `advertise_post_pinned` mutation on confirm | `boost_modal.rs` | ⚠️ | Mutation called; error path silently swallowed (`Err(_) => {}`), toast not used on error |
| 3.5 | Post-promotion badge insertion | `src/components/posts/post_card.rs` | ❌ | Not yet implemented |

### Phase 4: Styling & Polish (Estimate: S) — ✅ Done

| # | Task | File(s) | Status | Notes |
|---|------|---------|--------|-------|
| 4.1 | Create `style/my-ads.scss` | `style/my-ads.scss` | ✅ | 788 lines — stats header, ad cards, expand panels, badges |
| 4.2 | Add responsive breakpoints | `style/my-ads.scss` | ✅ | Stack stats, full-width cards on mobile |
| 4.3 | Stagger animation for ad cards | `style/my-ads.scss` | ✅ | Inline `animation-delay` per card index |
| 4.4 | Import new SCSS in main stylesheet | `style/main.scss` | ✅ | `@import "my-ads"` |

### Phase 5: Testing (Estimate: S) — ❌ Not Started

| # | Task | File(s) | Status | Notes |
|---|------|---------|--------|-------|
| 5.1 | Add ad history fixture data to mock backend | `packages/mock_backend/` | ❌ | Requires Mock Backend Phase 5 (Economy) |
| 5.2 | E2E: page loads, stats header visible | `end2end/` | ❌ | Playwright test |
| 5.3 | E2E: ad card expand/collapse | `end2end/` | ❌ | Click ad → detail panel opens |
| 5.4 | E2E: infinite scroll loads more ads | `end2end/` | ❌ | Scroll to bottom → new batch |
| 5.5 | E2E: empty state shown for user with no ads | `end2end/` | ❌ | Assert CTA link visible |

---

## Dependencies

| Dependency | Status | Impact |
|------------|--------|--------|
| Auth system (JWT server fns) | ✅ Done | Auth guard, API authorization |
| `get_balance()` server fn | ✅ Done | Boost modal eligibility check |
| `Post` & `PostUser` models | ✅ Done | Reused in `Advertisement.post` field |
| Mock Backend Phase 5 (Economy) | ❌ Not Started | Required for E2E tests; page can be built against real API or stubs first |
| `ProfileUser` model | ✅ Done | Reused in `Advertisement.user` field |

---

## Reusable Patterns from Existing Code

| Pattern | Source | Reuse in My Ads |
|---------|--------|-----------------|
| `AuthGuard` wrapper | `src/components/auth_guard.rs` | Page-level auth guard |
| Three-column layout | `dashboard.rs`, `profile.rs`, `wallet.rs` | Page structure |
| `MobileFooter` | `dashboard.rs`, `profile.rs` | Bottom nav bar |
| IntersectionObserver infinite scroll | `src/components/posts/post_list.rs` | Ad list pagination |
| `format_number()` utility | New (port from legacy `formatNumber()`) | Stats display |
| `format_time_ago()` / date formatting | `src/components/view_post/post_content.rs` | Ad dates |
| Toast notifications | `src/components/toast.rs` | Boost success/error feedback |
| Resource + Suspense pattern | Dashboard, Profile, Wallet pages | Data loading |
| Skeleton loading | `src/components/profile/header.rs` | Stats header skeleton |

---

## Known Issues

| # | Issue | Severity | File(s) | Notes |
|---|-------|----------|---------|-------|
| K1 | Boost modal not wired to Profile page | High | `src/components/profile/actions.rs` | `on_boost_posts` callback exists but never opens `BoostPostModal` (task 3.2) |
| K2 | Silent error handling in boost modal | Medium | `src/components/my_ads/boost_modal.rs` | `Err(_) => {}` swallows promotion failures; `_toast` binding created but unused on error |
| K3 | Duplicate `format_number()` utility | Low | `stats_header.rs`, `ad_card.rs` | Same function defined in both files; should be extracted to a shared utils module |
| K4 | Fragile `is_active()` date comparison | Low | `src/models/advertisement.rs` | Compares ISO strings lexicographically; should parse to `DateTime` for robustness |
| K5 | Clippy warnings in My Ads code | Low | `ad_list.rs`, `boost_modal.rs`, `api/ads.rs` | `sort_by` → `sort_by_key`; `match` → `if let`; server-only structs need `#[cfg(feature = "ssr")]` |
| K6 | Pagination `has_more` edge case | Low | `src/components/my_ads/ad_list.rs` | Stats-based check on subsequent pages may incorrectly stop if stats signal is `None`; safer to use `new_ads.len() == ADS_PER_PAGE` |
| K7 | `MobileFooter` is locally duplicated | Low | `src/pages/my_ads.rs` | Same footer exists in other pages; should be a shared component |

---

## Definition of Done

- [x] `/my-ads` route registered and renders behind auth guard
- [x] Stats header displays correct aggregated data
- [x] Ad cards render with correct post info, dates, status badges
- [x] Expand/collapse works for each ad card
- [x] Infinite scroll fetches next page and appends results
- [x] Empty state shown for users with zero ads
- [x] Hidden and illegal post badges render correctly
- [ ] Boost Post modal opens from Profile → Ads → "Boost post"
- [ ] Pinned ad creation completes and shows success toast
- [x] SCSS compiles, responsive at mobile/tablet/desktop
- [ ] No compiler warnings (`cargo clippy`)
