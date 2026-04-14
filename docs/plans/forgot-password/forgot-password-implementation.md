# Forgot Password Implementation Plan

**Feature:** Forgot Password  
**Priority:** #3b (auth flow completion, parallel to Dashboard)  
**Status:** 📋 Planning  
**Created:** 2026-04-14

---

## Overview

Implement the password reset flow for the Leptos frontend. This completes the auth trilogy (Login ✅, Register ✅, Forgot Password) and is essential for any production-ready authentication system. The flow is a 4-step multi-step form that reuses the Login/Register layout and several existing components.

### Goals

1. Full parity with legacy `forgotpassword.php` user experience
2. 4-step flow: Enter Email → Verify Code → New Password → Success
3. Rate-limited resend with escalating cooldowns (60s → 10min → locked)
4. Reuse existing components: `LeftPanel`, `BackButton`, `PasswordStrengthMeter`, `StepAnnouncer`, `Toast`
5. Client-side + server-side validation
6. Accessible: ARIA live regions, step announcements, keyboard navigation
7. Auto-redirect if already authenticated
8. Email masking for privacy in the verification step

---

## Scope

### In Scope

- [ ] Forgot password page (`/forgotpassword` route)
- [ ] Auto-redirect to `/dashboard` if already authenticated
- [ ] **Step 1 — Enter Email:**
  - [ ] Email input with real-time validation
  - [ ] Submit triggers `requestPasswordReset` mutation
  - [ ] Rate-limit error handling (31901, 31903)
  - [ ] On success, advance to Step 2
- [ ] **Step 2 — Verify Code:**
  - [ ] Code input field
  - [ ] Masked email display (`ca****@domain.com`)
  - [ ] Submit triggers `resetPasswordTokenVerify` mutation
  - [ ] Resend code button with countdown timer
  - [ ] Escalating cooldowns: 1st → 60s, 2nd → 10min, 3rd+ → locked with support message
  - [ ] Counter stored in cookie (2-hour expiry)
  - [ ] Invalid/expired token error handling (31904)
- [ ] **Step 3 — New Password:**
  - [ ] Password input with visibility toggle
  - [ ] `PasswordStrengthMeter` component (reuse existing)
  - [ ] Confirm password input with visibility toggle
  - [ ] Match validation
  - [ ] Submit triggers `resetPassword` mutation
  - [ ] Error display for failed update
- [ ] **Step 4 — Success:**
  - [ ] Success icon and message
  - [ ] "Continue to Login" link
- [ ] Back button navigation between steps
- [ ] `StepAnnouncer` for screen reader accessibility
- [ ] `Toast` notifications for API responses
- [ ] Loading states on submit buttons
- [ ] Add response codes to `response_codes.rs`
- [ ] 3 new server functions in API layer
- [ ] Route registration in `app.rs`

### Out of Scope (Future Work)

- Two-factor authentication
- Email change with verification code (handled in Settings)
- Account lockout after too many failed password attempts

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `forgotpassword.php` | Page template — 4-step multi-step form |
| `js/login/forgotpassword.js` | `AccessibleResetpasswordForm` class — validation, API calls, countdown timer, step navigation |
| `css/login-register.css` | Shared styles with login/register pages |
| `css/password.css` | Password strength meter styles |

### Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│ Same layout as Login / Register pages                      │
├──────────────────────────┬─────────────────────────────────┤
│                          │                                 │
│  LEFT PANEL              │  RIGHT PANEL                    │
│  (Phone mockup)          │                                 │
│                          │  ┌─────────────────────────┐    │
│  ┌──────────────────┐    │  │ [← Back]                │    │
│  │ Phone Screen     │    │  ├─────────────────────────┤    │
│  │ (register.webp)  │    │  │                         │    │
│  └──────────────────┘    │  │  ACTIVE STEP            │    │
│  ┌──────────────────┐    │  │  (1 of 4 visible)       │    │
│  │ Peer Logo        │    │  │                         │    │
│  └──────────────────┘    │  │  - Form fields          │    │
│                          │  │  - Submit button         │    │
│  ┌──────────────────┐    │  │  - Response messages     │    │
│  │ Logo (color)     │    │  │                         │    │
│  └──────────────────┘    │  └─────────────────────────┘    │
│                          │                                 │
│                          │  ┌─────────────────────────┐    │
│                          │  │ Version footer           │    │
│                          │  └─────────────────────────┘    │
├──────────────────────────┴─────────────────────────────────┤
```

### Step Flow

```
Step 1: Enter Email
  ├── Valid email → requestPasswordReset API call
  │   ├── Success (11901) → Go to Step 2
  │   ├── Rate limited (31901) → Show "try again later"
  │   └── Locked (31903) → Show "contact support"
  │
Step 2: Verify Code
  ├── Enter code → resetPasswordTokenVerify API call
  │   ├── Valid (11902) → Go to Step 3
  │   └── Invalid/expired (31904) → Show error
  ├── Resend Code → Back to requestPasswordReset
  │   ├── 1st resend: 60s cooldown
  │   ├── 2nd resend: 10min cooldown
  │   └── 3rd+: Locked → "Contact support"
  │
Step 3: New Password
  ├── Password + confirm → resetPassword API call
  │   ├── Success (11005) → Go to Step 4
  │   └── Token expired (31904) → Show error
  │
Step 4: Success
  └── "Continue to Login" → /login
```

### Key Features

1. **Rate-Limited Resend**
   - Cookie `reset_code_sent_counter` tracks attempt count (2-hour expiry)
   - Cookie `timerstart` tracks active countdown
   - 1st request: immediate send, then 60s cooldown
   - 2nd request: after cooldown, then 10min cooldown
   - 3rd+ request: locked — "Contact support at peernetworkpse@gmail.com"

2. **Email Masking**
   - `cameron@example.com` → `ca****@example.com`
   - Preserves domain, masks user portion after first 2 chars

3. **Password Validation**
   - Min 8 chars, 1 lowercase, 1 uppercase, 1 digit (required)
   - Special character or 12+ chars (bonus for "excellent")
   - Strength meter: Very Weak → Weak → Improvement → Good → Excellent
   - Confirm password must match

4. **Auto-Login Check**
   - If `authToken` cookie exists, redirect to `/dashboard`
   - In peer-web: check `AuthContext.is_authenticated`

5. **Countdown Timer**
   - Visual `MM:SS` display next to "Resend Code"
   - Button disabled during countdown
   - Timer resets on page reload via cookie persistence

---

## Backend API Reference

### `requestPasswordReset` Mutation (Guest Schema)

```graphql
mutation RequestPasswordReset($email: String!) {
  requestPasswordReset(email: $email) {
    status
    ResponseCode
    nextAttemptAt
  }
}
```

#### Response: `ResetPasswordRequestResponse`

```graphql
type ResetPasswordRequestResponse {
  meta: DefaultResponse!
  status: String!
  ResponseCode: String
  nextAttemptAt: String    # Timestamp when next attempt is allowed
}
```

> **Security Note:** Always returns success (11901) regardless of whether email exists — prevents user enumeration.

#### Response Codes

| Code | Description | User Message |
|------|-------------|--------------|
| `11901` | Email sent (if account exists) | "If an account exists, you'll receive an email with instructions." |
| `30104` | Invalid email format | "Please enter a valid email address." |
| `31901` | Rate limited — try again later | "Too many requests. Please try again later." |
| `31903` | Locked — contact support | "Too many requests. Please contact support at peernetworkpse@gmail.com." |

---

### `resetPasswordTokenVerify` Mutation (Guest Schema)

```graphql
mutation ResetPasswordTokenVerify($token: String!) {
  resetPasswordTokenVerify(token: $token) {
    status
    ResponseCode
  }
}
```

#### Response Codes

| Code | Description | User Message |
|------|-------------|--------------|
| `11902` | Token is valid | "Code verified." |
| `31904` | Invalid or expired token | "This password reset link isn't valid anymore. Please request a new one." |
| `41004` | Internal server error | "Something went wrong. Please try again." |

---

### `resetPassword` Mutation (Guest Schema)

```graphql
mutation ResetPassword($token: String!, $password: String!) {
  resetPassword(token: $token, password: $password) {
    status
    ResponseCode
  }
}
```

#### Password Constraints

| Rule | Constraint |
|------|-----------|
| Length | 8–128 characters |
| Pattern | Must contain uppercase + lowercase + digit (`^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).+$`) |

#### Side Effects

- Updates password hash in database
- Deletes stored access and refresh tokens (forces re-login)

#### Response Codes

| Code | Description | User Message |
|------|-------------|--------------|
| `11005` | Password updated | "Password changed successfully." |
| `31904` | Invalid or expired token | "This password reset link isn't valid anymore. Please request a new one." |
| `21001` | No user found for token | (silent success — no disclosure) |
| `41004` | Internal server error | "Something went wrong. Please try again." |

---

## Reusable Components

| Component | Source | Usage |
|-----------|--------|-------|
| `LeftPanel` | `src/components/left_panel.rs` | Phone mockup layout (identical to login/register) |
| `BackButton` | `src/components/back_button.rs` | Navigate between steps / back to login |
| `PasswordStrengthMeter` | `src/components/password_strength.rs` | Step 3 — password strength indicator |
| `StepAnnouncer` | `src/components/step_announcer.rs` | Screen reader step transition announcements |
| `Toast` / `use_toast` | `src/components/toast.rs` | API success/error notification toasts |
| `validation` helpers | `src/components/validation.rs` | `is_valid_email`, `validate_password`, `passwords_match` |

---

## New Files

### `src/pages/forgot_password.rs` — Page Component

```rust
//! Forgot password page — multi-step password reset flow.
//!
//! ## Step flow
//!
//! 1. Enter email → requestPasswordReset
//! 2. Verify code → resetPasswordTokenVerify
//! 3. New password → resetPassword
//! 4. Success → link to login

use leptos::prelude::*;
use leptos_meta::*;

use crate::components::back_button::BackButton;
use crate::components::left_panel::LeftPanel;
use crate::components::password_strength::PasswordStrengthMeter;
use crate::components::step_announcer::StepAnnouncer;
use crate::components::toast::{use_toast, ToastType};
use crate::components::validation::{
    is_valid_email, passwords_match, validate_password,
};
use crate::utils::response_codes::user_friendly_msg;

/// Step identifier for the forgot password flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgotStep {
    Email,
    VerifyCode,
    NewPassword,
    Success,
}

impl ForgotStep {
    pub fn data_step(&self) -> &'static str {
        match self {
            Self::Email => "1",
            Self::VerifyCode => "2",
            Self::NewPassword => "3",
            Self::Success => "4",
        }
    }
}

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    // Step state
    let current_step = RwSignal::new(ForgotStep::Email);

    // Form data signals
    let email = RwSignal::new(String::new());
    let verify_code = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // Token saved from Step 2 for use in Step 3
    let reset_token = RwSignal::new(String::new());

    // Loading state
    let pending = RwSignal::new(false);

    // Announcement for screen readers
    let announcement = RwSignal::new(String::new());

    // Toast
    let toast = use_toast();

    // Auto-redirect if already authenticated
    // (check auth context on mount)

    view! {
        <Title text="Peer Network - Forgot Password"/>
        <Meta name="description" content="Reset your Peer Network account password."/>

        <a href="#forgot-form" class="skip-link sr-only">
            "Skip to reset form"
        </a>

        <div class="container large_font">
            <LeftPanel
                image_src="/img/register.webp"
                image_alt="Forgot password"
            />

            <div class="container_right">
                <div class="container_inner">
                    <div class="top_head_area">
                        <BackButton
                            on_click=move |_| {
                                match current_step.get() {
                                    ForgotStep::Email => {
                                        // Navigate back to login
                                        // window.location.href = "/login"
                                    }
                                    ForgotStep::VerifyCode => current_step.set(ForgotStep::Email),
                                    ForgotStep::NewPassword => current_step.set(ForgotStep::VerifyCode),
                                    ForgotStep::Success => {} // No back from success
                                }
                            }
                            visible=Signal::derive(move || current_step.get() != ForgotStep::Success)
                        />
                    </div>

                    <div class="center_area" id="forgot-form" tabindex="-1">
                        <StepAnnouncer announcement=announcement.into()/>

                        // Step 1: Email
                        <EmailStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::Email)
                            email=email
                            pending=pending.into()
                            on_success=move || {
                                current_step.set(ForgotStep::VerifyCode);
                                announcement.set("Code sent to your email".into());
                            }
                        />

                        // Step 2: Verify Code
                        <VerifyCodeStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::VerifyCode)
                            email=email.into()
                            verify_code=verify_code
                            pending=pending.into()
                            on_success=move |token: String| {
                                reset_token.set(token);
                                current_step.set(ForgotStep::NewPassword);
                                announcement.set("Code verified. Enter new password.".into());
                            }
                        />

                        // Step 3: New Password
                        <NewPasswordStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::NewPassword)
                            password=password
                            confirm_password=confirm_password
                            reset_token=reset_token.into()
                            pending=pending.into()
                            on_success=move || {
                                current_step.set(ForgotStep::Success);
                                announcement.set("Password updated successfully!".into());
                            }
                        />

                        // Step 4: Success
                        <SuccessStep
                            active=Signal::derive(move || current_step.get() == ForgotStep::Success)
                        />
                    </div>

                    <div class="footer_area medium_font">
                        <p class="version version-number"></p>
                    </div>
                </div>
            </div>
        </div>
    }
}
```

### `src/api/forgot_password.rs` — Server Functions

```rust
//! Server functions for the password reset flow.
//!
//! All three mutations use the guest schema (no auth required).

use leptos::prelude::*;

use crate::models::auth::DefaultPayload;

// ── Step 1: Request password reset ──────────────────────────────────

#[server(RequestPasswordReset, "/api")]
pub async fn request_password_reset(email: String) -> Result<DefaultPayload, ServerFnError> {
    use crate::api::graphql::mutate;

    if email.is_empty() {
        return Err(ServerFnError::new("Email is required."));
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(ServerFnError::new("Invalid email format."));
    }

    const QUERY: &str = r#"
        mutation RequestPasswordReset($email: String!) {
            requestPasswordReset(email: $email) {
                status
                ResponseCode
            }
        }
    "#;

    #[derive(serde::Serialize)]
    struct Vars { email: String }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data { request_password_reset: DefaultPayload }

    let data: Data = mutate(QUERY, Vars { email }, None).await?;
    Ok(data.request_password_reset)
}

// ── Step 2: Verify reset token ──────────────────────────────────────

#[server(VerifyResetToken, "/api")]
pub async fn verify_reset_token(token: String) -> Result<DefaultPayload, ServerFnError> {
    use crate::api::graphql::mutate;

    if token.trim().is_empty() {
        return Err(ServerFnError::new("Code is required."));
    }

    const QUERY: &str = r#"
        mutation ResetPasswordTokenVerify($token: String!) {
            resetPasswordTokenVerify(token: $token) {
                status
                ResponseCode
            }
        }
    "#;

    #[derive(serde::Serialize)]
    struct Vars { token: String }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data { reset_password_token_verify: DefaultPayload }

    let data: Data = mutate(QUERY, Vars { token }, None).await?;
    Ok(data.reset_password_token_verify)
}

// ── Step 3: Reset password ──────────────────────────────────────────

#[server(ResetPassword, "/api")]
pub async fn reset_password(token: String, password: String) -> Result<DefaultPayload, ServerFnError> {
    use crate::api::graphql::mutate;
    use crate::api::validation::validate_password_server;

    if token.trim().is_empty() {
        return Err(ServerFnError::new("Reset token is missing."));
    }
    validate_password_server(&password)?;

    const QUERY: &str = r#"
        mutation ResetPassword($token: String!, $password: String!) {
            resetPassword(token: $token, password: $password) {
                status
                ResponseCode
            }
        }
    "#;

    #[derive(serde::Serialize)]
    struct Vars { token: String, password: String }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data { reset_password: DefaultPayload }

    let data: Data = mutate(QUERY, Vars { token, password }, None).await?;
    Ok(data.reset_password)
}
```

### Model Addition

The `DefaultPayload` struct (if not already available) should mirror:

```rust
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DefaultPayload {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: Option<String>,
}

impl DefaultPayload {
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}
```

---

## Step Components (Detail)

### Step 1 — Email Input

```rust
#[component]
fn EmailStep(
    active: Signal<bool>,
    email: RwSignal<String>,
    pending: Signal<bool>,
    on_success: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let is_email_valid = Memo::new(move |_| is_valid_email(&email.get()));
    let email_touched = RwSignal::new(false);
    let backend_error = RwSignal::new(Option::<String>::None);

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !is_email_valid.get() { return; }

        let on_success = on_success.clone();
        spawn_local(async move {
            pending.set(true);
            match request_password_reset(email.get()).await {
                Ok(payload) if payload.is_success() => {
                    toast.show("Check your email for the reset code.", ToastType::Success);
                    on_success();
                }
                Ok(payload) => {
                    let code = payload.response_code.unwrap_or_default();
                    match code.as_str() {
                        "31901" | "31903" => backend_error.set(Some(user_friendly_msg(&code).to_string())),
                        _ => toast.show(user_friendly_msg(&code), ToastType::Error),
                    }
                }
                Err(e) => toast.show(&e.to_string(), ToastType::Error),
            }
            pending.set(false);
        });
    };

    view! {
        <div
            class=move || if active.get() { "form-step active" } else { "form-step" }
            data-step="1"
        >
            <div class="step-header">
                <h2 class="x_large_font">"Forgot password"</h2>
                <p class="large_font">"Please enter your email and we'll send you a reset link."</p>
            </div>
            <form on:submit=on_submit novalidate>
                // Email input with validation icon
                // ... (standard input-group pattern)
                <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                    "Reset password"
                </button>
            </form>
        </div>
    }
}
```

### Step 2 — Verify Code with Countdown

```rust
#[component]
fn VerifyCodeStep(
    active: Signal<bool>,
    email: Signal<String>,
    verify_code: RwSignal<String>,
    pending: Signal<bool>,
    on_success: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    // Countdown timer state
    let countdown_seconds = RwSignal::new(0i32);
    let resend_count = RwSignal::new(0u32);
    let is_locked = Memo::new(move |_| resend_count.get() >= 3);

    // Masked email display
    let masked_email = Memo::new(move |_| mask_email(&email.get()));

    // Countdown display (MM:SS)
    let countdown_display = Memo::new(move |_| {
        let secs = countdown_seconds.get();
        if secs <= 0 { return String::new(); }
        format!("{:02}:{:02}", secs / 60, secs % 60)
    });

    // Can resend: countdown finished AND not locked
    let can_resend = Memo::new(move |_| countdown_seconds.get() <= 0 && !is_locked.get());

    // ... timer interval logic using set_interval ...

    view! {
        <div class=move || if active.get() { "form-step active" } else { "form-step" }>
            <div class="step-header">
                <h2 class="x_large_font">"Verify code"</h2>
                <p class="large_font">
                    "We sent a code to " {move || masked_email.get()} ". Enter it below."
                </p>
            </div>
            <form on:submit=handle_verify novalidate>
                // Code input ...
                <button type="submit" class="btn btn-primary">"Verify code"</button>
            </form>

            <div class="dont_get_code medium_font">
                <p>
                    "Didn't get the code? "
                    <Show when=move || countdown_seconds.get() > 0>
                        <span class="resend-txt">
                            "You can resend in " <span>{move || countdown_display.get()}</span>
                        </span>
                    </Show>
                    <Show when=move || can_resend.get()>
                        <a href="#" on:click=handle_resend>"Resend Code"</a>
                    </Show>
                    <Show when=move || is_locked.get()>
                        <span class="error-text">
                            "Email delivery failed. Please contact support at peernetworkpse@gmail.com"
                        </span>
                    </Show>
                </p>
            </div>
        </div>
    }
}
```

### Step 3 — New Password

```rust
#[component]
fn NewPasswordStep(
    active: Signal<bool>,
    password: RwSignal<String>,
    confirm_password: RwSignal<String>,
    reset_token: Signal<String>,
    pending: Signal<bool>,
    on_success: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let show_password = RwSignal::new(false);
    let show_confirm = RwSignal::new(false);

    // Password validation (reuse from registration)
    let password_validation = Memo::new(move |_| validate_password(&password.get()));
    let password_visible = Memo::new(move |_| !password.get().is_empty());
    let passwords_matching = Memo::new(move |_| {
        passwords_match(&password.get(), &confirm_password.get())
    });

    let can_submit = Memo::new(move |_| {
        password_validation.get().requirements.is_sufficient()
            && passwords_matching.get()
            && !pending.get()
    });

    // Submit: call resetPassword server function
    // ...

    view! {
        <div class=move || if active.get() { "form-step active" } else { "form-step" }>
            <div class="step-header">
                <h2 class="x_large_font">"Enter new password"</h2>
            </div>
            <form on:submit=handle_submit novalidate>
                // Password field + toggle
                // PasswordStrengthMeter (reuse)
                // Confirm password field + toggle
                // Error message area
                <button type="submit" class="btn btn-primary" disabled=move || !can_submit.get()>
                    "Update password"
                </button>
            </form>
        </div>
    }
}
```

### Step 4 — Success

```rust
#[component]
fn SuccessStep(active: Signal<bool>) -> impl IntoView {
    view! {
        <div class=move || if active.get() { "form-step active" } else { "form-step" }>
            <div class="success-message">
                <div class="step-header">
                    <span class="icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-good-tick-circle"></i>
                    </span>
                    <h2 class="x_large_font">"Password updated!"</h2>
                    <p class="large_font">
                        "Your password has been updated. Please use your new password to log in."
                    </p>
                </div>
                <a class="btn btn-primary" href="/login">"Continue to Login"</a>
            </div>
        </div>
    }
}
```

---

## Utility: Email Masking

```rust
/// Mask an email address for display privacy.
///
/// `cameron@example.com` → `ca****@example.com`
fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((user, domain)) => {
            if user.len() <= 2 {
                format!("{}@{}", "*".repeat(user.len()), domain)
            } else {
                format!("{}{}@{}", &user[..2], "*".repeat(user.len() - 2), domain)
            }
        }
        None => email.to_string(),
    }
}
```

---

## Countdown Timer (Client-Side)

The resend cooldown should use `gloo_timers::callback::Interval` (or `set_interval` via `web_sys`):

```rust
// Start countdown with the given duration in seconds
fn start_countdown(countdown_seconds: RwSignal<i32>, duration: i32) {
    countdown_seconds.set(duration);

    #[cfg(feature = "hydrate")]
    {
        use gloo_timers::callback::Interval;
        let interval = Interval::new(1_000, move || {
            countdown_seconds.update(|s| *s -= 1);
            if countdown_seconds.get_untracked() <= 0 {
                // Interval will be dropped when component unmounts
            }
        });
        // Store interval handle to prevent drop
        on_cleanup(move || drop(interval));
    }
}
```

Cooldown durations per attempt:
- **1st send:** 60 seconds
- **2nd send:** 600 seconds (10 minutes)
- **3rd+ send:** Locked (show support contact message)

The resend counter should be persisted in a cookie (`reset_code_sent_counter`, 2-hour expiry) to survive page refreshes.

---

## Response Codes to Add

Add to `src/utils/response_codes.rs`:

```rust
// ── Password Reset ───────────────────────────────────────────────
m.insert("11005", "Password changed successfully.");
m.insert("11901", "If an account exists, you'll receive an email with instructions.");
m.insert("11902", "Code verified.");
m.insert("30104", "Please enter a valid email address.");
m.insert("31901", "Too many requests. Please try again later.");
m.insert("31903", "Too many requests. Please contact support at peernetworkpse@gmail.com.");
m.insert("31904", "This password reset link isn't valid anymore. Please request a new one.");
m.insert("41004", "Something went wrong. Please try again.");
```

---

## Styles

Reuse `css/login-register.css` styles — the forgot password page uses the same `.container`, `.container_left`, `.container_right`, `.form-step`, `.input-group`, `.btn-primary` classes as login and register.

Additional styles needed (may already exist in `style/`):

- `.resend-txt` — grayed-out text for countdown display
- `.dont_get_code` — container for resend section
- `.disable` — utility class for disabled state on resend link
- `.response-message` — inline error messages within forms

---

## Integration Checklist

### Files to Create

| File | Purpose |
|------|---------|
| `src/pages/forgot_password.rs` | Page component with 4-step form |
| `src/api/forgot_password.rs` | 3 server functions |

### Files to Modify

| File | Change |
|------|--------|
| `src/pages/mod.rs` | Add `pub mod forgot_password;` + re-export `ForgotPasswordPage` |
| `src/api/mod.rs` | Add `pub mod forgot_password;` |
| `src/app.rs` | Add route: `<Route path=StaticSegment("forgotpassword") view=ForgotPasswordPage/>` |
| `src/utils/response_codes.rs` | Add password-reset response code mappings |
| `src/components/login_form.rs` | Verify "Forgot password?" link points to `/forgotpassword` |

### SCSS

| File | Change |
|------|--------|
| `style/` | Verify login-register styles cover all form step patterns; add countdown-specific styles if needed |

---

## Implementation Phases

### Phase 1: API Layer + Route Shell
- Create `src/api/forgot_password.rs` with 3 server functions
- Register module in `src/api/mod.rs`
- Add response codes to `src/utils/response_codes.rs`
- Create `src/pages/forgot_password.rs` with page shell + step enum
- Register in `src/pages/mod.rs` and `src/app.rs`

### Phase 2: Step 1 — Email Submission
- Email input with real-time validation (reuse `is_valid_email`)
- Form submit → `request_password_reset` server function
- Success/error handling with toast
- Rate-limit error display (31901, 31903)

### Phase 3: Step 2 — Code Verification + Resend
- Code input field
- Masked email display
- Submit → `verify_reset_token` server function
- Resend button with countdown timer
- Escalating cooldown logic
- Counter persistence (cookie or localStorage)

### Phase 4: Step 3 — New Password
- Password + confirm password inputs with toggles
- `PasswordStrengthMeter` integration (reuse component)
- Submit → `reset_password` server function
- Error display for expired tokens

### Phase 5: Step 4 + Polish
- Success screen with "Continue to Login" link
- Auto-redirect check (already authenticated → dashboard)
- Back button navigation
- StepAnnouncer integration
- Loading states on all buttons
- E2E test coverage

---

## Testing

### Unit Tests

- `mask_email()` — edge cases (short user, no @, empty)
- Response code mapping — all new codes resolve correctly

### E2E Tests (Playwright)

| Test | Description |
|------|-------------|
| Happy path | Email → code → password → success → login link works |
| Invalid email | Shows validation error, no API call |
| Rate limit | 3rd attempt shows "contact support" message |
| Invalid code | Shows "invalid or expired" error |
| Weak password | Submit disabled until strength requirements met |
| Password mismatch | Shows "passwords do not match" error |
| Back navigation | Back button returns to previous step |
| Auto-redirect | Authenticated user redirected to dashboard |
| Keyboard nav | Tab order correct, Enter submits forms |
| Screen reader | Step transitions announced via aria-live |

---

## Accessibility

| Feature | Implementation |
|---------|---------------|
| Skip link | `<a href="#forgot-form" class="skip-link sr-only">` |
| Step announcer | `StepAnnouncer` with `aria-live="polite"` |
| Form errors | `aria-describedby` linking inputs to validation messages |
| Validation messages | `role="alert"` with `aria-live="polite"` |
| Button states | `aria-busy="true"` during loading |
| Password toggle | `aria-label="Show/hide password"` |
| Strength meter | `role="meter"` with `aria-valuenow`, `aria-valuetext` |
| Focus management | Auto-focus first input when step changes |
