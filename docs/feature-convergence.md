# Feature Convergence Tracker

This document tracks the progress of migrating features from the legacy PHP/JS frontend to the new Leptos (Rust/WASM) rewrite.

**Last Updated:** 2026-04-12

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Implemented | 2 |
| 🚧 In Progress | 1 |
| 📋 Planning | 3 |
| ❌ Not Started | 14 |
| **Total** | **20** |

**Convergence:** ~10%

---

## Pages

| Feature | Legacy File | peer-web Status | Notes |
|---------|-------------|-----------------|-------|
| **Authentication** ||||
| Login | `login.php` | ✅ Implemented | Email/password, remember-me, auto-login, redirect handling |
| Register | `register.php` | ✅ Implemented | Multi-step: referral → email → password → confirmation |
| Forgot Password | `forgotpassword.php` | ❌ Not Started | Password reset flow |
| **Core Features** ||||
| Dashboard | `dashboard.php` | 🚧 In Progress | Post feed, filters, sort, infinite scroll |
| View Post | `post.php` | 📋 Planning | Single post view, comments, guest mode |
| New Post | `newpost.php` | 📋 Planning | Text/media creation, image cropping, video encoding ([docs](plans/new-post/new-post-implementation.md)) |
| **Profile** ||||
| My Profile | `profile.php` | 📋 Planning | User profile, posts, followers/following ([docs](plans/profile/profile-implementation.md)) |
| View Profile | `view-profile.php` | 📋 Planning | Other users' profiles ([docs](plans/profile/profile-implementation.md)) |
| Edit Profile | `edit_profile.php` | ❌ Not Started | Profile editing |
| Settings | `profileSettings.php` | ❌ Not Started | Preferences, account settings |
| **Social** ||||
| Chat | `chat.php` | ❌ Not Started | Real-time messaging (Firestore), group/private |
| Invite | `invite.php` | ❌ Not Started | Invite generation |
| Referral Board | `referralBoard.php` | ❌ Not Started | Referral tracking |
| **Economy** ||||
| Wallet | `wallet.php` | ❌ Not Started | Token balance, transfer, transaction history |
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
| Post Card | `js/posts.js` | ❌ Not Started | Post rendering |
| Post List | `js/load_posts.js` | ❌ Not Started | Infinite scroll, filters |
| Comments | `js/comments.js` | ❌ Not Started | Comment thread, form |
| **Chat** ||||
| Chat List | `js/chat/` | ❌ Not Started | Chat list UI |
| Chat Window | `js/chat/` | ❌ Not Started | Message bubbles, input |
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
| Image Cropper | `js/crop.js` | ❌ Not Started | Image cropping |
| Audio Player | `js/audio.js` | ❌ Not Started | Audio playback |
| Video Encoder | `js/ffmpeg/` | ❌ Not Started | FFmpeg WASM |
| **UI** ||||
| Modal | `js/lib/modal.js` | ❌ Not Started | Dialog system |
| Toast | — | ✅ Implemented | Notification toasts |
| Back Button | — | ✅ Implemented | Navigation |
| Step Announcer | — | ✅ Implemented | Accessibility |

---

## API Layer

| API Module | Legacy Location | peer-web Status | Notes |
|------------|----------------|-----------------|-------|
| GraphQL Client | `js/lib/const.js` | ✅ Implemented | `src/api/graphql.rs` |
| Auth (JWT) | `auth.php` | ✅ Implemented | Server fns, HttpOnly cookies, proactive refresh, 401 interceptor |
| Registration | `js/register/` | ✅ Implemented | Verify referral, register user |
| Posts | `js/posts.js` | ❌ Not Started | CRUD operations |
| Comments | `js/comments.js` | ❌ Not Started | Create, list comments |
| Chat | `js/chat/api.js` | ❌ Not Started | Chat queries/mutations |
| Wallet | `js/wallet.js` | ❌ Not Started | Balance, transfers |
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

## Migration Priority (Recommended Order)

1. ✅ ~~Registration~~ — Complete
2. ✅ ~~Login / Auth~~ — Complete ([docs](plans/login/login-auth-implementation.md))
3. � Dashboard — In progress ([docs](plans/dashboard/dashboard-implementation.md))
4. 📋 View Post — Planning ([docs](plans/view-post/view-post-implementation.md))
5. 📋 Profile — Planning ([docs](plans/profile/profile-implementation.md))
6. 📋 New Post — Content creation ([docs](plans/new-post/new-post-implementation.md))
7. ⬜ Chat — Real-time, complex
8. ⬜ Wallet — Financial features
9. ⬜ Settings — User preferences
10. ⬜ Admin — Moderation tools
11. ⬜ Remaining pages

---

## Changelog

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
