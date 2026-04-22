# Shared Layout Shell (Header / Footer / Sidebars)

**Feature:** Layout components — `SiteShell`, `SiteHeader`, `MobileFooter`, `RightRail`, `LeftRail`
**Priority:** Next — closes the three ❌ Layout rows in [feature-convergence.md](../../feature-convergence.md) and removes a documented bug source
**Status:** ✅ Implemented (Pass 1) — Pass 2 (`SiteHeader` / `StandardRightRail` adoption) deferred
**Created:** 2026-04-22
**Plan Quality Target:** ⭐⭐⭐⭐⭐

---

## Overview

Extract the page chrome that every authenticated page in `peer-web` reimplements inline today (header, mobile footer nav, left/right `<aside>` rails) into a small set of shared layout components. The convergence tracker has flagged `Header` / `Footer` / `Sidebars` as ❌ Not Started since the project began ([feature-convergence.md § Components / Layout](../../feature-convergence.md)). What's actually shipping is **divergent copy-paste**: `MobileFooter` is a private `fn` in six pages with different class names, different nav-item lists, and different — sometimes wrong — hard-coded `active` markers.

This plan promotes the layout chrome into a single, well-typed shell so:

1. The three ❌ Layout rows in the Components table can be retired.
2. Future pages (notably the planned [Home/Landing port](../home/home-implementation.md)) can use one component instead of inventing a seventh `MobileFooter`.
3. The active-route bug currently latent in `MobileFooter` (each page hard-codes which item is `active`, sometimes incorrectly) is replaced by a single computation against `use_location()`.
4. Inline `<aside class="left-sidebar …">` / `<aside class="right-sidebar …">` wrappers stop drifting in class names and `inner-scroll` markup.

### Goals

1. **One** `MobileFooter` component, route-aware, replacing six private copies.
2. **One** `RightRail` composition for the standard sidebar widget stack (`ProfileWidget` → `MainMenu` → `NewPostButton` → `VersionWidget`) used verbatim by Wallet, Chat, Referral Board, Settings, Version History (and likely Profile / View Profile / My Ads after audit).
3. **One** `LeftRail` shell that wraps page-specific sidebar content in the standard `<aside class="left-sidebar left-sidebar-{slug}"><div class="inner-scroll">…</div></aside>` markup. Page-specific *content* (filters, search box, etc.) stays page-local — the rail only owns the wrapper.
4. **One** `SiteHeader` with named slots (`title`, `actions`, optional `icon`) replacing five+ inline `<header>` blocks.
5. **One** `SiteShell` that composes `<div class="site_layout">` + `SiteHeader` + `LeftRail?` + `<main>` + `RightRail?` + `MobileFooter`, accepting a per-page `id` and `class` modifier (e.g. `wallet-layout`, `profile-layout`) so existing SCSS keeps targeting the same selectors.
6. Zero visual regression — selectors and DOM structure are preserved (tests assert this).
7. Zero new SCSS — the new components emit the same class names today's pages do.

### Non-Goals

- Extracting page-specific sidebar widgets (filter checkboxes, sort radios, profile widget chrome). Those are already separate components or stay page-local.
- A header/footer on the unauthenticated pages (`login`, `register`, `forgot_password`, `invite`). Those use the `LeftPanel` shared layout already and are out of scope.
- A new design / restyle. This is a pure refactor — pixel parity is the bar.
- Touching the SCSS files in [style/](../../..//style/). The shell emits the existing class names verbatim.
- Server-side route extraction (the SSR/CSR boundary is unchanged).

---

## Scope

### In Scope

- [ ] New module `src/components/layout/` with `mod.rs`, `site_shell.rs`, `site_header.rs`, `mobile_footer.rs`, `right_rail.rs`, `left_rail.rs`
- [ ] Migrate `dashboard.rs` to use the shell (this page already has `LeftSidebar` / `RightSidebar` components — verify the new `LeftRail`/`RightRail` wrappers compose cleanly with them, otherwise leave dashboard's bespoke sidebars alone for this sprint)
- [ ] Migrate `wallet.rs`, `chat.rs`, `settings.rs`, `referral_board.rs`, `version_history.rs`, `profile.rs`, `view_profile.rs`, `my_ads.rs`, `peer_shop.rs`, `admin.rs`, `new_post.rs` to the new shell where they currently roll their own header / footer / aside
- [ ] Delete the 10 private `fn MobileFooter` copies (and `peer_shop.rs`'s inline `<footer class="mobile-footer">`) once the shared component is live
- [ ] Delete the inline `<header class="site_header">…</header>` blocks and route them through `SiteHeader`
- [ ] Update [feature-convergence.md](../../feature-convergence.md): Header / Footer / Sidebars rows → ✅ Implemented; Summary counts; Migration Priority entry; Changelog
- [ ] One Playwright spec (`layout.spec.ts`) asserting `MobileFooter` highlights the active route on `/dashboard`, `/chat`, `/wallet`, `/profile` and that the `aside` / `header` / `footer` selectors still resolve

### Out of Scope (Future Work)

- A `<Header/>` for the marketing or guest paths (no consumer today).
- Extracting `LeftPanel` (used by login/register) into the same module — it's a different shape (split-screen branding) and isn't reused beyond auth.
- Mobile nav redesign / icon updates / unread badges. Tracked separately under chat completion sprint and any future nav refresh.
- Theming the shell. Existing SCSS variables remain the source of truth.

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| [`template-parts/footer.php`](../../../template-parts/footer.php) | Feedback popup + onboarding include + empty `<footer>` tag (legacy mobile nav lives in `phpheader.php`-rendered chrome, not in `footer.php`) |
| [`template-parts/sidebars/widget-*.php`](../../../template-parts/sidebars/) | 13 sidebar widgets (`widget-profile.php`, `widget-main-menu.php`, `widget-add-new-post.php`, `widget-web-version.php`, `widget-filter.php`, `widget-sort-filter.php`, `widget-create-post-filter.php`, `widget-back-button.php`, …) — already mostly ported as Leptos `widgets` components |
| [`css/style.css`](../../../css/style.css) | Owns `.site_layout`, `.left-sidebar`, `.right-sidebar`, `.mobile-footer` selectors |

### What Already Exists in `peer-web`

- `components/widgets/{profile_widget,main_menu,version_widget}.rs` — ported widget bodies.
- `components/auth_guard.rs` — guards the entire shell; the new `SiteShell` does **not** include it (callers wrap if needed, exactly as today).
- `components/back_button.rs` — already extracted.
- `components/dashboard/{left_sidebar,right_sidebar}.rs` — dashboard-specific rails composing widgets + filters.

### What's Duplicated Today

| Symbol | Sites | Drift |
|--------|-------|-------|
| `fn MobileFooter` | `dashboard.rs`, `chat.rs`, `wallet.rs`, `settings.rs`, `view_profile.rs`, `version_history.rs`, `profile.rs`, `my_ads.rs`, `new_post.rs`, `referral_board.rs` (10 private fns); `peer_shop.rs` ships the same `<footer class="mobile-footer">` markup inline (no fn). 11 sites total. | Class names differ (`nav-item` in Dashboard vs `mobile-nav-item` in Settings); item lists differ (Dashboard: Home/Search/NewPost/Alerts/Profile with labels; Settings: Home/Chat/NewPost/Wallet/Profile, no labels); each page hard-codes `active` (Dashboard pins Home; Wallet pins `/wallet`; Settings has neither pinning nor route awareness) |
| `<header class="site_header">…<div class="logo_box">…</div><div class="page-title"><h1>…</h1></div><div class="header-actions">…</div></header>` | `chat.rs`, `view_profile.rs`, `profile.rs`, `peer_shop.rs` (underscore spelling); `wallet.rs`, `settings.rs`, `version_history.rs`, `my_ads.rs`, `referral_board.rs` use the **hyphenated** `class="site-header header-{slug}"` form | `site_header` vs `site-header` class drift is **not** a cosmetic difference — see Open Question 3; per-page `header-{slug}` modifiers carry the actual styling rules |
| `<aside class="left-sidebar left-sidebar-{slug}"><div class="inner-scroll">…</div></aside>` | Every page with a sidebar | Class slug per page (`left-sidebar-wallet`, `left-sidebar-chats`, …) — kept by the new component |
| `<aside class="right-sidebar right-sidebar-{slug}"><div class="inner-scroll">…</div></aside>` with the same widget stack inside | Wallet, Chat, Referral Board, Settings, Version History | Identical body in 4 of 5 pages |

### Behavioural Contract (Preserved)

- `MobileFooter` renders the same five nav items in the same order; the `active` class is computed from `use_location().pathname` matching the link's `href` (or its prefix for `/profile/*`, `/chat/*`). Active computation is reactive on `pathname` so SSR (request URI) and post-hydration (`window.location`) agree without warning.
- `SiteHeader` emits `<header class="site-header header-{modifier}">` by default (the spelling used by Wallet / My Ads / Referral Board / Settings / Version History — verified in [style/](../../..//style/), where `.site-header.header-{slug}` selectors live in [dashboard.scss](../../..//style/dashboard.scss)). A `spelling: HeaderSpelling::Underscore` prop opts in to `<header class="site_header">` for callers that match Chat's shape ([chat.scss](../../..//style/chat.scss) scopes `.site_header` inside `.chat`).
- `LeftRail` / `RightRail` accept a `slug: &'static str` and emit `class="left-sidebar left-sidebar-{slug}"` / `class="right-sidebar right-sidebar-{slug}"` respectively. The `inner-scroll` wrapper is always present.
- `SiteShell`'s root element is `<div id="{id}" class="site_layout {modifier}">` so existing per-page CSS (`.wallet-layout`, `.profile-layout`, …) keeps working.

---

## Architecture

### Module Layout

```
src/components/
└── layout/
    ├── mod.rs           // re-exports
    ├── site_shell.rs    // SiteShell composition
    ├── site_header.rs   // SiteHeader with title/actions slots
    ├── mobile_footer.rs // route-aware MobileFooter
    ├── left_rail.rs     // LeftRail wrapper
    └── right_rail.rs    // RightRail wrapper + StandardRightRail preset
```

### Component Surface

> **Implementation note (2026-04-22):** the as-built shell collapses the
> per-region `#[slot]` design described below into a single `Children`
> prop on both `SiteShell` and `SiteHeader`. The shell wraps `children()`
> and auto-appends `<MobileFooter/>`; pages still hand-roll their own
> `<header>` / `<aside>` / `<main>` for Pass 1. The original slot-based
> surface is preserved here for context and remains the target shape if a
> future refactor wants the shell to enforce the `header / left / main /
> right / footer` rectangle.

```rust
// site_shell.rs
//
// NOTE: Leptos reserves `children` as a single special-cased prop, so multiple
// region slots are modelled with the `#[slot]` macro. Each slot is its own
// struct passed at the call site as `<HeaderSlot> … </HeaderSlot>`.

#[slot] pub struct HeaderSlot   { children: Children }
#[slot] pub struct LeftRailSlot { children: Children }
#[slot] pub struct MainSlot     { children: Children }
#[slot] pub struct RightRailSlot{ children: Children }

#[component]
pub fn SiteShell(
    /// Value for the root element's `id` attribute (e.g. "wallet-page").
    #[prop(into)] id: MaybeProp<String>,
    /// Optional CSS class appended after `site_layout` (e.g. "wallet-layout").
    #[prop(into, optional)] modifier: Option<String>,
    /// Optional header region.
    #[prop(optional)] header_slot: Option<HeaderSlot>,
    /// Optional left rail region.
    #[prop(optional)] left_rail_slot: Option<LeftRailSlot>,
    /// Main content. Wrapped in `<main id="main" class="site-main">`.
    main_slot: MainSlot,
    /// Optional right rail region.
    #[prop(optional)] right_rail_slot: Option<RightRailSlot>,
    /// If true, renders the shared `MobileFooter`. Default: true.
    #[prop(default = true)] mobile_footer: bool,
) -> impl IntoView
```

> Existing precedent in the workspace uses single-`Children` props ([toast.rs's `ToastProvider`](../../..//src/components/toast.rs), [auth_guard.rs's `AuthGuard`](../../..//src/components/auth_guard.rs)). The `#[slot]` macro from `leptos` is the supported way to expose more than one named region; verify against the `leptos` version pinned in [Cargo.toml](../../..//Cargo.toml) before Phase 1 lands and fall back to `ViewFn` props if the macro shape has shifted.

```rust
// site_header.rs

#[derive(Clone, Copy, Default)]
pub enum HeaderSpelling {
    /// `class="site-header header-{modifier}"` — used by Wallet, My Ads,
    /// Referral Board, Settings, Version History.
    #[default] Hyphen,
    /// `class="site_header"` — used by Chat (scoped under `.chat` in chat.scss),
    /// View Profile, Profile, Peer Shop.
    Underscore,
}

#[slot] pub struct HeaderActionsSlot { children: Children }

#[component]
pub fn SiteHeader(
    /// Page title displayed in the centre column.
    #[prop(into)] title: String,
    /// Per-page modifier appended after the base class (e.g. "wallet" →
    /// `header-wallet`). Required for the hyphen spelling, ignored for the
    /// underscore spelling.
    #[prop(into, optional)] modifier: Option<String>,
    /// Class spelling. Default: `Hyphen` (the majority of pages today).
    #[prop(default = HeaderSpelling::Hyphen)] spelling: HeaderSpelling,
    /// Optional peer-icon class for the title prefix (e.g. "peer-icon-wallet-filled").
    #[prop(into, optional)] icon: Option<String>,
    /// Optional right-aligned action region.
    #[prop(optional)] actions_slot: Option<HeaderActionsSlot>,
    /// If true, render the home-link logo on the left. Default: true.
    #[prop(default = true)] show_logo: bool,
) -> impl IntoView
```

```rust
// mobile_footer.rs
#[component]
pub fn MobileFooter() -> impl IntoView
// Reads use_location(); the `active` class is rendered from a reactive
// closure over `location.pathname` so SSR and post-hydration agree.
// See `components/widgets/main_menu.rs` for the established precedent.
```

```rust
// left_rail.rs
#[component]
pub fn LeftRail(
    /// Slug appended to the class list (e.g. "wallet" → `left-sidebar-wallet`).
    #[prop(into)] slug: String,
    children: Children, // single Children: idiomatic, matches ToastProvider
) -> impl IntoView
```

```rust
// right_rail.rs
#[component]
pub fn RightRail(
    #[prop(into)] slug: String,
    children: Children,
) -> impl IntoView

/// Convenience composition: ProfileWidget → MainMenu → AddPostButton → VersionWidget.
/// Used by Wallet, Chat, Referral Board, Settings, Version History.
/// `AddPostButton` is the shared widget at
/// `components/widgets/add_post_button.rs` (matches legacy
/// `template-parts/sidebars/widget-add-new-post.php`).
#[component]
pub fn StandardRightRail(#[prop(into)] slug: String) -> impl IntoView
```

### Active-Route Logic for `MobileFooter`

```rust
let location = use_location();
let is_active = move |prefix: &'static str| {
    let path = location.pathname.get();
    path == prefix || path.starts_with(&format!("{prefix}/"))
};
```

The class attribute must be reactive (e.g. `class:active=move || is_active("/dashboard")`), **not** computed once into a `String`, so SSR (request URI) and post-hydration (`window.location`) produce identical markup without a hydration warning. [components/widgets/main_menu.rs](../../..//src/components/widgets/main_menu.rs) is the precedent.

Active rules (one per nav item):

| Nav Item   | `href`         | Active when                                        |
|------------|----------------|----------------------------------------------------|
| Home       | `/dashboard`   | path == `/dashboard`                                |
| Search     | `/search`      | path starts with `/search`                          |
| New Post   | `/newpost`     | never highlighted (it's the floating action)        |
| Alerts     | `/notifications` | path starts with `/notifications`                 |
| Profile    | `/profile`     | path == `/profile` or starts with `/profile/`       |

### Why `#[slot]` regions (not multi-`Children`)

Leptos treats `children` as a single special-cased prop; declaring `header: Children`, `main: Children`, `right_rail: Children` as siblings does not compile. The supported pattern is the `#[slot]` macro, which expands each region into its own struct. Each region still accepts any `IntoView`, so a page can pass either a literal `view!` block or a dedicated component (`<DashboardHeader/>`, `<WalletHeader/>`, …). This matches how Dashboard already factors its sidebars and avoids forcing every page to migrate its header into `SiteHeader` on day one — a page can still pass `<MyCustomHeader/>` inside `<HeaderSlot>` and only adopt `SiteHeader` once it's ready.

### Migration Strategy

Two-pass refactor:

1. **Pass 1 — adoption (no behaviour change):** Each page's existing `view! { … }` is wrapped in `<SiteShell …>` with its current header / sidebar / footer code passed as slots. Private `MobileFooter` copies are deleted and `mobile_footer=true` (the default) is used. Net diff per page: small.
2. **Pass 2 — header/sidebar consolidation:** Pages whose header is identical-shaped to `SiteHeader` (Wallet, Referral Board, Chat) drop their bespoke header in favour of `<SiteHeader title="Wallet" icon="peer-icon-wallet-filled"/>`. Pages with the standard right-rail widget stack (Wallet, Chat, Settings, Referral Board, Version History) drop the inline composition for `<StandardRightRail slug="wallet"/>`. Pass 2 lands page-by-page; Pass 1 is the gate for closing the three ❌ rows.

---

## Implementation Plan

### Phase 1 — Component Skeleton (≈180 LOC)

**Files (new):**

- `src/components/layout/mod.rs` (re-exports)
- `src/components/layout/site_shell.rs`
- `src/components/layout/site_header.rs`
- `src/components/layout/mobile_footer.rs`
- `src/components/layout/left_rail.rs`
- `src/components/layout/right_rail.rs` (+ `StandardRightRail`)

**Files (modified):**

- `src/components/mod.rs` — `pub mod layout;`

**Tasks:**

1. Implement the five components per the surface above.
2. `MobileFooter` uses a reactive class binding on `location.pathname`; on SSR the signal returns the request path and on hydration it returns `window.location.pathname`, so a reactive binding produces identical markup on both sides (precedent: [main_menu.rs](../../..//src/components/widgets/main_menu.rs)). A non-reactive `String` would risk a hydration warning.
3. `SiteHeader`'s default spelling is `HeaderSpelling::Hyphen` (used by Wallet / My Ads / Referral Board / Settings / Version History) and emits `class="site-header header-{modifier}"`. The `Underscore` spelling emits `class="site_header"` for Chat / Profile / View Profile / Peer Shop. Both spellings reuse the same inner markup (`.site_header_inner` → `.logo_box` → `.page-title h1` → `.header-actions`); verify against [chat.scss](../../..//style/chat.scss) and [dashboard.scss](../../..//style/dashboard.scss) before deleting any per-page header.
4. `SiteShell` accepts an optional `modifier` and emits `class="site_layout {modifier}"` (no trailing space when modifier is `None`).
5. `StandardRightRail` composes `RightRail` + the four widgets; the widget stack imports are local to `right_rail.rs`.

**Acceptance for Phase 1:**

- [ ] `cargo leptos build` clean (this repo's canonical build — see the `cargo-leptos` terminal in `AGENTS.md`)
- [ ] `cargo build --features ssr` clean (SSR-only sanity check)
- [ ] `cargo clippy --all-features -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] No page imports the new module yet (Phase 1 is component-only)

### Phase 2 — Pass-1 Migration (≈−400 LOC across pages, mostly deletions)

**Files modified (audit-confirmed):**

Private `fn MobileFooter` (10): `dashboard.rs`, `chat.rs`, `wallet.rs`, `settings.rs`, `view_profile.rs`, `version_history.rs`, `profile.rs`, `my_ads.rs`, `new_post.rs`, `referral_board.rs`.

Inline `<footer class="mobile-footer">` without a fn (1): `peer_shop.rs`.

Bespoke chrome to evaluate during Pass 1 (and likely defer to a follow-up plan if the shape does not match `SiteShell`'s `header / left / main / right / footer` rectangle): `admin.rs`, `peer_shop.rs`.

**Per-page steps:**

1. Replace the outermost `<div id=… class="site_layout …">…</div>` with `<SiteShell id=… modifier=…>` and pass the page's existing header / left rail / main / right rail blocks as slots.
2. Delete the page's private `fn MobileFooter`.
3. Re-run `cargo build --features ssr`; fix any `unused_imports` warnings introduced by the deletion.

**Acceptance for Phase 2:**

- [ ] `grep_search "fn MobileFooter"` returns 1 hit (the shared component)
- [ ] `grep_search "class=\"mobile-footer"` returns 1 hit (catches `peer_shop.rs`'s inline copy, which the `fn` grep alone misses)
- [ ] `grep_search "class=\"site_layout"` returns 0 hits in `pages/` (all routed through `SiteShell`)
- [ ] All migrated pages render without console warnings (including no Leptos hydration warnings on the active-class binding) under `cargo leptos watch`
- [ ] Manual smoke against the Rust mock backend: `/dashboard`, `/wallet`, `/chat`, `/settings`, `/profile`, `/referralBoard`, `/version-history` — the mobile footer's `active` class follows the URL on each
- [ ] Visual diff: no per-page CSS changes; no new selectors in `style/`

### Phase 3 — Pass-2 Consolidation (optional, opportunistic)

Land in a follow-up PR — not gating for the ❌ → ✅ flip on Header / Footer / Sidebars (Pass 1 is sufficient because the **shared component now exists and is the path of least resistance for new pages**).

- [ ] Replace bespoke `WalletHeader` / `ChatHeader` / `ReferralHeader` with `<SiteHeader title=… icon=…/>` where shape matches
- [ ] Replace inline right-rail compositions with `<StandardRightRail slug=…/>` where contents are identical
- [ ] Delete the now-unused header/sidebar private fns

### Phase 4 — Convergence Tracker Update (≈30 LOC of markdown)

**File:** [docs/feature-convergence.md](../../feature-convergence.md)

- [ ] Components → Layout: `Header` (line 64), `Footer` (line 65), `Sidebars` (line 66) rows ❌ → ✅, with notes pointing at this plan
- [ ] Summary table: increment `✅ Implemented` for the Components section by 3
- [ ] Add Migration Priority entry (`Layout shell`) referencing this plan; verify the next index against the live document at merge time (the doc reorders frequently)
- [ ] Changelog entry summarising files added / deleted, line delta, the latent `MobileFooter` active-route bug fix, **and** the Settings mobile-nav behaviour change (see Open Question 2)

### Phase 5 — E2E Coverage (≈80 LOC)

**File (new):** `end2end/tests/layout.spec.ts`

Cases:

1. `/dashboard` → `footer.mobile-footer a[href="/dashboard"]` has class `active`
2. `/wallet` → `footer.mobile-footer a.active` does not exist (the canonical Dashboard preset omits Wallet, and `MobileFooter` no longer hard-codes pins; this is the explicit decision per Open Question 1's default — update the assertion if Open Question 1 is resolved differently)
3. `/profile/abc` → `footer.mobile-footer a[href="/profile"]` has class `active` (prefix match)
4. `/chat` → `header.site_header h1` text == `"Chat"` (sanity check that `SiteHeader` is wired with the `Underscore` spelling)
5. Selector smoke: on `/wallet`, all of `aside.right-sidebar`, `aside.left-sidebar`, `header.site-header.header-wallet`, `footer.mobile-footer` resolve

**Acceptance for Phase 5:**

- [ ] `npx playwright test layout.spec.ts` passes against the Rust mock backend
- [ ] No new fixtures (reuse the wallet test user)

---

## Open Questions

1. **Mobile-footer canonical nav.** The 11 existing copies disagree on which 5 items appear. The two distinct nav lists in production today are:
   - **Dashboard preset** (Home / Search / New Post / Alerts / Profile) — labelled, `nav-item` class.
   - **Settings preset** (Home / Chat / New Post / Wallet / Profile) — unlabelled, `mobile-nav-item` class.

   Default in this plan: ship the Dashboard preset as the canonical shared nav. **This is a UX regression for Settings**, which loses Wallet and Chat shortcuts. Resolution before Phase 2: either (a) accept the regression and document it, (b) extend the canonical nav to 6 items (add Wallet), or (c) make the nav list a prop until design ships a final answer. Owner: design.
2. **Settings drift — known behaviour change, not an open question.** Grep of [style/](../../..//style/) shows only `.nav-item` styled (in [dashboard.scss](../../..//style/dashboard.scss#L946)); `mobile-nav-item` is unstyled. Settings's mobile footer is therefore **silently unstyled today** and Phase 2 will fix it as a side effect. Call out as a behaviour change in the Phase 4 changelog entry.
3. **`SiteHeader` class spelling — resolved by audit.** Grep of [style/](../../..//style/) shows the hyphenated `.site-header.header-{slug}` pattern is the broadly-styled form ([dashboard.scss](../../..//style/dashboard.scss#L48), and the `header-{slug}` modifier is the load-bearing selector for Wallet / My Ads / Referral Board / Settings / Version History). The underscore `.site_header` is scoped under `.chat` ([chat.scss](../../..//style/chat.scss#L20)) and used by Chat / Profile / View Profile / Peer Shop. `SiteHeader` therefore exposes a `HeaderSpelling` enum (default `Hyphen`) and a required-when-`Hyphen` `modifier` prop.
4. **`peer_shop.rs` and `admin.rs`.** Both have bespoke layouts that may not fit `SiteShell`'s `header / left / main / right / footer` rectangle. Audit during Phase 1; if either resists adoption, defer it to Pass 2 / a follow-up plan and note the exception in the convergence tracker.

---

## Risks

| Risk | Mitigation |
|------|------------|
| Visual regression on a page with bespoke chrome (Admin, Peer Shop) | Phase 2 audits each page individually; pages that resist adoption stay on their bespoke implementation and are noted in the changelog. |
| `MobileFooter` route-active class breaks a CSS selector that depends on `data-active` or similar | Grep `style/` for `.active`, `[data-active]`, and `aria-current` before merging Phase 1. The contract is "match what one of the existing copies does" — pick the most-correct one. |
| Hydration warning on `MobileFooter` when SSR'd `pathname` differs from hydrate-time `window.location` | Use a reactive class binding (`class:active=move || is_active("/dashboard")`) so Leptos re-renders the attribute after hydration instead of comparing baked strings. Precedent: [main_menu.rs](../../..//src/components/widgets/main_menu.rs). |
| `#[slot]` macro shape differs from the version pinned in [Cargo.toml](../../..//Cargo.toml) | Verify in Phase 1; fall back to `ViewFn`-typed props (`header: Option<ViewFn>`, …) if needed. |
| Pass 2 widens the diff and slows review | Pass 2 is explicitly out-of-scope for this sprint; only Pass 1 + the new components are required to flip the ❌ rows. |

---

## Definition of Done

- [ ] Phase 1: 5 layout components compiled and exported, no consumers yet
- [ ] Phase 2: 11 sites migrated to `SiteShell` (10 `fn MobileFooter` deletions + `peer_shop.rs`'s inline footer); 0 private `MobileFooter` fns remain; `cargo leptos build` and `cargo build --features ssr` both clean
- [ ] Phase 4: convergence tracker updated; Header / Footer / Sidebars rows now ✅; Settings nav behaviour change noted in changelog
- [ ] Phase 5: `layout.spec.ts` green
- [ ] `cargo clippy --all-features -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] Manual smoke of every page that was migrated against `cargo leptos watch` + Rust mock backend
- [ ] Open Question 1 (canonical nav) resolved and documented in the changelog entry

---

## File Manifest (Estimated)

**New (≈180 LOC across 6 files):**

- `src/components/layout/mod.rs`
- `src/components/layout/site_shell.rs`
- `src/components/layout/site_header.rs`
- `src/components/layout/mobile_footer.rs`
- `src/components/layout/left_rail.rs`
- `src/components/layout/right_rail.rs`
- `end2end/tests/layout.spec.ts` (≈80 LOC)

**Modified (12 page files; net **negative** LOC after Pass 1):**

- `src/components/mod.rs`
- `src/pages/{dashboard,wallet,chat,settings,view_profile,version_history,profile,my_ads,peer_shop,referral_board,admin,new_post}.rs`
- `docs/feature-convergence.md`

**Deleted markup:** 10 private `fn MobileFooter` blocks (~25 LOC each → ~250 LOC) + `peer_shop.rs`'s inline footer; inline `<aside>` wrappers and `<header>` blocks where Pass 2 is applied.

**Net delta:** ~+260 LOC new, ~−500 LOC deleted, **≈ −240 LOC overall** with strictly more test coverage, the latent `MobileFooter` active-route bug fixed, and Settings's silently-unstyled mobile footer rendered correctly for the first time.
