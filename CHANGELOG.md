# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project tracks the migration from the legacy PHP/JS frontend to the
Leptos (Rust/WASM) rewrite. Entries are dated; semantic version numbers will
be introduced once the rewrite reaches feature parity.

> Convergence as of `2026-04-23`: 15/21 pages implemented (~71%), 19/21
> ≥ near-complete (~90%). For the live status matrix see
> [docs/feature-convergence.md](docs/feature-convergence.md).

## [Unreleased]

### In progress
- New Post — real-time waveform, video duration extraction, frame thumbnails,
  server-side trim plumbing, tag history, mobile review, E2E tests.
- Mock Backend — CI integration phase and acceptance-criteria sign-off gates.
- Dashboard — completion sprint published; only post-card click wiring + a
  `dashboard.spec.ts` E2E pass remain before ✅ promotion. See
  [docs/plans/dashboard/dashboard-completion-sprint.md](docs/plans/dashboard/dashboard-completion-sprint.md).

---

## [2026-04-23] — Dashboard completion sprint plan + Known Issues reconciliation

### Documentation
- New [docs/plans/dashboard/dashboard-completion-sprint.md](docs/plans/dashboard/dashboard-completion-sprint.md)
  — 6-task plan to promote Dashboard from 🟡 to ✅. P0 work is the post-card
  click handler ([src/components/posts/post_card.rs](src/components/posts/post_card.rs))
  plus a new `dashboard.spec.ts` Playwright spec; P1/P2 add IntersectionObserver-gated
  view tracking, a mock-backend round-trip Rust test, and doc reconciliation.
- Audit verified that **3 of the 4** Known Issues recorded in the parent
  [dashboard-implementation.md](docs/plans/dashboard/dashboard-implementation.md)
  are already fixed silently in the tree:
  `user_search.rs` hydrate-feature imports, `ProfileWidget` real-data wiring
  via `get_profile`, and `LIST_POSTS_QUERY` field completeness
  (`isreported`, `amounttrending`, `hasActiveReports`, `visibilityStatus`,
  `isHiddenForUsers`). Reflected in a new "Resolved" subsection of the
  parent plan's Known Issues list.
- Updated [docs/feature-convergence.md](docs/feature-convergence.md):
  Dashboard row notes + migration-priority entry now link to the sprint and
  reflect the narrowed gap; `Last Updated` bumped.

---

## [2026-04-23] — Forgot Password Tasks 5–6 (E2E + debug endpoint)

### Added
- **Mock backend `/debug/reset-token` endpoint**
  ([packages/mock_backend/src/lib.rs](packages/mock_backend/src/lib.rs))
  — GET handler that returns the most recent password-reset token issued
  for a given email (`?email=...`). Replaces the legacy SMTP transport for
  E2E coverage of the multi-step forgot-password flow. Mock-only — not part
  of the production GraphQL surface.
- **3 cross-cutting integration tests** for the debug endpoint
  ([packages/mock_backend/tests/cross_cutting.rs](packages/mock_backend/tests/cross_cutting.rs))
  — issued-token round-trip, unknown-email → 404, no-token-issued → 404.
  Brings mock-backend total to **269 tests** (was 266).
- **`getResetTokenForEmail` Playwright helper**
  ([end2end/helpers/mock-server.ts](end2end/helpers/mock-server.ts)) — thin
  wrapper around the new debug endpoint.
- **Playwright spec [end2end/tests/forgot-password.spec.ts](end2end/tests/forgot-password.spec.ts)**
  — 8 cases covering Step 1 render, client-side email validation, Step 2
  advance with masked-email assertion, code verification (using
  debug-issued token) advancing to Step 3, password-mismatch rejection,
  full happy-path with new-password-login follow-up, back-button callback
  path (Step 2 → Step 1), and authenticated-user auto-redirect.

### Status
- **Forgot Password** promoted from 🚧 In Progress → ✅ Implemented in
  [docs/feature-convergence.md](docs/feature-convergence.md) — all 6 sprint
  tasks complete (Tasks 1–4 landed earlier today, Tasks 5–6 land here).
  Convergence: **15/21 pages implemented (~71%)**.

---

## [2026-04-23] — Forgot Password gaps 1–4 closed

### Added
- `set_cookie_seconds(name, value, max_age_secs)` in
  [src/utils/cookies.rs](src/utils/cookies.rs) — companion to the existing
  days-based `set_cookie`, used by the forgot-password page to persist its
  resend counter for exactly 2 hours (legacy parity with
  `reset_code_sent_counter`).
- `ForgotPasswordView` private component in
  [src/pages/forgot_password.rs](src/pages/forgot_password.rs) — extracted
  form body so the new auto-redirect `<Show>` can short-circuit it cleanly
  for authenticated viewers.

### Changed
- **Forgot Password** ([src/pages/forgot_password.rs](src/pages/forgot_password.rs))
  — four convergence-tracker gaps closed:
  - **Auto-redirect for authenticated users** — `ForgotPasswordPage` now
    wraps its view in `<Show when=is_session_checked && is_authenticated>`
    with `<Redirect path="/dashboard"/>`, mirroring `HomePage` in
    [src/app.rs](src/app.rs). Matches legacy `forgotpassword.php`'s
    `autoLogin()` 302.
  - **Resend counter cookie-persisted** — `resend_count` is rehydrated from
    the `reset_code_sent_counter` cookie on mount and rewritten via
    `set_cookie_seconds(..., 7200)` after each successful resend, so a page
    reload can no longer reset the escalating cooldown (60 s → 10 min →
    locked) back to zero.
  - **Countdown interval stacking fixed** — `start_countdown` now lifts the
    `gloo_timers::callback::Interval` handle into a `StoredValue` declared
    once at the top of `VerifyCodeStep` and explicitly drops the previous
    interval (`take()`) before creating a new one. Prevents
    double/triple-speed countdowns when the user clicks resend rapidly. The
    same handle is dropped from a single `on_cleanup` registered at component
    mount time (no longer registered inside the closure on every call).
  - **`BackButton` component reused** — the inline `<a class="btn back-btn">`
    is replaced with `<BackButton visible href on_back/>`. Step 1 renders as
    a real `/login` link (browser navigation); Steps 2/3 render as a
    callback-driven button that returns to the previous step in-place; Step
    4 hides the button. DOM unchanged (same `id="backBtn"`, classes, and
    icon span).

### Removed
- Local `navigate_to(url)` helper in `forgot_password.rs` — its only caller
  was the inline back-button handler, which is now satisfied by
  `BackButton`'s `href` link behaviour.

### Notes
- SSR (`cargo build --features ssr`) and wasm (`cargo check --features
  hydrate --target wasm32-unknown-unknown`) both build clean. SSR clippy
  (`--all-targets -- -D warnings`) clean. The 5 existing `mask_email` unit
  tests still pass.
- Tasks 5 (Playwright `forgot-password.spec.ts`) and 6 (mock-backend
  `requestPasswordReset` / `resetPasswordTokenVerify` / `resetPassword`
  guest mutations + 6 integration tests) remain — see
  [forgot-password-completion-sprint.md](docs/plans/forgot-password/forgot-password-completion-sprint.md).
  Status stays 🚧 until those land and promote the page to ✅.

---

## [2026-04-23] — Repo layout promoted

### Changed
- Promoted the Leptos/Axum crate from `peer-web/` to the repository root.
- Moved the Rust mock GraphQL backend to `packages/mock_backend/`.
- Demoted the entire PHP-era surface (root `*.php`, `admin/`,
  `template-parts/`, `css/`, `js/`, `img/`, `svg/`, `svgnew/`, `fonts/`,
  `json/`) verbatim into `legacy/php/` and `legacy/assets/`.
- `Cargo.toml` is now the workspace root with `packages/mock_backend` and
  `tests-wasm` as members.
- `.github/workflows/{e2e,security}.yml` updated to drop the `peer-web/`
  prefix; `.gitignore` merged.
- See [adr-repo-layout-promote-peer-web.md](docs/adr-repo-layout-promote-peer-web.md)
  for full rationale and the `git mv` map.
- Commits: `c9fd16f` (legacy move), `43bfb94` (`mock_backend` →
  `packages/`), `a63ffc5` (`peer-web` → root), `a6bdf16` (workspace / paths
  / docs / CI finalization).

### Documentation
- Normalized `..//` → `../` in cross-doc relative links across the
  convergence tracker, chat / new-post / wallet completion sprints, layout /
  home / download / pwa plans, and the mock-backend rewrite ADR (`2fde283`).

### Notes
- Pure relocation — no feature deltas, no code changes inside the crate.
  `git log --follow` works through the renames. Builds (`cargo build
  --features ssr`, `cargo-leptos leptos build`) identical to the
  `2026-04-22` baseline.

---

## [2026-04-22] — Home / Landing, Layout shell, doc reconciliation

### Added
- **Home / Landing** ✅ implemented. `HomePage` ([src/app.rs](src/app.rs))
  rewritten as an auth-aware redirect mirroring legacy `index.php`'s 302:
  authed → `/dashboard`, guest → `/login?message=mustLogin`. Loading window
  renders the shared `auth-guard-loading` sentinel to suppress flash.
  Inbound `?redirect=…` is now preserved through to the login URL.
- Shared `encode_redirect()` helper in
  [src/state/auth.rs](src/state/auth.rs) used by both `HomePage` and
  `AuthGuard` to prevent drift.
- E2E coverage: [end2end/tests/home.spec.ts](end2end/tests/home.spec.ts)
  (guest redirect, authed redirect, `?redirect=` passthrough).
- **Layout shell** ✅ implemented under `src/components/layout/`:
  `SiteShell`, `SiteHeader` (`Hyphen` / `Underscore` spellings),
  `MobileFooter` (route-aware via `use_location()`), `LeftRail`,
  `RightRail`, and the `StandardRightRail` widget-stack preset.
- Pass-1 migration wraps `dashboard`, `chat`, `wallet`, `settings`,
  `view_profile`, `version_history`, `profile`, `my_ads`, `peer_shop`,
  `referral_board`, `new_post`, and `admin` in `<SiteShell…>`.

### Changed
- Components table: Header / Footer / Sidebars rows ❌ → ✅.
- Pages table: Home / Landing 🚧 → ✅.
- Summary counts: ✅ 13 → 14, 🚧 3 → 2; convergence 13/21 (~62%) → 14/21
  (~67%) implemented; 18/21 (~86%) → 19/21 (~90%) near-complete.
- Settings / Version History / My Ads now show the canonical Dashboard
  mobile-nav preset (Home / Search / NewPost / Alerts / Profile, labelled,
  `nav-item` class). The previously unstyled `mobile-nav-item` preset is
  dropped.
- Guest bounce uses `?message=mustLogin` rather than legacy's
  `?message=unauthorized` for consistency with `AuthGuard`.

### Removed
- Ten private `fn MobileFooter` copies and one inline
  `<footer class="mobile-footer">` (in `peer_shop.rs`). `grep "fn
  MobileFooter"` is now zero in `src/pages/`.

### Fixed
- Hard-coded `active` class on the mobile footer (Wallet pinned `/wallet`,
  Settings pinned nothing) is now computed reactively from the URL.

### Documentation
- New Post completion sprint Task 1 — flipped 8 checkboxes from `[ ]` to
  `[x]` (image cropping modal, aspect ratio toggle, cropped image preview,
  voice recording, recording timer, playback controls, record again,
  responsive layout). Audio Player and Video Encoder rows promoted 🚧 → 🟡.
- Documented the existing home-page placeholder before its replacement
  landed.

---

## [2026-04-22] — Wallet promoted to ✅

### Added
- Wallet shop delivery panel wired into the expanded transaction row (lazy
  `shopOrderDetails` fetch, gated to Peer Shop viewer).
- Mock backend shop-purchase seed transaction so the row appears in wallet
  history.
- Playwright [end2end/tests/wallet.spec.ts](end2end/tests/wallet.spec.ts)
  (6 cases including lazy-load network assertion + non-shop viewer gate).

### Changed
- `format_balance()` produces grouped thousands with 4 dp rounding,
  matching legacy `toLocaleString`.
- Pages table: Wallet 🟡 (tests pending) → ✅. Migration Priority #9
  promoted to ✅.
- Summary counts: ✅ 12 → 13, 🟡 6 → 5.

---

## [2026-04-21] — Download proxy, PWA, doc accuracy

### Added
- **Download** ✅ — Axum `/download` route ported from legacy
  `download.php`. New module `src/server/` (sibling to `api/`) for SSR-only
  HTTP routes outside Leptos. Handler, config, URL validator, filename
  sanitiser, streaming adapter, and 24 unit tests live in
  [src/server/download.rs](src/server/download.rs); 7 integration cases in
  [tests/download_proxy.rs](tests/download_proxy.rs).
- **PWA** ✅ — full manifest
  ([public/manifest.webmanifest](public/manifest.webmanifest)), hand-written
  service worker ([public/sw.js](public/sw.js), ~220L: install/activate/
  fetch, network-only for `/api/` + `/graphql` + `/admin`, network-first
  navigation with offline fallback, stale-while-revalidate for `/pkg/`,
  cache-first + 30-day expiry for static assets), branded
  [public/offline.html](public/offline.html), 9 PWA icons, 3 iOS splash
  PNGs.
- PWA Rust support: [src/utils/pwa.rs](src/utils/pwa.rs) (SW registration,
  `beforeinstallprompt` capture, update polling, `?nosw` escape hatch,
  `apply_service_worker_update()`),
  [src/components/pwa.rs](src/components/pwa.rs) (`InstallBanner`,
  `IosHint`, `UpdateBanner`), [style/pwa.scss](style/pwa.scss).
- PWA tests: [tests/pwa_manifest.rs](tests/pwa_manifest.rs) and
  [end2end/tests/pwa.spec.ts](end2end/tests/pwa.spec.ts).
- New `Install Prompt` row added under UI components.

### Changed
- Pages table: Download ❌ → ✅; description corrected — `download.php`
  was always a force-download media proxy, never an "App download page".
- Infrastructure table: PWA / Manifest ❌ → ✅.
- Summary counts: ✅ 11 → 12, ❌ 1 → 0; convergence ~82% → ~87%
  (**100% Started**). Migration Priority added PWA at #17 (Download
  renumbered to #18).
- Mock Backend table: added Integration Test Refactor row — discovered
  during plan audit that commit `c8cf7b7` (2026-04-16) split the monolithic
  `packages/mock_backend/tests/integration.rs` (7,675L) into 18 per-domain
  test files (7,391L total) plus shared `common/` helpers.
- Summary counts corrected: ✅ 10 → 11 (Pages table actually contained 11
  ✅ rows after Admin + Version History promotions on 2026-04-16).
- Migration Priority renumbered 1–17 (removed `3b.` duplicate numbering).

### Security
- Download proxy hardened: HTTPS-only + host allow-list (closes SSRF), no
  redirect following, no userinfo, `Cache-Control: private, no-store`,
  forced `application/octet-stream` + `X-Content-Type-Options: nosniff`,
  RFC 5987 `Content-Disposition` with ASCII fallback, per-request connect
  + total timeouts, streaming body with hard byte cap that aborts
  mid-response if upstream exceeds the limit.
- Configuration (env vars, read once at server start):
  `DOWNLOAD_ALLOWED_HOSTS` (default `media.peer.network,cdn.peer.network`
  — **confirm against production CDN before shipping**),
  `DOWNLOAD_MAX_BYTES` (default 256 MiB), `DOWNLOAD_TIMEOUT_SECS` (default
  300).
- Range / resumable downloads, auth-gated downloads, and rate limiting
  intentionally not implemented; rate limiting **must** be enforced at the
  reverse proxy / WAF before production.

### Deferred
- WASM `force_download()` client helper — legacy caller in
  `js/global.js` is commented out; helper will ship with the first feature
  that needs a "Save to device" action.

---

## [2026-04-16] — Admin Dashboard, Mock Backend Phases 5 & 6, Version History, Profile sprint

### Added
- **Admin Dashboard** ✅ implemented (all 6 phases). Role-gated content
  moderation surface: stats header, filterable ticket list (All / Posts /
  Comments / Accounts), expandable detail with content previews,
  moderation actions (hide / restore / illegal) with confirmations,
  infinite scroll, dark theme. **Total: 2,514 lines** across 15 files +
  1,044L SCSS.
- New: [src/models/moderation.rs](src/models/moderation.rs) (191L),
  [src/api/moderation.rs](src/api/moderation.rs) (102L),
  [src/pages/admin.rs](src/pages/admin.rs) (101L), `src/components/admin/`
  (11 files, 1,470L), [style/admin.scss](style/admin.scss) (1,044L).
- GraphQL constants: `MODERATION_STATS_QUERY`, `MODERATION_ITEMS_QUERY`,
  `PERFORM_MODERATION_MUTATION` + 3 wrapper types in `api/graphql.rs`.
- Response codes: moderation constants (`12101`–`12103`, `22103`, `32101`,
  `32103`, `62101`, `60501`) added to `models/common.rs`.
- Route `/admin` registered with `AuthGuard` + `RoleGuard`. `RoleGuard`
  uses a server-side `moderationStats` probe (tri-state: authorized /
  denied / error).
- **Version History** ✅ implemented. Auth-guarded two-panel layout with
  static JSON fetch.
- New: [src/models/version.rs](src/models/version.rs),
  [src/api/version.rs](src/api/version.rs),
  [src/pages/version_history.rs](src/pages/version_history.rs),
  `src/components/version_history/` (3 files),
  [style/version-history.scss](style/version-history.scss).
- Server function fetches from `json/version_releases.json` (with path
  fallback). External links include `rel="noopener noreferrer"`
  (improvement over legacy).
- **Mock Backend Phase 5 — Economy (Wallet, Tokenomics, Shop, Ads)** ✅.
  219 total tests (49 new). 16 wallet tests (balance, transferTokens with
  fee calculation: burn 1%, peer 2%, inviter 1%, transaction history); 8
  tokenomics tests; 12 ads tests; 8 shop tests. Token deduction
  integrated into Phase 3 / 4 action resolvers with daily-free-action
  logic.
- **Mock Backend Phase 6 — Admin & Moderation** ✅. 266 total tests (47
  new). `RoleGuard` implementing async-graphql `Guard` trait with bitmask
  checking (ADMIN=16, MODERATOR=256). `moderationStats`,
  `moderationItems`, `performModeration` (hide/restore/illegal),
  `listUsersAdminV2`, `allfriends`, `postcomments`, `generateLeaderboard`,
  admin-gem operations. Content visibility integration: hidden / illegal
  content filtered from list queries. Report-to-ticket integration:
  `reportUser`, `postAction(REPORT)`, `reportComment` auto-create
  moderation tickets.
- Shared `use_infinite_scroll` hook
  ([src/hooks/use_infinite_scroll.rs](src/hooks/use_infinite_scroll.rs))
  extracted from 6 components (~180 lines removed). Fixes
  `callback.forget()` memory leak; observer + closure now stored together
  in cleanup.
- Profile sprint Tasks 1–8: `/edit-profile` redirect, infinite scroll for
  profile posts, filter sidebar wired, `?user=` query-param redirect,
  `ProfileWidget` wired to auth, `RelationsModal` infinite scroll,
  Deactivate-account UI, settings save fix.

### Changed
- 4 features promoted to ✅ Implemented: My Profile, View Profile, Edit
  Profile, Settings. Convergence ~64% → ~79%. Summary counts: ✅ 5 → 9,
  🟡 9 → 6, ❌ 4 → 3.
- Summary count fix: 🟡 Near-Complete corrected 8 → 9, ❌ Not Started
  corrected 5 → 4 (My Ads off-by-one). Convergence recalculated ~60% →
  ~64%.
- Referral Board (#10): removed "pending mock backend endpoints" note.

### Fixed
- `use_navigate()` moved out of `spawn_local` async block in
  `DeactivateAccountPanel` (Leptos hook correctness).
- Parallel save race condition in `settings/profile.rs` replaced with a
  single sequential `spawn_local` (race-free, no `futures` dep needed).
- `callback.forget()` memory leak fixed in all `IntersectionObserver` call
  sites.

---

## [2026-04-14] — Invite, Referral Board, Mock Backend Phases 1–4, planning blitz

### Added
- **Invite** ✅ implemented. Deep-link relay page for referral links
  (`/invite?referralUuid=…`): platform detection (Android / iOS /
  Desktop), `peer://invite/{uuid}` deep-link attempt, auto-fallback timer
  (1.5s), manual "Click Here" button, localStorage persistence, clipboard
  copy before redirect.
- New: [src/pages/invite.rs](src/pages/invite.rs) (185L),
  [style/invite.scss](style/invite.scss) (26L). Route `/invite`
  registered. All client logic `#[cfg(feature = "hydrate")]` gated; SSR
  renders static fallback HTML.
- **Referral Board** ✅ implemented. Referral link display + clipboard
  copy, "Invited Friends" / "My Inviter" tabs, user-card grid with
  navigation, auth guard, loading skeletons, empty states, responsive
  layout.
- New: [src/pages/referral_board.rs](src/pages/referral_board.rs) (150L),
  `src/components/referral_board/` (4 components),
  [src/api/referral.rs](src/api/referral.rs) (65L),
  [src/models/referral.rs](src/models/referral.rs) (150L),
  [style/referral-board.scss](style/referral-board.scss) (180L). 13 tests
  (11 unit + 2 fixture).
- **Mock Backend Phase 1 — Login & Session Flows** ✅. 9 auth/account
  mutations: `login`, `refreshToken`, `logout`, `deleteAccount`,
  `requestPasswordReset`, `resetPasswordTokenVerify`, `resetPassword`,
  `updatePassword`, `contactus`. Auth middleware: `Authorization: Bearer
  <token>` → `CurrentUser` context injection. 2 seeded users. 24 new
  integration tests (33 total). Mock token strategy: `mock-access-<uid>-
  <ts>-<seq>`.
- **Mock Backend Phase 2 — Users & Profiles** ✅. 10 user queries
  (`getProfile`, `searchUser`, `listUsersV2`, `getUser`, follow/blocks/
  friends/relations, `getUserInfo`, `getReferralInfo`, `referralList`); 8
  profile mutations (follow/block/report, preferences, profile/bio/
  username/email edits). 73 tests total (40 new). 6 seeded users. Content
  filtering pipeline with 5 filter specs (`IllegalContentFilterSpec`,
  `SystemUserSpec`, `DeletedUserSpec`, `UserIsBlockedByMeSpec`,
  `CurrentUserIsBlockedUserSpec`).
- **Mock Backend Phase 3 — Posts & Content** ✅. `listPosts`,
  `guestListPost`, `postAction`, `createPost`, `searchTags`, ads. 121
  total tests (48 new). 0 clippy warnings.
- **Mock Backend Phase 4 — Social (Comments, Chat)** ✅. 5 comment
  resolvers (`listComments`, `listChildComments`, `createComment`,
  `likeComment`, `unlikeComment`, `reportComment`); 3 chat resolvers
  (`listChats`, `createChat`, `sendChatMessage`). 170 total tests (49
  new). Daily-free-action logic for comments. Private-chat dedup in
  `createChat`.
- **My Ads** 🟡 In Progress. Phases 1–4 implemented: stats header, ad
  listing with infinite scroll, boost-post modal (multi-step),
  `advertisementHistory` query, `advertisePostPinned` mutation, skeleton
  loading.
- Plan documents created for Forgot Password completion sprint, Mock
  Backend Phase 6, Mock Backend CI Integration, Acceptance Criteria,
  Settings, Wallet, Chat, New Post, Profile, Forgot Password.

### Changed
- 5 features promoted after cross-referencing plan-doc statuses: Dashboard
  🚧 → 🟡, Chat 🚧 → 🟡, Wallet 🚧 → 🟡, Settings 🚧 → 🟡, Profile 🚧 →
  🟡. Summary counts 3/1/8/8 → 3/6/3/8; convergence ~40% → ~50%.
- Mock Backend section added — 7 phases tracked (Phase 0–6).
- New Post status updated 📋 → 🚧 (~70% structurally complete).
- Full codebase audit: summary counts corrected to 3 Implemented, 1
  Near-Complete, 8 In Progress, 8 Not Started; convergence ~10% → ~40%.
  Multiple pages and components upgraded based on actual line counts.

### Documentation
- Plan-quality reviews recorded for Dashboard, Chat, New Post, Mock
  Backend Phase 2 (each rated ⭐⭐⭐⭐ or ⭐⭐⭐⭐⭐). Phase 0 plan doc
  reconciled with actual implementation; deviations catalogued in an
  appendix.

---

## [2026-04-12] — Initial planning blitz

### Documentation
- Implementation planning documents created for: View Post, Wallet, Chat,
  New Post, Profile.
- View Post audited: ~95% implemented (page, 8 components, 7 API fns, 7
  GraphQL ops, 2 models, 952 lines SCSS). Tasks 1–5 confirmed complete
  (follow API wiring, mobile deep-link hook, view tracking,
  `format_time_ago`).
- Dashboard marked "In Progress"; View Post promoted from "Planning" to
  "🟡 ~95% Implemented".

---

## [2026-04-10] — Foundations

### Added
- Login / Auth ✅ marked complete.
- Auth API layer: JWT, refresh, 401 handling, HttpOnly cookies, proactive
  refresh, 401 interceptor.
- State management upgraded from Basic to Implemented (`AuthContext`).
- Dashboard implementation planning document.
- Initial convergence tracker.
- Registration flow ✅ marked complete.
- Basic infrastructure (SSR, routing, build) confirmed working.

### Changed
- Convergence updated to ~10%.
