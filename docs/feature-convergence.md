# Feature Convergence Tracker

This document tracks the progress of migrating features from the legacy PHP/JS frontend to the new Leptos (Rust/WASM) rewrite.

**Last Updated:** 2026-04-14

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Implemented | 3 |
| 🟡 Near-Complete | 6 |
| 🚧 In Progress | 3 |
| ❌ Not Started | 8 |
| **Total** | **20** |

**Convergence:** ~50%

---

## Pages

| Feature | Legacy File | peer-web Status | Notes |
|---------|-------------|-----------------|-------|
| **Authentication** ||||
| Login | `login.php` | ✅ Implemented | Email/password, remember-me, auto-login, redirect handling ([docs](plans/login/login-auth-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5) |
| Register | `register.php` | ✅ Implemented | Multi-step: referral → email → password → confirmation |
| Forgot Password | `forgotpassword.php` | 🚧 In Progress | 990-line page, full 4-step flow (email → verify → reset → success), rate-limited resend, API fns implemented ([docs](plans/forgot-password/forgot-password-implementation.md)). **Gaps:** no auto-redirect for authenticated users, resend counter not cookie-persisted (cooldown bypass on reload), countdown interval stacking bug, `BackButton` not reused |
| **Core Features** ||||
| Dashboard | `dashboard.php` | � Mostly Implemented | Post feed, filters, sort, infinite scroll ([docs](plans/dashboard/dashboard-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5). **Gaps:** post click → view post overlay is a TODO stub, profile widget not wired to auth context |
| View Post | `post.php` | 🟡 ~95% Implemented | Single post view, comments, guest mode ([docs](plans/view-post/view-post-implementation.md), [sprint](plans/view-post/view-post-completion-sprint.md)) |
| New Post | `newpost.php` | 🚧 In Progress | Text/media creation, image cropping, video encoding ([docs](plans/new-post/new-post-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5) |
| **Profile** ||||
| My Profile | `profile.php` | 🟡 Mostly Implemented | 238-line page, profile header, post list, skeleton loading, API integration ([docs](plans/profile/profile-implementation.md)). **Gaps:** infinite scroll not wired (fixed page), filter sidebar placeholder, legacy `?user=` query param unsupported |
| View Profile | `view-profile.php` | 🟡 Mostly Implemented | 274-line page, follow/block/report actions, profile fetch, post list ([docs](plans/profile/profile-implementation.md)). **Gaps:** infinite scroll not wired, filter sidebar placeholder |
| Edit Profile | `edit_profile.php` | ❌ Not Started | Profile editing (covered partially by Settings) |
| Settings | `profileSettings.php` | � Implemented (with gaps) | 142-line page + 7 sub-components (profile, passwords, email, username, content, notifications, preferences), API layer (282L) ([docs](plans/settings/settings-implementation.md)). **Gaps:** deactivate/delete account UI not wired (API exists, menu is a no-op), profile save runs sequentially instead of in parallel |
| **Social** ||||
| Chat | `chat.php` | 🟡 Core Implemented | 141-line page + 7 components (chat_list, contacts_overlay, group_review, chat_input, chat_messages, chat_container, chat_item), API layer (151L), state module, SCSS (851L) ([docs](plans/chat/chat-implementation.md)) — Plan quality: ⭐⭐⭐⭐ (4/5). **Gaps:** Firebase real-time listener missing (no polling fallback), unread indicators missing, chat search logic not connected |
| Invite | `invite.php` | ❌ Not Started | Invite generation |
| Referral Board | `referralBoard.php` | ❌ Not Started | Referral link, invited/inviter tabs, user cards ([docs](plans/referral-board/referral-board-implementation.md)) |
| **Economy** ||||
| Wallet | `wallet.php` | 🟡 Implemented (tests pending) | 146-line page + transfer_modal (760L), balance_header, transaction_history, transaction_item, API layer (246L), SCSS (1092L) ([docs](plans/wallet/wallet-implementation.md)). **Gaps:** shop purchase order details UI not wired (model + query exist), thousand-separator formatting missing |
| Peer Shop | `viewPeerShop.php` | ❌ Not Started | Shop view |
| My Ads | `myAds.php` | ❌ Not Started | Ad management |
| **Admin** ||||
| Admin Dashboard | `admin/index.php` | ❌ Not Started | Content moderation |
| **Misc** ||||
| Download | `download.php` | ❌ Not Started | App download page |
| Version History | `version_history.php` | ❌ Not Started | Release notes |
| 404 Page | `404.php` | ✅ Implemented | Fallback route in router |

---

## Components

| Component | Legacy Location | peer-web Status | Notes |
|-----------|----------------|-----------------|-------|
| **Layout** ||||
| Header | `template-parts/` | ❌ Not Started | Navigation, user menu |
| Footer | `template-parts/footer.php` | ❌ Not Started | Site footer |
| Sidebars | `template-parts/sidebars/` | ❌ Not Started | Filter, navigation sidebars |
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
| Comments | `js/comments.js` | 🚧 In Progress | `src/api/comments.rs` (204L) — get_post, guest_get_post, list_comments, create_comment, like/unlike |
| Chat | `js/chat/api.js` | 🚧 In Progress | `src/api/chat.rs` (151L) — list_chats, send_message, create_chat |
| Wallet | `js/wallet.js` | 🚧 In Progress | `src/api/wallet.rs` (246L) — get_balance, transaction_history, transfer_tokens |
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
| PWA / Manifest | `json/webmanifest.json` | ❌ Not Started | Service worker |
| E2E Tests | None | 🚧 In Progress | Playwright setup |

---

## Mock Backend (Rust Rewrite)

Tracks the incremental Rust mock backend that replaces the Node.js mock for offline E2E development. Parent plan: [mock-backend-rust-rewrite.md](plans/mock-backend/mock-backend-rust-rewrite.md)

| Phase | Scope | Plan | Quality | Status |
|-------|-------|------|---------|--------|
| 0 — Skeleton & Parity | 3 registration mutations, health check | [phase-0](plans/mock-backend/phase-0-mock-backend-skeleton.md) | — | ✅ Done (Node.js parity) |
| 1 — Login & Session | login, refreshToken, logout, deleteAccount, updatePassword, password reset | [phase-1](plans/mock-backend/phase-1-login-session-flows.md) | ⭐⭐⭐⭐⭐ (5/5) | 🚧 In Progress |
| 2 — Users & Profiles | getProfile, searchUser, follow/block/report, preferences, profile edits | [phase-2](plans/mock-backend/phase-2-users-and-profiles.md) | — | ❌ Not Started |
| 3 — Posts & Content | listPosts, guestListPost, postAction, createPost, searchTags, ads | [phase-3](plans/mock-backend/phase-3-posts-content.md) | ⭐⭐⭐⭐⭐ (5/5) | ❌ Not Started |
| 4 — Social (Comments, Chat) | listComments, createComment, like/unlike, listChats, sendMessage, createChat | [phase-4](plans/mock-backend/phase-4-social-comments-chat.md) | ⭐⭐⭐⭐⭐ (5/5) | ❌ Not Started |
| 5 — Economy (Wallet, Shop, Ads) | balance, transferTokens, transactionHistory, shopOrderDetails, ads | [phase-5](plans/mock-backend/phase-5-economy-wallet-tokenomics-shop-ads.md) | ⭐⭐⭐⭐⭐ (5/5) | ❌ Not Started |
| 6 — Admin & Moderation | (not yet planned) | — | — | ❌ Not Started |

---

## Migration Priority (Recommended Order)

1. ✅ ~~Registration~~ — Complete
2. ✅ ~~Login / Auth~~ — Complete ([docs](plans/login/login-auth-implementation.md))
3. 🟡 Dashboard — Mostly implemented, view-post overlay stub remains ([docs](plans/dashboard/dashboard-implementation.md))
3b. 🚧 Forgot Password — In progress, 990-line page implemented ([docs](plans/forgot-password/forgot-password-implementation.md))
4. 🟡 View Post — ~95% implemented, polish & testing remain ([docs](plans/view-post/view-post-implementation.md))
5. 🟡 Profile — Mostly implemented, infinite scroll + filters remain ([docs](plans/profile/profile-implementation.md))
6. 🚧 New Post — In progress, ~70% structural ([docs](plans/new-post/new-post-implementation.md))
7. 🟡 Chat — Core implemented, Firebase real-time missing ([docs](plans/chat/chat-implementation.md))
8. 🟡 Wallet — Implemented (tests pending), shop order UI not wired ([docs](plans/wallet/wallet-implementation.md))
9. 🟡 Settings — Implemented (with gaps), delete account UI not wired ([docs](plans/settings/settings-implementation.md))
10. ⬜ Admin — Moderation tools (no plan yet)
11. ⬜ Remaining pages — Invite, Referral Board, Peer Shop, My Ads, Download, Version History (no plans yet)

---

## Changelog

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
- **Missing plans noted:** 8 "Not Started" features + Header/Footer/Sidebars + Firebase + PWA have no planning docs yet

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
