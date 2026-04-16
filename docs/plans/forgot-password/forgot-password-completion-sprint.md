# Forgot Password — Completion Sprint Plan

**Feature:** Forgot Password (#3b)  
**Priority:** Highest-priority in-progress feature after Dashboard (#3)  
**Status:** 🚧 In Progress → target ✅ Implemented  
**Created:** 2026-04-14  
**Updated:** 2026-04-14

---

## Summary

Forgot Password is a 990-line multi-step password reset page that is **structurally complete** — all four steps work end-to-end (email → verify code → new password → success), the API layer is wired, and accessibility features (ARIA, step announcements, keyboard navigation) are in place.

This sprint addresses **4 known gaps** identified in the convergence tracker. All are small-to-medium effort and require no new API endpoints or major architectural changes.

**Verdict:** ~90% complete. The remaining work is hardening: auto-redirect guard, cookie-persisted resend counter, interval cleanup, and component reuse.

---

## Code Audit

### What Exists ✅

| Layer | File | Lines | Status |
|-------|------|-------|--------|
| **Page** | `src/pages/forgot_password.rs` | 990 | ✅ All 4 steps, step navigation, back button, masked email |
| **Route** | `src/app.rs` | — | ✅ `/forgotpassword` registered |
| **API — Request Reset** | `src/api/forgot_password.rs` → `request_password_reset` | — | ✅ Server fn, guest schema, server-side email validation |
| **API — Verify Token** | `src/api/forgot_password.rs` → `verify_reset_token` | — | ✅ Server fn, token validation |
| **API — Reset Password** | `src/api/forgot_password.rs` → `reset_password` | — | ✅ Server fn, token + new password |
| **Component — LeftPanel** | `src/components/left_panel.rs` | — | ✅ Reused (phone mockup) |
| **Component — PasswordStrengthMeter** | `src/components/password_strength.rs` | — | ✅ Reused in Step 3 |
| **Component — StepAnnouncer** | `src/components/step_announcer.rs` | — | ✅ Reused for ARIA live announcements |
| **Component — Toast** | `src/components/toast.rs` | — | ✅ Reused for success/error notifications |
| **Validation** | `src/components/validation.rs` | — | ✅ `is_valid_email`, `validate_password`, `passwords_match` |
| **Response Codes** | `src/utils/response_codes.rs` | — | ✅ 31901, 31903, 31904, 11901, 11902, 11005 mapped |
| **Unit Tests** | `forgot_password.rs` (bottom) | — | ✅ 5 `mask_email` tests |
| **SCSS** | `style/login-register.scss` | — | ✅ Shared with Login + Register pages |

### What Remains 🔲

| # | Task | Effort | Gap Source |
|---|------|--------|------------|
| 1 | **Auto-redirect for authenticated users** | S | Convergence tracker |
| 2 | **Persist resend counter in cookie** | M | Convergence tracker |
| 3 | **Fix countdown interval stacking** | S | Convergence tracker |
| 4 | **Reuse `BackButton` component** | S | Convergence tracker |
| 5 | **E2E test coverage** | L | Testing (new) |
| 6 | **Mock backend: password reset endpoints** | M | Testing prerequisite |

**Legend:** S = Small (< 1 hour), M = Medium (1–3 hours), L = Large (3+ hours)

---

## Task Details

### Task 1: Auto-Redirect for Authenticated Users

**Problem:** If an already logged-in user navigates to `/forgotpassword`, they see the form instead of being redirected to `/dashboard`. The legacy JS calls `autoLogin()` on load: if `authToken` cookie exists → `window.location.href = "dashboard.php"`.

**Fix:** Check `AuthContext.is_authenticated` on mount and redirect.

**File:** `src/pages/forgot_password.rs` — `ForgotPasswordPage` component

```rust
// Add after signal declarations, inside ForgotPasswordPage:
use crate::state::auth::use_auth;

let auth = use_auth();

// Auto-redirect if already authenticated
Effect::new(move |_| {
    if auth.is_authenticated.get() {
        navigate_to("/dashboard");
    }
});
```

**Placement:** After the `let toast = use_toast();` line, before the `handle_back` closure.

**Acceptance criteria:**
- [ ] Authenticated user visiting `/forgotpassword` is redirected to `/dashboard`
- [ ] Unauthenticated user sees the form normally
- [ ] SSR does not panic (the `navigate_to` already gates on `#[cfg(feature = "hydrate")]`)

---

### Task 2: Persist Resend Counter in Cookie

**Problem:** The `resend_count` signal resets to 0 on page reload, allowing users to bypass the escalating cooldown (60s → 10min → locked). The legacy implementation stores the counter in a cookie with a 2-hour expiry (`reset_code_sent_counter`).

**Fix:** Read `resend_count` from a cookie on mount, write to cookie on each resend.

**File:** `src/pages/forgot_password.rs` — `VerifyCodeStep` component signals + resend handler

**Implementation:**

1. Add cookie helper functions (or reuse if they exist):

```rust
/// Read a cookie value by name (client-side only).
#[cfg(feature = "hydrate")]
fn get_cookie(name: &str) -> Option<String> {
    let document = web_sys::window()?.document()?;
    let cookies = document.cookie().ok()?;
    for pair in cookies.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix(&format!("{}=", name)) {
            return Some(value.to_string());
        }
    }
    None
}

/// Set a cookie with max-age in seconds (client-side only).
#[cfg(feature = "hydrate")]
fn set_cookie(name: &str, value: &str, max_age_secs: u32) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        let cookie = format!("{}={}; max-age={}; path=/; SameSite=Lax", name, value, max_age_secs);
        let _ = document.set_cookie(&cookie);
    }
}
```

2. Initialize `resend_count` from cookie in `ForgotPasswordPage`:

```rust
// Replace: let resend_count = RwSignal::new(0u32);
// With:
let initial_resend_count: u32 = {
    #[cfg(feature = "hydrate")]
    {
        get_cookie("reset_code_sent_counter")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }
    #[cfg(not(feature = "hydrate"))]
    { 0 }
};
let resend_count = RwSignal::new(initial_resend_count);
```

3. In the resend handler (`handle_resend`), after incrementing `resend_count`, persist to cookie:

```rust
resend_count.set(count + 1);
#[cfg(feature = "hydrate")]
set_cookie("reset_code_sent_counter", &(count + 1).to_string(), 7200); // 2 hours
```

4. If locked (3+ attempts), also clear the timer cookie:

```rust
// Already handled by is_locked memo; no UI change needed.
// The cookie naturally expires after 2 hours, matching legacy behavior.
```

**Acceptance criteria:**
- [ ] Resend counter survives page reload (persisted in cookie)
- [ ] Cookie expires after 2 hours (fresh start)
- [ ] After 3 resends, user sees "contact support" message even after reload
- [ ] SSR build compiles (cookie access gated behind `#[cfg(feature = "hydrate")]`)

---

### Task 3: Fix Countdown Interval Stacking

**Problem:** The `start_countdown` closure captures an `Interval` whose cleanup is registered via `leptos::on_cleanup`. However, if the user triggers resend multiple times before the page unmounts, each call creates a new `Interval` without stopping the previous one. This causes multiple intervals to decrement `countdown_seconds` simultaneously, making the timer count down faster than real time.

**Current code (in `VerifyCodeStep`):**

```rust
let start_countdown = move |duration: i32| {
    countdown_seconds.set(duration);
    #[cfg(feature = "hydrate")]
    {
        use gloo_timers::callback::Interval;
        let interval = Interval::new(1_000, move || {
            countdown_seconds.update(|s| { if *s > 0 { *s -= 1; } });
        });
        leptos::on_cleanup(move || drop(interval));
    }
};
```

**Fix:** Store the interval handle in a `StoredValue` and explicitly drop the previous interval before creating a new one.

```rust
#[cfg(feature = "hydrate")]
let interval_handle: StoredValue<Option<gloo_timers::callback::Interval>> = StoredValue::new(None);

let start_countdown = move |duration: i32| {
    countdown_seconds.set(duration);
    #[cfg(feature = "hydrate")]
    {
        use gloo_timers::callback::Interval;

        // Drop previous interval to prevent stacking
        interval_handle.update_value(|h| { h.take(); });

        let interval = Interval::new(1_000, move || {
            countdown_seconds.update(|s| {
                if *s > 0 {
                    *s -= 1;
                }
            });
        });

        interval_handle.set_value(Some(interval));
        leptos::on_cleanup(move || {
            interval_handle.update_value(|h| { h.take(); });
        });
    }
};
```

**Acceptance criteria:**
- [ ] Pressing resend twice rapidly does not cause double-speed countdown
- [ ] Timer always counts at 1 second per tick
- [ ] Previous interval is cleaned up when a new one starts
- [ ] Interval is cleaned up on component unmount

---

### Task 4: Reuse `BackButton` Component

**Problem:** The page inlines a custom `<a>` back-button instead of using the existing `BackButton` component from `src/components/back_button.rs`. The component has the same API surface needed: `visible`, `href`, and `on_back` callback.

**Current code (in `ForgotPasswordPage` view):**

```rust
<a
    href=move || back_href.get().unwrap_or_default()
    class="btn btn-secondary back-btn"
    id="backBtn"
    style:display=move || if show_back.get() { "flex" } else { "none" }
    on:click=handle_back
>
    <span aria-hidden="true">
        <i class="peer-icon medium_font peer-icon-arrow-left"></i>
    </span>
    "Back"
</a>
```

**Fix:** Replace with the `BackButton` component.

```rust
use crate::components::back_button::BackButton;

// Replace the inline <a> back button with:
<BackButton
    visible=show_back.into()
    href=Signal::derive(move || back_href.get())
    on_back=Callback::new(move |_| {
        match current_step.get() {
            ForgotStep::VerifyCode => {
                current_step.set(ForgotStep::Email);
                announcement.set(ForgotStep::Email.announcement().into());
            }
            ForgotStep::NewPassword => {
                current_step.set(ForgotStep::VerifyCode);
                announcement.set(ForgotStep::VerifyCode.announcement().into());
            }
            _ => {}
        }
    })
/>
```

The existing `handle_back` closure also navigates to `/login` for Step 1, but the `BackButton` component handles that automatically via the `href` prop — when `href` is `Some("/login")`, it renders as a link; when `None`, it fires `on_back`. The `back_href` memo already provides this logic.

**Acceptance criteria:**
- [ ] `BackButton` component used instead of inline `<a>`
- [ ] Step 1 → clicking back navigates to `/login` (link behavior)
- [ ] Step 2 → clicking back returns to Step 1 (callback behavior)
- [ ] Step 3 → clicking back returns to Step 2 (callback behavior)
- [ ] Step 4 → back button hidden
- [ ] Visual appearance unchanged

---

### Task 5: E2E Tests (Playwright)

**Prerequisite:** Task 6 (mock backend endpoints) must be complete.

**File:** `peer-web/end2end/tests/forgot-password.spec.ts` (new)

| # | Test | Steps |
|---|------|-------|
| T1 | Navigate to forgot password | Visit `/forgotpassword`, verify email form visible |
| T2 | Invalid email validation | Enter "not-an-email", blur, verify error message shown |
| T3 | Valid email → advance to Step 2 | Enter valid email, submit, verify code input visible |
| T4 | Verify code → advance to Step 3 | Enter valid code, submit, verify password form visible |
| T5 | Password mismatch | Enter mismatched passwords, verify error shown |
| T6 | Password too weak | Enter "abc", verify strength meter shows weak |
| T7 | Successful reset → Step 4 | Enter valid password + confirm, submit, verify success message |
| T8 | Back button navigation | On Step 2, click back, verify Step 1 shown |
| T9 | Resend code with cooldown | On Step 2, click resend, verify countdown timer appears |
| T10 | Authenticated redirect | Login first, then visit `/forgotpassword`, verify redirect to `/dashboard` |

**Acceptance criteria:**
- [ ] All 10 E2E tests pass against mock backend
- [ ] Tests cover the 4 fixed gaps (redirect, resend persistence, timer, back button)

---

### Task 6: Mock Backend — Password Reset Endpoints

**Context:** The mock backend (Rust, axum + async-graphql) currently supports Phases 0–3 (registration, login/session, users/profiles, posts/content). Password reset endpoints are not yet implemented. These 3 mutations belong to the guest schema and are needed for E2E testing.

**File:** `tests/mock_backend/src/schema/mutation/auth.rs` (extend existing)

| Mutation | Schema | Behavior |
|----------|--------|----------|
| `requestPasswordReset(email)` | Guest | Always return success (prevent enumeration). Store token `"MOCK_RESET_TOKEN"` in state. Second call within cooldown → return 31901. |
| `resetPasswordTokenVerify(token)` | Guest | Accept `"MOCK_RESET_TOKEN"` → success (11902). Anything else → 31904. |
| `resetPassword(token, password)` | Guest | Accept `"MOCK_RESET_TOKEN"` + valid password → success (11005), update user password in state. Invalid token → 31904. |

**State additions:**

```rust
// In MockState:
pub reset_tokens: HashMap<String, String>,  // token → user_email
pub reset_cooldowns: HashMap<String, Instant>,  // email → last_request_time
```

**Test coverage (integration):**

| # | Test | Assertions |
|---|------|------------|
| T1 | `test_request_password_reset_success` | Returns status "success", ResponseCode "11901" |
| T2 | `test_request_password_reset_nonexistent_email` | Still returns success (anti-enumeration) |
| T3 | `test_verify_reset_token_valid` | Returns "11902" |
| T4 | `test_verify_reset_token_invalid` | Returns "31904" |
| T5 | `test_reset_password_success` | Returns "11005", password updated |
| T6 | `test_reset_password_invalid_token` | Returns "31904" |

**Acceptance criteria:**
- [ ] 3 new guest mutations in mock backend
- [ ] 6 integration tests pass
- [ ] Existing 121+ tests unaffected

---

## Implementation Order

```
Task 1 (auto-redirect)       ← S effort, no dependencies
Task 4 (BackButton reuse)    ← S effort, no dependencies
Task 3 (interval fix)        ← S effort, no dependencies
Task 2 (cookie persistence)  ← M effort, no dependencies
─── above: frontend fixes (can be done in parallel) ───
Task 6 (mock backend)        ← M effort, independent
Task 5 (E2E tests)           ← L effort, depends on Task 6
```

Tasks 1–4 can be implemented in a single PR. Task 6 is a separate backend PR. Task 5 depends on Task 6 and should be its own PR.

---

## Definition of Done

- [ ] All 4 convergence tracker gaps resolved (Tasks 1–4)
- [ ] Automated redirect for logged-in users
- [ ] Resend counter survives page reload
- [ ] Countdown timer never stacks intervals
- [ ] `BackButton` component reused (no inline back button)
- [ ] Cargo clippy clean, `cargo fmt` clean
- [ ] SCSS unchanged (no styling regressions)
- [ ] Mock backend password reset endpoints (6 tests)
- [ ] E2E tests for full reset flow (10 tests)
- [ ] Feature convergence status promoted: 🚧 → ✅ Implemented
