# Step 12 — Toast Notification Component

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Build a reusable toast/notification component that matches the existing PHP/JS behaviour, provide a reactive context for showing toasts from anywhere in the component tree, and implement a `user_friendly_msg()` utility that maps backend response codes to human-readable messages using `json/response-codes.json`.

---

## 12.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 7 complete | `cargo leptos build` | Compiles; referral verification shows errors on failure |
| Step 9 complete | `cargo leptos build` | Compiles; registration submission handles success/error paths |
| Mock backend running | `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` | Returns `{"data":{"_health":true}}` |
| Response codes JSON exists | `cat json/response-codes.json \| jq '.data["10601"]'` | Returns the registration success entry |
| Existing toast CSS exists | Check `css/login-register.css` | `.toast`, `.toast.show`, `.toast.success`, `.toast.error` classes defined |

---

## 12.2 — Architecture Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                                App Component                                 │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │  ToastProvider (context)                                                 ││
│  │                                                                          ││
│  │  ┌──────────────────┐     ┌──────────────────────────────────────────┐  ││
│  │  │  toasts           │     │  ToastContainer Component               │  ││
│  │  │  RwSignal<Vec<    │────►│                                          │  ││
│  │  │    ToastEntry>>   │     │  ┌────────────────────────────────────┐  │  ││
│  │  └──────────────────┘     │  │  <For> loop over active toasts     │  │  ││
│  │                            │  │                                    │  │  ││
│  │  ┌──────────────────┐     │  │  ┌──────────────────────────────┐  │  │  ││
│  │  │  show_toast()     │     │  │  │  Toast div                  │  │  │  ││
│  │  │  (write fn from   │     │  │  │  .toast .show .{type}       │  │  │  ││
│  │  │   context)        │     │  │  │  role="alert"               │  │  │  ││
│  │  └──────────────────┘     │  │  │  aria-live="assertive"       │  │  │  ││
│  │         ▲                  │  │  └──────────────────────────────┘  │  │  ││
│  │         │                  │  └────────────────────────────────────┘  │  ││
│  │         │                  └──────────────────────────────────────────┘  ││
│  │         │                                                                ││
│  │  ┌──────┴───────────────────────────────────────────────────────────────┐││
│  │  │  Child Components (RegisterPage, ReferralStep, etc.)                │││
│  │  │                                                                      │││
│  │  │  let toast = use_context::<ToastContext>().unwrap();                 │││
│  │  │  toast.show("Message here", ToastType::Error);                      │││
│  │  │  toast.show_code("30601"); // auto-maps response code               │││
│  │  └──────────────────────────────────────────────────────────────────────┘││
│  └──────────────────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────────────────┘
```

### Key Behaviour

1. **`ToastProvider`** wraps the app (or the register page subtree) and provides a `ToastContext` via Leptos context
2. **Any child component** calls `toast.show(msg, type)` or `toast.show_code(code)` to trigger a toast
3. **`ToastContainer`** renders all active toasts as a stack at the top-right of the viewport
4. **Auto-dismiss** after 3 seconds with a slide-out animation (matching existing JS: 300ms transition)
5. **Only one toast visible** at a time — showing a new toast removes any existing one (matching existing JS behaviour)
6. **`user_friendly_msg()`** maps response code strings to human-readable messages from a compiled-in lookup table

---

## 12.3 — Reference: Existing JS Implementation

The Leptos component must replicate the behaviour of the `showToast()` method in `js/register/register.js`:

```javascript
showToast(message, type = 'info') {
    // Remove existing toast
    const existingToast = document.querySelector('.toast');
    existingToast?.remove();

    // Create new toast
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;
    toast.setAttribute('role', 'alert');
    toast.setAttribute('aria-live', 'assertive');

    document.body.appendChild(toast);

    // Show toast
    setTimeout(() => toast.classList.add('show'), 100);

    // Hide toast after 3 seconds
    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}
```

And the `userfriendlymsg()` function in `js/lib/modal.js`:

```javascript
function userfriendlymsg(code) {
    let msg;
    if (code in responsecodes.data) {
        msg = responsecodes.data[code].userFriendlyComment;
    } else {
        msg = code;
    }
    return msg;
}
```

### CSS Classes Used (from `css/login-register.css`)

| Class | Purpose |
|-------|---------|
| `.toast` | Fixed position `top: 20px; right: 20px`, dark background (`#333`), white text, `border-radius: 8px`, off-screen right (`translateX(100%)`), `z-index: 1000`, `transition: transform 0.3s ease` |
| `.toast.show` | Slides in: `translateX(0)` |
| `.toast.success` | Green background: `var(--Green-Accent)`, black text |
| `.toast.error` | Red background: `var(--Red-Accent)` |

Responsive overrides:

| Breakpoint | Change |
|------------|--------|
| `≤ 1220px` | `font-size: clamp(1.25rem, 0rem + 1vw, 1.875rem)` |
| `≤ 768px` | `font-size: 16px` |

---

## 12.4 — Implementation: Toast Types & Data Model

### 12.4.1 — Create `src/components/toast.rs`

```rust
//! Reusable toast notification component.
//!
//! Provides a `ToastProvider` context and `ToastContainer` renderer.
//! Any child component can trigger toasts via `use_context::<ToastContext>()`.

use leptos::prelude::*;
use std::time::Duration;

/// The visual style of a toast notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastType {
    Info,
    Success,
    Error,
}

impl ToastType {
    /// Returns the CSS class suffix for this toast type.
    fn css_class(&self) -> &'static str {
        match self {
            ToastType::Info => "",
            ToastType::Success => "success",
            ToastType::Error => "error",
        }
    }
}

/// A single toast entry in the notification queue.
#[derive(Clone)]
struct ToastEntry {
    /// Unique ID for keyed rendering.
    id: u64,
    /// The message to display.
    message: String,
    /// The toast type (determines styling).
    toast_type: ToastType,
    /// Whether the `.show` class has been applied (for slide-in animation).
    visible: RwSignal<bool>,
}
```

---

## 12.5 — Implementation: Toast Context

### 12.5.1 — `ToastContext` struct and `ToastProvider` component

```rust
/// Handle for showing toasts, obtained via `use_context::<ToastContext>()`.
#[derive(Clone, Copy)]
pub struct ToastContext {
    /// The list of active toasts.
    toasts: RwSignal<Vec<ToastEntry>>,
    /// Monotonically increasing ID counter.
    next_id: RwSignal<u64>,
}

impl ToastContext {
    /// Show a toast with a custom message and type.
    ///
    /// Removes any existing toast first (matching the JS behaviour of
    /// only one toast visible at a time), then creates a new one that
    /// auto-dismisses after 3 seconds.
    pub fn show(&self, message: impl Into<String>, toast_type: ToastType) {
        let message = message.into();
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);

        let visible = RwSignal::new(false);

        let entry = ToastEntry {
            id,
            message,
            toast_type,
            visible,
        };

        // Replace any existing toasts (only one at a time)
        self.toasts.set(vec![entry]);

        // Trigger slide-in after a short delay (matches JS: setTimeout 100ms)
        let toasts = self.toasts;
        set_timeout(
            move || {
                visible.set(true);
            },
            Duration::from_millis(100),
        );

        // Auto-dismiss after 3 seconds
        set_timeout(
            move || {
                // Start slide-out
                visible.set(false);

                // Remove from DOM after slide-out animation (300ms)
                set_timeout(
                    move || {
                        toasts.update(|t| t.retain(|e| e.id != id));
                    },
                    Duration::from_millis(300),
                );
            },
            Duration::from_millis(3000),
        );
    }

    /// Show a toast by response code.
    ///
    /// Looks up the code in the compiled response-code map.
    /// Determines the toast type from the code prefix:
    /// - `1xxxx` → Success
    /// - `2xxxx` → Info
    /// - `3xxxx` / `4xxxx` → Error
    pub fn show_code(&self, code: &str) {
        let message = user_friendly_msg(code).to_string();
        let toast_type = toast_type_from_code(code);
        self.show(message, toast_type);
    }
}

/// Infer the toast type from a response code prefix.
///
/// Codes starting with `1` are successes, `2` are informational,
/// and `3`/`4` are errors (validation failures, server errors).
fn toast_type_from_code(code: &str) -> ToastType {
    match code.chars().next() {
        Some('1') => ToastType::Success,
        Some('2') => ToastType::Info,
        _ => ToastType::Error,
    }
}

/// Wraps children with toast context. Place this near the top of your
/// component tree (e.g. inside `App` or `RegisterPage`).
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toasts = RwSignal::new(Vec::<ToastEntry>::new());
    let next_id = RwSignal::new(0u64);

    let ctx = ToastContext { toasts, next_id };
    provide_context(ctx);

    view! {
        {children()}
        <ToastContainer toasts=toasts />
    }
}
```

---

## 12.6 — Implementation: Toast Container & Rendering

### 12.6.1 — `ToastContainer` component

```rust
/// Renders the active toast notifications.
///
/// Placed at the end of the `ToastProvider` output so it's above
/// all other content (positioned fixed via CSS).
#[component]
fn ToastContainer(toasts: RwSignal<Vec<ToastEntry>>) -> impl IntoView {
    view! {
        <For
            each=move || toasts.get()
            key=|entry| entry.id
            children=move |entry| {
                let type_class = entry.toast_type.css_class().to_string();
                let visible = entry.visible;

                view! {
                    <div
                        class=move || {
                            let mut classes = String::from("toast");
                            if !type_class.is_empty() {
                                classes.push(' ');
                                classes.push_str(&type_class);
                            }
                            if visible.get() {
                                classes.push_str(" show");
                            }
                            classes
                        }
                        role="alert"
                        aria-live="assertive"
                    >
                        {entry.message.clone()}
                    </div>
                }
            }
        />
    }
}
```

### 12.6.2 — Export from `src/components/mod.rs`

```rust
pub mod toast;
```

---

## 12.7 — Implementation: Response Code Mapping

### 12.7.1 — Create `src/utils/mod.rs`

```rust
pub mod response_codes;
```

### 12.7.2 — Create `src/utils/response_codes.rs`

This module provides a compile-time lookup table mapping backend response codes to user-friendly messages. The map is built from the codes relevant to the registration flow (and a selection of commonly encountered codes), with a fallback for unknown codes.

```rust
//! Maps backend response codes to user-friendly messages.
//!
//! Based on `json/response-codes.json`. Only codes that can appear
//! during the registration flow are included here; additional codes
//! can be added as more pages are migrated.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Compiled map of response code → user-friendly message.
///
/// Populated with registration-relevant codes. Extended as
/// new pages are migrated to Leptos.
static RESPONSE_CODES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // ── Registration & Account ───────────────────────────────────────
    m.insert("10601", "Registration successful! Please check your email to verify your account.");
    m.insert("10701", "Account verified! You can now log in.");
    m.insert("10801", "Login successful! Welcome back.");

    // ── Referral ─────────────────────────────────────────────────────
    m.insert("11011", "Referral information loaded.");
    m.insert("21002", "Creating new referral link for you.");
    m.insert("21003", "No referral information available.");
    m.insert("31010", "The referral code you entered is invalid. Please check and try again.");

    // ── Validation (30xxx) ───────────────────────────────────────────
    m.insert("30101", "Please fill in all required fields.");
    m.insert("30102", "Some fields are empty. Please complete them.");
    m.insert("30103", "Some fields have invalid format. Please correct them.");
    m.insert("30202", "Username must be 3-23 chars with letters, numbers, or underscores.");
    m.insert("30224", "Please enter a valid email address.");
    m.insert("30226", "Password must be 8-128 chars with uppercase, lowercase, numbers and special characters.");
    m.insert("30231", "Please fill in all required fields correctly.");

    // ── Duplicate / Conflict (306xx) ─────────────────────────────────
    m.insert("30601", "Email already registered. Use a different one.");
    m.insert("30701", "Account already verified!");

    // ── Invalid Action (31xxx) ───────────────────────────────────────
    m.insert("30801", "Invalid email or password. Please try again.");

    // ── Server Errors (4xxxx) ────────────────────────────────────────
    m.insert("40601", "We couldn't complete your registration. Please try again or contact support.");
    m.insert("40602", "We're having trouble creating your account. Please try again.");
    m.insert("40701", "We couldn't verify your account. Please try again or contact support.");

    m
});

/// Look up the user-friendly message for a response code.
///
/// Returns the mapped message if the code is known, otherwise
/// returns the code itself as a fallback (matching the JS
/// `userfriendlymsg()` behaviour).
pub fn user_friendly_msg(code: &str) -> &str {
    RESPONSE_CODES.get(code).copied().unwrap_or(code)
}

/// Convenience re-export for use in components.
pub use user_friendly_msg as userfriendlymsg;
```

### 12.7.3 — Wire into `src/lib.rs`

Add the `utils` module declaration:

```rust
pub mod utils;
```

---

## 12.8 — Implementation: Integration into RegisterPage

### 12.8.1 — Wrap RegisterPage content with `ToastProvider`

In `src/pages/register.rs`, wrap the page content so all child components have access to the toast context:

```rust
use crate::components::toast::{ToastProvider, ToastContext, ToastType};
use crate::utils::response_codes::user_friendly_msg;

#[component]
pub fn RegisterPage() -> impl IntoView {
    view! {
        <ToastProvider>
            <div class="container">
                // ... step 1 (referral) ...
                // ... step 2 (registration form) ...
                // ... step 3 (success) ...
            </div>
        </ToastProvider>
    }
}
```

### 12.8.2 — Replace inline error handling in Step 7 (Referral Verification)

Current pattern (from step 7 — ad-hoc error display):

```rust
// BEFORE: inline error message
error_message.set(Some("Error verifying referral code".into()));
```

Updated pattern using toast context:

```rust
// AFTER: toast notification
let toast = use_context::<ToastContext>().expect("ToastProvider missing");

// On server error:
toast.show("Error verifying referral code.", ToastType::Error);

// On known response code:
toast.show_code(&response.response_code);
```

### 12.8.3 — Replace inline error handling in Step 9 (Registration Submission)

```rust
// BEFORE: ad-hoc error display
// error_signal.set(Some(format!("Registration failed: {}", msg)));

// AFTER: toast notification
let toast = use_context::<ToastContext>().expect("ToastProvider missing");

Effect::new(move |_| {
    if let Some(result) = register_action.value().get() {
        match result {
            Ok(response) if response.status == "success" => {
                toast.show_code(&response.response_code);  // "10601" → success toast
                current_step.set(3);
            }
            Ok(response) if response.response_code == "30601" => {
                // Duplicate email — show as toast AND highlight field
                toast.show_code(&response.response_code);
                email_error.set(Some(user_friendly_msg(&response.response_code).to_string()));
            }
            Ok(response) => {
                toast.show_code(&response.response_code);
            }
            Err(e) => {
                toast.show(
                    "Something went wrong. Please try again.",
                    ToastType::Error,
                );
            }
        }
    }
});
```

---

## 12.9 — SSR Considerations

| Concern | Approach |
|---------|----------|
| `set_timeout` not available on server | `set_timeout` from `leptos` is a no-op on the server — toasts are a client-only interaction |
| Initial SSR HTML | No toasts rendered in initial HTML (the `toasts` signal starts empty) — correct behaviour |
| `LazyLock` for response codes | Thread-safe, zero-cost after first access; works on both server and client |
| Hydration | `ToastContainer` hydrates as empty `<For>` — no mismatch since no toasts are active on page load |
| `provide_context` / `use_context` | Works identically on server and client in Leptos |

---

## 12.10 — File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `src/components/toast.rs` | **Create** | `ToastType`, `ToastEntry`, `ToastContext`, `ToastProvider`, `ToastContainer` |
| `src/components/mod.rs` | **Edit** | Add `pub mod toast;` |
| `src/utils/mod.rs` | **Create** | Add `pub mod response_codes;` |
| `src/utils/response_codes.rs` | **Create** | `RESPONSE_CODES` map, `user_friendly_msg()`, `toast_type_from_code()` |
| `src/lib.rs` | **Edit** | Add `pub mod utils;` |
| `src/pages/register.rs` | **Edit** | Wrap page in `ToastProvider`, replace inline error displays with `toast.show()` / `toast.show_code()` calls |

---

## 12.11 — Testing

### 12.11.1 — Unit Test: Response Code Lookup

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("10601"),
            "Registration successful! Please check your email to verify your account."
        );
    }

    #[test]
    fn duplicate_email_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("30601"),
            "Email already registered. Use a different one."
        );
    }

    #[test]
    fn invalid_referral_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("31010"),
            "The referral code you entered is invalid. Please check and try again."
        );
    }

    #[test]
    fn server_error_code_returns_friendly_message() {
        assert_eq!(
            user_friendly_msg("40601"),
            "We couldn't complete your registration. Please try again or contact support."
        );
    }

    #[test]
    fn unknown_code_returns_code_itself() {
        assert_eq!(user_friendly_msg("99999"), "99999");
    }

    #[test]
    fn empty_code_returns_empty_string() {
        assert_eq!(user_friendly_msg(""), "");
    }
}
```

Run:

```bash
cargo test --lib utils::response_codes
```

### 12.11.2 — Unit Test: Toast Type Inference from Code

```rust
#[cfg(test)]
mod toast_type_tests {
    use super::*;

    #[test]
    fn success_codes_start_with_1() {
        assert_eq!(toast_type_from_code("10601"), ToastType::Success);
        assert_eq!(toast_type_from_code("10701"), ToastType::Success);
        assert_eq!(toast_type_from_code("11011"), ToastType::Success);
    }

    #[test]
    fn info_codes_start_with_2() {
        assert_eq!(toast_type_from_code("21002"), ToastType::Info);
        assert_eq!(toast_type_from_code("21003"), ToastType::Info);
    }

    #[test]
    fn validation_errors_start_with_3() {
        assert_eq!(toast_type_from_code("30601"), ToastType::Error);
        assert_eq!(toast_type_from_code("31010"), ToastType::Error);
    }

    #[test]
    fn server_errors_start_with_4() {
        assert_eq!(toast_type_from_code("40601"), ToastType::Error);
        assert_eq!(toast_type_from_code("40701"), ToastType::Error);
    }

    #[test]
    fn empty_code_defaults_to_error() {
        assert_eq!(toast_type_from_code(""), ToastType::Error);
    }
}
```

### 12.11.3 — SSR Render Test: ToastContainer Renders Empty

```rust
#[cfg(test)]
mod ssr_tests {
    use super::*;
    use leptos::prelude::*;

    #[test]
    fn toast_container_renders_empty_initially() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let html = leptos::ssr::render_to_string(|| {
                view! {
                    <ToastProvider>
                        <p>"Page content"</p>
                    </ToastProvider>
                }
            });

            // Toast container should be present but empty (no .toast divs)
            assert!(html.contains("Page content"), "Child content should render");
            assert!(!html.contains("class=\"toast"), "No toast should be visible on initial SSR");
        });
    }
}
```

### 12.11.4 — WASM Integration Test: Toast Appears and Auto-Dismisses

```rust
// tests/wasm/toast.rs
#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    use web_sys::window;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_toast_appears_in_dom() {
        // Mount the app with ToastProvider
        // Programmatically trigger a toast via context
        // After 200ms, assert a .toast.show element exists in the DOM
        // After 3500ms, assert the .toast element has been removed

        let document = window().unwrap().document().unwrap();

        // Initial state: no toasts
        let toasts = document.query_selector_all(".toast").unwrap();
        assert_eq!(toasts.length(), 0, "No toasts initially");

        // (Mount and trigger toast via leptos test utilities)
        // ... test implementation depends on leptos test harness ...
    }
}
```

Run:

```bash
wasm-pack test --headless --chrome -- --test wasm_toast
```

### 12.11.5 — Manual Browser Testing Checklist

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | Success toast renders | Submit valid registration data against mock backend | Green toast slides in from the right with "Registration successful! Please check your email to verify your account." |
| 2 | Error toast renders | Submit registration with already-used email (mock returns `30601`) | Red toast slides in with "Email already registered. Use a different one." |
| 3 | Info toast renders (if applicable) | Trigger an info-level response code | Dark grey toast slides in with the mapped message |
| 4 | Auto-dismiss timing | Observe any toast | Toast slides in after ~100ms, remains for 3s, slides out over 300ms, then is removed from DOM |
| 5 | New toast replaces existing | Trigger two errors quickly in succession | First toast is immediately replaced by the second (only one visible at a time) |
| 6 | Screen reader announces toast | Enable VoiceOver (macOS: ⌘+F5) and trigger a toast | VoiceOver reads the toast message aloud (via `role="alert"` + `aria-live="assertive"`) |
| 7 | Referral error uses toast | Enter an invalid referral code and click "Verify Code" | Error toast shows "The referral code you entered is invalid. Please check and try again." |
| 8 | Network error uses toast | Stop the mock backend, then try to register | Error toast shows "Something went wrong. Please try again." |
| 9 | Unknown code fallback | (Dev only) Force a response with code `"99999"` | Toast displays "99999" as its message |
| 10 | Responsive positioning | Test at 375px, 768px, and 1440px widths | Toast appears at top-right, text is legible, does not overflow viewport |

---

## 12.12 — CSS: No Changes Required

The existing `css/login-register.css` already defines all necessary toast styles:

```css
.toast {
    position: fixed;
    top: 20px;
    right: 20px;
    background: #333;
    color: var(--White-primary);
    padding: 12px 24px;
    border-radius: 8px;
    font-size: 28px;
    font-size: clamp(1.5rem, 1.25rem + 0.2vw, 1.75rem);
    transform: translateX(100%);
    transition: transform 0.3s ease;
    z-index: 1000;
}

.toast.show {
    transform: translateX(0);
}

.toast.success {
    background: var(--Green-Accent);
    color: var(--Black);
}

.toast.error {
    background: var(--Red-Accent);
}
```

The Leptos component must produce elements with these exact class names. No additional CSS is needed — the existing stylesheet is imported in Step 5.

---

## 12.13 — Accessibility Checklist

| Requirement | Implementation |
|-------------|----------------|
| Screen reader announcement | `role="alert"` + `aria-live="assertive"` on each toast div — screen readers interrupt to read the message immediately |
| Sufficient auto-dismiss duration | 3 seconds matches the existing JS; WCAG 2.2.1 allows auto-dismissing non-essential notifications |
| Colour contrast | `.toast.success` uses `var(--Green-Accent)` bg with `var(--Black)` text (high contrast); `.toast.error` uses `var(--Red-Accent)` bg with `var(--White-primary)` text |
| Not the sole error indicator | Toasts supplement inline field errors (e.g. `30601` also highlights the email field) — toasts are never the only way an error is communicated |
| Reduced motion support | `css/login-register.css` already has `@media (prefers-reduced-motion: reduce)` that sets `transition-duration: 0.01ms` |

---

## 12.14 — Edge Cases & Error Handling

| Edge Case | Handling |
|-----------|----------|
| Rapid-fire toast triggers | New toast replaces existing one immediately (`.toasts.set(vec![entry])` clears previous) |
| Component unmount during timeout | `set_timeout` callbacks may fire after navigation; the `toasts` signal update is a no-op if the owner is disposed — Leptos handles this safely |
| Empty message string | Allowed — renders an empty toast div (no crash). Callers should avoid this |
| Very long message | CSS `padding: 12px 24px` with no `max-width` — message will expand. Consider adding `max-width: 400px; word-wrap: break-word;` if messages can be long (optional enhancement) |
| Server-side rendering | `set_timeout` is a no-op on the server — no toasts in SSR HTML (correct behaviour) |
| Missing `ToastProvider` in tree | `use_context::<ToastContext>().expect("ToastProvider missing")` panics with a clear message during development |

---

## 12.15 — Future Extensibility

This toast component is designed to be reused across the entire Leptos app, not just the registration page. As more pages are migrated:

| Extension | Approach |
|-----------|----------|
| Add more response codes | Append entries to the `RESPONSE_CODES` map in `response_codes.rs` |
| Move `ToastProvider` to `App` level | Once multiple pages need toasts, hoist the provider from `RegisterPage` to `App` in `app.rs` |
| Add toast stacking | Change `toasts.set(vec![entry])` to `toasts.update(\|t\| t.push(entry))` and add vertical stacking CSS |
| Add manual dismiss (click to close) | Add an `on:click` handler that triggers the slide-out + removal |
| Add toast icons | Extend `ToastType` with an `icon()` method returning the peer-icon class name |
| Load codes from JSON at build time | Use a Rust build script (`build.rs`) to parse `json/response-codes.json` and generate the HashMap at compile time |

---

## 12.16 — Dependency Graph

```
Step 7 (Referral Verification)
    │
    ├──► currently shows inline errors
    │
Step 9 (Registration Submission)
    │
    ├──► currently shows inline errors
    │
    ▼
Step 12 (Toast Notification)  ◄── YOU ARE HERE
    │
    ├──► Step 13 (Accessibility) — audits toast a11y compliance
    ├──► Step 14 (CSS & Visual Parity) — verifies toast looks identical to PHP version
    └──► Step 15 (E2E Tests) — exercises toast appearance in test scenarios
```

---

## 12.17 — Definition of Done

- [ ] `ToastType` enum created with `Info`, `Success`, `Error` variants
- [ ] `ToastContext` struct with `show(msg, type)` and `show_code(code)` methods
- [ ] `ToastProvider` component provides context and renders `ToastContainer`
- [ ] `ToastContainer` renders toast divs with correct CSS classes (`.toast`, `.toast.show`, `.success`, `.error`)
- [ ] Toast auto-dismisses after 3 seconds with slide-in/slide-out animation
- [ ] Only one toast visible at a time (new toast replaces existing)
- [ ] `role="alert"` and `aria-live="assertive"` on toast elements
- [ ] `user_friendly_msg()` maps all registration-relevant response codes
- [ ] Unknown codes fall back to returning the code string itself
- [ ] `toast_type_from_code()` correctly infers toast type from code prefix
- [ ] `RegisterPage` wrapped in `ToastProvider`
- [ ] Step 7 (referral verification) error handling uses toast
- [ ] Step 9 (registration submission) error handling uses toast
- [ ] `cargo leptos build` compiles with no errors
- [ ] Unit tests pass (`cargo test`)  — response code lookup + toast type inference
- [ ] SSR render test passes — `ToastProvider` renders children with empty toast container
- [ ] Manual browser testing checklist (12.11.5) passes all 10 items
