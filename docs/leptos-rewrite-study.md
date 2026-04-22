# Leptos Rewrite Study — Peer Network Web Frontend

## 1. Executive Summary

This document evaluates rewriting the **Peer Network** web frontend from its current PHP + vanilla JavaScript stack to [Leptos](https://leptos.dev/), a full-stack Rust web framework. It covers the current architecture, a proposed Leptos architecture, migration strategy, risks, and a recommended phased plan.

**Recommendation:** A Leptos rewrite is feasible and would yield significant benefits in type safety, performance, and developer experience. However, the team must weigh the Rust learning curve and ecosystem maturity against those benefits. A phased, route-by-route migration is recommended over a big-bang rewrite.

---

## 2. Current Architecture Analysis

### 2.1 Tech Stack

| Layer | Technology |
|-------|-----------|
| Server-side rendering | PHP (no framework — raw `.php` files) |
| Client-side interactivity | Vanilla JavaScript (no bundler, no framework) |
| Styling | Plain CSS (per-page stylesheets) |
| Realtime / Analytics | Firebase (Firestore, Analytics) via compat SDK |
| API communication | GraphQL over `fetch()` to a backend at `$domain/graphql` |
| Auth | JWT (access + refresh tokens stored in cookies), server-side validation via PHP `stream_context_create` |
| Media processing | FFmpeg (WASM, bundled in `js/ffmpeg/`) |
| PWA | Web manifest, service worker (commented out) |

### 2.2 Page Inventory

The app is a **blockchain-based social network** with the following pages:

| Page | File | Auth Required | Key Features |
|------|------|--------------|--------------|
| Login | `login.php` | No | Email/password login, remember-me |
| Register | `register.php` | No | Multi-step: referral code → email → password → confirmation |
| Forgot Password | `forgotpassword.php` | No | Password reset flow |
| Dashboard | `dashboard.php` | Yes | Post feed, filters, sort, infinite scroll |
| Profile | `profile.php` | Yes | User profile, posts, followers/following, peers |
| View Profile | `view-profile.php` | Yes | Other users' profiles |
| New Post | `newpost.php` | Yes | Text/media post creation, image cropping, video encoding |
| View Post | `post.php` | No (guest mode) | Single post view, comments |
| Chat | `chat.php` | Yes | Real-time messaging (Firestore), group/private chats |
| Wallet | `wallet.php` | Yes | Token balance, transfer, transaction history |
| Settings | `profileSettings.php` | Yes | Edit profile, preferences |
| Edit Profile | `edit_profile.php` | Yes | Profile editing |
| Ads | `myAds.php` | Yes | Ad management |
| Peer Shop | `viewPeerShop.php` | Yes | Shop view |
| Referral Board | `referralBoard.php` | Yes | Referral tracking |
| Invite | `invite.php` | Yes | Invite generation |
| Download | `download.php` | Yes | Force-download media proxy (Axum route) — [plan](plans/download/download-implementation.md) |
| Admin / Moderation | `admin/index.php` | Yes (admin role) | Content moderation dashboard |
| Version History | `version_history.php` | No | Release notes |

### 2.3 JavaScript Module Map

```
js/
├── global.js           — App init, auth check, user info, onboarding
├── posts.js            — Post rendering, GraphQL post queries
├── load_posts.js       — Infinite scroll, filters, sort, post click handlers
├── comments.js         — Comment creation and rendering
├── dashboard.js        — Dashboard-specific logic
├── profile.js          — Profile page logic
├── wallet.js           — Wallet transactions, balance, transfer
├── add_post.js         — Post creation form
├── crop.js             — Image cropping
├── audio.js            — Audio player
├── voiceRecorderApi.js — Voice recording
├── chat/
│   ├── index.js        — Chat init
│   ├── api.js          — GraphQL fetch wrapper
│   ├── graphql.js      — Chat query/mutation strings
│   ├── state.js        — Chat state object
│   ├── ui.js           — Chat DOM manipulation
│   ├── loader.js       — Chat list loading
│   └── utils.js        — Chat utilities
├── lib/
│   ├── const.js        — GraphQL endpoint config
│   ├── cookie.js       — Cookie helpers
│   ├── modal.js        — Modal dialog system
│   ├── scroll.js       — Scroll utilities
│   ├── string.js       — String sanitization
│   ├── time.js         — Time formatting
│   └── user.js         — User data helpers
├── login/login.js      — Login form logic
├── register/register.js— Registration flow
├── settings/           — Settings modules
├── ads/                — Ad-related modules
└── ffmpeg/             — Client-side video processing
```

### 2.4 Backend API Pattern

All data flows through a **single GraphQL endpoint** (`https://{domain}/graphql`). The frontend sends:
- **Queries:** `listPosts`, `getProfile`, `listChats`, `listFriends`, `getWalletBalance`, etc.
- **Mutations:** `sendChatMessage`, `createChat`, `createComment`, `createPost`, etc.

Auth is handled via `Authorization: Bearer {JWT}` headers. Tokens are stored in cookies and refreshed server-side in PHP (`auth.php`).

### 2.5 Current Pain Points

1. **No component model** — UI is built with raw PHP includes and DOM manipulation in JS.
2. **No build system** — JS files loaded via `<script>` tags with `filemtime()` cache busting.
3. **Global mutable state** — Variables like `likeCost`, `balance`, `storedUserInfo` leak across scripts.
4. **Duplicated logic** — Each page repeats header/meta/CSS/JS includes manually.
5. **No type safety** — Both PHP and JS are dynamically typed; GraphQL responses are untyped.
6. **Tight coupling** — PHP does server-side auth checks, then JS does everything else client-side.
7. **Manual DOM manipulation** — All UI updates done via `document.createElement()`, `innerHTML`, etc.

---

## 3. Why Leptos?

### 3.1 Framework Overview

Leptos is a **full-stack Rust web framework** that supports:
- **Server-side rendering (SSR)** with hydration
- **Client-side rendering (CSR)** via WebAssembly
- Fine-grained **reactive signals** (similar to SolidJS)
- **Server functions** — type-safe RPC between client/server
- Built-in **routing** with nested layouts
- Compile-time checked **HTML templates** (RSX macro)

### 3.2 Advantages for Peer Network

| Benefit | Details |
|---------|---------|
| **Type safety end-to-end** | Rust's type system catches errors at compile time. GraphQL response types can be modeled as structs. |
| **Performance** | Leptos' fine-grained reactivity + WASM execution is significantly faster than vanilla JS DOM manipulation. SSR provides fast initial loads. |
| **Component model** | Replaces PHP includes + JS DOM manipulation with declarative, composable components. |
| **Single language** | Rust for both server and client eliminates the PHP/JS split. |
| **Built-in routing** | Replaces the current approach of separate `.php` files per page. |
| **Server functions** | Auth validation, token refresh, and GraphQL proxying can be type-safe server functions, replacing `auth.php` and `host.php`. |
| **SEO via SSR** | Leptos SSR provides the same SEO benefits as current PHP-rendered HTML. |
| **WASM ecosystem** | FFmpeg WASM, image processing, and crypto operations work well in the Rust/WASM ecosystem. |

### 3.3 Risks and Challenges

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Rust learning curve** | High | Invest in training; Leptos syntax is approachable for web devs |
| **Ecosystem maturity** | Medium | Leptos is actively developed (v0.7+); community growing rapidly |
| **Firebase SDK** | Medium | No native Rust Firebase SDK — use REST API or JS interop via `wasm-bindgen` |
| **FFmpeg WASM** | Low | Existing WASM approach can be called from Rust via JS interop |
| **Build tooling** | Low | `cargo-leptos` and `trunk` provide good DX |
| **Deployment** | Medium | Requires a Rust application server (Actix/Axum) instead of PHP+Apache/Nginx |
| **Development speed** | Medium | Slower initial velocity; faster long-term due to fewer runtime bugs |

---

## 4. Proposed Leptos Architecture

### 4.1 Project Structure

```
/
├── Cargo.toml
├── src/
│   ├── main.rs                  — Server entry point (Axum)
│   ├── app.rs                   — Root <App/> component + router
│   ├── lib.rs                   — Shared types and re-exports
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── graphql.rs           — GraphQL client (reqwest)
│   │   ├── auth.rs              — JWT handling, token refresh (server fns)
│   │   └── firebase.rs          — Firebase REST/interop wrapper
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs              — User, Profile structs
│   │   ├── post.rs              — Post, Comment structs
│   │   ├── chat.rs              — Chat, Message structs
│   │   ├── wallet.rs            — Transaction, Balance structs
│   │   └── admin.rs             — Moderation structs
│   │
│   ├── components/
│   │   ├── mod.rs
│   │   ├── layout.rs            — SiteLayout, Header, Footer, Sidebars
│   │   ├── post_card.rs         — PostCard, PostList
│   │   ├── comment.rs           — CommentThread, CommentForm
│   │   ├── chat_list.rs         — ChatList, ChatItem
│   │   ├── chat_window.rs       — ChatWindow, MessageBubble
│   │   ├── wallet_balance.rs    — WalletBalance, TransactionItem
│   │   ├── profile_header.rs    — ProfileHeader, ProfileStats
│   │   ├── filter_sidebar.rs    — FilterWidget, SortWidget
│   │   ├── modal.rs             — Modal dialog component
│   │   ├── media_upload.rs      — MediaUpload, ImageCropper
│   │   ├── audio_player.rs      — AudioPlayer
│   │   └── onboarding.rs        — OnboardingPopup
│   │
│   ├── pages/
│   │   ├── mod.rs
│   │   ├── login.rs             — LoginPage
│   │   ├── register.rs          — RegisterPage (multi-step)
│   │   ├── forgot_password.rs   — ForgotPasswordPage
│   │   ├── dashboard.rs         — DashboardPage
│   │   ├── profile.rs           — ProfilePage
│   │   ├── view_profile.rs      — ViewProfilePage
│   │   ├── new_post.rs          — NewPostPage
│   │   ├── view_post.rs         — ViewPostPage (guest-accessible)
│   │   ├── chat.rs              — ChatPage
│   │   ├── wallet.rs            — WalletPage
│   │   ├── settings.rs          — SettingsPage
│   │   ├── admin.rs             — AdminPage (moderation)
│   │   ├── referral.rs          — ReferralBoardPage
│   │   ├── invite.rs            — InvitePage
│   │   ├── peer_shop.rs         — PeerShopPage
│   │   └── not_found.rs         — 404 page
│   │
│   └── state/
│       ├── mod.rs
│       ├── auth.rs              — AuthContext (signals for user, token)
│       ├── theme.rs             — Theme/zoom preferences
│       └── app_state.rs         — Global app state (tokenomics, etc.)
│
├── style/
│   ├── main.scss                — Global styles (migrated from css/)
│   └── components/              — Per-component styles
│
├── public/
│   ├── fonts/                   — Poppins, Peer icon font
│   ├── img/
│   ├── svg/
│   └── manifest.json
│
└── end2end/                     — Playwright or similar E2E tests
```

### 4.2 Router Design

```rust
use leptos::*;
use leptos_router::*;

#[component]
fn App() -> impl IntoView {
    provide_context(AuthState::new());

    view! {
        <Router>
            <Routes>
                // Public routes
                <Route path="/login" view=LoginPage />
                <Route path="/register" view=RegisterPage />
                <Route path="/forgot-password" view=ForgotPasswordPage />
                <Route path="/post/:id" view=ViewPostPage />
                <Route path="/download" view=DownloadPage />
                <Route path="/version-history" view=VersionHistoryPage />

                // Protected routes (wrapped in auth guard)
                <ProtectedRoute path="/" view=AuthLayout redirect="/login">
                    <Route path="" view=DashboardPage />
                    <Route path="profile" view=ProfilePage />
                    <Route path="user/:slug" view=ViewProfilePage />
                    <Route path="new-post" view=NewPostPage />
                    <Route path="chat" view=ChatPage />
                    <Route path="wallet" view=WalletPage />
                    <Route path="settings" view=SettingsPage />
                    <Route path="referrals" view=ReferralBoardPage />
                    <Route path="invite" view=InvitePage />
                    <Route path="shop" view=PeerShopPage />
                    <Route path="ads" view=MyAdsPage />
                    <Route path="admin" view=AdminPage />
                </ProtectedRoute>

                <Route path="/*" view=NotFoundPage />
            </Routes>
        </Router>
    }
}
```

### 4.3 Auth Pattern (Replacing `auth.php`)

```rust
// Server function — runs on the server, called from client
#[server(Login, "/api")]
pub async fn login(email: String, password: String, remember_me: bool) -> Result<UserSession, ServerFnError> {
    let response = graphql_client::login(&email, &password).await?;
    let session = extract_session(response)?;

    // Set HTTP-only cookies server-side
    set_cookie("authToken", &session.access_token, remember_me);
    set_cookie("refreshToken", &session.refresh_token, remember_me);

    Ok(session)
}

#[server(RefreshToken, "/api")]
pub async fn refresh_token() -> Result<String, ServerFnError> {
    let refresh = get_cookie("refreshToken")?;
    let new_tokens = graphql_client::refresh(&refresh).await?;
    set_cookie("authToken", &new_tokens.access_token, true);
    Ok(new_tokens.access_token)
}
```

### 4.4 GraphQL Integration

```rust
// src/api/graphql.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GraphQLRequest<V: Serialize> {
    query: &'static str,
    variables: V,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

pub async fn query<T: for<'de> Deserialize<'de>, V: Serialize>(
    query: &'static str,
    variables: V,
    token: &str,
) -> Result<T, ApiError> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("https://{}/graphql", get_domain()))
        .bearer_auth(token)
        .json(&GraphQLRequest { query, variables })
        .send()
        .await?
        .json::<GraphQLResponse<T>>()
        .await?;

    match (resp.data, resp.errors) {
        (Some(data), _) => Ok(data),
        (_, Some(errors)) => Err(ApiError::GraphQL(errors)),
        _ => Err(ApiError::EmptyResponse),
    }
}
```

### 4.5 Reactive State (Replacing Global JS Variables)

```rust
// src/state/auth.rs
#[derive(Clone)]
pub struct AuthState {
    pub user: RwSignal<Option<User>>,
    pub token: RwSignal<Option<String>>,
    pub balance: RwSignal<f64>,
    pub tokenomics: RwSignal<Option<Tokenomics>>,
}

// Provided at app root, consumed anywhere:
let auth = use_context::<AuthState>();
```

### 4.6 Firebase Integration

Since there is no native Rust Firebase SDK, two options exist:

**Option A — JS Interop (recommended for Firestore realtime)**
```rust
// Use wasm-bindgen to call the existing Firebase JS SDK
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = firebase)]
    fn firestore() -> JsValue;
}
```

**Option B — Firebase REST API (for non-realtime operations)**
```rust
// Use reqwest to call Firebase REST endpoints server-side
pub async fn log_analytics_event(event: &str, params: &serde_json::Value) {
    // POST to Firebase Analytics Measurement Protocol
}
```

### 4.7 CSS Migration

The existing CSS can be largely reused. Recommended approach:
1. Copy all CSS files as-is into `style/legacy/`.
2. Import them globally during migration.
3. Gradually refactor into per-component CSS modules or use Leptos' `style` macro.
4. The Poppins font and Peer icon font remain in `public/fonts/`.

---

## 5. Migration Strategy

### 5.1 Recommended Approach: Phased Route-by-Route

A big-bang rewrite is risky. Instead, use a **strangler fig pattern**:

1. Stand up the Leptos app alongside the existing PHP app.
2. Migrate one route at a time, starting with the simplest.
3. Use a reverse proxy (Nginx/Caddy) to route traffic between PHP and Leptos.
4. Once all routes are migrated, decommission the PHP app.

### 5.2 Phase Plan

#### Phase 0 — Foundation (Weeks 1–3)
- [ ] Initialize `cargo-leptos` project
- [ ] Set up Axum server with SSR + hydration
- [ ] Implement `AuthState`, cookie handling, token refresh
- [ ] Build `GraphQL` client module with typed response structs
- [ ] Create shared layout components: `SiteLayout`, `Header`, `Footer`, `Sidebar`
- [ ] Migrate CSS files into the project
- [ ] Set up CI/CD pipeline for Rust/WASM builds

#### Phase 1 — Public Pages (Weeks 3–5)
- [ ] `LoginPage` — form, validation, auth flow
- [ ] `RegisterPage` — multi-step referral/email/password flow
- [ ] `ForgotPasswordPage`
- [ ] `ViewPostPage` (guest mode)
- [ ] `NotFoundPage` (404)
- [ ] `VersionHistoryPage`
- [ ] `DownloadPage`

#### Phase 2 — Core Feed (Weeks 5–8)
- [ ] `DashboardPage` — post feed, infinite scroll, filters, sort
- [ ] `PostCard` component — likes, dislikes, comments, views
- [ ] `CommentThread` component
- [ ] `FilterSidebar` and `SortWidget` components
- [ ] `Modal` component (replaces `js/lib/modal.js`)
- [ ] `AudioPlayer` component

#### Phase 3 — User Features (Weeks 8–11)
- [ ] `ProfilePage` — own profile, stats, post grid
- [ ] `ViewProfilePage` — other users, follow/unfollow
- [ ] `SettingsPage` — edit profile, preferences
- [ ] `NewPostPage` — post creation, media upload, image cropping
- [ ] `InvitePage` and `ReferralBoardPage`

#### Phase 4 — Chat & Wallet (Weeks 11–14)
- [ ] `ChatPage` — chat list, message window, Firebase Firestore integration
- [ ] `WalletPage` — balance display, transfer, transaction history
- [ ] `PeerShopPage`
- [ ] `MyAdsPage`

#### Phase 5 — Admin & Polish (Weeks 14–16)
- [x] `AdminPage` — content moderation dashboard
- [ ] Onboarding popup flow
- [ ] PWA manifest + service worker
- [ ] Performance optimization (code splitting, lazy loading)
- [ ] E2E test suite
- [ ] Decommission PHP app

### 5.3 Parallel Operation During Migration

```
                   ┌──────────────────────┐
                   │   Reverse Proxy       │
    Browser ──────►│   (Nginx / Caddy)     │
                   └──────┬───────┬────────┘
                          │       │
              Leptos      │       │  PHP (legacy)
              routes      │       │  routes
                    ▼             ▼
           ┌──────────┐   ┌──────────┐
           │  Axum +   │   │  Apache/  │
           │  Leptos   │   │  Nginx +  │
           │  (Rust)   │   │  PHP      │
           └──────────┘   └──────────┘
                    │              │
                    └──────┬───────┘
                           ▼
                   ┌──────────────┐
                   │  GraphQL     │
                   │  Backend     │
                   └──────────────┘
```

Both apps share the same backend GraphQL API and auth cookies. The proxy forwards `/login`, `/register` etc. to Leptos once migrated, and everything else to PHP until its route is also migrated.

---

## 6. Key Technical Decisions

### 6.1 SSR vs CSR vs Islands

| Mode | Use Case |
|------|----------|
| **SSR + Hydration** | Default for all pages — good SEO, fast first paint |
| **CSR-only islands** | Chat (Firestore realtime), media upload/crop |
| **Server-only** | Auth token refresh, admin data fetching |

Leptos supports all three in the same app.

### 6.2 State Management

| Current (JS) | Leptos Equivalent |
|--------------|-------------------|
| Global `let` variables | `RwSignal<T>` in context |
| `localStorage.getItem("userData")` | `leptos_use::use_local_storage` |
| Cookie-based auth | `tower-cookies` on server, `leptos_use::use_cookie` on client |
| Event-driven DOM updates | Reactive `view!` macro with signals |

### 6.3 Build & Deploy

| Concern | Solution |
|---------|----------|
| Build tool | `cargo-leptos` (builds server + WASM client) |
| CSS | Sass via `grass` crate or plain CSS |
| Bundling | Built-in WASM optimization + `wasm-opt` |
| Deployment | Single binary (Axum server) + WASM + static assets |
| Container | Single Dockerfile (`rust:slim` → `debian:slim` multi-stage) |

### 6.4 Dependencies (Cargo.toml)

```toml
[dependencies]
leptos = { version = "0.7", features = ["ssr", "csr"] }
leptos_router = "0.7"
leptos_axum = "0.7"
leptos_meta = "0.7"
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json"] }
jsonwebtoken = "9"
tower-cookies = "0.10"
chrono = "0.4"
thiserror = "2"
tracing = "0.1"

# Client-only
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Document", "Window", "HtmlElement"] }
gloo = "0.11"
leptos_use = "0.14"
```

---

## 7. Effort Estimation

| Phase | Scope | Estimated Effort |
|-------|-------|-----------------|
| Phase 0 | Foundation | 2–3 weeks (1 dev) |
| Phase 1 | Public pages | 2 weeks |
| Phase 2 | Core feed | 3 weeks |
| Phase 3 | User features | 3 weeks |
| Phase 4 | Chat & Wallet | 3 weeks |
| Phase 5 | Admin & Polish | 2 weeks |
| **Total** | | **15–16 weeks (1 dev)** |

With 2 developers working in parallel on different phases, this could compress to ~10 weeks.

---

## 8. Alternatives Considered

| Framework | Pros | Cons | Verdict |
|-----------|------|------|---------|
| **Leptos** | Full-stack Rust, SSR+CSR, fine-grained reactivity | Rust learning curve, smaller ecosystem | **Recommended** |
| **Dioxus** | Rust, React-like, multi-platform | Less mature SSR, different mental model | Good alternative |
| **Next.js** | Huge ecosystem, easy JS migration | Still JavaScript, no Rust benefits | Easier but less transformative |
| **SvelteKit** | Small bundle, great DX | Still JavaScript | Lower migration cost, fewer long-term gains |
| **Yew** | Rust + WASM, mature | React-like VDOM (slower), less ergonomic | Leptos is the better Rust choice |

---

## 9. Conclusion

The current PHP + vanilla JS architecture has served its purpose but is reaching its limits in maintainability, type safety, and developer experience. Leptos offers a modern, performant, and type-safe alternative that aligns well with Peer Network's blockchain-oriented identity.

**Key takeaways:**
1. The GraphQL-only backend makes this migration clean — no PHP backend logic needs porting.
2. PHP only serves as a template engine and auth proxy — both are well-handled by Leptos SSR + server functions.
3. The existing CSS can be reused during migration.
4. Firebase integration requires JS interop but is tractable.
5. A phased migration minimizes risk and allows continuous delivery.

The biggest investment is the team's Rust proficiency. If the team is committed to Rust, this rewrite will pay dividends in reliability, performance, and long-term velocity.
