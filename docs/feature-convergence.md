# Feature Convergence Tracker

This document tracks the progress of migrating features from the legacy PHP/JS frontend to the new Leptos (Rust/WASM) rewrite.

**Last Updated:** 2026-04-10

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Implemented | 2 |
| 🚧 In Progress | 0 |
| ❌ Not Started | 18 |
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
| Dashboard | `dashboard.php` | ❌ Not Started | Post feed, filters, sort, infinite scroll |
| View Post | `post.php` | ❌ Not Started | Single post view, comments, guest mode |
| New Post | `newpost.php` | ❌ Not Started | Text/media creation, image cropping, video encoding |
| **Profile** ||||
| My Profile | `profile.php` | ❌ Not Started | User profile, posts, followers/following |
| View Profile | `view-profile.php` | ❌ Not Started | Other users' profiles |
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
3. 📋 Dashboard — [Planning doc](plans/dashboard/dashboard-implementation.md)
4. ⬜ View Post — Guest-accessible, SEO important
5. ⬜ Profile — User identity
6. ⬜ New Post — Content creation
7. ⬜ Chat — Real-time, complex
8. ⬜ Wallet — Financial features
9. ⬜ Settings — User preferences
10. ⬜ Admin — Moderation tools
11. ⬜ Remaining pages

---

## Changelog

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
