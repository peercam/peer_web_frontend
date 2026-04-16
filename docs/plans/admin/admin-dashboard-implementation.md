# Admin Dashboard Implementation Plan

**Feature:** Admin Dashboard (Content Moderation)  
**Priority:** #14 (after Peer Shop)  
**Status:** Not Started  
**Created:** 2026-04-16  
**Mock Backend:** ✅ Phase 6 complete (266 tests, RBAC + moderation + admin gems)

---

## Overview

Implement the admin/moderation dashboard for the Leptos frontend. This is a role-gated page allowing moderators to review reported content (posts, comments, users) and take action (hide, restore, mark as illegal). The legacy PHP admin panel (`admin/index.php` + `admin/content.php`) comprises ~4,150 lines of JS across 7 modules (`schema.js`, `store.js`, `helpers.js`, `service.js`, `fetcher.js`, `view.js`, `main.js`) plus ~2,240 lines of CSS.

The mock backend already has full support: `moderationStats`, `moderationItems` (with status/contentType filtering), and `performModeration` mutation — all behind `RoleGuard` (MODERATOR bitmask 256).

### Goals

1. Full parity with legacy `admin/index.php` content moderation experience
2. Role-gated route: require MODERATOR (256) or ADMIN (16) role, redirect others
3. Stats header: awaiting review, hidden, restored, illegal counts
4. Filterable ticket list: All / Posts / Comments / Accounts, "waiting for review" toggle
5. Expandable ticket detail: content preview (post/comment/user), reporters list, moderation actions
6. Moderation actions: Restore, Hide, Mark as illegal — with confirmation dialogs
7. Infinite scroll pagination (reuse `use_infinite_scroll` hook)
8. Skeleton loading states and error handling
9. Responsive layout consistent with other Leptos pages

---

## Scope

### In Scope

- [ ] Admin page (`/admin` route) with role-based auth guard
- [ ] Moderation stats header (4 stat boxes with live counts)
- [ ] Content type filter tabs (All, Posts, Comments, Accounts)
- [ ] "Waiting for review" checkbox filter
- [ ] Moderation ticket list with summary row (content preview, moderation ID, date, report count, status)
- [ ] Expandable ticket detail panel:
  - [ ] **Post detail:** author info, post title/description/media, hashtags, "See full post" link
  - [ ] **Comment detail:** commenter info, comment text, linked parent post preview
  - [ ] **User detail:** profile image, username, slug, biography, profile stats, "View profile" link
- [ ] Right-side detail panel: reporters list, action buttons, moderation result display
- [ ] Confirmation dialogs for Hide / Restore / Mark as illegal
- [ ] `performModeration` mutation integration with optimistic UI update
- [ ] Infinite scroll pagination for ticket list
- [ ] Admin header: "Admin" logo, logged-in username, "Back to user mode" link
- [ ] Skeleton loading for stats and ticket list
- [ ] SCSS styles (port from legacy `admin/css/style.css`)

### Out of Scope (Future Phases)

- Admin user search (`listUsersAdminV2`) — separate admin sub-page
- Friendship graph (`allfriends`) — separate admin sub-page
- Admin post comments (`postcomments`) — separate admin sub-page
- Leaderboard generation (`generateLeaderboard`) — separate admin sub-page
- Gem/mint administration (`gemster`, `dailygemstatus`, `globalwins`, etc.) — separate admin sub-page
- Media gallery slider (image/video carousel in post detail) — post card component handles this
- Firebase real-time reporting notifications

---

## Architecture

### Route & Auth

```
/admin → AdminPage
  ├── RoleGuard (MODERATOR | ADMIN)
  ├── AdminHeader
  ├── StatsHeader
  ├── FilterBar (type tabs + review checkbox)
  └── ModerationList
       └── ModerationTicketItem (per ticket)
            ├── TicketSummaryRow (collapsed view)
            └── TicketDetail (expanded view)
                 ├── ContentPreview (left)
                 │    ├── PostPreview
                 │    ├── CommentPreview
                 │    └── UserPreview
                 └── ActionPanel (right)
                      ├── ReportersList
                      ├── ActionButtons (restore/hide/illegal)
                      ├── ConfirmationDialog
                      └── ModeratedByInfo
```

### File Tree (New Files)

```
peer-web/src/
├── api/
│   └── moderation.rs           # NEW — server functions for moderation queries/mutations
├── models/
│   └── moderation.rs           # NEW — ModerationStats, ModerationItem, TargetContent, etc.
├── pages/
│   └── admin.rs                # NEW — AdminPage component
├── components/
│   └── admin/                  # NEW — admin component module
│       ├── mod.rs
│       ├── admin_header.rs     # Admin-specific header bar
│       ├── stats_header.rs     # 4 stat boxes
│       ├── filter_bar.rs       # Type tabs + review checkbox
│       ├── moderation_list.rs  # Ticket list with infinite scroll
│       ├── ticket_item.rs      # Single ticket row + expandable detail
│       ├── content_preview.rs  # Dispatch to post/comment/user preview (left panel)
│       ├── post_preview.rs     # Post detail in expanded ticket
│       ├── comment_preview.rs  # Comment detail in expanded ticket
│       ├── user_preview.rs     # User detail in expanded ticket
│       └── action_panel.rs     # Reporters, action buttons, confirmation (right panel)
└── style/
    └── admin.scss              # NEW — admin-specific styles
```

### Modified Files

```
peer-web/src/
├── api/mod.rs                  # Add `pub mod moderation;`
├── models/mod.rs               # Add `pub mod moderation;`
├── pages/mod.rs                # Add `pub mod admin;`, `pub use admin::AdminPage;`
├── components/mod.rs           # Add `pub mod admin;`
├── app.rs                      # Add `/admin` route
└── style/main.scss             # Import `admin.scss`
```

---

## Phase Breakdown

### Phase 1 — Models & API Layer

**Goal:** Define data models and server functions so components can fetch moderation data.

#### Task 1.1 — Moderation Models (`models/moderation.rs`)

```rust
// Key types to define:

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationStats {
    #[serde(rename = "AmountAwaitingReview")]
    pub amount_awaiting_review: i32,
    #[serde(rename = "AmountHidden")]
    pub amount_hidden: i32,
    #[serde(rename = "AmountRestored")]
    pub amount_restored: i32,
    #[serde(rename = "AmountIllegal")]
    pub amount_illegal: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationStatsResponse {
    pub status: String,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<ModerationStats>,
    pub meta: Option<DefaultResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationItem {
    #[serde(rename = "moderationTicketId")]
    pub moderation_ticket_id: String,
    #[serde(rename = "targetContentId")]
    pub target_content_id: String,
    pub targettype: String,               // "post", "comment", "user"
    pub reportscount: i32,
    pub status: String,                    // "waiting for review", "hidden", "restored", "illegal"
    pub createdat: String,
    pub targetcontent: TargetContent,
    pub reporters: Vec<BasicReporterInfo>,
    #[serde(rename = "moderatedBy")]
    pub moderated_by: Option<BasicReporterInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetContent {
    pub post: Option<ModerationPost>,
    pub comment: Option<ModerationComment>,
    pub user: Option<ModerationUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicReporterInfo {
    pub userid: Option<String>,
    pub img: Option<String>,
    pub username: Option<String>,
    pub slug: Option<String>,
    pub updatedat: Option<String>,
}

/// Valid moderation actions (typed instead of raw strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationAction {
    Hidden,
    Restored,
    Illegal,
}

// + ModerationPost, ModerationComment, ModerationUser structs
// matching the GraphQL field selections in admin/js/schema.js
```

**Acceptance:** Types compile, serde round-trips match mock backend responses.

#### Task 1.2 — GraphQL Queries (`api/graphql.rs` additions)

Add the following query/mutation constants. These are **new queries** — not direct ports of the legacy `admin/js/schema.js` constants. The legacy `LIST_ITEMS` takes only `$offset`/`$limit`; our version adds `$contentType` and `$status` variables to push filtering to the backend (already supported by the mock backend resolver).

| Constant | GraphQL Operation | Mock Backend Support |
|----------|-------------------|-----------------------|
| `MODERATION_STATS_QUERY` | `query { moderationStats { ... } }` | `tests/mock_backend/tests/moderation.rs` |
| `MODERATION_ITEMS_QUERY` | `query($offset: Int!, $limit: Int!, $contentType: ModerationContentType, $status: ModerationStatus) { moderationItems(offset: $offset, limit: $limit, contentType: $contentType, status: $status) { ... } }` | `tests/mock_backend/tests/moderation.rs` |
| `PERFORM_MODERATION_MUTATION` | `mutation($moderationTicketId: ID!, $moderationAction: ModerationStatus!) { performModeration(moderationTicketId: $moderationTicketId, moderationAction: $moderationAction) { ... } }` | `tests/mock_backend/tests/moderation.rs` |

**Note:** The legacy panel has separate queries for `LIST_POST`, `LIST_COMMENT`, `LIST_USER` with narrower field selections. The Leptos version uses a single `MODERATION_ITEMS_QUERY` with the full field set and passes `contentType` + `status` as variables. This simplifies the API layer (1 query vs 4).

#### Task 1.3 — Server Functions (`api/moderation.rs`)

```rust
/// Fetch moderation stats (ticket counts by status).
#[server(GetModerationStats, "/api")]
pub async fn get_moderation_stats() -> Result<ModerationStatsResponse, ServerFnError>

/// Fetch moderation tickets with optional filters.
#[server(GetModerationItems, "/api")]
pub async fn get_moderation_items(
    content_type: Option<String>,    // "post", "comment", "user", or None for all
    status: Option<String>,          // "waiting_for_review", "hidden", "restored", "illegal", or None for all
    offset: i32,
    limit: i32,
) -> Result<ModerationItemListResponse, ServerFnError>

/// Perform a moderation action on a ticket.
/// Uses the actual GraphQL argument names: `moderationTicketId` and `moderationAction`.
#[server(PerformModeration, "/api")]
pub async fn perform_moderation(
    moderation_ticket_id: String,
    moderation_action: ModerationAction,  // typed enum: Hidden, Restored, Illegal
) -> Result<DefaultResponse, ServerFnError>
```

Each server function:
1. Extracts the access token from HttpOnly cookies via `get_access_token_from_cookies()`
2. Calls the GraphQL backend via `query()` / `mutate()`
3. Returns the typed response or `ServerFnError`

**Acceptance:** All 3 server functions compile and work against the mock backend.

---

### Phase 2 — Admin Page Shell & Route

**Goal:** Create the page component, register the route, and implement role-based access control.

#### Task 2.1 — Role Guard Component

The existing `AuthGuard` only checks whether a user is logged in. The admin page needs an additional role check. Two approaches:

**Option A (recommended):** Create a `RoleGuard` wrapper component:
```rust
#[component]
pub fn RoleGuard(
    #[prop(into)] required_role: u32,
    children: Children,
) -> impl IntoView
```

This component:
1. Reads `AuthContext` to get the current user
2. Checks `roles_mask` from the auth context (see Task 2.2)
3. If `roles_mask & required_role != 0`, renders children
4. Otherwise, redirects to `/dashboard` with a toast: "You do not have permission to access this page"

**Option B:** Do the role check inside `get_moderation_stats` / `get_moderation_items` and handle the 62101 (not authorized) error code on the client side, showing an access denied view.

**Decision:** Use **Option A** for UX (redirect immediately rather than showing a broken page) + **Option B** as a safety net (server functions still validate roles).

#### Task 2.2 — Add `roles_mask` to Auth Context

The existing `AuthContext` (`src/state/auth.rs`) does not currently track user roles. Rather than adding a separate server function + extra round-trip, extend the existing login/session flow:

1. Add `roles_mask: RwSignal<u32>` to `AuthContext` (default `0`)
2. When the login/profile-fetch server function returns user data, populate `roles_mask` from the response
3. The `RoleGuard` component reads `roles_mask` directly from `AuthContext` — no additional API call needed

If the existing profile response doesn't include `roles_mask`, add it to the profile GraphQL query. The mock backend already returns it via the user model.

**Fallback:** If extending the auth context is blocked (e.g., the real backend doesn't expose `roles_mask` on the profile query), use a dedicated server function:
```rust
#[server(CheckUserRole, "/api")]
pub async fn check_user_role() -> Result<u32, ServerFnError>
```
Cache the result in a signal so it's only fetched once per session.

#### Task 2.3 — Admin Page Component (`pages/admin.rs`)

```rust
#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Title text="Admin - Peer Network"/>
        <AuthGuard>
            <RoleGuard required_role=256>  // MODERATOR
                <div id="admin-page" class="site_layout admin-layout">
                    <AdminHeader/>
                    <main class="site-main site-main-admin">
                        <h2 class="page-title">"Content moderation"</h2>
                        <StatsHeader/>
                        <FilterBar/>
                        <ModerationList/>
                    </main>
                </div>
            </RoleGuard>
        </AuthGuard>
    }
}
```

#### Task 2.4 — Route Registration (`app.rs`)

Add to the router:
```rust
<Route path=StaticSegment("admin") view=AdminPage/>
```

**Acceptance:** Navigating to `/admin` as a non-moderator redirects to `/dashboard`. As a moderator, shows the page shell with the "Content moderation" heading.

---

### Phase 3 — Stats Header

**Goal:** Display the four stat boxes with live counts from the moderation backend.

#### Task 3.1 — StatsHeader Component (`components/admin/stats_header.rs`)

```rust
#[component]
pub fn StatsHeader() -> impl IntoView
```

- Calls `get_moderation_stats` as a resource on mount
- Renders 4 stat boxes in a `.main_stats` grid:
  - **Awaiting Review** (yellow warning icon) — `AmountAwaitingReview`
  - **Hidden** (red eye-close icon) — `AmountHidden`
  - **Restored** (green check icon) — `AmountRestored`
  - **Illegal** (red cross icon) — `AmountIllegal`
- Skeleton loading state while resource is pending
- Error fallback if fetch fails

**Legacy reference:** `admin/index.php` lines 23–52 (stat box HTML), `admin/js/view.js` `renderStats()`, `admin/js/fetcher.js` `loadStats()`.

**Stats refresh:** Accepts a `stats_version: ReadSignal<u32>` prop. The resource depends on this signal so it re-fetches whenever a moderation action completes (see Phase 5, Task 5.3).

**Acceptance:** Stats display correctly with data from mock backend. Skeleton shows during loading.

---

### Phase 4 — Filter Bar & Ticket List

**Goal:** Implement content type filtering and the scrollable ticket list.

#### Task 4.1 — FilterBar Component (`components/admin/filter_bar.rs`)

```rust
#[component]
pub fn FilterBar(
    active_filter: RwSignal<Option<String>>,    // None = "All", Some("post"), Some("comment"), Some("user")
    review_only: RwSignal<bool>,
) -> impl IntoView
```

- Renders filter tabs: All | Posts | Comments | Accounts
- Renders "Waiting for review" checkbox
- On tab click: updates `active_filter` signal → triggers re-fetch in `ModerationList`
- On checkbox toggle: updates `review_only` signal → triggers re-fetch

**Legacy reference:** `admin/index.php` lines 54–71 (filter HTML), `admin/js/view.js` `initFilters()`.

#### Task 4.2 — ModerationList Component (`components/admin/moderation_list.rs`)

```rust
#[component]
pub fn ModerationList() -> impl IntoView
```

- Holds the filter signals (`active_filter`, `review_only`)
- Creates a reactive resource that re-fetches when filters change
- Contains `FilterBar` and the list of `TicketItem` components
- Uses `use_infinite_scroll` hook for pagination (offset + limit = 20)
- Table header row: Content | Moderation ID | Moderation date | Reports | Status
- Empty state: "No items found"
- Skeleton loading with 5 placeholder rows

**State management:**
```rust
let active_filter = RwSignal::new(None::<String>);
let review_only = RwSignal::new(false);
let items = RwSignal::new(Vec::<ModerationItem>::new());
let offset = RwSignal::new(0i32);
let has_more = RwSignal::new(true);
let loading = RwSignal::new(false);
```

**Legacy reference:** `admin/js/store.js` (state), `admin/js/fetcher.js` `loadItems()`, `admin/js/view.js` `renderItems()` + `initWindowInfiniteScroll()`.

**Acceptance:** Ticket list loads with 20 items, infinite scroll appends more. Filter tabs and review checkbox re-fetch correctly. Skeleton shows during loading.

---

### Phase 5 — Ticket Item & Expandable Detail

**Goal:** Implement the per-ticket row and expandable detail panel.

#### Task 5.1 — TicketItem Component (`components/admin/ticket_item.rs`)

```rust
#[component]
pub fn TicketItem(
    item: ModerationItem,
    on_moderation_action: Callback<(String, ModerationAction)>,  // (ticket_id, typed action)
) -> impl IntoView
```

**Collapsed state (summary row):** Clicking the row toggles the detail panel.

| Column | Content | Notes |
|--------|---------|-------|
| Content | Thumbnail + icon + username (+ post title for posts, comment ID for comments) | Icon by type: camera (image), text (text), audio (audio), play (video), comment (comment), profile (user) |
| Moderation ID | `#<ticket_id>` | Truncated UUID display |
| Moderation date | Formatted date | `"20 Jun 2025, 15:03"` format |
| Reports | Count + flag icon (red if ≥ 5) + "Not visible in feed" for hidden/illegal or high-report items | |
| Status | Color-coded label | Yellow = waiting, Red = hidden/illegal, Green = restored |

**Expanded state (detail panel):** Two-column layout.

**Legacy reference:** `admin/js/view.js` `renderItems()` (lines 110–300 for summary, 300+ for detail).

#### Task 5.2 — ContentPreview & Sub-Components

`content_preview.rs` dispatches to the appropriate sub-component based on `targettype`.
Each type-specific preview is in its own file to keep complexity manageable (the legacy equivalent spans ~200 lines of dense JS *per type*).

| File | Component | Renders |
|------|-----------|---------|
| `content_preview.rs` | `ContentPreview` | Match on `targettype`, delegate to sub-component |
| `post_preview.rs` | `PostPreview` | Post author, title, text, hashtags, "See full post" link |
| `comment_preview.rs` | `CommentPreview` | Commenter info, comment text, parent post link |
| `user_preview.rs` | `UserPreview` | Profile image, bio, stats, "View profile" link |

Left side of the expanded detail. Renders differently based on `targettype`:

**Post preview:**
- Author profile image + username + slug
- "See full post" link → `/post/<id>`
- Post title + time ago
- Post text/description
- Hashtags

**Comment preview:**
- "Reported comment" heading with comment icon
- Commenter profile image + username + slug + time ago
- Comment text
- Linked parent post info (if available — requires secondary lookup via `get_post`)

**User preview:**
- Profile image + username + slug
- Biography text
- Profile stats (posts, followers, following, peers)
- "View profile" link → `/profile/<slug>`

**Legacy reference:** `admin/js/view.js` lines 300–500 (post/user/comment detail blocks), `admin/js/fetcher.js` `normalizeItems()` + `enrichCommentsWithPosts()`.

#### Task 5.3 — ActionPanel Component (`components/admin/action_panel.rs`)

Right side of the expanded detail:

1. **Reporters list:** Profile image + username + slug + report date for each reporter
2. **Action buttons** (only shown when status is "waiting for review"):
   - "Restore" (blue button)
   - "Hide" (transparent button)
   - "Mark as illegal" (red transparent button)
3. **Confirmation dialog** (replaces action buttons when clicked):
   - Hide: "Are you sure you want to hide this content?" → "It will require additional confirmation from users to be shown."
   - Restore: "Are you sure you want to restore this content?" → "It will reappear in everyone's feed."
   - Illegal: "Are you sure this content is illegal?" (red text) → "It will never be shown to anyone again."
   - Yes / No buttons
4. **Moderated by info** (shown after action or for already-moderated tickets):
   - Moderator profile image + username + slug
   - Moderation date
   - Action taken (color-coded: green for restored, red for hidden/illegal)

**Optimistic UI:** After confirming an action:
1. Call `perform_moderation(moderation_ticket_id, moderation_action)` server function
2. On success: update the item's status in the signal, swap action buttons for moderated-by info, bump `stats_version` trigger signal (see below)
3. On error: show toast with error message, revert UI

**Stats refresh mechanism:** `ModerationList` owns a `stats_version: RwSignal<u32>` signal, passed to both `StatsHeader` and `ActionPanel`. `StatsHeader` uses it as a dependency in its resource (re-fetches when bumped). `ActionPanel` increments it after a successful moderation action. This avoids global state while keeping stats in sync.

**Legacy reference:** `admin/js/view.js` lines 500–780 (action buttons, confirmation dialogs, moderation handlers).

**Acceptance:** Clicking a ticket row expands/collapses the detail. Only one ticket can be expanded at a time (accordion). Moderation actions work end-to-end against mock backend.

---

### Phase 6 — Admin Header & Styles

**Goal:** Implement the admin-specific header and port CSS to SCSS.

#### Task 6.1 — AdminHeader Component (`components/admin/admin_header.rs`)

```rust
#[component]
pub fn AdminHeader() -> impl IntoView
```

- "Admin" logo/title on the left
- "Logged in as <username>" in the center (reads from `AuthContext`)
- "Back to user mode" link → `/dashboard` on the right

**Legacy reference:** `admin/template-parts/header.php`.

#### Task 6.2 — SCSS Styles (`style/admin.scss`)

Port relevant styles from `admin/css/style.css` (2,239 lines). Key sections to port:

| Section | Lines (approx.) | Priority |
|---------|-----------------|----------|
| Layout (`.site_layout`, `.site-main-admin`) | ~50 | Must |
| Header (`.site-header`, `.inner-header`) | ~40 | Must |
| Stats (`.main_stats`, `.stat_box`) | ~80 | Must |
| Filter bar (`.content_filter_row`, `.item_filters`) | ~60 | Must |
| Content list (`.content_list`, `.content_item`) | ~150 | Must |
| Content box detail (`.content_box`, `.content_box_left`, `.content_box_right`) | ~200 | Must |
| Post/Comment/User type blocks | ~200 | Must |
| Action buttons & confirmation (`.action_buttons`, `.action_box`) | ~80 | Must |
| Reporters list (`.reported_by`) | ~60 | Must |
| Moderated by (`.moderated_by_box`) | ~40 | Must |
| Typography & colors (reuse CSS custom properties) | ~100 | Must |
| Responsive breakpoints | ~100 | Should |
| Animations/transitions | ~50 | Nice to have |
| SVG background decoration | ~20 | Nice to have |

**Approach:** Don't copy-paste the entire legacy CSS. Instead, write clean SCSS using the existing `peer-web` design system variables and mixins where possible. Reference the legacy CSS for layout geometry, spacing, and color choices.

**Acceptance:** The admin page visually matches the legacy PHP admin panel in dark mode.

---

## API Reference

### GraphQL Operations Used

| Operation | Type | Variables | Response Type | Role Required |
|-----------|------|-----------|---------------|---------------|
| `moderationStats` | Query | — | `ModerationStatsResponse` | MODERATOR (256) |
| `moderationItems` | Query | `offset: Int!, limit: Int!, contentType: ModerationContentType, status: ModerationStatus` | `ModerationItemListResponse` | MODERATOR (256) |
| `performModeration` | Mutation | `moderationTicketId: ID!, moderationAction: ModerationStatus!` | `DefaultResponse` | MODERATOR (256) |

### Response Codes to Handle

| Code | Meaning | UI Action |
|------|---------|-----------|
| `12101` | Stats retrieved | Display stats |
| `12102` | Items retrieved | Display items |
| `12103` | Moderation action performed | Update item status + refresh stats |
| `22103` | Ticket not found | Toast error |
| `32101` | Invalid action | Toast error |
| `32103` | Already in terminal state | Toast warning |
| `62101` | Not authorized | Redirect to `/dashboard` |
| `60501` | Not authenticated | Redirect to `/login` |

### Mock Backend Endpoints

All operations are available in the mock backend (Phase 6):
- 2 seed moderator-like users: `admin` (roles_mask=16) and `moderator` (roles_mask=256)
- 3 seed moderation tickets (1 post, 1 comment, 1 user)
- Report actions (Phase 3/4) auto-create moderation tickets

---

## Component Sizing Estimates

| Component | Estimated Lines | Notes |
|-----------|-----------------|-------|
| `models/moderation.rs` | ~120 | 8-10 structs with serde |
| `api/moderation.rs` | ~120 | 3 server functions + variable structs |
| `pages/admin.rs` | ~60 | Page shell + signal setup |
| `components/admin/admin_header.rs` | ~30 | Simple header bar |
| `components/admin/stats_header.rs` | ~80 | 4 stat boxes + skeleton |
| `components/admin/filter_bar.rs` | ~60 | Tabs + checkbox |
| `components/admin/moderation_list.rs` | ~120 | List + infinite scroll + loading states |
| `components/admin/ticket_item.rs` | ~100 | Summary row + expand toggle |
| `components/admin/content_preview.rs` | ~40 | Dispatch to type-specific sub-component |
| `components/admin/post_preview.rs` | ~80 | Post detail in expanded ticket |
| `components/admin/comment_preview.rs` | ~80 | Comment detail in expanded ticket |
| `components/admin/user_preview.rs` | ~60 | User detail in expanded ticket |
| `components/admin/action_panel.rs` | ~180 | Reporters + actions + confirmation + moderated-by |
| `style/admin.scss` | ~500 | Ported/adapted from legacy CSS |
| **Total** | **~1,630** | |

---

## Testing Strategy

### E2E Tests (Playwright)

| Test | Description |
|------|-------------|
| Admin access denied | Non-moderator user navigates to `/admin` → redirected to `/dashboard` |
| Stats display | Moderator sees correct stat counts |
| Filter tabs | Clicking "Posts" tab shows only post tickets |
| Review checkbox | Toggling "Waiting for review" filters list |
| Ticket expand/collapse | Clicking a row expands detail, clicking again collapses |
| Accordion behaviour | Expanding one row collapses the previously expanded row |
| Hide action | "Hide" → confirm → item status updates to "hidden", stats refresh |
| Restore action | "Restore" → confirm → item status updates to "restored", stats refresh |
| Illegal action | "Mark as illegal" → confirm → item status updates to "illegal", stats refresh |
| Cancel confirmation | Click action → click "No" → returns to action buttons |
| Infinite scroll | Scroll to bottom → more items load |

### Unit / Component Tests

- Model deserialization from mock backend JSON snapshots
- Server function error handling (invalid token, permission denied)
- Filter signal reactivity

---

## Migration Decisions

| Decision | Rationale |
|----------|-----------|
| **Single query vs 4 queries** | Legacy uses separate `LIST_ITEMS`, `LIST_POST`, `LIST_COMMENT`, `LIST_USER` queries with narrower field selections. Leptos uses a single query with the full field set + `contentType` variable. Simpler code, negligible overhead for a moderation page. |
| **No comment post enrichment** | Legacy makes a secondary `loadPostById` call for each comment ticket to show the parent post. The mock backend's `moderationItems` response already includes the post data via `targetcontent.comment.postid`. We can display the post ID/link directly and load full post data lazily on expand if needed. |
| **Accordion (single expand)** | Legacy uses an accordion pattern where only one ticket detail is open at a time. We keep this — it prevents the page from becoming unwieldy with multiple expanded details. |
| **No media slider** | The legacy admin panel builds custom image/video sliders in JS. For Phase 1, post media will render as a simple image/thumbnail. Full post view is available via the "See full post" link at `/post/<id>`. Media slider can be added later. |
| **RoleGuard component** | New component that wraps `AuthGuard` and adds role checking. Reusable for future admin sub-pages. |
| **SCSS vs legacy CSS** | Write fresh SCSS rather than porting 2,239 lines of CSS. Reference legacy for layout/spacing values. Much of the legacy CSS handles typography, normalize, and utility classes that already exist in peer-web's design system. |

---

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| Mock backend Phase 6 | ✅ Complete | RBAC, moderation, admin — 266 tests |
| `AuthGuard` component | ✅ Exists | `src/components/auth_guard.rs` |
| `use_infinite_scroll` hook | ✅ Exists | `src/hooks/use_infinite_scroll.rs` |
| `auth_fetch` / `get_access_token_from_cookies` | ✅ Exists | `src/api/auth_fetch.rs` |
| `query()` / `mutate()` GraphQL helpers | ✅ Exists | `src/api/graphql.rs` |
| Toast system | ✅ Exists | `src/components/toast.rs` |
| `DefaultResponse` model | ✅ Exists | `src/models/common.rs` |

---

## Implementation Order

```
Phase 1 (Models + API)     → Foundation, no UI
Phase 2 (Page + Route)     → Navigable at /admin, role-gated
Phase 3 (Stats Header)     → First visible data
Phase 4 (Filters + List)   → Core experience
Phase 5 (Detail + Actions) → Full moderation workflow
Phase 6 (Header + Styles)  → Visual polish
```

Phases 1–2 can be implemented in a single session. Phases 3–5 are the core work. Phase 6 is polish.

---

## Changelog

*(To be updated during implementation)*
