# Referral Board Implementation Plan

**Feature:** Referral Board  
**Priority:** #10 (after Settings — first truly new feature after all in-progress items)  
**Status:** ❌ Not Started  
**Created:** 2026-04-14

---

## Overview

Implement the Referral Board page for the Leptos frontend. This page is the core of Peer's organic-growth engine: users share a personal referral link, track who they've invited, and see who invited them. It directly complements the Registration flow (✅ complete), which already accepts referral codes.

### Goals

1. Full parity with legacy `referralBoard.php` user experience
2. Display and copy the authenticated user's personal referral link
3. Tabbed UI: "Invited Friends" (users I referred) and "My Inviter" (who referred me)
4. User cards with avatar, username, slug, and click-through to profile
5. Authenticated-only page (requires login)
6. Responsive layout matching existing 3-column page pattern
7. Loading, empty, and error states

---

## Scope

### In Scope

- [ ] Referral Board page (`/referral` route)
- [ ] Auth guard (redirect to `/login` if unauthenticated)
- [ ] **Referral Link section:**
  - [ ] Fetch referral link via `getReferralInfo` query
  - [ ] Display shareable referral link
  - [ ] Copy-to-clipboard on click (Clipboard API)
  - [ ] Toast notification on copy ("Link copied!")
  - [ ] Loading and error states for link fetch
- [ ] **Tab navigation:**
  - [ ] "Invited Friends" tab (default active)
  - [ ] "My Inviter" tab
  - [ ] Active state styling
- [ ] **User list (Invited Friends tab):**
  - [ ] Fetch via `referralList` query
  - [ ] Grid of user cards (avatar, username, slug)
  - [ ] Click user card → navigate to `/profile/:slug` or `/u/:slug`
  - [ ] Empty state ("You haven't referred anyone yet…")
  - [ ] Loading skeleton
- [ ] **User list (My Inviter tab):**
  - [ ] Single user card (or empty if direct signup)
  - [ ] Empty state ("No inviter found — you joined directly")
  - [ ] Loading skeleton
- [ ] **Right sidebar:**
  - [ ] Profile widget
  - [ ] Main menu
  - [ ] New post button
  - [ ] Version widget
- [ ] SCSS styling (new `referral-board.scss`)
- [ ] Responsive layout (desktop/tablet/mobile)

### Out of Scope (Future Work)

- Invite deep-link page (`/invite?referralUuid=...`) — separate small feature
- Referral earnings tracking (token economy detail)
- Referral count badges
- Pagination/infinite scroll for invited list (API supports `offset`/`limit`, wire later)
- Share via native share sheet (Web Share API)
- QR code generation

---

## Legacy Implementation Analysis

### Files

| File | Purpose | Lines |
|------|---------|-------|
| `referralBoard.php` | Page template with 3-column layout, tab HTML, user grid containers | ~140 |
| `js/referral.js` | Fetch referral link, fetch referral list, tab switching, user card rendering, clipboard copy, toast | ~360 |
| `css/style.css` | Shared styles (referral-specific styles are inline in the page/JS) | — |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│  HEADER: 🏠 Dashboard                                      │
├──────────────┬─────────────────────────┬───────────────────┤
│              │                         │                   │
│  LEFT        │      MAIN CONTENT       │  RIGHT            │
│  SIDEBAR     │                         │  SIDEBAR          │
│  (filters)   │  ┌─────────────────────┐│                   │
│              │  │ Referral Program    ││  - Profile widget │
│              │  │ description text    ││  - Main menu      │
│              │  │                     ││  - New post btn   │
│              │  │ [referral-link] 📋  ││  - Version        │
│              │  │   "Link copied!" ✅ ││                   │
│              │  └─────────────────────┘│                   │
│              │                         │                   │
│              │  ┌─────────────────────┐│                   │
│              │  │ [ Invited Friends ] │|                   │
│              │  │ [ My Inviter     ]  ││                   │
│              │  │ ─────────────────── ││                   │
│              │  │ Account             ││                   │
│              │  │ ┌────┐ ┌────┐      ││                   │
│              │  │ │ 👤 │ │ 👤 │ ...  ││                   │
│              │  │ │name│ │name│      ││                   │
│              │  │ │#slug│ │#slug│     ││                   │
│              │  │ └────┘ └────┘      ││                   │
│              │  └─────────────────────┘│                   │
│              │                         │                   │
├──────────────┴─────────────────────────┴───────────────────┤
│  FOOTER (mobile nav)                                       │
└────────────────────────────────────────────────────────────┘
```

### User Card Layout

```
┌──────────────────────────────┐
│  [Avatar]  Username          │
│            #slug             │
└──────────────────────────────┘
```

### Key Behaviors

1. **Referral Link Fetching**
   - `getReferralInfo` returns `referralUuid` and `referralLink`
   - Legacy JS rewrites the domain in the link to match the current host
   - Legacy replaces `register.php` → `invite.php` in the link
   - In Leptos, we'll construct the link as `{origin}/invite?referralUuid={uuid}` (or just display the raw link and later wire invite page)

2. **Referral List Fetching**
   - `referralList(offset: 0, limit: 20)` returns both `invitedBy` and `iInvited` in a single call
   - `invitedBy` is a single `ProfileUser` (or null) — the person who referred the current user
   - `iInvited` is an array of `ProfileUser` — people the current user referred
   - Response code `21003` / codes starting with `2` indicate empty data (no referrals)
   - Both arrays may be a single object or an array — legacy JS normalizes this

3. **Tab Switching**
   - Two tabs: "Invited Friends" (default), "My Inviter"
   - Tab click re-renders the grid content from in-memory data
   - No separate API call per tab (single fetch populates both)

4. **Clipboard Copy**
   - Uses `navigator.clipboard.writeText()`
   - Shows a toast notification for 3 seconds

5. **User Card Click**
   - Navigates to `view-profile.php?user={uuid}` in legacy
   - In Leptos, navigate to `/profile/{slug}` or `/u/{slug}`

---

## GraphQL API Reference

### Query: `getReferralInfo`

```graphql
query GetReferralInfo {
    getReferralInfo {
        status
        ResponseCode
        referralUuid
        referralLink
    }
}
```

**Auth:** Required (Bearer token)

| Response Code | Description |
|---------------|-------------|
| `11011` | Referral info loaded |
| `21002` | No referral info found (new referral generated) |
| `60501` | Not authenticated |

### Query: `referralList`

```graphql
query ReferralList($offset: Int, $limit: Int) {
    referralList(offset: $offset, limit: $limit) {
        status
        counter
        ResponseCode
        affectedRows {
            invitedBy {
                id
                username
                slug
                img
            }
            iInvited {
                id
                username
                slug
                img
            }
        }
    }
}
```

**Auth:** Required (Bearer token)

| Response Code | Description |
|---------------|-------------|
| `11011` | Referral list loaded |
| `21003` | No referral data |
| `60501` | Not authenticated |

---

## Leptos Implementation

### Architecture Overview

```
pages/referral_board.rs          ← Page component + header
components/referral_board/       ← Sub-components
├── mod.rs
├── referral_header.rs           ← Referral link display + copy
├── referral_tabs.rs             ← Tab navigation
├── referral_user_card.rs        ← Individual user card
└── referral_user_grid.rs        ← Grid of user cards (per tab)
api/referral.rs                  ← Server functions (getReferralInfo, referralList)
models/referral.rs               ← Response types
style/referral-board.scss        ← Page styles
```

### Models (`src/models/referral.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Response from the `getReferralInfo` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReferralInfoResponse {
    #[serde(rename = "status")]
    pub status: String,
    pub response_code: String,
    #[serde(default, rename = "referralUuid")]
    pub referral_uuid: Option<String>,
    #[serde(default, rename = "referralLink")]
    pub referral_link: Option<String>,
}

impl ReferralInfoResponse {
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }
}

/// A user in the referral list (lighter than ProfileUser — only id, username, slug, img).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralListUser {
    pub id: String,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub img: Option<String>,
}

impl ReferralListUser {
    pub fn avatar_url(&self) -> &str {
        self.img.as_deref().unwrap_or("/svg/noname.svg")
    }

    pub fn display_slug(&self) -> String {
        format!("#{}", self.slug)
    }
}

/// The `affectedRows` envelope inside `referralList` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralUsers {
    /// The user who invited the current user (single or null).
    #[serde(default, deserialize_with = "deserialize_optional_single_or_vec")]
    pub invited_by: Vec<ReferralListUser>,
    /// Users the current user invited (array).
    #[serde(default, deserialize_with = "deserialize_optional_single_or_vec")]
    pub i_invited: Vec<ReferralListUser>,
}

/// Response from the `referralList` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReferralListResponse {
    #[serde(rename = "status")]
    pub status: String,
    #[serde(default)]
    pub counter: i32,
    pub response_code: String,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Option<ReferralUsers>,
}

impl ReferralListResponse {
    pub fn is_success(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }

    pub fn is_empty_response(&self) -> bool {
        self.response_code.starts_with('2')
    }

    pub fn invited_by(&self) -> Vec<ReferralListUser> {
        if self.is_empty_response() {
            return vec![];
        }
        self.affected_rows
            .as_ref()
            .map(|r| r.invited_by.clone())
            .unwrap_or_default()
    }

    pub fn i_invited(&self) -> Vec<ReferralListUser> {
        if self.is_empty_response() {
            return vec![];
        }
        self.affected_rows
            .as_ref()
            .map(|r| r.i_invited.clone())
            .unwrap_or_default()
    }
}
```

> **Note:** The backend may return `invitedBy` as a single object or an array. The custom `deserialize_optional_single_or_vec` serde helper normalizes this (same pattern as the legacy JS normalization). This helper should live in `src/models/common.rs` for reuse.

### API Layer (`src/api/referral.rs`)

```rust
use leptos::prelude::*;

/// Fetch the current user's referral info (UUID + shareable link).
#[server(GetReferralInfo, "/api")]
pub async fn get_referral_info() -> Result<ReferralInfoResponse, ServerFnError> {
    use crate::api::graphql::{query, GetReferralInfoData, GET_REFERRAL_INFO_QUERY};
    use crate::api::auth_fetch::get_auth_token;

    let token = get_auth_token().await?;
    let data: GetReferralInfoData = query(GET_REFERRAL_INFO_QUERY, (), Some(&token)).await?;
    Ok(data.get_referral_info)
}

/// Fetch the referral list (who I invited + who invited me).
#[server(GetReferralList, "/api")]
pub async fn get_referral_list(
    offset: i32,
    limit: i32,
) -> Result<ReferralListResponse, ServerFnError> {
    use crate::api::graphql::{query, GetReferralListData, GET_REFERRAL_LIST_QUERY};
    use crate::api::auth_fetch::get_auth_token;

    let token = get_auth_token().await?;

    #[derive(serde::Serialize)]
    struct Vars { offset: i32, limit: i32 }

    let data: GetReferralListData = query(
        GET_REFERRAL_LIST_QUERY,
        Vars { offset, limit },
        Some(&token),
    ).await?;

    Ok(data.referral_list)
}
```

### GraphQL Constants (add to `src/api/graphql.rs`)

```rust
/// Query: Get the current user's referral info.
pub const GET_REFERRAL_INFO_QUERY: &str = r#"
query GetReferralInfo {
    getReferralInfo {
        status
        ResponseCode
        referralUuid
        referralLink
    }
}
"#;

/// Query: List referral relationships.
pub const GET_REFERRAL_LIST_QUERY: &str = r#"
query ReferralList($offset: Int, $limit: Int) {
    referralList(offset: $offset, limit: $limit) {
        status
        counter
        ResponseCode
        affectedRows {
            invitedBy {
                id
                username
                slug
                img
            }
            iInvited {
                id
                username
                slug
                img
            }
        }
    }
}
"#;

/// Wrapper for `getReferralInfo` query response.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetReferralInfoData {
    pub get_referral_info: crate::models::referral::ReferralInfoResponse,
}

/// Wrapper for `referralList` query response.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetReferralListData {
    pub referral_list: crate::models::referral::ReferralListResponse,
}
```

### Page Component (`src/pages/referral_board.rs`)

```rust
use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::referral::{get_referral_info, get_referral_list};
use crate::components::auth_guard::AuthGuard;
use crate::components::referral_board::{ReferralHeader, ReferralTabs};
use crate::components::widgets::{MainMenu, ProfileWidget, VersionWidget};

/// Active tab in the referral board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferralTab {
    Invited,
    Inviter,
}

#[component]
pub fn ReferralBoardPage() -> impl IntoView {
    let active_tab = RwSignal::new(ReferralTab::Invited);

    // Fetch referral info (link) and referral list in parallel
    let referral_info = Resource::new(|| (), |_| async { get_referral_info().await.ok() });
    let referral_list = Resource::new(
        || (),
        |_| async { get_referral_list(0, 20).await.ok() },
    );

    view! {
        <Title text="Referral Program - Peer Network"/>
        <AuthGuard>
            <div id="referral-board" class="site_layout referral-board-layout">
                <header class="site-header header-referralBoard">
                    <img class="logo" src="/svg/Home.svg" alt="Peer Network"/>
                    <h1 class="referralBoard_h1">"Dashboard"</h1>
                </header>

                <aside class="left-sidebar left-sidebar-referralBoard">
                    <div class="inner-scroll">
                        // Left sidebar — empty or filters (matching legacy)
                    </div>
                </aside>

                <main class="site-main site-main-referralBoard">
                    <div class="referralBoard_container">
                        <Suspense fallback=move || view! { <ReferralHeaderSkeleton/> }>
                            {move || referral_info.get().flatten().map(|info| {
                                view! { <ReferralHeader info=info/> }
                            })}
                        </Suspense>

                        <Suspense fallback=move || view! { <ReferralListSkeleton/> }>
                            {move || referral_list.get().flatten().map(|list| {
                                view! {
                                    <ReferralTabs
                                        active_tab=active_tab
                                        referral_list=list
                                    />
                                }
                            })}
                        </Suspense>
                    </div>
                </main>

                <aside class="right-sidebar right-sidebar-referralBoard">
                    <div class="inner-scroll">
                        <ProfileWidget/>
                        <MainMenu/>
                        <NewPostButton/>
                        <VersionWidget/>
                    </div>
                </aside>

                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}
```

### Sub-Components

#### `ReferralHeader` — Link display + copy

```rust
/// Displays the referral program description and shareable link with copy button.
#[component]
pub fn ReferralHeader(info: ReferralInfoResponse) -> impl IntoView {
    let copied = RwSignal::new(false);

    let link_text = info.referral_link.clone().unwrap_or_default();

    let on_copy = move |_| {
        let text = link_text.clone();
        spawn_local(async move {
            // Use wasm_bindgen Clipboard API
            if let Some(clipboard) = window().navigator().clipboard() {
                let _ = JsFuture::from(clipboard.write_text(&text)).await;
                copied.set(true);
                // Auto-reset after 3 seconds
                set_timeout(move || copied.set(false), Duration::from_secs(3));
            }
        });
    };

    view! {
        <div class="referralBoard_header">
            <h1>"Referral Program"</h1>
            <p>
                "Invite a friend and earn "
                <em class="bold">"1% of their earnings"</em>
                " every time they transfer or cash out "
                <em class="bold">"forever"</em>
                ". The more you refer, the more you earn!"
            </p>
            <p>"Copy the code and share it with a friend. Make sure they enter it during registration."</p>
            <div class="referral_link_container" on:click=on_copy>
                <span class="link_text">{&info.referral_link.unwrap_or("Loading...".into())}</span>
                <img src="/svg/refCopy.svg" alt="Copy icon"/>
            </div>
            <Show when=move || copied.get()>
                <div class="toast show">
                    <svg class="toast_icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/>
                    </svg>
                    <span class="toast_message">"Link copied!"</span>
                </div>
            </Show>
        </div>
    }
}
```

#### `ReferralTabs` — Tab switching + user grid

```rust
#[component]
pub fn ReferralTabs(
    active_tab: RwSignal<ReferralTab>,
    referral_list: ReferralListResponse,
) -> impl IntoView {
    let invited = referral_list.i_invited();
    let inviter = referral_list.invited_by();

    view! {
        <div class="referralBoard_body">
            <div class="referral_tabs">
                <div
                    class=move || if active_tab.get() == ReferralTab::Invited { "button referral_tab active" } else { "button referral_tab" }
                    on:click=move |_| active_tab.set(ReferralTab::Invited)
                >
                    <span class="tab_label">"Invited Friends"</span>
                </div>
                <div
                    class=move || if active_tab.get() == ReferralTab::Inviter { "button referral_tab active" } else { "button referral_tab" }
                    on:click=move |_| active_tab.set(ReferralTab::Inviter)
                >
                    <span class="tab_label">"My Inviter"</span>
                </div>
            </div>

            <div class="referral_top">
                <h3>"Account"</h3>
            </div>

            <div class="referral_content">
                <Show
                    when=move || active_tab.get() == ReferralTab::Invited
                    fallback=move || view! { <ReferralUserGrid users=inviter.clone() empty_message="No inviter found — you joined directly without a referral"/> }
                >
                    <ReferralUserGrid users=invited.clone() empty_message="You haven't referred anyone yet — share your referral link to invite friends"/>
                </Show>
            </div>
        </div>
    }
}
```

#### `ReferralUserCard` — Individual user card

```rust
#[component]
pub fn ReferralUserCard(user: ReferralListUser) -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let slug = user.slug.clone();

    view! {
        <div
            class="user_card button"
            on:click=move |_| {
                navigate(&format!("/u/{}", slug), Default::default());
            }
        >
            <div class="ref_user_info">
                <img
                    src=user.avatar_url().to_string()
                    alt=user.username.clone()
                    class="user_avatar"
                    on:error=|ev| {
                        let target = event_target::<web_sys::HtmlImageElement>(&ev);
                        target.set_src("/svg/noname.svg");
                    }
                />
                <div class="user_info">
                    <span class="user_name">{user.username.clone()}</span>
                    <span class="user_slug">{user.display_slug()}</span>
                </div>
            </div>
        </div>
    }
}
```

#### `ReferralUserGrid` — Grid container with empty state

```rust
#[component]
pub fn ReferralUserGrid(
    users: Vec<ReferralListUser>,
    #[prop(into)] empty_message: String,
) -> impl IntoView {
    view! {
        <div class="referral_list active">
            {if users.is_empty() {
                view! {
                    <div class="empty_state">
                        <p>{empty_message}</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="users_grid">
                        <For
                            each=move || users.clone()
                            key=|u| u.id.clone()
                            children=|user| view! { <ReferralUserCard user=user/> }
                        />
                    </div>
                }.into_any()
            }}
        </div>
    }
}
```

---

## SCSS Styling (`style/referral-board.scss`)

The page follows the same 3-column layout as Dashboard, Profile, and Wallet. Key style areas:

```scss
// Referral Board page styles

.referral-board-layout {
  // Inherits site_layout grid
}

.site-main-referralBoard {
  display: flex;
  flex-direction: column;
  height: 100%;
}

// -- Header section --
.referralBoard_header {
  margin-top: 75px;

  h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin-bottom: 16px;
  }

  p {
    margin-bottom: 12px;
    line-height: 1.5;
    color: var(--white-opacity-70);
  }

  em.bold {
    font-weight: 700;
    font-style: normal;
  }
}

// -- Referral link container --
.referral_link_container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--card-bg);
  border-radius: 12px;
  cursor: pointer;
  transition: background 0.2s ease;
  margin-bottom: 8px;

  &:hover {
    background: var(--card-bg-hover);
  }

  .link_text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.875rem;
  }

  img {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }
}

// -- Copy toast (inline, not global ToastProvider) --
.referralBoard_header .toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 8px;
  background: var(--success-bg);
  color: var(--success-text);
  font-size: 0.875rem;
  margin-top: 8px;

  .toast_icon {
    width: 18px;
    height: 18px;
  }
}

// -- Tab navigation --
.referral_tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.referral_tab {
  padding: 10px 20px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-weight: 500;

  &.active {
    background: var(--accent);
    color: var(--white);
  }
}

.referral_top {
  margin-bottom: 16px;

  h3 {
    font-weight: 600;
    color: var(--white-opacity-50);
  }
}

// -- User card grid --
.users_grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.user_card {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-radius: 12px;
  background: var(--card-bg);
  cursor: pointer;
  transition: background 0.2s ease;

  &:hover {
    background: var(--card-bg-hover);
  }
}

.ref_user_info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user_avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
}

.user_info {
  display: flex;
  flex-direction: column;

  .user_name {
    font-weight: 600;
    font-size: 0.9375rem;
  }

  .user_slug {
    font-size: 0.8125rem;
    color: var(--white-opacity-50);
  }
}

// -- Empty & loading states --
.empty_state {
  padding: 40px 20px;
  text-align: center;
  color: var(--white-opacity-50);
  line-height: 1.6;
}

// -- Skeletons --
.referral-header-skeleton {
  // Placeholder pulse animation for the header section
}

.referral-list-skeleton {
  // Placeholder pulse animation for the user grid
}

// -- Responsive --
@media (max-width: 768px) {
  .referralBoard_header {
    margin-top: 20px;
  }

  .users_grid {
    grid-template-columns: 1fr;
  }
}
```

---

## Routing

Add to `src/app.rs`:

```rust
<Route path=StaticSegment("referral") view=ReferralBoardPage/>
```

Add to `src/pages/mod.rs`:

```rust
pub mod referral_board;
pub use referral_board::ReferralBoardPage;
```

---

## File Manifest

### New Files

| File | Purpose | Est. Lines |
|------|---------|------------|
| `src/pages/referral_board.rs` | Page component | ~80 |
| `src/components/referral_board/mod.rs` | Module declarations | ~10 |
| `src/components/referral_board/referral_header.rs` | Link display + copy | ~60 |
| `src/components/referral_board/referral_tabs.rs` | Tab navigation + content switching | ~60 |
| `src/components/referral_board/referral_user_card.rs` | User card component | ~35 |
| `src/components/referral_board/referral_user_grid.rs` | Grid + empty state | ~35 |
| `src/api/referral.rs` | Server functions | ~50 |
| `src/models/referral.rs` | Response types | ~90 |
| `style/referral-board.scss` | Page styles | ~180 |

### Modified Files

| File | Change |
|------|--------|
| `src/app.rs` | Add `/referral` route |
| `src/pages/mod.rs` | Add `referral_board` module + re-export |
| `src/components/mod.rs` | Add `referral_board` module |
| `src/api/mod.rs` | Add `referral` module |
| `src/api/graphql.rs` | Add 2 query constants + 2 data wrappers |
| `src/models/mod.rs` | Add `referral` module |
| `style/main.scss` | Import `referral-board.scss` |

---

## Implementation Phases

### Phase 1: Models & API Layer
1. Create `src/models/referral.rs` with response types
2. Add `deserialize_optional_single_or_vec` helper to `common.rs` (if not already present)
3. Add GraphQL query constants and data wrappers to `graphql.rs`
4. Create `src/api/referral.rs` with server functions
5. Register modules in `mod.rs` files

### Phase 2: Page & Components
1. Create `src/pages/referral_board.rs` with page layout
2. Create `src/components/referral_board/` directory with sub-components
3. Wire `ReferralHeader` (link display + clipboard copy)
4. Wire `ReferralTabs` + `ReferralUserGrid` + `ReferralUserCard`
5. Add loading skeletons and empty states

### Phase 3: Routing & Styling
1. Add `/referral` route in `app.rs`
2. Create `style/referral-board.scss`
3. Import in `main.scss`
4. Responsive layout adjustments

### Phase 4: Polish & Testing
1. Error handling (API failures, auth expiry)
2. Avatar fallback (on error → `/svg/noname.svg`)
3. Clipboard fallback (older browsers)
4. E2E test with mock backend (when mock Phase 2 is available)
5. Accessibility: keyboard navigation for tabs, aria-labels

---

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| Auth context | ✅ Available | `AuthGuard` for protected route |
| Profile widget | ✅ Available | Right sidebar |
| Main menu widget | ✅ Available | Right sidebar |
| Toast component | ✅ Available | Could use `ToastProvider` instead of inline toast |
| `ProfileUser` model | ✅ Available | Referral uses a lighter type (`ReferralListUser`) but same pattern |
| Mock backend Phase 2 | ❌ Not Started | `getReferralInfo` and `referralList` endpoints needed for E2E |

---

## Existing Code to Reuse

- **`AuthGuard`** — Protected route wrapper (same as Dashboard, Wallet, etc.)
- **`ProfileWidget`, `MainMenu`, `VersionWidget`** — Right sidebar widgets
- **`ToastProvider`** — Consider using the global toast instead of a local inline toast
- **`ProfileUser` model pattern** — `ReferralListUser` follows the same avatar + slug pattern
- **`ReferralUser` model** (`src/models/user.rs`) — Already exists for registration, but has different fields (`uid` vs `id`). The Referral Board needs a new type matching the `referralList` response shape.
- **Page layout pattern** — Same 3-column grid as Wallet, Dashboard, Profile pages

---

## Open Questions

1. **Invite page:** Should `/invite?referralUuid=...` be a separate route that does deep-link detection, or just redirect to `/register?referralUuid=...`? (Can be deferred to a separate small ticket.)
2. **Global vs inline toast:** The existing `ToastProvider` provides global toasts. Should the copy notification use that, or keep a local inline toast matching the legacy design?
3. **Pagination:** The API supports `offset`/`limit` — should we wire infinite scroll from the start, or implement a simple single-page load for v1?
