# Step 6 — Step 1 UI: Referral Code Entry

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Build the first registration step — the referral code input form with client-side UUID validation, a "default referral code" sub-step, and URL parameter prefill. This step produces the reactive UI only; the server round-trip (actually calling `verify_referral`) is wired in Step 7.

**Depends on:** Step 5 (Router & Page Shell)

---

## 6.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 5 complete | Navigate to `http://localhost:3000/register` | Renders the page shell with `.container` div |
| Page component exists | Check `src/pages/register.rs` | Contains stub `RegisterPage` component |
| Router configured | Check `src/app.rs` | Has `<Route path="/register" view=RegisterPage />` |
| CSS imported | Check `style/main.scss` or `public/` | `login-register.css` is loadable |
| Shared types | Check `src/models/user.rs` | `ReferralVerifyResponse` type defined |
| Leptos dev server runs | `cargo leptos watch` | Compiles and serves at `localhost:3000` |

---

## 6.2 — Architecture Overview

Step 6 is purely client-side UI. No server functions are called yet — that happens in Step 7. The goal is to render the referral step HTML, validate input reactively, and manage the sub-step toggle ("Don't have a code?" → default referral display).

```
┌──────────────────────────────────────────────────────────────┐
│  RegisterPage Component (src/pages/register.rs)              │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Signals:                                              │  │
│  │    current_step: RwSignal<RegistrationStep>            │  │
│  │    referral_code: RwSignal<String>                     │  │
│  │    show_default_referral: RwSignal<bool>               │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  ReferralStep Component (src/components/referral.rs)   │  │
│  │    ├── Referral code input + validation icon           │  │
│  │    ├── Validation message (reactive)                   │  │
│  │    ├── "Verify Code" button (disabled until valid)     │  │
│  │    ├── "Don't have a code?" link                       │  │
│  │    └── DefaultReferral sub-view (toggled)              │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Validation (src/components/validation.rs)             │  │
│  │    └── validate_uuid(input) → bool                    │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Separate `ReferralStep` component | Keeps `RegisterPage` clean; each step is its own component (pattern continues in Steps 8, 10) |
| Client-side validation only (no server call) | Step 7 adds the server round-trip; separation makes both steps independently testable |
| `RwSignal<String>` for referral code | Parent owns the signal so it persists when navigating back from step 2 (Step 11) |
| Derived signal for validity | `is_valid` is computed from `referral_code`, not stored separately — single source of truth |
| Default referral code is a constant | The default UUID `85d5f836-b1f5-4c4e-9381-1b058e13df93` is hardcoded (matches current PHP) |
| URL param read on mount | Use `leptos_router::hooks::use_query_map` to read `?ref=` on initial render |

---

## 6.3 — Files to Create / Modify

| File | Action | Purpose |
|------|--------|---------|
| `src/components/mod.rs` | Modify | Add `pub mod referral;` and `pub mod validation;` |
| `src/components/referral.rs` | **Create** | `ReferralStep` and `DefaultReferralView` components |
| `src/components/validation.rs` | **Create** | Client-side UUID regex validation function |
| `src/pages/register.rs` | Modify | Replace stub with full step-management shell + `ReferralStep` |
| `src/pages/mod.rs` | Verify | Ensure `pub mod register;` exists |

---

## 6.4 — Client-Side Validation Module

### Create `src/components/validation.rs`

This module provides client-side validation helpers. The UUID validator mirrors the existing JavaScript regex: `/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i`

```rust
//! Client-side validation helpers for registration form fields.
//!
//! These run in the browser (WASM) for instant feedback.
//! Server-side validation in `src/api/validation.rs` (Step 4) 
//! provides defense-in-depth.

/// Validate that a string matches UUID format.
///
/// Accepts: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (hex digits, case-insensitive).
/// Does not enforce UUID v4 variant bits — the backend handles that.
///
/// # Examples
/// ```
/// assert!(is_valid_uuid("85d5f836-b1f5-4c4e-9381-1b058e13df93"));
/// assert!(!is_valid_uuid("abc123"));
/// assert!(!is_valid_uuid(""));
/// ```
pub fn is_valid_uuid(input: &str) -> bool {
    let input = input.trim();
    if input.len() != 36 {
        return false;
    }

    input
        .chars()
        .enumerate()
        .all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit(),
        })
}
```

> **Why not use the `regex` crate client-side?** The `regex` crate adds ~200 KB to the WASM binary. The character-by-character check above is zero-dependency, compiles to tiny WASM, and is equivalent to the regex. The `regex` crate is already used on the server side (Step 4's `validation.rs`) where binary size is irrelevant.

### Unit tests (in the same file)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_uuid_lowercase() {
        assert!(is_valid_uuid("85d5f836-b1f5-4c4e-9381-1b058e13df93"));
    }

    #[test]
    fn valid_uuid_uppercase() {
        assert!(is_valid_uuid("85D5F836-B1F5-4C4E-9381-1B058E13DF93"));
    }

    #[test]
    fn valid_uuid_mixed_case() {
        assert!(is_valid_uuid("85d5F836-b1F5-4c4E-9381-1b058E13df93"));
    }

    #[test]
    fn valid_uuid_with_whitespace_trimmed() {
        assert!(is_valid_uuid("  85d5f836-b1f5-4c4e-9381-1b058e13df93  "));
    }

    #[test]
    fn invalid_empty_string() {
        assert!(!is_valid_uuid(""));
    }

    #[test]
    fn invalid_short_string() {
        assert!(!is_valid_uuid("abc123"));
    }

    #[test]
    fn invalid_no_hyphens() {
        assert!(!is_valid_uuid("85d5f836b1f54c4e93811b058e13df93"));
    }

    #[test]
    fn invalid_wrong_hyphen_positions() {
        assert!(!is_valid_uuid("85d5f83-6b1f5-4c4e-9381-1b058e13df93"));
    }

    #[test]
    fn invalid_non_hex_characters() {
        assert!(!is_valid_uuid("85d5f836-b1f5-4c4e-9381-1b058e13dg93"));
    }

    #[test]
    fn invalid_too_long() {
        assert!(!is_valid_uuid("85d5f836-b1f5-4c4e-9381-1b058e13df930"));
    }
}
```

---

## 6.5 — Registration Step Enum

Before building the referral component, define the step enum used by `RegisterPage` to control which step is visible. This will be used across Steps 6, 8, and 10.

### Add to `src/pages/register.rs` (or a shared module)

```rust
/// The three main steps of the registration flow.
/// Step 1b (default referral) is handled as a sub-view within Step 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationStep {
    /// Step 1: Enter referral code
    Referral,
    /// Step 2: Fill registration form (email, username, password)
    Registration,
    /// Step 3: Success confirmation
    Success,
}

impl Default for RegistrationStep {
    fn default() -> Self {
        Self::Referral
    }
}
```

---

## 6.6 — Referral Step Component

### Create `src/components/referral.rs`

This is the main deliverable of Step 6. The component renders the referral code input form (matching the current `register.php` Step 1 markup) with reactive validation.

```rust
use leptos::prelude::*;
use crate::components::validation::is_valid_uuid;

/// The default referral code shown when the user clicks "Don't have a code?"
/// Matches the value hardcoded in the current PHP version.
const DEFAULT_REFERRAL_CODE: &str = "85d5f836-b1f5-4c4e-9381-1b058e13df93";

/// Props for the ReferralStep component.
///
/// The parent (`RegisterPage`) owns the signals so that:
/// - `referral_code` persists when navigating back from step 2
/// - `on_verify` can be swapped between a no-op (Step 6) and the real 
///   server call (Step 7)
#[component]
pub fn ReferralStep(
    /// The referral code signal, owned by the parent.
    referral_code: RwSignal<String>,
    /// Callback invoked when the user clicks "Verify Code" with a valid UUID.
    /// In Step 6 this simply advances to step 2; in Step 7 it triggers the
    /// server function.
    on_verify: Action<String, ()>,
) -> impl IntoView {
    // ...
}
```

### 6.6.1 — Signals & Derived State

Inside the component body:

```rust
// Whether to show the default-referral sub-step (step 1b)
let show_default = RwSignal::new(false);

// Derived: is the current input a valid UUID?
let is_valid = Memo::new(move |_| {
    is_valid_uuid(&referral_code.get())
});

// Derived: validation message text
let validation_message = Memo::new(move |_| {
    let code = referral_code.get();
    if code.is_empty() {
        String::new() // no message when empty
    } else if is_valid.get() {
        String::new() // valid — hide message
    } else {
        "Hmm\u{2026} that referral code doesn't seem to work. \
         Ask your friend to send you a new link, or use a Peer code."
            .to_string()
    }
});

// Derived: CSS class for the input field wrapper
let field_class = Memo::new(move |_| {
    let code = referral_code.get();
    if code.is_empty() {
        "input-field"
    } else if is_valid.get() {
        "input-field valid"
    } else {
        "input-field invalid"
    }
});
```

### 6.6.2 — View Markup (Step 1: Referral Code)

The `view!` macro output should mirror the existing `register.php` DOM structure to ensure CSS parity. Key classes like `form-step`, `step-header`, `input-group`, `input-field`, `validation-icon`, `validation-message`, `btn btn-primary`, and `step-footer` must match exactly.

```rust
view! {
    // Step 1: Referral Code Entry
    <div class="form-step active" id="referralStep" data-step="1">
        <div class="step-header">
            <h2 class="x_large_font">"Welcome to " <strong>"peer!"</strong></h2>
            <p class="large_font">
                "One quick step left! Enter your referral code to complete registration."
            </p>
        </div>

        <form
            on:submit=move |ev| {
                ev.prevent_default();
                if is_valid.get() {
                    on_verify.dispatch(referral_code.get_untracked());
                }
            }
            novalidate=true
        >
            <div class="input-group">
                <div class=move || field_class.get() id="referralCodeField">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-referral"></i>
                    </span>
                    <input
                        type="text"
                        id="referralCode"
                        name="referralCode"
                        placeholder="Enter your referral code"
                        required=true
                        aria-describedby="referralValidation referralHelp"
                        autocomplete="off"
                        prop:value=move || referral_code.get()
                        on:input=move |ev| {
                            referral_code.set(event_target_value(&ev));
                        }
                    />

                    // Validation icon: green check, shown only when valid
                    <span
                        class=move || {
                            if is_valid.get() {
                                "validation-icon show"
                            } else {
                                "validation-icon"
                            }
                        }
                        id="referralCodeValidIcon"
                        aria-hidden="true"
                    >
                        <i class="peer-icon peer-icon-tick-circle"></i>
                    </span>
                </div>

                // Validation message (reactive)
                <div
                    class="validation-message medium_font"
                    id="referralCodeValidation"
                    role="alert"
                    aria-live="polite"
                >
                    {move || validation_message.get()}
                </div>

                <div id="referralHelp" class="sr-only">
                    "Enter the referral code provided to you"
                </div>
            </div>

            <button
                type="submit"
                class="btn btn-primary"
                id="verifyReferralBtn"
                disabled=move || !is_valid.get()
            >
                "Verify Code"
            </button>
        </form>

        <div class="step-footer medium_font">
            <p>
                "Don't have a code? "
                <a
                    href="#"
                    on:click=move |ev| {
                        ev.prevent_default();
                        show_default.set(true);
                    }
                >
                    "Click here"
                </a>
                " to get peer code"
            </p>
        </div>
    </div>

    // Step 1b: Default Referral Code (conditionally shown)
    <Show when=move || show_default.get()>
        <DefaultReferralView
            on_use_code=move |code: String| {
                referral_code.set(code);
                show_default.set(false);
            }
        />
    </Show>
}
```

### 6.6.3 — Default Referral Sub-View (Step 1b)

This is the "Claim Your Invitation" panel shown when the user clicks "Don't have a code?" — matching `register.php`'s `#defaultReferralStep`.

```rust
/// Displays the default referral code for users who don't have one.
#[component]
fn DefaultReferralView(
    /// Callback when user clicks "Use This Code" — passes the default UUID back.
    on_use_code: impl Fn(String) + 'static,
) -> impl IntoView {
    view! {
        <div class="form-step active" id="defaultReferralStep" data-step="1b">
            <div class="step-header">
                <h2 class="x_large_font">"Claim Your Invitation"</h2>
                <p class="large_font">
                    "Earning starts the moment you enter this magic code"
                </p>
            </div>
            <div class="input-group">
                <div class="referral-code-display medium_font">
                    <span class="input-icon" aria-hidden="true">
                        <i class="peer-icon peer-icon-referral"></i>
                    </span>
                    <span id="defaultReferralCode">{DEFAULT_REFERRAL_CODE}</span>
                </div>
            </div>

            <button
                type="button"
                class="btn btn-primary"
                id="useThisCodeBtn"
                on:click=move |_| {
                    on_use_code(DEFAULT_REFERRAL_CODE.to_string());
                }
            >
                "Use This Code"
            </button>
        </div>
    }
}
```

### 6.6.4 — Step Visibility Logic

When `show_default` is `true`, the main referral step should be hidden and the default referral step shown. There are two approaches:

**Option A — CSS class toggle (matches PHP behaviour):**

Both `<div class="form-step">` elements are always in the DOM; the `active` class controls visibility. The `ReferralStep` wrapping div's class becomes:

```rust
class=move || {
    if show_default.get() { "form-step" } else { "form-step active" }
}
```

And the default referral div uses:

```rust
class=move || {
    if show_default.get() { "form-step active" } else { "form-step" }
}
```

**Option B — Conditional rendering with `<Show>`:**

Use `<Show when=...>` to mount/unmount the sub-step. Slightly cleaner if we don't need the hidden DOM.

**Recommendation:** Use **Option A** (CSS class toggle) to maximise CSS parity with the PHP version. The existing `login-register.css` relies on `.form-step.active` for display logic.

---

## 6.7 — URL Parameter Prefill

When a user arrives via a referral link like `/register?ref=85d5f836-...`, the input should be auto-filled on mount.

### Implementation in `RegisterPage` (src/pages/register.rs)

```rust
use leptos_router::hooks::use_query_map;

// Inside RegisterPage component body:
let query_params = use_query_map();
let referral_code = RwSignal::new(String::new());

// Read ?ref= or ?referralUuid= on first render
Effect::new(move |_| {
    let params = query_params.get();
    if let Some(ref_code) = params.get("ref").or_else(|| params.get("referralUuid")) {
        if !ref_code.is_empty() {
            referral_code.set(ref_code.clone());
        }
    }
});
```

This mirrors the existing JS logic:

```javascript
// Current JS (for reference — DO NOT use in Leptos):
const referralCode = urlParams.get('ref') || urlParams.get('referralUuid');
```

Both `?ref=` and `?referralUuid=` parameter names are supported for backwards compatibility.

---

## 6.8 — RegisterPage Shell Update

Replace the stub `RegisterPage` component with the step-management shell. This sets up the signals that persist across the registration flow and renders the current step.

### Update `src/pages/register.rs`

```rust
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use crate::components::referral::ReferralStep;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationStep {
    Referral,
    Registration,
    Success,
}

impl Default for RegistrationStep {
    fn default() -> Self {
        Self::Referral
    }
}

/// Root component for the `/register` route.
///
/// Owns all top-level registration signals and renders the active step.
#[component]
pub fn RegisterPage() -> impl IntoView {
    let current_step = RwSignal::new(RegistrationStep::default());
    let referral_code = RwSignal::new(String::new());

    // URL parameter prefill (?ref= or ?referralUuid=)
    let query_params = use_query_map();
    Effect::new(move |_| {
        let params = query_params.get();
        if let Some(ref_code) = params.get("ref").or_else(|| params.get("referralUuid")) {
            if !ref_code.is_empty() {
                referral_code.set(ref_code.clone());
            }
        }
    });

    // Temporary action for Step 6: just advance to step 2 on verify.
    // Step 7 replaces this with the real server function call.
    let on_verify = Action::new(move |_code: &String| {
        let step = current_step;
        async move {
            step.set(RegistrationStep::Registration);
        }
    });

    view! {
        <div class="container large_font">
            <div class="container_left">
                <div class="phone">
                    <div class="screen">
                        <img
                            src="/img/register.webp"
                            alt="Login"
                            width="612"
                            height="612"
                        />
                    </div>
                    <div class="home-button">
                        <img
                            src="/svg/logo_sw.svg"
                            alt="PeerLogo"
                            width="96"
                            height="96"
                        />
                    </div>
                </div>
                <img
                    class="logo"
                    src="/svg/logo_farbe.svg"
                    alt="Peer logo"
                    width="96"
                    height="96"
                />
            </div>

            <div class="container_right">
                <div class="container_inner">
                    <div class="top_head_area">
                        // Back button (Step 11 will add full logic)
                        <a href="/login" class="btn btn-secondary back-btn" id="backBtn">
                            <span aria-hidden="true">
                                <i class="peer-icon medium_font peer-icon-arrow-left"></i>
                            </span>
                            "Back"
                        </a>
                    </div>

                    <div class="center_area">
                        // Render the active step
                        {move || match current_step.get() {
                            RegistrationStep::Referral => view! {
                                <ReferralStep
                                    referral_code=referral_code
                                    on_verify=on_verify
                                />
                            }.into_any(),
                            RegistrationStep::Registration => view! {
                                // Placeholder until Step 8
                                <div class="form-step active" data-step="2">
                                    <p>"Registration form placeholder (Step 8)"</p>
                                </div>
                            }.into_any(),
                            RegistrationStep::Success => view! {
                                // Placeholder until Step 10
                                <div class="form-step active" data-step="3">
                                    <p>"Success placeholder (Step 10)"</p>
                                </div>
                            }.into_any(),
                        }}
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

### Key points:

- **`referral_code` is owned by the parent** — it persists even when `ReferralStep` is replaced by the registration form, so the back button (Step 11) can restore it.
- **`on_verify` is a temporary no-op action** — it advances to step 2 immediately. Step 7 replaces it with the real `verify_referral` server function call.
- **Step 2 and Step 3 are placeholders** — they'll be implemented in Steps 8 and 10 respectively.
- **The outer container markup matches `register.php`** — `.container`, `.container_left`, `.container_right`, `.container_inner`, `.top_head_area`, `.center_area`, `.footer_area` all preserved for CSS parity.

---

## 6.9 — Module Declarations

### Update `src/components/mod.rs`

```rust
pub mod referral;
pub mod validation;
```

### Verify `src/pages/mod.rs`

Ensure it contains:

```rust
pub mod register;
```

---

## 6.10 — Static Assets

The referral step requires several static assets from the existing PHP project. These need to be available to the Leptos dev server.

### Assets to copy (or symlink) into `peer-web/public/`

| Source (repo root) | Destination (`peer-web/public/`) | Used by |
|--------------------|----------------------------------|---------|
| `img/register.webp` | `img/register.webp` | Phone mockup image |
| `svg/logo_sw.svg` | `svg/logo_sw.svg` | Phone home-button logo |
| `svg/logo_farbe.svg` | `svg/logo_farbe.svg` | Left panel logo |
| `fonts/font-poppins/` | `fonts/font-poppins/` | Poppins font face |
| `fonts/peer-icon-font/` | `fonts/peer-icon-font/` | Peer icon glyphs (`peer-icon-referral`, `peer-icon-tick-circle`, `peer-icon-arrow-left`) |

### Symlink approach (recommended for development)

```bash
cd peer-web/public
ln -s ../../img img
ln -s ../../svg svg
ln -s ../../fonts fonts
```

> **Note:** For production builds, these will be copied properly. Symlinks are a development convenience.

---

## 6.11 — CSS Integration

The referral step relies on classes defined in `css/login-register.css`. This was imported in Step 5, but verify the following classes are present and render correctly:

### Required CSS classes (spot-check list)

| Class | Element | Visual effect |
|-------|---------|---------------|
| `.form-step` | Step wrapper | `display: none` by default |
| `.form-step.active` | Active step | `display: block` (or flex) |
| `.step-header` | Title area | Spacing, typography |
| `.input-group` | Form field container | Vertical spacing |
| `.input-field` | Input wrapper | Border, flex layout |
| `.input-field.valid` | Valid state | Green border |
| `.input-field.invalid` | Invalid state | Red border |
| `.input-icon` | Left icon in input | Positioning |
| `.validation-icon` | Right icon (tick) | Hidden by default |
| `.validation-icon.show` | Valid tick | Visible |
| `.validation-message` | Error text | Red text, assertive |
| `.btn.btn-primary` | Primary button | Blue gradient, hover states |
| `.btn.btn-primary:disabled` | Disabled button | Greyed out, no hover |
| `.step-footer` | Bottom link area | Centered, muted text |
| `.referral-code-display` | Default code display | Styled code box |
| `.container_left` | Left panel (phone) | Gradient background, border radius |
| `.container_right` | Right panel (form) | Dark background |
| `.sr-only` | Screen reader only | Visually hidden |

If any class is missing or renders differently, note it for Step 14 (Visual Parity).

---

## 6.12 — Accessibility Requirements

These must be implemented now (not deferred to Step 13) as they are structural:

| Feature | Implementation | Matches PHP |
|---------|----------------|-------------|
| `aria-describedby` on input | Links to `#referralValidation` and `#referralHelp` | Yes |
| `role="alert"` on validation message | Automatic screen reader announcement on change | Yes |
| `aria-live="polite"` on validation message | Non-interruptive announcements | Yes |
| `aria-hidden="true"` on decorative icons | Icons not read by screen readers | Yes |
| `sr-only` help text | "Enter the referral code provided to you" | Yes |
| Disabled button when invalid | `disabled` attribute prevents premature submit | New (improvement) |
| Form `novalidate` | Prevents browser default validation UI | Yes |
| Keyboard submit (Enter) | Native `<form>` submit on Enter key in input | Yes |

---

## 6.13 — Testing Outcomes

### 6.13.1 — Manual Testing

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | Valid UUID shows green check | Type `85d5f836-b1f5-4c4e-9381-1b058e13df93` into the input | Green tick icon appears, input field has green border, "Verify Code" button is enabled |
| 2 | Invalid input shows error | Type `abc123` | Error message "Hmm… that referral code doesn't seem to work…" appears below input, field has red border, button stays disabled |
| 3 | Empty input shows no message | Clear the input field | No validation message, no border colour, button disabled |
| 4 | URL prefill works | Navigate to `/register?ref=85d5f836-b1f5-4c4e-9381-1b058e13df93` | Input is pre-filled with the UUID, green check shown, button enabled |
| 5 | URL prefill alternate param | Navigate to `/register?referralUuid=85d5f836-b1f5-4c4e-9381-1b058e13df93` | Same as test 4 |
| 6 | "Don't have a code?" link | Click "Click here" | Default referral sub-step appears with the default UUID displayed |
| 7 | "Use This Code" button | On the default referral sub-step, click "Use This Code" | Returns to main referral step, input filled with default UUID, green check shown |
| 8 | Verify button advances step | With a valid UUID, click "Verify Code" | Transitions to step 2 placeholder (Step 7 will add server call) |
| 9 | SSR renders HTML | View page source at `/register` | HTML contains the referral form markup (not just a WASM shell) |
| 10 | Keyboard navigation | Tab through the form with keyboard only | Focus order: Back button → referral input → Verify Code → "Click here" link |

### 6.13.2 — Automated Unit Tests

Run with `cargo test`:

```bash
cargo test --lib -- validation
```

**Expected:** All 10 UUID validation tests from §6.4 pass.

### 6.13.3 — Compilation Check

```bash
cargo leptos build
```

**Expected:** Both SSR (native) and CSR (WASM) targets compile without errors or warnings from our code.

---

## 6.14 — Edge Cases & Gotchas

| Issue | Handling |
|-------|----------|
| User pastes UUID with leading/trailing spaces | `is_valid_uuid` trims whitespace; `event_target_value` preserves it in the signal, but validation still passes |
| User pastes uppercase UUID | Validation is case-insensitive (hex digits include A-F) |
| Browser auto-fill | `autocomplete="off"` on the referral input discourages browser auto-fill (referral codes aren't a standard auto-fill type) |
| Double-click "Verify Code" | Step 7 adds loading state; for Step 6, the action is idempotent (re-setting the same step is harmless) |
| `?ref=` with invalid value | Input is pre-filled, but validation shows the error state — user must correct it |
| `?ref=` with empty value | `if !ref_code.is_empty()` guard prevents overwriting the empty default |
| SSR vs CSR discrepancy | URL params are available during SSR via `use_query_map`; the `Effect` runs on both server and client, initial render is consistent |

---

## 6.15 — Implementation Checklist

- [ ] **6.15a** Create `src/components/validation.rs` with `is_valid_uuid` function and unit tests (§6.4).
- [ ] **6.15b** Run `cargo test --lib -- validation` — confirm all 10 tests pass.
- [ ] **6.15c** Add `RegistrationStep` enum to `src/pages/register.rs` (§6.5).
- [ ] **6.15d** Create `src/components/referral.rs` with `ReferralStep` component (§6.6.1–§6.6.2).
- [ ] **6.15e** Add `DefaultReferralView` sub-component in the same file (§6.6.3).
- [ ] **6.15f** Implement step visibility toggle with CSS class approach (§6.6.4, Option A).
- [ ] **6.15g** Update `src/components/mod.rs` to declare `referral` and `validation` modules (§6.9).
- [ ] **6.15h** Update `src/pages/register.rs` with full `RegisterPage` shell, URL param prefill, and temporary `on_verify` action (§6.7, §6.8).
- [ ] **6.15i** Symlink or copy static assets into `peer-web/public/` (§6.10).
- [ ] **6.15j** Run `cargo leptos build` — confirm both targets compile without errors (§6.13.3).
- [ ] **6.15k** Run `cargo leptos watch` and manually test all 10 scenarios in §6.13.1.
- [ ] **6.15l** Verify SSR by viewing page source — referral form HTML should be present (§6.13.1, test 9).
- [ ] **6.15m** Spot-check CSS classes from §6.11 — confirm input validation states render correctly.
- [ ] **6.15n** Test keyboard navigation through the referral step (§6.13.1, test 10).

---

## 6.16 — What's Next

After completing Step 6:

- **Step 7** wires the "Verify Code" button to the `verify_referral` server function (replacing the temporary `on_verify` action), adds loading state, and handles success/error responses from the mock backend.
- **Step 8** can proceed in parallel — it builds the Step 2 registration form UI (email, username, password fields), which is independent of the referral verification logic.
- **Step 11** adds back-button navigation that relies on the `referral_code` signal persisting across step transitions (already handled by the parent-owned signal in §6.8).
