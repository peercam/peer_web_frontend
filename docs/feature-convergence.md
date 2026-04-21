# Feature Convergence Tracker

This document tracks the progress of migrating features from the legacy PHP/JS frontend to the new Leptos (Rust/WASM) rewrite.

**Last Updated:** 2026-04-21

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Implemented | 11 |
| 🟡 Near-Complete | 6 |
| 🚧 In Progress | 2 |
| ❌ Not Started | 1 |
| **Total** | **20** |

**Convergence:** ~82%

---

## Pages

| Feature | Legacy File | peer-web Status | Notes |
|---------|-------------|-----------------|-------|
| **Authentication** ||||
| Login | `login.php` | ✅ Implemented | Email/password, remember-me, auto-login, redirect handling ([docs](plans/login/login-auth-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5) |
| Register | `register.php` | ✅ Implemented | Multi-step: referral → email → password → confirmation |
| Forgot Password | `forgotpassword.php` | 🚧 In Progress | 990-line page, full 4-step flow (email → verify → reset → success), rate-limited resend, API fns implemented ([docs](plans/forgot-password/forgot-password-implementation.md), [sprint](plans/forgot-password/forgot-password-completion-sprint.md)). **Gaps:** no auto-redirect for authenticated users, resend counter not cookie-persisted (cooldown bypass on reload), countdown interval stacking bug, shared `BackButton` component exists but not wired in |
| **Core Features** ||||
| Dashboard | `dashboard.php` | 🟡 Mostly Implemented | Post feed, filters, sort, infinite scroll ([docs](plans/dashboard/dashboard-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5). **Gaps:** post click → view post overlay is a TODO stub |
| View Post | `post.php` | 🟡 ~95% Implemented | Single post view, comments, guest mode ([docs](plans/view-post/view-post-implementation.md), [sprint](plans/view-post/view-post-completion-sprint.md)) |
| New Post | `newpost.php` | 🚧 In Progress | Text/media creation, image cropping, video encoding ([docs](plans/new-post/new-post-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5) |
| **Profile** ||||
| My Profile | `profile.php` | ✅ Implemented | Profile header, post list with infinite scroll, filter sidebar, auth guard, `?user=` redirect, `use_infinite_scroll` hook ([docs](plans/profile/profile-implementation.md), [sprint](plans/profile/profile-completion-sprint.md)) |
| View Profile | `view-profile.php` | ✅ Implemented | Slug route, follow/block/report actions, infinite scroll, filter sidebar, relations modal with pagination ([docs](plans/profile/profile-implementation.md)) |
| Edit Profile | `edit_profile.php` | ✅ Implemented | Redirect routes (`/edit-profile`, `/edit_profile`) → `/settings` ([sprint](plans/profile/profile-completion-sprint.md)) |
| Settings | `profileSettings.php` | ✅ Implemented | 142-line page + 8 sub-components (profile, passwords, email, username, content, notifications, preferences, deactivate), race-free bio+image save, API layer (282L) ([docs](plans/settings/settings-implementation.md)) |
| **Social** ||||
| Chat | `chat.php` | 🟡 Core Implemented | 141-line page + 7 components (chat_list, contacts_overlay, group_review, chat_input, chat_messages, chat_container, chat_item), API layer (151L), state module, SCSS (851L) ([docs](plans/chat/chat-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5). **Gaps:** Firebase real-time listener missing (no polling fallback), unread indicators missing, chat search logic not connected |
| Invite | `invite.php` | ✅ Implemented | Deep-link relay page: platform detection, `peer://invite/{uuid}` deep link, app store / registration fallback, localStorage persistence, clipboard copy ([docs](plans/invite/invite-implementation.md)) |
| Referral Board | `referralBoard.php` | ✅ Implemented | Referral link + copy, invited/inviter tabs, user cards, auth guard ([docs](plans/referral-board/referral-board-implementation.md)) |
| **Economy** ||||
| Wallet | `wallet.php` | 🟡 Implemented (tests pending) | 146-line page + transfer_modal (760L), balance_header, transaction_history, transaction_item, API layer (246L), SCSS (1092L) ([docs](plans/wallet/wallet-implementation.md)). **Gaps:** shop purchase order details UI not wired (model + query exist), thousand-separator formatting missing |
| Peer Shop | `viewPeerShop.php` | 🟡 Core Implemented | `/shop` route, profile header, product feed with price badges, checkout popup (multi-step), FAQ popup, `performShopOrder` API, SCSS ([docs](plans/peer-shop/peer-shop-implementation.md)) — **Gaps:** Firebase product data (sizes/stock), infinite scroll, View Post overlay integration, functional filters |
| My Ads | `myAds.php` | 🟡 In Progress | 144-line page + stats header, ad listing with infinite scroll, boost post modal (multi-step), `advertisementHistory` query, `advertisePostPinned` mutation, skeleton loading, staggered animations ([docs](plans/my-ads/my-ads-implementation.md)). **Gaps:** Boost modal not wired to Profile page button, success/error toast not wired on error path, Basic (time-based) ad flow not started |
| **Admin** ||||
| Admin Dashboard | `admin/index.php` | ✅ Implemented | Content moderation — role-gated, stats header, filterable ticket list, expandable detail, moderation actions, infinite scroll, 1,044L SCSS ([docs](plans/admin/admin-dashboard-implementation.md)) |
| **Misc** ||||
| Download | `download.php` | ❌ Not Started | App download page |
| Version History | `version_history.php` | ✅ Implemented | Release notes ([docs](plans/version-history/version-history-implementation.md)) |
| 404 Page | `404.php` | ✅ Implemented | Fallback route in router |

---

## Components

| Component | Legacy Location | peer-web Status | Notes |
|-----------|----------------|-----------------|-------|
| **Layout** ||||
| Header | `template-parts/` | ❌ Not Started | Not extracted as a shared component; pages render navigation/user menu inline |
| Footer | `template-parts/footer.php` | ❌ Not Started | Not extracted as a shared component; pages render footer inline (or omit) |
| Sidebars | `template-parts/sidebars/` | ❌ Not Started | Filter sidebars implemented per-page (dashboard, profile, view-profile); no shared component |
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
| Image Cropper | `js/crop.js` | 🚧 In Progress | 226 lines, canvas-based cropping (some stubs remain) |
| Audio Player | `js/audio.js` | 🚧 In Progress | audio_upload (236L) + voice_recorder (159L), MediaRecorder stubs |
| Video Encoder | `js/ffmpeg/` | 🚧 In Progress | video_upload (180L) + video_trimmer (189L) + video_cover (102L), FFmpeg WASM stubs |
| **UI** ||||
| Modal | `js/lib/modal.js` | 🚧 In Progress | image_modal (settings 151L, view_post 74L), share_modal (138L), relations_modal (271L), transfer_modal (760L) |
| Toast | — | ✅ Implemented | Notification toasts |
| Install Prompt | — | ✅ Implemented | PWA install banner + iOS hint + update-available banner (`src/components/pwa.rs`) |
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
| Firebase | `js/firebase_config.js` | ❌ Not Started | Real-time, analytics |

---

## Infrastructure

| Feature | Legacy | peer-web Status | Notes |
|---------|--------|-----------------|-------|
| SSR | PHP | ✅ Implemented | Leptos SSR + hydration |
| Routing | PHP files | ✅ Implemented | leptos_router |
| State Management | Global JS vars | ✅ Implemented | Leptos signals, AuthContext |
| Build System | None | ✅ Implemented | cargo-leptos |
| CSS | Plain CSS | ✅ Implemented | SCSS |
| PWA / Manifest | `json/webmanifest.json` | ✅ Implemented | Full manifest + hand-written service worker (offline shell, per-route caching, update toast), install prompt + iOS Add-to-Home-Screen hint ([docs](plans/pwa/pwa-implementation.md)) |
| E2E Tests | None | 🚧 In Progress | Playwright setup |

---

## Mock Backend (Rust Rewrite)

Tracks the incremental Rust mock backend that replaces the Node.js mock for offline E2E development. Parent plan: [mock-backend-rust-rewrite.md](plans/mock-backend/mock-backend-rust-rewrite.md)

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
3. 🟡 Dashboard — Mostly implemented, view-post overlay stub remains ([docs](plans/dashboard/dashboard-implementation.md))
4. 🚧 Forgot Password — In progress, 990-line page implemented, completion sprint created ([docs](plans/forgot-password/forgot-password-implementation.md), [sprint](plans/forgot-password/forgot-password-completion-sprint.md))
5. 🟡 View Post — ~95% implemented, polish & testing remain ([docs](plans/view-post/view-post-implementation.md))
6. ✅ ~~Profile~~ — Implemented, infinite scroll + filters + `use_infinite_scroll` hook ([docs](plans/profile/profile-implementation.md), [sprint](plans/profile/profile-completion-sprint.md))
7. 🚧 New Post — In progress, ~70% structural ([docs](plans/new-post/new-post-implementation.md))
8. 🟡 Chat — Core implemented, Firebase real-time missing ([docs](plans/chat/chat-implementation.md))
9. 🟡 Wallet — Implemented (tests pending), shop order UI not wired ([docs](plans/wallet/wallet-implementation.md))
10. ✅ ~~Settings~~ — Implemented, deactivate account UI wired, save race condition fixed ([docs](plans/settings/settings-implementation.md))
11. ✅ ~~Referral Board~~ — Complete, full UI + API, mock backend endpoints available (Phase 2) ([docs](plans/referral-board/referral-board-implementation.md))
12. 🟡 My Ads — In progress (Phases 1–4 done), boost modal wiring + basic ad flow remaining ([docs](plans/my-ads/my-ads-implementation.md))
13. ✅ ~~Invite~~ — Complete, deep-link relay page ([docs](plans/invite/invite-implementation.md))
14. 🟡 Peer Shop — Core implemented (Phases 1–4), Firebase integration + polish remaining ([docs](plans/peer-shop/peer-shop-implementation.md))
15. ✅ ~~Admin~~ — Implemented, role-gated moderation dashboard: stats header, filterable ticket list (All/Posts/Comments/Accounts), expandable detail with content previews, moderation actions (hide/restore/illegal) with confirmations, infinite scroll, 2,514 lines across 15 files + 1,044L SCSS ([docs](plans/admin/admin-dashboard-implementation.md))
16. ✅ ~~Version History~~ — Complete, auth-guarded two-panel layout, static JSON fetch, responsive styles ([docs](plans/version-history/version-history-implementation.md))
17. ✅ ~~PWA~~ — Complete, manifest + service worker + install prompt + offline shell + update toast ([docs](plans/pwa/pwa-implementation.md))
18. ⬜ Download — App download page (no plan yet)

---

## Changelog

### 2026-04-21 (PWA Implemented)
- **PWA fully implemented** — manifest, service worker, install prompt, iOS hint, update toast, offline shell ([docs](plans/pwa/pwa-implementation.md))
- **Infrastructure table:** PWA / Manifest ❌ Not Started → ✅ Implemented
- **Components table:** added `Install Prompt` row under UI
- **Migration Priority:** added PWA at #17 (Download renumbered to #18)
- **New files (18):**
  - `peer-web/public/manifest.webmanifest` — full v1 manifest (id, scope, start_url, display_override, screenshots, shortcuts, 5 icons incl. maskable + monochrome)
  - `peer-web/public/sw.js` — hand-written service worker (~220L): install/activate/fetch, network-only for `/api/` + `/graphql` + `/admin`, network-first navigation with offline fallback, stale-while-revalidate for `/pkg/`, cache-first + 30-day expiry for static assets
  - `peer-web/public/offline.html` — branded offline fallback shell
  - `peer-web/public/img/pwa/` — 9 icons (any + maskable + monochrome + apple-touch + 2 screenshots) + 3 iOS splash PNGs
  - `peer-web/src/utils/pwa.rs` — SW registration (+ SSR no-op stub), `beforeinstallprompt` capture, update polling (visibilitychange + 60s interval), `?nosw` escape hatch, `apply_service_worker_update()` for `SKIP_WAITING`
  - `peer-web/src/components/pwa.rs` — `InstallBanner` (Install / Not-now-14d / Never), `IosHint`, `UpdateBanner`
  - `peer-web/style/pwa.scss` — bottom-sheet on mobile, top-right card on desktop, update-banner pill
  - `peer-web/tests/pwa_manifest.rs` — asserts manifest shape + `.webmanifest` MIME resolution
  - `peer-web/end2end/tests/pwa.spec.ts` — Playwright coverage: manifest MIME, controller after reload, offline fallback
- **Modified files (5):** `src/app.rs` (manifest link, Apple meta, 3 splash media queries, mount `<InstallPrompt/>`, `register_service_worker()`), `src/components/mod.rs` + `src/utils/mod.rs` (module registration), `style/main.scss` (`@use "pwa"`), `Cargo.toml` (web-sys features for `ServiceWorker*` + `MessageEvent` + `MediaQueryList` + `VisibilityState`, `hash-files = false` pinned, `mime_guess` dev-dep)
- **Build hash:** `BUILD_HASH` composed at compile time from `env!("CARGO_PKG_VERSION")` + `option_env!("GIT_SHA")`; passed to the SW via `/sw.js?v=<HASH>` query string (no `build.rs` / template substitution)
- **Builds clean** on both `cargo build --features ssr` and `cargo build --features hydrate --target wasm32-unknown-unknown`

### 2026-04-21 (Doc Accuracy Pass)
- **Integration Test Refactor row added** to Mock Backend table — discovered during plan audit that commit `c8cf7b7` (2026-04-16) split the monolithic `tests/mock_backend/tests/integration.rs` (7,675L) into 18 per-domain test files (7,391L total) plus shared `common/` helpers (`assertions.rs`, `auth.rs`, `client.rs`, `fragments.rs`, `state.rs`); all 266 tests still pass, build clean — refactor was implemented but not previously tracked
- **Summary counts corrected:** ✅ 10→11, ❌ 2→1 (Pages table actually contains 11 ✅ rows after Admin + Version History promotions on 2026-04-16; convergence ~82% was already correct)
- **Migration Priority renumbered** 1–17 (removed `3b.` duplicate numbering, every item now sequential)
- **Forgot Password gap clarified:** "BackButton not reused" → "shared `BackButton` component exists but not wired in" (avoids contradiction with Components table where BackButton is ✅ Implemented)
- **Layout components clarified:** Header/Footer/Sidebars notes now explain these are not extracted as shared components (pages render inline) rather than implying pages are missing chrome
- **Phase 3 test count:** stray `+` removed ("48+ new" → "48 new") for consistency with phases 4/5/6
- **Last Updated** bumped to 2026-04-21
- No code or implementation changes; documentation accuracy fixes only

### 2026-04-16 (Admin Dashboard Implemented)
- **Admin Dashboard fully implemented** — all 6 phases complete, promoted 🟡 → ✅ ([docs](plans/admin/admin-dashboard-implementation.md))
- **New files (15):** `src/models/moderation.rs` (191L), `src/api/moderation.rs` (102L), `src/pages/admin.rs` (101L), `src/components/admin/` (11 files, 1,470L), `style/admin.scss` (1,044L)
- **GraphQL constants:** `MODERATION_STATS_QUERY`, `MODERATION_ITEMS_QUERY`, `PERFORM_MODERATION_MUTATION` + 3 wrapper types in `api/graphql.rs`
- **Response codes:** Added moderation constants (`12101`–`12103`, `22103`, `32101`, `32103`, `62101`, `60501`) to `models/common.rs`
- **Route:** `/admin` registered in `app.rs` with `AuthGuard` + `RoleGuard`
- **Architecture:** `RoleGuard` uses server-side `moderationStats` probe (tri-state: authorized/denied/error); `AdminHeader` fetches username via `get_profile`; single atomic `Effect` for filter reset + reload; named response code constants in action panel
- **SCSS:** 1,044 lines with responsive breakpoints at 980px, 768px, 600px; dark theme using CSS custom properties + admin-specific color palette
- **Total: 2,514 lines** (54% over initial estimate of 1,630, mostly from comprehensive SCSS)

### 2026-04-16 (Mock Backend Phases 5 & 6 Complete)
- **Phase 5 — Economy (Wallet, Tokenomics, Shop, Ads)** marked ✅ Done — 219 total tests (49 new for Phase 5), 0 clippy warnings
  - 16 wallet tests: balance, transferTokens with fee calculation (burn 1%, peer 2%, inviter 1%), transaction history
  - 8 tokenomics tests: action prices, daily free status, gems, minting
  - 12 ads tests: basic/pinned creation, listing, history, cost calculation
  - 8 shop tests: performShopOrder, shopOrderDetails, delivery validation
  - Token deduction integrated into Phase 3/4 action resolvers with daily free action logic
  - New files: `types/wallet.rs`, `types/tokenomics.rs`, `types/ad.rs`, `types/shop.rs`, `schema/query/wallet.rs`, `schema/mutation/wallet.rs`, `schema/query/tokenomics.rs`, `schema/query/ads.rs`, `schema/mutation/ads.rs`, `schema/mutation/shop.rs`, `schema/query/shop.rs`
- **Phase 6 — Admin & Moderation** marked ✅ Done — 266 total tests (47 new for Phase 6), 0 clippy warnings
  - `RoleGuard` implementing async-graphql `Guard` trait with bitmask checking (ADMIN=16, MODERATOR=256)
  - Moderation: `moderationStats`, `moderationItems` (filtered/paginated), `performModeration` (hide/restore/illegal)
  - Admin: `listUsersAdminV2` (extended search by email/IP/verified/roles), `allfriends`, `postcomments`, `generateLeaderboard`
  - Admin gems: `gemster`, `dailygemstatus`, `dailygemsresults`, `getMintAccount`, `globalwins`, `distributeTokensForGems`, `gemsters`, `alphaMint`
  - Content visibility integration: hidden/illegal content filtered from Phase 3/4 list queries
  - Report-to-ticket integration: `reportUser`, `postAction(REPORT)`, `reportComment` auto-create moderation tickets
  - New files: `guards.rs`, `types/moderation.rs`, `types/admin.rs`, `types/admin_gems.rs`, `schema/query/moderation.rs`, `schema/mutation/moderation.rs`, `schema/query/admin.rs`, `schema/query/admin_gems.rs`, `schema/mutation/admin_gems.rs`
- **Admin migration priority** updated from ⬜ Not Started to 🟡 (mock backend complete, Leptos admin page not started)

### 2026-04-16 (Version History Implemented)
- **Version History fully implemented** — all 6 phases complete, auth-guarded two-panel layout with static JSON fetch ([docs](plans/version-history/version-history-implementation.md))
- **New files:** `src/models/version.rs`, `src/api/version.rs`, `src/pages/version_history.rs`, `src/components/version_history/` (3 files), `style/version-history.scss`
- **Server function** fetches from `json/version_releases.json` (with path fallback), proper error handling
- **Responsive breakpoints** ported from legacy `settings.css` (`@media min-width: 1000px`)
- **Security:** external links include `rel="noopener noreferrer"` (improvement over legacy)
- **No backend changes needed** — uses static JSON data, no GraphQL operations required

### 2026-04-16 (Profile Completion Sprint + Code Quality Pass)
- **4 features promoted to ✅ Implemented:** My Profile, View Profile, Edit Profile, Settings — bumps convergence from ~64% to ~79%
- **Summary counts updated:** ✅ 5→9, 🟡 9→6, ❌ 4→3
- **Profile sprint Tasks 1–8 implemented:** `/edit-profile` redirect, infinite scroll for profile posts, filter sidebar wired, `?user=` query param redirect, ProfileWidget wired to auth, RelationsModal infinite scroll, Deactivate account UI, settings save fix
- **New shared hook:** `use_infinite_scroll` (`src/hooks/use_infinite_scroll.rs`) — extracted IntersectionObserver pattern from 6 components (~180 lines removed), fixes `callback.forget()` memory leak, proper cleanup of observer + JS closure on unmount
- **Bug fixes:**
  - `use_navigate()` moved out of `spawn_local` async block in `DeactivateAccountPanel` (Leptos hook correctness)
  - Parallel save race condition in `settings/profile.rs` replaced with single sequential `spawn_local` (race-free, no `futures` dep needed)
  - `callback.forget()` memory leak fixed in all IntersectionObserver call sites (observer + closure now stored together in cleanup)
- **Files modified:** `src/hooks/mod.rs`, `src/hooks/use_infinite_scroll.rs` (new), `src/pages/profile.rs`, `src/pages/view_profile.rs`, `src/components/posts/post_list.rs`, `src/components/profile/relations_modal.rs`, `src/components/my_ads/ad_list.rs`, `src/components/wallet/transaction_history.rs`, `src/components/settings/deactivate.rs`, `src/components/settings/profile.rs`
- **Task 9 (E2E tests) remains ❌ Not Started** — `peer-web/end2end/tests/` has no profile/settings test coverage yet
- **Dashboard gap note updated** — removed "profile widget not wired to auth context" (now resolved)

### 2026-04-16 (Plan Status Audit)
- **Summary count fix:** 🟡 Near-Complete corrected 8 → 9, ❌ Not Started corrected 5 → 4 (My Ads was promoted to 🟡 but counters were off-by-one)
- **Convergence recalculated:** ~60% → ~64%
- **Referral Board (#10)** — removed "pending mock backend endpoints" note; Phase 2 (which includes `getReferralInfo` and `referralList`) has been ✅ Done since 2026-04-14
- **All 15 plan documents audited** — no status changes required; all plan statuses match convergence tracker entries
- **Mock backend verified:** 170 tests passing, Phases 0–4 ✅ Done, Phases 5–6 + CI + Acceptance ❌ Not Started (no new types/resolvers added since Phase 4)
- **No new implementations detected** since 2026-04-14 (10 commits, all plan/doc work)

### 2026-04-14 (Mock Backend Phase 4 Complete)
- **Phase 4 — Social (Comments, Chat)** marked ✅ Done
- **5 new query/mutation resolvers (comments):** `listComments`, `listChildComments`, `createComment`, `likeComment`, `unlikeComment`, `reportComment`
- **3 new query/mutation resolvers (chat):** `listChats`, `createChat`, `sendChatMessage`
- **New types:** `types/comment.rs` (CommentType enum, CommentUser, Comment, CommentListResponse, CreateCommentResponse), `types/chat.rs` (ChatParticipant, ChatMessage, Chat, ListChatsResponse, SendMessageResponse, CreateChatResponse, CreateChatResult)
- **State extensions:** `comments`, `comment_likes`, `comment_reports`, `daily_comment_count` (comments); `chats`, `chat_messages` (chat)
- **Seed data:** 6 comments (4 top-level, 2 replies), 2 comment likes, 2 chats (1 private, 1 group), 5 chat messages
- **Cross-cutting:** `amountcomments` now computed from actual comment count in `post_record_to_graphql()`; `Comments` sort uses real counts; trending score includes comments
- **Daily free action logic:** first 4 comments per day → `11608` (free), subsequent → `11605` (paid, Phase 5 token deduction stubbed)
- **Private chat deduplication:** `createChat` with 1 recipient returns existing chat if already exists (`11803`)
- **49 new integration tests** (170 total), all passing; `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean
- **Review findings (minor):** `img` field correctly resolves from user profile (plan hardcoded placeholder); 2 planned tests absent (offset-beyond-range, reply-to-different-post) — both code paths exist and are tested indirectly
- New files: `types/comment.rs`, `types/chat.rs`, `schema/query/comments.rs`, `schema/query/chat.rs`, `schema/mutation/comment.rs`, `schema/mutation/chat.rs`
- Modified: `state.rs` (6 new record types + fields), `seed.rs` (comment/chat seed data + UUIDs), `types/mod.rs`, `schema/mod.rs`, `schema/query/mod.rs`, `schema/mutation/mod.rs`

### 2026-04-14 (Forgot Password Completion Sprint)
- **Forgot Password completion sprint plan created** — 6-task sprint to promote 🚧 → ✅: auto-redirect for authenticated users, cookie-persisted resend counter, countdown interval stacking fix, BackButton component reuse, mock backend password reset endpoints (6 tests), E2E tests (10 tests) ([sprint](plans/forgot-password/forgot-password-completion-sprint.md))

### 2026-04-14 (Plan Status Sync)
- **My Ads promoted** ❌ Not Started → 🟡 In Progress — plan created and Phases 1–4 implemented: stats header, ad listing with infinite scroll, boost post modal (multi-step), `advertisementHistory` query, `advertisePostPinned` mutation, skeleton loading ([docs](plans/my-ads/my-ads-implementation.md))
- **Mock Backend Phase 3 completed** ❌ Not Started → ✅ Done — 121 total tests (48+ new for Phase 3), 0 clippy warnings, `cargo fmt` clean; adds all post-related queries and mutations (listPosts, guestListPost, postAction, createPost, searchTags, ads)
- **Mock Backend Phase 6 plan created** — RBAC, content moderation, admin operations fully planned at ⭐⭐⭐⭐⭐ quality ([docs](plans/mock-backend/phase-6-admin-moderation.md))
- **Mock Backend CI Integration plan created** — build/test pipeline, Leptos E2E integration, schema snapshots at ⭐⭐⭐⭐⭐ quality ([docs](plans/mock-backend/phase-ci-integration.md))
- **Mock Backend Acceptance Criteria plan created** — per-phase gates, cross-cutting quality reqs, verification tooling at ⭐⭐⭐⭐⭐ quality ([docs](plans/mock-backend/acceptance-criteria.md))
- Summary updated: 🟡 7→8, ❌ 6→5, convergence ~57%→~60%
- Migration priority updated: My Ads broken out from "Remaining pages", Admin now has plan link

### 2026-04-14 (Mock Backend Phase 2 Complete)
- **Phase 2 implemented and reviewed** — 73 tests pass (33 existing + 40 new), cargo clippy/fmt clean
- **10 user queries:** getProfile, searchUser, listUsersV2, getUser, listFollowRelations, listFriends, listBlockedUsers, getUserInfo, getReferralInfo, referralList
- **8 profile mutations:** toggleUserFollowStatus, toggleBlockUserStatus, reportUser, updateProfileImage, updateBio, updateUsername, updateEmail, updateUserPreferences
- **6 new files:** `types/user.rs` (~400L), `schema/query/users.rs` (~620L), `schema/mutation/profile.rs` (~320L), `filters.rs` (~50L), `schema/query/health.rs` (extracted), `schema/query/mod.rs` (MergedObject refactor)
- **State extensions:** follows, blocks, reports, preferences, referral_invitations; 15 helper methods on MockState
- **Seed data:** 6 users (test, unverified, alice, bob, carol, dave), 3 follow edges, 1 block edge, 1 referral
- **Content filtering pipeline:** IllegalContentFilterSpec, SystemUserSpec, DeletedUserSpec, UserIsBlockedByMeSpec, CurrentUserIsBlockedUserSpec
- **Review findings (minor):** referral link domain uses `peer.com` (plan says `getpeer.eu`, frontend tests use `peer.network`); `listFollowRelations`/`listBlockedUsers` ignore offset/limit params; regex recompiled per `updateUsername` call; `updateBio`/`updateProfileImage` reject empty strings (undocumented validation)
- Mock backend tracker updated: Phase 2 → ✅ Done

### 2026-04-14 (Mock Backend Phase 2 Plan Updated)
- **All review gaps resolved** — plan doc updated to be fully self-contained and copy-paste ready
- **9 missing query resolvers added:** `listUsersV2`, `getUser`, `listFollowRelations`, `listFriends`, `listBlockedUsers`, `getUserInfo`, `getReferralInfo`, `referralList` — all fully coded with response construction, error handling, and content filtering
- **`require_auth()` helper:** added §4.5a documenting promotion of existing Phase 1 `fn require_auth` in `auth.rs` to `pub`, with `lib.rs` re-export
- **`convert_visibility()` helper:** added §4.5b for state enum → GraphQL enum conversion (fixes `/* convert from state enum */` placeholder)
- **`filter_users` module:** resolved ambiguity — now concretely `src/filters.rs` with `pub mod filters;` in `lib.rs`
- **Schema assembly (§4.8):** expanded from 1 snippet to 5 concrete steps: split `query.rs` into `query/mod.rs` + `query/health.rs` + `query/users.rs`, update `schema/mod.rs`, update `mutation/mod.rs`, update `lib.rs`
- **Builder pattern helpers:** added `build_profile_user()` and `build_basic_user_info()` helper fns to reduce duplication across query resolvers
- **Test helper signatures fixed:** all tests now correctly use `&state` (reference) pattern matching actual `graphql_stateful`/`graphql_with_auth` signatures; removed all redundant `app_with_state()` calls
- **4 new tests added:** `test_list_users_v2_by_username`, `test_list_users_v2_excludes_blocked`, `test_get_user_by_id`, `test_referral_list` — total now 46 (was 42)
- **Test table (§3 G):** expanded from G1–G42 to G1–G46
- **Definition of Done:** test count updated ≥42 → ≥46
- **Prerequisites (§2):** checkboxes updated to match actual Phase 1 state (8/9 checked, QueryRoot refactoring flagged)
- **`BasicUserInfoGql.updatedat`:** annotated with rationale (backend schema returns it; frontend ignores extra fields via serde)
- **`ProfileMutation`:** added `#[derive(Default)]` and `use crate::require_auth;` import

### 2026-04-14 (Mock Backend Phase 2 Plan Quality Review)
- Phase 2 plan reviewed and rated ⭐⭐⭐⭐⭐ (5/5)
- **Strong:** Exhaustive 75+ task breakdown across 8 sub-phases (A–H); full Rust code for all types (26 structs/enums), state extensions, 2 query resolvers (getProfile, searchUser) + 9 mutation resolvers; seed data with 4 new users, pre-existing follow/block/referral relationships; content filtering module with 5 filter specs + pagination helper; 42 integration tests covering success, error, auth, pagination, and multi-step interaction flows; comprehensive Definition of Done (50+ checklist items) with build gates, response shape compatibility checks, and state isolation requirements
- **Perfect model alignment** — `ProfileGql`, `ProfileUserGql`, `BasicUserInfoGql`, `FollowStatusResponseGql`, `UpdateResponseGql`, `UserPreferencesResponseGql` all verified field-by-field against `peer-web/src/models/profile.rs` and `peer-web/src/models/settings.rs`; GraphQL query shapes match `SEARCH_USERS_QUERY` and `GET_USER_QUERY` exactly
- **Correctly identifies** QueryRoot `#[Object]` → `MergedObject` refactoring needed for schema assembly
- **Minor gaps (now resolved above):** 9 of 11 query resolvers left as "follows the same pattern"; `require_auth()` helper referenced but not defined; `filter_users()` module location ambiguous; test helper signatures inconsistent with codebase; `referralList` query missing from test section

### 2026-04-14 (Invite Page Review & Doc Update)
- **Summary table fixed:** 🟡 count corrected 6→7, 🚧 count corrected 3→2 (stale after Invite moved to ✅)
- **Invite plan updated** (`plans/invite/invite-implementation.md`):
  - File manifest: added actual line counts (185L Rust, 26L SCSS)
  - SSR Considerations: documented `#[cfg(feature = "hydrate")]` gating pattern and SSR no-op fallback
  - Appendix added: 6 implementation deviations catalogued (inlined helpers, `#[cfg]` gating, `detect_platform` signature, case-insensitive iOS detection, `Rc<Cell<Option<Timeout>>>` cancellation pattern, no legacy clipboard fallback)

### 2026-04-14 (Invite Page Implemented)
- **Invite page implemented** — `src/pages/invite.rs` (185L), `style/invite.scss` (26L)
- Route `/invite` added to `app.rs`, module registered in `pages/mod.rs`
- SCSS imported in `main.scss`
- All client logic `#[cfg(feature = "hydrate")]` gated; SSR renders static fallback HTML
- `Rc<Cell<Option<Timeout>>>` pattern for cancellable fallback timer
- Builds clean on both `--features hydrate` and `--features ssr`
- Plan status updated to ✅ Complete, scope checkboxes marked done, open questions resolved

### 2026-04-14 (Mock Backend Phase 1 Complete)
- **Phase 1 — Login & Session Flows** marked ✅ Done
- 9 new auth/account mutations implemented: `login`, `refreshToken`, `logout`, `deleteAccount`, `requestPasswordReset`, `resetPasswordTokenVerify`, `resetPassword`, `updatePassword`, `contactus`
- Auth middleware: `Authorization: Bearer <token>` header extraction → `CurrentUser` context injection
- 2 seeded users: verified (`test@peer.com` / `TestPass123`) and unverified (`unverified@peer.com` / `TestPass456`)
- `register` mutation updated to create `User` records so login works for newly registered users
- 24 new integration tests (33 total), all passing with `cargo clippy` + `cargo fmt` clean
- New files: `src/types/auth.rs`, `src/schema/mutation/auth.rs`
- Modified: `state.rs` (User struct, token maps), `seed.rs` (seed users/credentials), `lib.rs` (auth extraction), `schema/mod.rs` (MutationRoot expanded)
- Mock token strategy: `mock-access-<uid>-<ts>-<seq>` with atomic counter for uniqueness
- Phase 1 plan doc, parent plan, acceptance criteria, and README all updated

### 2026-04-14 (Invite Page Planning)
- **Invite page plan created** ([docs](plans/invite/invite-implementation.md))
- Deep-link landing page for referral links (`/invite?referralUuid=...`)
- Scope: platform detection (Android/iOS/Desktop), `peer://` deep-link attempt, auto-fallback timer (1.5s), manual "Click Here" button, localStorage persistence, clipboard copy before redirect
- Lightweight: ~150 lines Rust, ~30 lines SCSS, no API calls, no auth required
- Connects Referral Board (✅ generates links) → Invite (this) → Register (✅ accepts `?referralUuid=`)
- **Priority list updated:** Invite split out of "Remaining pages" at #12

### 2026-04-14 (Mock Backend Phase 0 Documentation Update)
- **Phase 0 plan doc updated** to reflect actual implementation:
  - All task checkboxes and Definition of Done items marked complete
  - Code snippets updated: removed `rename_fields = "PascalCase"` (broke queries), added `#[graphql(name)]` per-field, fixed `_health` query naming, replaced `GraphQL` service with explicit handler
  - Cargo.toml: `async-graphql` v8→v7 (v8 still RC), added `regex = "1"`, `chrono = "0.4"`
  - Test section updated: 7→9 tests, `graphql_stateful()` helper documented
  - Migration checklist fully checked off (Node.js files deleted, fixtures kept)
  - Appendix B added: 8 implementation deviations catalogued
- **Parent plan updated** (`mock-backend-rust-rewrite.md`):
  - Phase 0 section marked ✅ Complete with all definition-of-done items checked
  - Directory structure, Cargo.toml, code snippets, and test table updated to match reality
  - Deviation notes added for async-graphql naming, axum routing, and POST-only GraphQL
- **Feature convergence table updated**: Phase 0 row now shows quality rating ⭐⭐⭐⭐ (4/5), expanded scope note, and detailed status

### 2026-04-14 (Referral Board Implementation)
- **Referral Board promoted** ❌→✅ Implemented
- New files: `src/pages/referral_board.rs` (150L), `src/components/referral_board/` (4 components: header, tabs, user_card, user_grid), `src/api/referral.rs` (65L), `src/models/referral.rs` (150L), `style/referral-board.scss` (180L)
- Features: referral link display + clipboard copy, "Invited Friends" / "My Inviter" tabs, user card grid with navigation, auth guard, loading skeletons, empty states, responsive layout
- Tests: 13 passing (11 unit + 2 fixture)
- API layer entry added for Referral module
- **Summary counts updated:** 3/6/3/8 → 4/6/3/7; convergence ~50% → ~55%
- **Priority list updated:** Referral Board marked complete at #10

### 2026-04-14 (Plan vs Tracker Reconciliation)
- **5 features promoted** after cross-referencing plan doc statuses against tracker:
  - Dashboard: 🚧→🟡 Mostly Implemented (plan says "Mostly Implemented, minor gaps")
  - Chat: 🚧→🟡 Core Implemented (plan says "✅ Core Implemented")
  - Wallet: 🚧→🟡 Implemented, tests pending (plan says "✅ Implemented, tests pending")
  - Settings: 🚧→🟡 Implemented with gaps (plan says "Implemented with gaps")
  - Profile (My + View): 🚧→🟡 Mostly Implemented (plan says "Mostly Implemented, core complete")
- **Summary counts updated:** 3/1/8/8 → 3/6/3/8; convergence ~40% → ~50%
- **Known gaps surfaced in Notes columns** from plan docs:
  - Dashboard: post click overlay is a TODO stub, profile widget not wired
  - Forgot Password: no auth redirect, resend counter not cookie-persisted, interval stacking bug, BackButton not reused
  - Profile: no infinite scroll, filter sidebar placeholder, no legacy `?user=` query param
  - Settings: delete account UI not wired (no-op), sequential profile save
  - Chat: no Firebase real-time listener, no unread indicators, search not connected
  - Wallet: shop order details UI not wired, no thousand-separator formatting
- **Mock Backend section added** — 7 phases tracked (Phase 0–6), 4 plans rated ⭐⭐⭐⭐⭐
- **Priority list updated** with accurate statuses and remaining-work summaries
- **Missing plans noted:** 8 "Not Started" features + Header/Footer/Sidebars + Firebase have no planning docs yet (PWA now ✅ Implemented)

### 2026-04-14 (Full Codebase Audit & Status Refresh)
- Audited every page, component, API module, and SCSS file against tracker claims
- **Summary counts corrected:** 3 Implemented, 1 Near-Complete, 8 In Progress, 8 Not Started (was: 2/0/2/6/10)
- **Convergence updated:** ~10% → ~40%
- **Pages upgraded:** Forgot Password (📋→🚧, 990L page), Profile (📋→🚧, 238L), View Profile (📋→🚧, 274L), Settings (📋→🚧, 142L page + 7 sub-components), Chat (📋→🚧, 141L page + 7 components + state), Wallet (📋→🚧, 146L page + transfer modal 760L)
- **Components upgraded:** Post Card (❌→🚧, 304L), Post List (❌→🚧, 209L), Comments (❌→🚧, 509L), Chat List (❌→🚧, 166L), Chat Window (❌→🚧, 289L combined), Image Cropper (❌→🚧, 226L), Audio (❌→🚧, 395L), Video (❌→🚧, 471L), Modal (❌→🚧, multiple: 151+74+138+271+760L)
- **API layer upgraded:** Posts (❌→🚧, 409L), Comments (❌→🚧, 204L), Chat (❌→🚧, 151L), Wallet (❌→🚧, 246L)
- **Fixed:** broken Unicode emojis (�) on View Post and New Post rows, merged priority list lines 3/3b/4
- SCSS coverage confirmed: dashboard (1015L), new-post (1378L), wallet (1092L), view-post (952L), chat (851L), settings (819L), profile (739L)

### 2026-04-14 (New Post Implementation Review)
- New Post status updated from "📋 Planning" to "🚧 In Progress"
- ~70% structurally complete: all scaffolding, state management, API calls (4 endpoints), routing, models, and styles (1378-line SCSS) in place
- Completed: page layout, content type tabs, text/image/audio/video upload UIs, image slider, tag autocomplete, full/card preview, 3-step submit flow (eligibility → upload → createPost), form validation, toast feedback
- Remaining stubs: image cropper canvas logic, voice recorder MediaRecorder integration, video trimmer drag/FFmpeg WASM, tag localStorage history, drag-and-drop events, responsive breakpoints
- Updated summary counts (2 In Progress, 6 Planning, 10 Not Started)

### 2026-04-14 (Dashboard Plan Quality Review)
- Dashboard implementation plan reviewed and rated ⭐⭐⭐⭐ (4/5)
- Strong: thorough legacy analysis with layout diagram, complete API reference (4 GraphQL ops + response codes), substantial Leptos 0.8 component code (IntersectionObserver, debounce, post card, filters, widgets), well-structured 7-phase plan, good model design (FeedItem enum), iterative v1→v2 changelog, complete file manifest (22 new + 9 modified)
- Gaps: feed/sort filter components under-specified (one-liners), ad interleaving logic missing, sparse error handling (no API failure states), view post overlay hollow, mobile layout unspecified, filter state provider not shown, no SCSS code

### 2026-04-14 (Chat Plan Quality Review)
- Chat implementation plan reviewed and rated ⭐⭐⭐⭐ (4/5)
- Strong: legacy analysis, architecture diagrams, component decomposition, API reference, testing strategy
- Gaps: Firebase interop under-specified, group review screen missing, no dual-source deduplication, sparse error handling

### 2026-04-14 (New Post Plan Quality Review)
- New Post implementation plan reviewed and rated ⭐⭐⭐⭐ (4/5)
- Strong: excellent scope definition with detailed in-scope/out-of-scope checklists, thorough legacy file mapping, 6 ASCII layout diagrams (main layout + each content type + preview modes), complete API reference (4 endpoints — postEligibility, /upload-post REST, createPost, searchTags — with full GraphQL schemas, response codes, media limits, token costs), substantial Leptos component code (page, image cropper, voice recorder, video trimmer, tag input, submit logic), clean component decomposition (~15 files), 6-phase plan, comprehensive testing strategy (unit/component/integration/E2E), 30-item migration checklist
- Gaps: FFmpeg WASM interop hand-wavy (hardest technical challenge left as "consider" options), ~5 components listed but never detailed (image slider, drop zone, preview full/card), no SCSS code provided, several types undefined (MediaFile, AudioBlob, VideoBlob, FileData), no error recovery strategy (upload failure mid-flow, rate limiting, token rejection), no mobile/responsive layout, no file size limits documented, no accessibility considerations, multi-image cropping workflow unclear (sequential vs batch)

### 2026-04-14 (Forgot Password Planning)
- Forgot Password implementation planning document created
- Comprehensive documentation of 4-step password reset flow (Email → Verify Code → New Password → Success)
- Detailed analysis of 3 backend mutations (requestPasswordReset, resetPasswordTokenVerify, resetPassword)
- Rate-limited resend with escalating cooldowns (60s → 10min → locked)
- Identified reusable components: LeftPanel, BackButton, PasswordStrengthMeter, StepAnnouncer, Toast, validation helpers
- 5-phase implementation plan with Rust component code for all steps
- Forgot Password marked as "Planning" in tracker
- Updated summary counts (7 Planning, 10 Not Started)

### 2026-04-14 (Settings Planning)
- Settings implementation planning document created
- Comprehensive documentation of profile editing, credential changes, content preferences
- Detailed analysis of 7 backend mutations (updateProfileImage, updateBio, updateUsername, updatePassword, updateEmail, updateUserPreferences, deleteAccount)
- 5-phase implementation plan with Rust component code for all sub-panels
- Settings marked as "Planning" in tracker
- Updated summary counts (6 Planning, 11 Not Started)

### 2026-04-12 (Wallet Planning)
- Wallet implementation planning document created
- Comprehensive documentation of balance display, P2P transfers, transaction history
- Detailed analysis of transfer modal flow, fee calculations, validation rules
- Transaction categories and expandable details documented
- Wallet marked as "Planning" in tracker
- Updated summary counts (5 Planning, 12 Not Started)

### 2026-04-12 (Chat Planning)
- Chat implementation planning document created
- Comprehensive documentation of private/group chat, Firebase real-time integration
- Detailed analysis of chat UI components, message handling, contact list
- Chat marked as "Planning" in tracker
- Updated summary counts (4 Planning, 13 Not Started)

### 2026-04-12 (New Post Planning)
- New Post implementation planning document created
- Comprehensive documentation of all four content types (text, image, audio, video)
- Detailed analysis of image cropping, voice recording, video trimming features
- New Post marked as "Planning" in tracker
- Updated summary counts (3 Planning, 14 Not Started)

### 2026-04-12 (Profile Planning)
- Profile (My Profile + View Profile) implementation planning document created
- Profile marked as "Planning" in tracker
- Updated priority list with Profile documentation link
- Summary counts updated (2 Planning, 15 Not Started)

### 2026-04-14
- View Post audited: ~95% implemented (page, 8 components, 7 API fns, 7 GraphQL ops, 2 models, 952 lines SCSS)
- Tasks 1–5 confirmed complete: follow API wiring, mobile deep-link hook, view tracking, format_time_ago
- Remaining: error toasts, mock backend fixtures, E2E tests, responsive polish, dashboard overlay
- View Post status updated from "Planning" to "🟡 ~95% Implemented"

### 2026-04-12
- View Post implementation planning document created
- Dashboard marked as "In Progress" (substantial code exists)
- View Post marked as "Planning"
- Updated summary counts

### 2026-04-10
- Login / Auth marked as complete
- Auth API layer implemented (JWT, refresh, 401 handling)
- State management upgraded from Basic to Implemented (AuthContext)
- Updated convergence to ~10%

### 2026-04-10 (earlier)
- Added Dashboard implementation planning document
- Initial tracking document created
- Registration flow marked as complete
- Basic infrastructure (SSR, routing, build) confirmed working
