# Login & Authentication Implementation Plan

**Feature:** Login & Authentication  
**Priority:** #2 (after Registration)  
**Status:** 📋 Planning  
**Created:** 2026-04-10

---

## Overview

Implement the login page and authentication infrastructure for the Leptos frontend, enabling users to sign in with email/password and maintain authenticated sessions via JWT tokens.

### Goals

1. Full parity with legacy `login.php` user experience
2. JWT token handling (access + refresh tokens)
3. "Remember me" persistent sessions
4. Auto-login for returning users with valid tokens
5. Protected route infrastructure for authenticated pages

---

## Scope

### In Scope

- [x] Login page UI (`/login`)
- [x] Email/password authentication
- [x] Remember me checkbox
- [x] Token storage (cookies)
- [x] Auto-login on page load (silent refresh)
- [x] Redirect to dashboard on success
- [x] Link to forgot password
- [x] Link to registration
- [x] Error messaging (invalid credentials, session expired, etc.)

### Out of Scope (Future Work)

- Forgot password flow (separate feature)
- OAuth/social login
- Two-factor authentication
- Admin authentication

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `login.php` | Page template with login form |
| `js/login/login.js` | Form validation, API calls, auto-login |
| `auth.php` | PHP token utilities (decode, refresh, cookies) |
| `css/login-register.css` | Shared styles with registration |

### Key Features

1. **Query Parameter Messages**
   - `?message=unauthorized` → "You do not have access. Please log in to continue."
   - `?message=sessionExpired` → "Your session has expired. Please log in again."
   - `?message=mustLogin` → "Please log in to access your dashboard."
   - `?message=walletAccessDenied` → "Please log in to access your wallet."

2. **Form Fields**
   - Email (required, validated format)
   - Password (required, non-empty)
   - Remember me (checkbox, default checked)

3. **Auto-Login Flow**
   ```
   Page Load
     ↓
   Check rememberMe flag (localStorage/cookie)
     ↓ (if true)
   Try refresh token first
     ↓ (if fails)
   Try stored email/password
     ↓ (if success)
   Redirect to dashboard
   ```

4. **Token Storage**
   - `authToken` (cookie) — JWT access token
   - `refreshToken` (cookie) — JWT refresh token
   - `userEmail` (cookie) — stored for auto-login
   - `userPassword` (cookie) — stored for auto-login (⚠️ security concern)
   - `rememberMe` (cookie + localStorage) — persistence flag

---

## Backend API Reference

### `login` Mutation

```graphql
mutation Login($email: String!, $password: String!) {
  login(email: $email, password: $password) {
    status
    ResponseCode
    accessToken
    refreshToken
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `10801` | Login successful |
| `30801` | Invalid credentials |
| `60801` | Account not verified |
| `40801` | Internal server error |

### `refreshToken` Mutation

```graphql
mutation RefreshToken($refreshToken: String!) {
  refreshToken(refreshToken: $refreshToken) {
    status
    ResponseCode
    accessToken
    refreshToken
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `10901` | Token refreshed successfully |
| `30101` | Missing refresh token |
| `30901` | Invalid/expired/revoked token |
| `40901` | Internal server error |

### `logout` Mutation

```graphql
mutation Logout($refreshToken: String!) {
  logout(refreshToken: $refreshToken) {
    status
    ResponseCode
  }
}
```

#### Response Codes

| Code | Description |
|------|-------------|
| `11001` | Logout successful |
| `30101` | Missing refresh token |
| `41001` | Internal server error |

---

## Implementation Plan

### Phase 1: Core Infrastructure

#### 1.1 Auth Models (`src/models/auth.rs`)

```rust
/// JWT token pair returned by login/refresh.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthPayload {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

/// Stored auth state for the application.
#[derive(Debug, Clone, Default)]
pub struct AuthState {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub is_authenticated: bool,
}

/// Login form data.
#[derive(Debug, Clone, Serialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}
```

#### 1.2 Auth API (`src/api/auth.rs`)

Server functions for authentication:

```rust
/// Login user with email and password.
#[server(Login, "/api")]
pub async fn login(
    email: String,
    password: String,
) -> Result<AuthPayload, ServerFnError>;

/// Refresh access token using refresh token.
#[server(RefreshToken, "/api")]
pub async fn refresh_token(
    refresh_token: String,
) -> Result<AuthPayload, ServerFnError>;

/// Logout and invalidate refresh token.
#[server(Logout, "/api")]
pub async fn logout(
    refresh_token: String,
) -> Result<(), ServerFnError>;
```

GraphQL queries:

```rust
pub const LOGIN_MUTATION: &str = r#"
    mutation Login($email: String!, $password: String!) {
        login(email: $email, password: $password) {
            status
            ResponseCode
            accessToken
            refreshToken
        }
    }
"#;

pub const REFRESH_TOKEN_MUTATION: &str = r#"
    mutation RefreshToken($refreshToken: String!) {
        refreshToken(refreshToken: $refreshToken) {
            status
            ResponseCode
            accessToken
            refreshToken
        }
    }
"#;
```

#### 1.3 Auth State Management (`src/state/auth.rs`)

```rust
use leptos::prelude::*;

/// Global auth context provider.
#[derive(Clone)]
pub struct AuthContext {
    pub is_authenticated: RwSignal<bool>,
    pub access_token: RwSignal<Option<String>>,
    // Methods
    pub login: Action<LoginCredentials, Result<(), String>>,
    pub logout: Action<(), ()>,
    pub refresh: Action<(), Result<(), String>>,
}

/// Provide auth context at app root.
pub fn provide_auth_context();

/// Use auth context in components.
pub fn use_auth() -> AuthContext;
```

### Phase 2: Login Page UI

#### 2.1 Login Page Component (`src/pages/login.rs`)

Structure matching legacy layout:

```rust
#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <Title text="Peer Network - Login"/>
        
        <div class="container large_font">
            // Left panel: phone mockup (reuse from register)
            <LeftPanel image="login"/>
            
            // Right panel: login form
            <div class="container_right">
                <div class="container_inner">
                    <TopMessageArea/>
                    <LoginForm/>
                    <FooterArea/>
                </div>
            </div>
        </div>
    }
}
```

#### 2.2 Login Form Component (`src/components/login_form.rs`)

```rust
#[component]
pub fn LoginForm() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let remember_me = RwSignal::new(true); // default checked
    let show_password = RwSignal::new(false);
    let is_loading = RwSignal::new(false);
    
    // Validation states
    let email_error = RwSignal::new(Option::<String>::None);
    let password_error = RwSignal::new(Option::<String>::None);
    
    // ... form implementation
}
```

#### 2.3 Shared Layout Components

Extract reusable components from registration:

- `LeftPanel` — phone mockup with image and logo
- `FooterArea` — privacy policy link + version number

### Phase 3: Token Management (Client-Side)

#### 3.1 Cookie Utilities (`src/utils/cookies.rs`)

```rust
/// Get a cookie value by name.
pub fn get_cookie(name: &str) -> Option<String>;

/// Set a cookie with optional expiry.
pub fn set_cookie(name: &str, value: &str, days: Option<i32>);

/// Delete a cookie.
pub fn delete_cookie(name: &str);

/// Set all auth-related cookies.
pub fn set_auth_cookies(
    access_token: &str,
    refresh_token: &str,
    remember_me: bool,
    email: Option<&str>,
);

/// Clear all auth-related cookies.
pub fn clear_auth_cookies();
```

#### 3.2 Auto-Login Hook (`src/hooks/use_auto_login.rs`)

```rust
/// Auto-login on mount if remember_me is set.
/// Returns loading state.
pub fn use_auto_login() -> Signal<bool> {
    // 1. Check remember_me flag
    // 2. Try refresh_token first
    // 3. If fails and stored credentials exist, try login
    // 4. Redirect to dashboard on success
}
```

### Phase 4: Protected Routes

#### 4.1 Auth Guard Component

```rust
/// Wraps protected routes, redirecting to login if not authenticated.
/// Preserves the original URL for redirect after login.
#[component]
pub fn AuthGuard(children: Children) -> impl IntoView {
    let auth = use_auth();
    let location = use_location();
    
    // Build redirect URL with original path
    let redirect_url = move || {
        let path = location.pathname.get();
        format!("/login?message=mustLogin&redirect={}", urlencoding::encode(&path))
    };
    
    view! {
        <Show
            when=move || auth.is_authenticated.get()
            fallback=move || view! { <Redirect to=redirect_url()/> }
        >
            {children()}
        </Show>
    }
}
```

### Phase 5: Token Refresh Strategy

#### 5.1 Proactive Token Refresh

Refresh tokens proactively before expiry to avoid jarring UX interruptions:

```rust
/// Decode JWT to extract expiry time (exp claim).
pub fn token_expiry_secs(token: &str) -> Option<i64>;

/// Set up an effect that refreshes the access token 1 minute before expiry.
pub fn use_proactive_refresh() {
    let auth = use_auth();
    
    Effect::new(move || {
        if let Some(token) = auth.access_token.get() {
            if let Some(exp) = token_expiry_secs(&token) {
                let now = js_sys::Date::now() as i64 / 1000;
                let refresh_in = (exp - now - 60).max(0) as u32; // 1 min before expiry
                
                set_timeout(
                    move || auth.refresh.dispatch(()),
                    Duration::from_secs(refresh_in.into()),
                );
            }
        }
    });
}
```

#### 5.2 401 Response Interceptor

Handle expired tokens during API calls:

```rust
/// Wrapper for authenticated API calls that handles token refresh on 401.
pub async fn auth_fetch<T>(
    request: impl Fn() -> Future<Output = Result<T, ServerFnError>>,
) -> Result<T, ServerFnError> {
    match request().await {
        Err(e) if is_unauthorized(&e) => {
            // Attempt refresh
            refresh_token_action().await?;
            // Retry original request
            request().await
        }
        result => result,
    }
}
```

---

## Files to Create/Modify

### New Files

| File | Purpose |
|------|---------|
| `src/models/auth.rs` | Auth-related types |
| `src/api/auth.rs` | Login/refresh server functions |
| `src/state/auth.rs` | Auth context provider |
| `src/pages/login.rs` | Login page component |
| `src/components/login_form.rs` | Login form component |
| `src/components/left_panel.rs` | Extracted shared layout |
| `src/utils/cookies.rs` | Cookie manipulation |
| `src/hooks/use_auto_login.rs` | Auto-login logic |
| `src/hooks/use_proactive_refresh.rs` | Token refresh before expiry |
| `src/components/auth_guard.rs` | Protected route wrapper |
| `src/utils/auth_fetch.rs` | 401 interceptor for API calls |

### Modified Files

| File | Changes |
|------|---------|
| `src/app.rs` | Add `/login` route, wrap with AuthProvider |
| `src/models/mod.rs` | Export auth module |
| `src/api/mod.rs` | Export auth module |
| `src/api/graphql.rs` | Add login/refresh mutations |
| `src/components/mod.rs` | Export new components |
| `src/pages/mod.rs` | Export login page |
| `src/pages/register.rs` | Extract LeftPanel to shared component |

---

## UI/UX Requirements

### Visual Parity

- Same two-column layout as registration
- Animated gradient left panel with phone mockup
- Blue glow background SVG
- Poppins font family
- Same color palette (dark theme)

### Form Behavior

1. **Email Field**
   - Icon: envelope
   - Validation: valid email format
   - Validate on blur
   - Green checkmark on valid

2. **Password Field**
   - Icon: lock
   - Toggle visibility button (eye open/close)
   - Validate on blur (non-empty)

3. **Remember Me**
   - Checkbox, default checked
   - Persists preference to cookie/localStorage

4. **Submit Button**
   - Loading state with spinner
   - Disabled during submission
   - `aria-busy="true"` when loading

### Error States

- Field-level validation messages
- Toast notifications for API errors
- Query param messages displayed in top area

### Accessibility

- Proper form labels and `aria-describedby`
- Focus management on errors
- Live regions for validation messages
- Keyboard navigation support
- Screen reader announcements

---

## Response Code Handling

Use typed enums for response codes:

```rust
/// Typed login response codes for compile-time safety.
#[derive(Debug, Clone, PartialEq)]
pub enum LoginResponseCode {
    Success,            // 10801
    InvalidCredentials, // 30801
    NotVerified,        // 60801
    ServerError,        // 40801
    Unknown(String),
}

impl From<&str> for LoginResponseCode {
    fn from(code: &str) -> Self {
        match code {
            "10801" => Self::Success,
            "30801" => Self::InvalidCredentials,
            "60801" => Self::NotVerified,
            "40801" => Self::ServerError,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl LoginResponseCode {
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Success => "Login successful.",
            Self::InvalidCredentials => "Invalid email or password. Please try again.",
            Self::NotVerified => "Your account is not verified. Please check your email.",
            Self::ServerError => "Something went wrong. Please try again later.",
            Self::Unknown(_) => "Login failed. Please try again.",
        }
    }
}
```

---

## Testing Plan

### Unit Tests

- [ ] Email validation regex
- [ ] Password validation (non-empty)
- [ ] Response code message mapping
- [ ] Cookie utilities

### Integration Tests (E2E)

- [ ] Successful login → redirect to dashboard
- [ ] Invalid credentials → error message
- [ ] Unverified account → appropriate error
- [ ] Remember me checked → tokens persisted
- [ ] Remember me unchecked → session-only tokens
- [ ] Auto-login with valid refresh token
- [ ] Auto-login with expired token → stay on login
- [ ] Query param messages display correctly
- [ ] Link to registration works
- [ ] Link to forgot password works

---

## Security Considerations

### Current Legacy Issues

⚠️ The legacy implementation stores the raw password in cookies for auto-login. This is a security anti-pattern.

### Required Security Measures

1. **Do NOT store raw passwords** — rely solely on refresh token (breaking change from legacy, but necessary)
2. Use `httpOnly` cookies for tokens (prevents XSS access)
3. Use `Secure` flag (HTTPS only)
4. Use `SameSite=Strict` (CSRF protection)
5. Implement refresh token rotation (new token on each refresh)
6. Access token expiry: 15 minutes
7. Refresh token expiry: 7 days (30 days with "remember me")

### SSR Token Flow

Leptos SSR requires careful token handling:

```rust
// Server-side: Set cookies in response headers
pub fn set_auth_cookies_ssr(
    access_token: &str,
    refresh_token: &str,
    remember_me: bool,
) {
    use leptos_axum::ResponseOptions;
    let response = expect_context::<ResponseOptions>();
    
    let max_age = if remember_me { 30 * 24 * 60 * 60 } else { 7 * 24 * 60 * 60 };
    
    response.append_header(
        header::SET_COOKIE,
        format!(
            "access_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
            access_token, 15 * 60
        ),
    );
    response.append_header(
        header::SET_COOKIE,
        format!(
            "refresh_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
            refresh_token, max_age
        ),
    );
}

// Server-side: Read cookies from request
pub fn get_auth_cookies_ssr() -> Option<(String, String)> {
    use leptos_axum::extract;
    let cookies = extract::<TypedHeader<Cookie>>().await.ok()?;
    let access = cookies.get("access_token")?.to_string();
    let refresh = cookies.get("refresh_token")?.to_string();
    Some((access, refresh))
}
```

### Auth State Hydration

On SSR page load, initialize auth state from existing cookies:

```rust
pub fn provide_auth_context() {
    // On server: read from request cookies
    // On client: cookies auto-sent with requests
    let initial_auth = create_blocking_resource(|| (), |_| async {
        // Server function reads cookies and validates
        validate_session().await.ok()
    });
    
    // ... rest of context setup
}
```

---

## Dependencies

### Rust Crates

Already in use:
- `leptos` — framework
- `serde` / `serde_json` — serialization
- `reqwest` — HTTP client (SSR)

May need:
- `gloo-storage` — localStorage access (WASM)
- `wasm-cookies` — cookie manipulation (WASM) or use `web_sys` directly

### Shared Resources

- `css/login-register.css` → convert to SCSS in `style/login.scss`
- `svg/logo_sw.svg`, `svg/logo_farbe.svg`, `svg/PeerLogoWhite.svg`
- `img/register.webp` (or create `img/login.webp`)

---

## Estimated Effort

| Phase | Effort |
|-------|--------|
| Phase 1: Infrastructure | 4-6 hours |
| Phase 2: Login UI | 3-4 hours |
| Phase 3: Token Management | 2-3 hours |
| Phase 4: Protected Routes | 2-3 hours |
| Phase 5: Token Refresh Strategy | 2-3 hours |
| Testing & Polish | 2-3 hours |
| **Total** | **15-22 hours** |

---

## Resolved Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Password storage | **Refresh token only** | Security vulnerability in legacy; refresh token is sufficient |
| Auto-login UX | **Subtle loading indicator** | Show spinner in auth area; avoid full-page blank flash |
| Post-login redirect | **Original URL via `?redirect=`** | `/login?redirect=%2Fwallet` → redirect to `/wallet` after success; default to `/dashboard` |
| Token refresh strategy | **Proactive + 401 fallback** | Refresh 1 min before expiry; 401 interceptor as backup |

---

## Changelog

### 2026-04-10 (Review Update)
- Added logout mutation to API reference
- Added Phase 5: Token Refresh Strategy
- Added SSR token handling and auth state hydration
- Added original URL redirect pattern to AuthGuard
- Changed password storage decision: refresh token only (security fix)
- Added typed ResponseCode enum
- Resolved all open questions
- Updated effort estimate (+2-3 hours for Phase 5)

### 2026-04-10
- Initial planning document created
- Analyzed legacy implementation
- Documented backend API
- Created implementation plan
