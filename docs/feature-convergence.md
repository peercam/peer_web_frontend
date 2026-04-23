# Feature Convergence Tracker

This document tracks the progress of migrating features from the legacy PHP/JS frontend to the new Leptos (Rust/WASM) rewrite.

**Last Updated:** 2026-04-23 (Dashboard — completion sprint plan published; reconciles 3 already-resolved Known Issues)

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Implemented | 15 |
| 🟡 Near-Complete | 5 |
| 🚧 In Progress | 1 |
| ❌ Not Started | 0 |
| **Total** | **21** |

**Convergence:** 15/21 pages implemented (~71%); 20/21 ≥ near-complete (~95%).

See [CHANGELOG.md](../CHANGELOG.md) for the dated history of how these counts evolved.

---

## Pages

| Feature | Legacy File | peer-web Status | Notes |
|---------|-------------|-----------------|-------|
| **Landing** ||||
| Home / Landing | `index.php` | ✅ Implemented | Auth-aware redirect node at `/` (`src/app.rs::HomePage`): authed → `/dashboard`, guest → `/login?message=mustLogin` (preserves inbound `?redirect=…`). Matches legacy `index.php`'s server-side 302 chain (`/` → `dashboard.php` → `login.php?message=…`). Loading sentinel reuses the shared `auth-guard-loading` selector to avoid placeholder flash during the session-check window. Percent-encoding factored into shared [`encode_redirect`](../src/state/auth.rs) helper used by both `HomePage` and `AuthGuard` ([docs](plans/home/home-implementation.md), [changelog](../CHANGELOG.md#2026-04-22--home--landing-layout-shell-doc-reconciliation)). |
| **Authentication** ||||
| Login | `login.php` | ✅ Implemented | Email/password, remember-me, auto-login, redirect handling ([docs](plans/login/login-auth-implementation.md), [changelog](../CHANGELOG.md#2026-04-10--foundations)) — Plan quality: ⭐⭐⭐⭐ (4/5) |
| Register | `register.php` | ✅ Implemented | Multi-step: referral → email → password → confirmation ([changelog](../CHANGELOG.md#2026-04-10--foundations)) |
| Forgot Password | `forgotpassword.php` | ✅ Implemented | 990-line page, full 4-step flow (email → verify → reset → success), rate-limited resend (60s → 10min → locked), cookie-persisted resend counter (2h TTL), countdown interval no longer stacks on rapid resends, shared `BackButton` reused, auto-redirect for authed users (`<Show>` + `<Redirect>`), mock-backend `/debug/reset-token` endpoint + 3 cross-cutting tests, Playwright coverage (8 cases) ([docs](plans/forgot-password/forgot-password-implementation.md), [sprint](plans/forgot-password/forgot-password-completion-sprint.md), [changelog](../CHANGELOG.md#2026-04-23--forgot-password-tasks-56-e2e--debug-endpoint)). | |
| **Core Features** ||||
| Dashboard | `dashboard.php` | 🟡 Mostly Implemented | Post feed, filters, sort, infinite scroll ([docs](plans/dashboard/dashboard-implementation.md), [sprint](plans/dashboard/dashboard-completion-sprint.md)) — Plan quality: ⭐⭐⭐⭐ (4/5). **Gap:** post-card click → View Post navigation is a TODO stub (sprint Task 1). 3 of the 4 originally-recorded Known Issues (user-search hydrate imports, ProfileWidget wiring, `LIST_POSTS_QUERY` field completeness) verified resolved on 2026-04-23. |
| View Post | `post.php` | 🟡 ~95% Implemented | Single post view, comments, guest mode ([docs](plans/view-post/view-post-implementation.md), [sprint](plans/view-post/view-post-completion-sprint.md)) |
| New Post | `newpost.php` | 🚧 In Progress | Text/media creation, image cropping, video encoding ([docs](plans/new-post/new-post-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5) |
| **Profile** ||||
| My Profile | `profile.php` | ✅ Implemented | Profile header, post list with infinite scroll, filter sidebar, auth guard, `?user=` redirect, `use_infinite_scroll` hook ([docs](plans/profile/profile-implementation.md), [sprint](plans/profile/profile-completion-sprint.md), [changelog](../CHANGELOG.md#2026-04-16--admin-dashboard-mock-backend-phases-5--6-version-history-profile-sprint)) |
| View Profile | `view-profile.php` | ✅ Implemented | Slug route, follow/block/report actions, infinite scroll, filter sidebar, relations modal with pagination ([docs](plans/profile/profile-implementation.md), [changelog](../CHANGELOG.md#2026-04-16--admin-dashboard-mock-backend-phases-5--6-version-history-profile-sprint)) |
| Edit Profile | `edit_profile.php` | ✅ Implemented | Redirect routes (`/edit-profile`, `/edit_profile`) → `/settings` ([sprint](plans/profile/profile-completion-sprint.md)) |
| Settings | `profileSettings.php` | ✅ Implemented | 142-line page + 8 sub-components (profile, passwords, email, username, content, notifications, preferences, deactivate), race-free bio+image save, API layer (282L) ([docs](plans/settings/settings-implementation.md), [changelog](../CHANGELOG.md#2026-04-16--admin-dashboard-mock-backend-phases-5--6-version-history-profile-sprint)) |
| **Social** ||||
| Chat | `chat.php` | 🟡 Core Implemented | 141-line page + 7 components (chat_list, contacts_overlay, group_review, chat_input, chat_messages, chat_container, chat_item), API layer (151L), state module, SCSS (851L) ([docs](plans/chat/chat-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5). **Gaps:** Firebase real-time listener missing (no polling fallback), unread indicators missing, chat search logic not connected |
| Invite | `invite.php` | ✅ Implemented | Deep-link relay page: platform detection, `peer://invite/{uuid}` deep link, app store / registration fallback, localStorage persistence, clipboard copy ([docs](plans/invite/invite-implementation.md), [changelog](../CHANGELOG.md#2026-04-14--invite-referral-board-mock-backend-phases-1-4-planning-blitz)) |
| Referral Board | `referralBoard.php` | ✅ Implemented | Referral link + copy, invited/inviter tabs, user cards, auth guard ([docs](plans/referral-board/referral-board-implementation.md), [changelog](../CHANGELOG.md#2026-04-14--invite-referral-board-mock-backend-phases-1-4-planning-blitz)) |
| **Economy** ||||
| Wallet | `wallet.php` | ✅ Implemented | 146-line page + transfer_modal (760L), balance_header, transaction_history, transaction_item with lazy-loaded shop delivery panel (Peer Shop account only), `format_balance()` thousand separators (4dp rounding parity with legacy `toLocaleString`), API layer (246L), SCSS (1092L), Playwright `wallet.spec.ts` (6 cases incl. lazy-load network assertion) ([docs](plans/wallet/wallet-implementation.md), [sprint](plans/wallet/wallet-completion-sprint.md), [changelog](../CHANGELOG.md#2026-04-22--wallet-promoted-to-)) |
| Peer Shop | `viewPeerShop.php` | 🟡 Core Implemented | `/shop` route, profile header, product feed with price badges, checkout popup (multi-step), FAQ popup, `performShopOrder` API, SCSS ([docs](plans/peer-shop/peer-shop-implementation.md)) — **Gaps:** Firebase product data (sizes/stock), infinite scroll, View Post overlay integration, functional filters |
| My Ads | `myAds.php` | 🟡 In Progress | 144-line page + stats header, ad listing with infinite scroll, boost post modal (multi-step), `advertisementHistory` query, `advertisePostPinned` mutation, skeleton loading, staggered animations ([docs](plans/my-ads/my-ads-implementation.md)). **Gaps:** Boost modal not wired to Profile page button, success/error toast not wired on error path, Basic (time-based) ad flow not started |
| **Admin** ||||
| Admin Dashboard | `admin/index.php` | ✅ Implemented | Content moderation — role-gated, stats header, filterable ticket list, expandable detail, moderation actions, infinite scroll, 1,044L SCSS ([docs](plans/admin/admin-dashboard-implementation.md), [changelog](../CHANGELOG.md#2026-04-16--admin-dashboard-mock-backend-phases-5--6-version-history-profile-sprint)) |
| **Misc** ||||
| Download | `download.php` | ✅ Implemented | Force-download media proxy — Axum `/download` route with HTTPS-only host allow-list, streaming body, byte/time caps, RFC 5987 filename sanitisation ([docs](plans/download/download-implementation.md), [changelog](../CHANGELOG.md#2026-04-21--download-proxy-pwa-doc-accuracy)). **Note:** legacy row description said "App download page" — that was incorrect; `download.php` was always a media proxy, and this port preserves that behaviour.
| Version History | `version_history.php` | ✅ Implemented | Release notes ([docs](plans/version-history/version-history-implementation.md), [changelog](../CHANGELOG.md#2026-04-16--admin-dashboard-mock-backend-phases-5--6-version-history-profile-sprint)) |
| 404 Page | `404.php` | ✅ Implemented | Fallback route in router |

---

## Components

| Component | Legacy Location | peer-web Status | Notes |
|-----------|----------------|-----------------|-------|
| **Layout** ||||
| Header | `template-parts/` | ✅ Implemented | `SiteHeader` component (`src/components/layout/site_header.rs`) — Hyphen / Underscore spelling, optional icon + actions slot. Adopted opportunistically (Pass 2); Pass 1 wraps existing inline headers in [`SiteShell`](../src/components/layout/site_shell.rs) ([docs](plans/layout/layout-shell-implementation.md), [changelog](../CHANGELOG.md#2026-04-22--home--landing-layout-shell-doc-reconciliation)) |
| Footer | `template-parts/footer.php` | ✅ Implemented | Route-aware `MobileFooter` (`src/components/layout/mobile_footer.rs`) — single shared component replaces 10 private `fn MobileFooter` copies + 1 inline `<footer>`; reactive `active` class against `use_location()`. Auto-rendered by [`SiteShell`](../src/components/layout/site_shell.rs). **Behaviour change:** Settings's previously unstyled `mobile-nav-item` markup is now the canonical Dashboard preset (Home / Search / New Post / Alerts / Profile) — see Open Question 1 in [the plan](plans/layout/layout-shell-implementation.md). |
| Sidebars | `template-parts/sidebars/` | ✅ Implemented | `LeftRail` / `RightRail` wrappers + `StandardRightRail` widget-stack preset (`src/components/layout/{left_rail,right_rail}.rs`) reusing the shared `AddPostButton` widget; page-specific filter sidebars stay page-local ([docs](plans/layout/layout-shell-implementation.md)) |
| **Posts** ||||
| Post Card | `js/posts.js` | 🚧 In Progress | 304-line component with like/dislike/save actions, view tracking |
| Post List | `js/load_posts.js` | 🚧 In Progress | 209 lines, infinite scroll, ad interleaving, filter integration |
| Comments | `js/comments.js` | 🚧 In Progress | 509 lines, nested replies, create, like/unlike, infinite scroll |
| **Chat** ||||
| Chat List | `js/chat/` | 🚧 In Progress | 166 lines, tabs (private/group), chat items |
| Chat Window | `js/chat/` | 🚧 In Progress | chat_container (85L) + chat_messages (80L) + chat_input (124L) |
| **Forms** ||||
| Login Form | `js/login/login.js` | ✅ Implemented | Email/password, remember-me, validation |
| Registration Form | `js/register/` | ✅ Implemented | Multi-step form |
| Password Strength | — | ✅ Implemented | New component |
| Form Validation | — | ✅ Implemented | Client + server validation |
| **Auth** ||||
| Auth Guard | — | ✅ Implemented | Protected routes, redirect preservation |
| Auth Context | — | ✅ Implemented | Global auth state, actions |
| Left Panel | — | ✅ Implemented | Shared login/register layout |
| **Media** ||||
| Image Cropper | `js/crop.js` | ✅ Implemented | Canvas draw + drag + scroll-zoom (0.3–8×) + 1:1/4:5 toggle, 1080px PNG data-URL output (`image_cropper.rs`) |
| Audio Player | `js/audio.js` | 🟡 Mostly Implemented | MediaRecorder + chunked capture + preview `<audio>` + timer + play/pause + reset wired; real-time waveform pending — see [sprint](plans/new-post/new-post-completion-sprint.md) Task 2 |
| Video Encoder | `js/ffmpeg/` | 🟡 Mostly Implemented | Trimmer timeline + draggable handles (`MIN_DURATION` clamp) + cover wired; frame thumbnails + server-side trim plumbing pending, FFmpeg WASM intentionally deferred — see [sprint](plans/new-post/new-post-completion-sprint.md) Tasks 3–5 |
| **UI** ||||
| Modal | `js/lib/modal.js` | 🚧 In Progress | image_modal (settings 151L, view_post 74L), share_modal (138L), relations_modal (271L), transfer_modal (760L) |
| Toast | — | ✅ Implemented | Notification toasts |
| Install Prompt | — | ✅ Implemented | PWA install banner + iOS hint + update-available banner (`src/components/pwa.rs`) ([changelog](../CHANGELOG.md#2026-04-21--download-proxy-pwa-doc-accuracy)) |
| Back Button | — | ✅ Implemented | Navigation |
| Step Announcer | — | ✅ Implemented | Accessibility |

---

## API Layer

| API Module | Legacy Location | peer-web Status | Notes |
|------------|----------------|-----------------|-------|
| GraphQL Client | `js/lib/const.js` | ✅ Implemented | `src/api/graphql.rs` |
| Auth (JWT) | `auth.php` | ✅ Implemented | Server fns, HttpOnly cookies, proactive refresh, 401 interceptor. Mock backend: [Phase 1 plan](plans/mock-backend/phase-1-login-session-flows.md) — ⭐⭐⭐⭐⭐ (5/5) |
| Registration | `js/register/` | ✅ Implemented | Verify referral, register user |
| Posts | `js/posts.js` | 🚧 In Progress | `src/api/posts.rs` (409L) — list_posts, list_ad_posts, post_action, create_post, search_tags |
| Comments | `js/comments.js` | 🚧 In Progress | `src/api/comments.rs` (204L) — get_post, guest_get_post, list_comments, create_comment, like/unlike. Mock backend: Phase 4 complete |
| Chat | `js/chat/api.js` | 🚧 In Progress | `src/api/chat.rs` (151L) — list_chats, send_message, create_chat. Mock backend: Phase 4 complete |
| Wallet | `js/wallet.js` | 🚧 In Progress | `src/api/wallet.rs` (246L) — get_balance, transaction_history, transfer_tokens |
| Referral | `js/referral.js` | ✅ Implemented | `src/api/referral.rs` (65L) — get_referral_info, get_referral_list |
| Firebase / Real-time chat transport | `js/firebase_config.js` | ❌ Not Started | Client row renamed from "Firebase" — the vendor choice is now transport-agnostic. v1 ships **polling** against GraphQL; Firestore/WS/SSE tracked in [ADR](adr-chat-realtime-transport.md). Backend write path is Postgres, not Firestore ([chat-completion-sprint.md § Blocker Resolution](plans/chat/chat-completion-sprint.md#blocker-resolution-2026-04-21)) |

---

## Infrastructure

| Feature | Legacy | peer-web Status | Notes |
|---------|--------|-----------------|-------|
| SSR | PHP | ✅ Implemented | Leptos SSR + hydration |
| Routing | PHP files | ✅ Implemented | leptos_router |
| State Management | Global JS vars | ✅ Implemented | Leptos signals, AuthContext |
| Build System | None | ✅ Implemented | cargo-leptos |
| CSS | Plain CSS | ✅ Implemented | SCSS |
| PWA / Manifest | `json/webmanifest.json` | ✅ Implemented | Full manifest + hand-written service worker (offline shell, per-route caching, update toast), install prompt + iOS Add-to-Home-Screen hint ([docs](plans/pwa/pwa-implementation.md), [changelog](../CHANGELOG.md#2026-04-21--download-proxy-pwa-doc-accuracy)) |
| E2E Tests | None | 🚧 In Progress | Playwright setup |

---

## Mock Backend (Rust Rewrite)

Tracks the incremental Rust mock backend that replaces the Node.js mock for offline E2E development. Parent plan: [mock-backend-rust-rewrite.md](plans/mock-backend/mock-backend-rust-rewrite.md). Per-phase landing dates and detailed deltas live in [CHANGELOG.md](../CHANGELOG.md).

| Phase | Scope | Plan | Quality | Status |
|-------|-------|------|---------|--------|
| 0 — Skeleton & Parity | 3 registration mutations, health check, 9 integration tests | [phase-0](plans/mock-backend/phase-0-mock-backend-skeleton.md) | ⭐⭐⭐⭐ (4/5) | ✅ Done — Node.js replaced, async-graphql v7 + axum 0.8 |
| 1 — Login & Session | login, refreshToken, logout, deleteAccount, updatePassword, password reset, contactus — 9 mutations, auth middleware, 24 tests | [phase-1](plans/mock-backend/phase-1-login-session-flows.md) | ⭐⭐⭐⭐⭐ (5/5) | ✅ Done — 33 total tests, 2 seeded users, `CurrentUser` context |
| 2 — Users & Profiles | 10 queries + 8 mutations: getProfile, searchUser, listUsersV2, getUser, follow/block/report, preferences, profile edits, referrals — 40 new tests (73 total), 6 seeded users, content filtering pipeline | [phase-2](plans/mock-backend/phase-2-users-and-profiles.md) | ⭐⭐⭐⭐⭐ (5/5) | ✅ Done — [implementation notes](plans/mock-backend/phase-2-implementation.md) |
| 3 — Posts & Content | listPosts, guestListPost, postAction, createPost, searchTags, ads | [phase-3](plans/mock-backend/phase-3-posts-content.md) | ⭐⭐⭐⭐⭐ (5/5) | ✅ Done — 121 total tests (48 new), 0 clippy warnings |
| 4 — Social (Comments, Chat) | listComments, createComment, like/unlike, listChats, sendMessage, createChat | [phase-4](plans/mock-backend/phase-4-social-comments-chat.md) | ⭐⭐⭐⭐⭐ (5/5) | ✅ Done — 170 total tests (49 new), 0 clippy warnings |
| 5 — Economy (Wallet, Shop, Ads) | balance, transferTokens, transactionHistory, shopOrderDetails, ads | [phase-5](plans/mock-backend/phase-5-economy-wallet-tokenomics-shop-ads.md) | ⭐⭐⭐⭐⭐ (5/5) | ✅ Done — 219 total tests (49 new), 0 clippy warnings |
| 6 — Admin & Moderation | RBAC, moderation tickets/stats, hide/restore/illegal actions, admin user search, leaderboard, gem/mint ops | [phase-6](plans/mock-backend/phase-6-admin-moderation.md) | ⭐⭐⭐⭐⭐ (5/5) | ✅ Done — 266 total tests (47 new), 0 clippy warnings |
| Integration Test Refactor | Split monolithic 7,675L `integration.rs` into 18 per-domain test files + shared `common/` helpers (assertions, auth, client, fragments, state) | [refactor](plans/mock-backend/phase-integration-test-refactor.md) | ⭐⭐⭐⭐⭐ (5/5) | ✅ Done — 266 tests preserved, 18 files (7,391L), build clean |
| CI — Integration | Build/test pipeline, Leptos E2E integration, schema snapshots, Node.js replacement | [phase-ci](plans/mock-backend/phase-ci-integration.md) | ⭐⭐⭐⭐⭐ (5/5) | ❌ Not Started |
| Acceptance Criteria | Per-phase gates, cross-cutting quality reqs, automated verification, sign-off checklist | [acceptance](plans/mock-backend/acceptance-criteria.md) | ⭐⭐⭐⭐⭐ (5/5) | ❌ Not Started |

---

## Migration Priority (Recommended Order)

1. ✅ ~~Registration~~ — Complete
2. ✅ ~~Login / Auth~~ — Complete ([docs](plans/login/login-auth-implementation.md))
3. 🟡 Dashboard — Mostly implemented; only the post-card click handler remains, plus a `dashboard.spec.ts` E2E pass ([docs](plans/dashboard/dashboard-implementation.md), [sprint](plans/dashboard/dashboard-completion-sprint.md))
4. ✅ ~~Forgot Password~~ — Complete, all 6 sprint tasks done: auto-redirect, cookie-persisted resend counter, interval stacking fix, `BackButton` reuse, mock-backend `/debug/reset-token` + integration tests, Playwright spec (8 cases) ([docs](plans/forgot-password/forgot-password-implementation.md), [sprint](plans/forgot-password/forgot-password-completion-sprint.md))
5. 🟡 View Post — ~95% implemented, polish & testing remain ([docs](plans/view-post/view-post-implementation.md))
6. ✅ ~~Profile~~ — Implemented, infinite scroll + filters + `use_infinite_scroll` hook ([docs](plans/profile/profile-implementation.md), [sprint](plans/profile/profile-completion-sprint.md))
7. 🚧 New Post — In progress, ~70% structural ([docs](plans/new-post/new-post-implementation.md))
8. 🟡 Chat — Core implemented, Firebase real-time missing ([docs](plans/chat/chat-implementation.md))
9. ✅ ~~Wallet~~ — Implemented, shop delivery panel + thousand-separator formatting + E2E coverage ([docs](plans/wallet/wallet-implementation.md), [sprint](plans/wallet/wallet-completion-sprint.md))
10. ✅ ~~Settings~~ — Implemented, deactivate account UI wired, save race condition fixed ([docs](plans/settings/settings-implementation.md))
11. ✅ ~~Referral Board~~ — Complete, full UI + API, mock backend endpoints available (Phase 2) ([docs](plans/referral-board/referral-board-implementation.md))
12. 🟡 My Ads — In progress (Phases 1–4 done), boost modal wiring + basic ad flow remaining ([docs](plans/my-ads/my-ads-implementation.md))
13. ✅ ~~Invite~~ — Complete, deep-link relay page ([docs](plans/invite/invite-implementation.md))
14. 🟡 Peer Shop — Core implemented (Phases 1–4), Firebase integration + polish remaining ([docs](plans/peer-shop/peer-shop-implementation.md))
15. ✅ ~~Admin~~ — Implemented, role-gated moderation dashboard: stats header, filterable ticket list (All/Posts/Comments/Accounts), expandable detail with content previews, moderation actions (hide/restore/illegal) with confirmations, infinite scroll, 2,514 lines across 15 files + 1,044L SCSS ([docs](plans/admin/admin-dashboard-implementation.md))
16. ✅ ~~Version History~~ — Complete, auth-guarded two-panel layout, static JSON fetch, responsive styles ([docs](plans/version-history/version-history-implementation.md))
17. ✅ ~~PWA~~ — Complete, manifest + service worker + install prompt + offline shell + update toast ([docs](plans/pwa/pwa-implementation.md))
18. ✅ ~~Download~~ — Force-download media proxy: HTTPS-only host allow-list, streaming body, byte + time caps, sanitised `Content-Disposition` ([docs](plans/download/download-implementation.md))
19. ✅ ~~Layout shell~~ — Shared `SiteShell` / `SiteHeader` / `MobileFooter` / `LeftRail` / `RightRail` (+ `StandardRightRail` preset). Closes Header / Footer / Sidebars rows; deletes 10 private `fn MobileFooter` copies + 1 inline `<footer>`; route-aware mobile-footer active class via `use_location()` ([docs](plans/layout/layout-shell-implementation.md))
20. ✅ ~~Home / Landing~~ — Auth-aware redirect at `/`: authed → `/dashboard`, guest → `/login?message=mustLogin` (with inbound `?redirect=…` passthrough); shared `encode_redirect` helper now used by both `HomePage` and `AuthGuard` ([docs](plans/home/home-implementation.md))

---

## Changelog

The dated, narrative history of this tracker has moved to the
repository-root **[CHANGELOG.md](../CHANGELOG.md)**. New status changes
should:

1. Update the relevant table row(s) above (status glyph, notes, plan link).
2. Update the **Summary** counts and **Last Updated** date at the top.
3. Add a dated entry to [CHANGELOG.md](../CHANGELOG.md) under the appropriate
   section (`Added`, `Changed`, `Removed`, `Fixed`, `Security`,
   `Documentation`, `Deferred`, `Notes`).
4. Where useful, add a `[changelog](../CHANGELOG.md#…)` link to the row's
   Notes column so readers can jump from the matrix to the rationale.

This split keeps the tracker scannable as a status board while preserving
the full migration narrative in the conventional Keep-a-Changelog location.
