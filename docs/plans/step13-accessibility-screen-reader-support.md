# Step 13 — Accessibility & Screen Reader Support

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Ensure the Leptos registration flow meets **WCAG 2.1 AA** compliance, matching or exceeding the accessibility features present in the existing PHP/JS implementation. This step is a dedicated audit-and-fix pass across all UI components built in Steps 6–12, adding missing ARIA attributes, fixing keyboard navigation, and introducing automated accessibility testing.

---

## 13.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Steps 6–12 complete | `cargo leptos build` | Compiles; full registration flow works end-to-end against mock backend |
| Mock backend running | `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` | Returns `{"data":{"_health":true}}` |
| axe-core installed | `npm ls axe-core` or `npm install -D @axe-core/cli @axe-core/playwright` | Package available |
| VoiceOver available | macOS System Settings → Accessibility → VoiceOver | Can be toggled with ⌘+F5 |
| Leptos dev server running | `cargo leptos watch` | `http://localhost:3000/register` renders all 3 steps |

---

## 13.2 — Audit Scope

The accessibility pass covers every component and interaction in the registration flow:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         Registration Flow Audit Scope                        │
│                                                                              │
│  ┌────────────────────────┐  ┌────────────────────────┐  ┌───────────────┐  │
│  │  Step 1: Referral      │  │  Step 2: Registration  │  │  Step 3:      │  │
│  │  ───────────────       │  │  ──────────────────    │  │  Success      │  │
│  │  • Referral input      │  │  • Email input         │  │  ─────────    │  │
│  │  • Validation msg      │  │  • Username input      │  │  • Success    │  │
│  │  • Verify button       │  │  • Password input      │  │  • Login link │  │
│  │  • Default code link   │  │  • Confirm password    │  │               │  │
│  │  • Step 1b sub-step    │  │  • Strength meter      │  │               │  │
│  │                        │  │  • Eye toggle buttons   │  │               │  │
│  │                        │  │  • Privacy checkbox     │  │               │  │
│  │                        │  │  • EULA checkbox        │  │               │  │
│  │                        │  │  • Register button      │  │               │  │
│  └────────────────────────┘  └────────────────────────┘  └───────────────┘  │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │  Cross-cutting concerns                                                  ││
│  │  • Step announcer (aria-live region)                                     ││
│  │  • Toast notifications (role="alert")                                    ││
│  │  • Back button behaviour                                                 ││
│  │  • Focus management on step transitions                                  ││
│  │  • Keyboard navigation (Tab, Shift+Tab, Enter, Escape, Space)           ││
│  │  • Colour contrast (all states)                                          ││
│  │  • Reduced motion / high contrast media queries                          ││
│  └──────────────────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 13.3 — Reference: Existing PHP/JS Accessibility Features

The current `register.php` + `js/register/register.js` already implements several accessibility patterns. The Leptos version must preserve **all** of these and close any gaps.

### 13.3.1 — ARIA Attributes in `register.php`

| Element | Attribute | Purpose |
|---------|-----------|---------|
| Back button icon `<i>` | `aria-hidden="true"` | Hides decorative icon from screen readers |
| Referral input | `aria-describedby="referralValidation referralHelp"` | Links input to validation message and help text |
| Referral validation div | `role="alert" aria-live="polite"` | Announces validation changes |
| Referral help div | `class="sr-only"` | Provides invisible help text for screen readers |
| Email input | `aria-describedby="emailValidation emailHelp"` | Links to validation + help |
| Username input | `aria-describedby="usernameValidation usernameHelp"` | Links to validation + help |
| Password input | `aria-describedby="passwordValidation passwordHelp passwordStrength"` | Links to validation + help + strength meter |
| Confirm password input | `aria-describedby="confirmPasswordValidation confirmPasswordHelp"` | Links to validation + help |
| Password toggle buttons | `aria-label="Show/hide password"` | Accessible name for icon-only buttons |
| Password requirements `<ul>` | `role="list" aria-label="Password requirements"` | Groups requirements for screen readers |
| Checkbox inputs | `aria-describedby="checkboxValidation"` | Links to shared checkbox error message |
| Validation icon `<span>`s | `aria-hidden="true"` | Hides decorative check icons |
| Input icon `<span>`s | `aria-hidden="true"` | Hides decorative field icons |
| Success icon `<i>` | `aria-hidden="true"` | Hides decorative success icon |

### 13.3.2 — JS Accessibility Behaviours in `register.js`

| Feature | Implementation |
|---------|----------------|
| **Step announcer** | `getOrCreateAnnouncer()` creates a `div#step-announcer.sr-only` with `aria-live="polite"` and `aria-atomic="true"` — announces step transitions and custom messages |
| **Focus management** | `showStep()` focuses the first interactive element (`input, button, select, textarea`) in the new step after 100ms delay |
| **Focus on first error** | `focusFirstError(stepId)` focuses the first `.input-field.invalid input` when form submission fails validation |
| **Loading state** | `setButtonLoading()` sets `aria-busy="true"` on buttons during async operations |
| **Password toggle label** | `togglePasswordVisibility()` updates `aria-label` between "Show password" and "Hide password" |
| **Checkbox error state** | `highlightCheckbox()` sets `aria-invalid="true"` on checkboxes with errors |
| **Toast announcements** | `showToast()` creates elements with `role="alert"` and `aria-live="assertive"` |

### 13.3.3 — CSS Accessibility in `login-register.css`

| Feature | Implementation |
|---------|----------------|
| `.sr-only` | Standard screen-reader-only class (absolute position, 1px × 1px, clipped) |
| `@media (prefers-contrast: high)` | Increases `.input-field` border width to 3px, uses solid `var(--Blue)` for `.btn-primary` |
| `@media (prefers-reduced-motion: reduce)` | Sets all animation/transition durations to 0.01ms |

### 13.3.4 — Known Gaps in the Current PHP Version

These issues exist in the PHP version and should be **fixed** in the Leptos version:

| Gap | Issue | WCAG Criterion |
|-----|-------|----------------|
| No visible focus indicator | `input { outline: none; }` removes the focus ring without providing a custom one | 2.4.7 Focus Visible |
| Missing `aria-invalid` on text inputs | Only checkboxes get `aria-invalid="true"` on error; text inputs rely solely on CSS class `.invalid` | 4.1.2 Name, Role, Value |
| No `<label>` elements on inputs | Inputs use `placeholder` as the only label; no `<label>` or `aria-label` | 1.3.1 Info and Relationships |
| Password strength not announced | Strength meter changes are visual-only; no `aria-live` region or `aria-valuenow` for screen readers | 4.1.3 Status Messages |
| Toggle buttons not `<button>` | Password visibility toggles are `<span>` elements — not keyboard-focusable natively | 2.1.1 Keyboard |
| Missing `autocomplete` on referral | Referral input has `autocomplete="off"` but no semantic `autocomplete` value | 1.3.5 Identify Input Purpose |
| No skip link | No mechanism to skip repeated header content | 2.4.1 Bypass Blocks |
| `lang="de"` but content is English | HTML `lang` attribute is set to German | 3.1.1 Language of Page |

---

## 13.4 — Implementation: Form Input Labels

### 13.4.1 — Add Visually-Hidden `<label>` Elements

Every input field must have a programmatically-associated label. The current PHP version relies on `placeholder` attributes, which do not satisfy WCAG 1.3.1.

Add a visually-hidden `<label>` before each input, using the `sr-only` class so the visual design is unchanged.

**In `src/components/referral_step.rs` (Step 1 referral input):**

```rust
view! {
    <div class="input-group">
        <div class="input-field" id="referralCodeField">
            <span class="input-icon" aria-hidden="true">
                <i class="peer-icon peer-icon-referral"></i>
            </span>
            // ADD: visually-hidden label
            <label for="referralCode" class="sr-only">"Referral code"</label>
            <input
                type="text"
                id="referralCode"
                name="referralCode"
                placeholder="Enter your referral code"
                required=true
                aria-describedby="referralValidation referralHelp"
                aria-invalid=move || if referral_error.get().is_some() { "true" } else { "false" }
                autocomplete="off"
                prop:value=referral_code
                on:input=on_referral_input
            />
            // ...
        </div>
        // ...
    </div>
}
```

**In `src/components/registration_form.rs` (Step 2 — all inputs):**

```rust
// Email
<label for="email" class="sr-only">"Email address"</label>
<input
    type="email"
    id="email"
    name="email"
    placeholder="Enter your email"
    required=true
    aria-describedby="emailValidation emailHelp"
    aria-invalid=move || if email_error.get().is_some() { "true" } else { "false" }
    autocomplete="email"
    // ...
/>

// Username
<label for="username" class="sr-only">"Username"</label>
<input
    type="text"
    id="username"
    name="username"
    placeholder="Choose a username"
    required=true
    aria-describedby="usernameValidation usernameHelp"
    aria-invalid=move || if username_error.get().is_some() { "true" } else { "false" }
    autocomplete="username"
    // ...
/>

// Password
<label for="password" class="sr-only">"Password"</label>
<input
    type=move || if password_visible.get() { "text" } else { "password" }
    id="password"
    name="password"
    placeholder="Create a strong password"
    required=true
    aria-describedby="passwordValidation passwordHelp passwordStrength"
    aria-invalid=move || if password_error.get().is_some() { "true" } else { "false" }
    autocomplete="new-password"
    // ...
/>

// Confirm Password
<label for="confirmPassword" class="sr-only">"Confirm password"</label>
<input
    type=move || if confirm_visible.get() { "text" } else { "password" }
    id="confirmPassword"
    name="confirmPassword"
    placeholder="Confirm your password"
    required=true
    aria-describedby="confirmPasswordValidation confirmPasswordHelp"
    aria-invalid=move || if confirm_error.get().is_some() { "true" } else { "false" }
    autocomplete="new-password"
    // ...
/>
```

---

## 13.5 — Implementation: `aria-invalid` on All Inputs

The current JS only sets `aria-invalid` on checkboxes. In the Leptos version, every input field must reflect its validation state via `aria-invalid`.

### 13.5.1 — Pattern for Reactive `aria-invalid`

Each input's `aria-invalid` attribute should be derived from the corresponding error signal:

```rust
// Generic pattern — apply to every input
aria-invalid=move || {
    if field_error_signal.get().is_some() {
        "true"
    } else {
        "false"
    }
}
```

This covers:
- `referral_error: RwSignal<Option<String>>`
- `email_error: RwSignal<Option<String>>`
- `username_error: RwSignal<Option<String>>`
- `password_error: RwSignal<Option<String>>`
- `confirm_error: RwSignal<Option<String>>`
- Checkbox error signals

### 13.5.2 — Checkbox `aria-invalid`

Checkboxes already have `aria-describedby="checkboxValidation"`. Add `aria-invalid` tied to the checkbox validation signal:

```rust
<input
    type="checkbox"
    id="readPrivacy"
    name="readPrivacy"
    aria-describedby="checkboxValidation"
    aria-invalid=move || {
        if checkbox_error.get().is_some() && !privacy_checked.get() {
            "true"
        } else {
            "false"
        }
    }
/>

<input
    type="checkbox"
    id="agreementEULA"
    name="agreementEULA"
    aria-describedby="checkboxValidation"
    aria-invalid=move || {
        if checkbox_error.get().is_some() && !eula_checked.get() {
            "true"
        } else {
            "false"
        }
    }
/>
```

---

## 13.6 — Implementation: Visible Focus Indicators

### 13.6.1 — Problem

The current CSS has `input { outline: none; }` which removes the browser's default focus ring. This violates **WCAG 2.4.7 (Focus Visible)** — keyboard users cannot see which element has focus.

### 13.6.2 — CSS Addition: `src/style/accessibility.css`

Create a dedicated accessibility stylesheet imported by the Leptos app. This adds custom focus indicators that match the design language while meeting WCAG requirements.

```css
/* ─── Focus Indicators (WCAG 2.4.7) ────────────────────────────────────── */

/*
 * Custom focus ring for all interactive elements.
 * Uses :focus-visible to only show on keyboard navigation,
 * not on mouse clicks (better UX).
 * 
 * The 2px outline with 2px offset ensures a minimum 3:1 contrast
 * ratio against adjacent colours (WCAG 2.4.11 Focus Appearance).
 */

.input-field:focus-within {
    outline: 2px solid var(--Hover, #1EBCFB);
    outline-offset: 2px;
    border-color: var(--Hover, #1EBCFB);
}

input:focus-visible {
    outline: none; /* Handled by .input-field:focus-within above */
}

/* Standalone inputs (checkboxes) that aren't inside .input-field */
input[type="checkbox"]:focus-visible {
    outline: 2px solid var(--Hover, #1EBCFB);
    outline-offset: 2px;
}

.btn:focus-visible {
    outline: 2px solid var(--Hover, #1EBCFB);
    outline-offset: 4px;
}

a:focus-visible {
    outline: 2px solid var(--Hover, #1EBCFB);
    outline-offset: 2px;
    border-radius: 2px;
}

/* Password toggle button focus */
.toggle-passwordBtn-icon:focus-visible {
    outline: 2px solid var(--Hover, #1EBCFB);
    outline-offset: 2px;
    border-radius: 4px;
}

/* ─── High Contrast Mode Enhancements ──────────────────────────────────── */

@media (prefers-contrast: high) {
    .input-field:focus-within {
        outline-width: 3px;
        outline-color: var(--White-primary, #FFF);
    }

    .btn:focus-visible {
        outline-width: 3px;
        outline-color: var(--White-primary, #FFF);
    }

    input[type="checkbox"]:focus-visible {
        outline-width: 3px;
        outline-color: var(--White-primary, #FFF);
    }

    /* Ensure validation messages have sufficient contrast */
    .validation-message {
        font-weight: 600;
    }
}
```

### 13.6.3 — Import in `app.rs`

```rust
<Stylesheet id="a11y" href="/pkg/accessibility.css"/>
```

Or if using cargo-leptos style bundling, import in the main `style/` directory and ensure it's included in the build.

---

## 13.7 — Implementation: Password Visibility Toggle as `<button>`

### 13.7.1 — Problem

The current PHP uses `<span class="toggle-passwordBtn-icon">` for the password visibility toggle. `<span>` elements are not keyboard-focusable and have no implicit button role —  keyboard-only users cannot toggle password visibility.

### 13.7.2 — Leptos Implementation

Replace the `<span>` with a `<button>` element. The button must:
1. Have `type="button"` (prevent form submission)
2. Have a dynamic `aria-label` reflecting the current state
3. Have `aria-pressed` to indicate toggle state
4. Not have a visible text label (icon-only) — the `aria-label` provides the accessible name

```rust
/// Password visibility toggle button.
///
/// Renders as a `<button>` (not `<span>`) for keyboard accessibility.
/// Updates `aria-label` and `aria-pressed` reactively.
#[component]
fn PasswordToggle(
    /// Which field this toggle controls: "password" or "confirmPassword"
    #[prop(into)]
    field_id: String,
    /// Signal controlling visibility state
    visible: RwSignal<bool>,
) -> impl IntoView {
    let toggle = move |_| visible.update(|v| *v = !*v);

    view! {
        <button
            type="button"
            class="toggle-passwordBtn-icon"
            on:click=toggle
            aria-label=move || {
                if visible.get() {
                    "Hide password"
                } else {
                    "Show password"
                }
            }
            aria-pressed=move || if visible.get() { "true" } else { "false" }
            aria-controls=field_id.clone()
        >
            <i class=move || {
                if visible.get() {
                    "peer-icon peer-icon-eye-open"
                } else {
                    "peer-icon peer-icon-eye-close"
                }
            } aria-hidden="true" />
        </button>
    }
}
```

### 13.7.3 — CSS Adjustment

Since the element changes from `<span>` to `<button>`, add a reset to maintain visual parity:

```css
/* In accessibility.css or login-register.css override */
button.toggle-passwordBtn-icon {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0 12px;
    color: var(--White-secondary);
    font-size: inherit;
    line-height: inherit;
    display: flex;
    align-items: center;
}
```

---

## 13.8 — Implementation: Step Announcer (Live Region)

### 13.8.1 — Create `src/components/step_announcer.rs`

The step announcer is an `aria-live="polite"` region that notifies screen reader users when the current step changes or when important status messages occur (e.g. "Referral code verified, proceeding to registration").

The existing JS creates this dynamically via `getOrCreateAnnouncer()`. In Leptos, it should be a static component in the DOM from the start (better for SSR hydration).

```rust
//! Screen reader step announcer.
//!
//! Provides a visually-hidden aria-live region that announces
//! step transitions and status messages to assistive technology.

use leptos::prelude::*;

/// Reactive context for announcing messages to screen readers.
#[derive(Clone, Copy)]
pub struct StepAnnouncerContext {
    message: RwSignal<String>,
}

impl StepAnnouncerContext {
    /// Announce a message to screen readers.
    ///
    /// The message replaces the current content of the aria-live region.
    /// Screen readers will read the new content based on the politeness level.
    pub fn announce(&self, msg: impl Into<String>) {
        self.message.set(msg.into());
    }
}

/// Step titles for automatic announcement on step change.
const STEP_TITLES: &[&str] = &[
    "",                        // 0 — unused
    "Referral Code Entry",     // 1
    "Registration Form",       // 2
    "Registration Complete",   // 3
];

/// Visually-hidden aria-live region for step announcements.
///
/// Place this once inside the `RegisterPage` component.
/// Child components can announce messages via `use_context::<StepAnnouncerContext>()`.
#[component]
pub fn StepAnnouncer(
    /// The current step number (1-indexed). Changes trigger automatic announcement.
    current_step: Signal<u8>,
) -> impl IntoView {
    let message = RwSignal::new(String::new());
    let ctx = StepAnnouncerContext { message };
    provide_context(ctx);

    // Auto-announce when step changes
    Effect::new(move |_| {
        let step = current_step.get() as usize;
        if step > 0 && step < STEP_TITLES.len() {
            message.set(format!("Now on step {}: {}", step, STEP_TITLES[step]));
        }
    });

    view! {
        <div
            id="step-announcer"
            class="sr-only"
            aria-live="polite"
            aria-atomic="true"
            role="status"
        >
            {move || message.get()}
        </div>
    }
}
```

### 13.8.2 — Export from `src/components/mod.rs`

```rust
pub mod step_announcer;
```

### 13.8.3 — Usage in `RegisterPage`

```rust
use crate::components::step_announcer::{StepAnnouncer, StepAnnouncerContext};

#[component]
pub fn RegisterPage() -> impl IntoView {
    let current_step = RwSignal::new(1u8);

    view! {
        <ToastProvider>
            <StepAnnouncer current_step=current_step.into() />
            <div class="container large_font">
                // ... steps ...
            </div>
        </ToastProvider>
    }
}
```

### 13.8.4 — Custom Announcements from Child Components

```rust
// In referral verification handler (Step 7):
let announcer = use_context::<StepAnnouncerContext>().expect("StepAnnouncer missing");

// On referral verified:
announcer.announce("Referral code verified, proceeding to registration");

// On form validation failure (Step 9):
announcer.announce("Please correct the errors in the form");

// On registration success:
announcer.announce("Registration successful! Welcome to peer!");
```

---

## 13.9 — Implementation: Focus Management on Step Transitions

### 13.9.1 — Problem

When the registration flow transitions between steps, keyboard focus must move to the first interactive element in the new step. Without this, a keyboard user's focus remains on a now-hidden element.

### 13.9.2 — Leptos Implementation

Create a utility that focuses the first focusable element within a step container after the step becomes visible. This mirrors the existing JS `showStep()` behaviour.

**In `src/utils/focus.rs`:**

```rust
//! Focus management utilities for multi-step forms.

use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

/// Focus the first interactive element inside the given container element.
///
/// Searches for `input`, `button`, `select`, `textarea`, and `a[href]`
/// elements and focuses the first one found.
///
/// Call this after a step transition with a short delay to allow the DOM
/// to update.
pub fn focus_first_interactive(container_id: &str) {
    let container_id = container_id.to_string();
    // Use request_animation_frame to defer until after DOM update
    request_animation_frame(move || {
        if let Some(document) = document() {
            if let Some(container) = document.get_element_by_id(&container_id) {
                let selectors = "input:not([type='hidden']), button, select, textarea, a[href]";
                if let Ok(Some(first)) = container.query_selector(selectors) {
                    if let Ok(el) = first.dyn_into::<web_sys::HtmlElement>() {
                        let _ = el.focus();
                    }
                }
            }
        }
    });
}

/// Focus the first invalid input inside the given container.
///
/// Looks for `.input-field.invalid input` or `input[aria-invalid="true"]`.
/// Used when form submission fails validation.
pub fn focus_first_error(container_id: &str) {
    let container_id = container_id.to_string();
    request_animation_frame(move || {
        if let Some(document) = document() {
            if let Some(container) = document.get_element_by_id(&container_id) {
                let selectors = ".input-field.invalid input, input[aria-invalid='true']";
                if let Ok(Some(first)) = container.query_selector(selectors) {
                    if let Ok(el) = first.dyn_into::<web_sys::HtmlElement>() {
                        let _ = el.focus();
                    }
                }
            }
        }
    });
}

/// Helper: get the browser document.
fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}
```

### 13.9.3 — Wire into Step Navigation (Step 11)

In the step transition logic (from Step 11), call `focus_first_interactive` after changing `current_step`:

```rust
use crate::utils::focus::{focus_first_interactive, focus_first_error};

// When advancing to step 2:
current_step.set(2);
focus_first_interactive("registrationStep");

// When going back to step 1:
current_step.set(1);
focus_first_interactive("referralStep");

// When advancing to step 3 (success):
current_step.set(3);
focus_first_interactive("successStep");

// On form validation failure:
focus_first_error("registrationStep");
```

### 13.9.4 — Export from `src/utils/mod.rs`

```rust
pub mod focus;
pub mod response_codes;
```

---

## 13.10 — Implementation: Password Strength Meter Accessibility

### 13.10.1 — Problem

The password strength meter is purely visual — the coloured segments and strength labels are not communicated to screen readers. WCAG 4.1.3 (Status Messages) requires that status information be programmatically determinable.

### 13.10.2 — Approach

1. Add an `aria-live="polite"` region that announces the current strength level as text
2. Add `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, and `aria-valuetext` on the meter container (treating it as a custom progress indicator)
3. Mark the individual segment `<span>` elements as `aria-hidden="true"` (decorative)

### 13.10.3 — Leptos Implementation

```rust
/// Password strength levels with their numeric value and text description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    NeedsImprovement,
    Good,
    Excellent,
}

impl PasswordStrength {
    /// CSS class for the strength-fill element.
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::VeryWeak => "weak",
            Self::Weak => "weak2",
            Self::NeedsImprovement => "medium",
            Self::Good => "strong",
            Self::Excellent => "excellent",
        }
    }

    /// Numeric value (1–5) for aria-valuenow.
    pub fn numeric(&self) -> u8 {
        match self {
            Self::VeryWeak => 1,
            Self::Weak => 2,
            Self::NeedsImprovement => 3,
            Self::Good => 4,
            Self::Excellent => 5,
        }
    }

    /// Human-readable label for aria-valuetext and screen reader announcement.
    pub fn label(&self) -> &'static str {
        match self {
            Self::VeryWeak => "Very weak",
            Self::Weak => "Weak",
            Self::NeedsImprovement => "Needs improvement",
            Self::Good => "Good",
            Self::Excellent => "Excellent",
        }
    }
}

/// Accessible password strength meter component.
#[component]
pub fn PasswordStrengthMeter(
    /// The current password strength level.
    strength: Signal<PasswordStrength>,
    /// Whether the password field has any input.
    has_input: Signal<bool>,
    /// Unmet password requirements (for the requirements list).
    requirements: Signal<Vec<(String, bool)>>,
) -> impl IntoView {
    view! {
        <div
            class="password-strength"
            class:none=move || !has_input.get()
            class:show=move || has_input.get()
            id="passwordStrength"
        >
            // Visible strength label
            <div class="strength-labels medium_font">
                {move || {
                    let s = strength.get();
                    view! {
                        <span class=format!("strength-text {}", s.label().to_lowercase().replace(' ', "-"))>
                            {s.label()}
                        </span>
                    }
                }}
            </div>

            // Meter bar with ARIA attributes
            <div
                class="strength-meter"
                role="meter"
                aria-label="Password strength"
                aria-valuenow=move || strength.get().numeric().to_string()
                aria-valuemin="1"
                aria-valuemax="5"
                aria-valuetext=move || {
                    format!("Password strength: {}", strength.get().label())
                }
            >
                <div class=move || {
                    format!("strength-fill {}", strength.get().css_class())
                }>
                    <span class="strength-segment segment-weak" aria-hidden="true"></span>
                    <span class="strength-segment segment-weak2" aria-hidden="true"></span>
                    <span class="strength-segment segment-medium" aria-hidden="true"></span>
                    <span class="strength-segment segment-strong" aria-hidden="true"></span>
                    <span class="strength-segment segment-excellent" aria-hidden="true"></span>
                </div>
            </div>

            // Screen reader announcement for strength changes
            <div class="sr-only" aria-live="polite" aria-atomic="true">
                {move || {
                    if has_input.get() {
                        format!("Password strength: {}", strength.get().label())
                    } else {
                        String::new()
                    }
                }}
            </div>

            // Requirements list
            <ul class="strength-requirements medium_font" role="list" aria-label="Password requirements">
                <For
                    each=move || requirements.get()
                    key=|(label, _)| label.clone()
                    children=move |(label, met)| {
                        view! {
                            <li
                                class:met=met
                                class:none=met
                                aria-label=format!(
                                    "{}: {}",
                                    label,
                                    if met { "met" } else { "not met" }
                                )
                            >
                                {label}
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}
```

---

## 13.11 — Implementation: `aria-busy` on Submit Buttons

### 13.11.1 — Pattern

When an async action is pending (referral verification, registration), the submit button must:
1. Be `disabled` (prevent double-submission)
2. Have `aria-busy="true"` (tell screen readers the action is in progress)
3. Show the `.loading` CSS class (visual spinner)

```rust
/// Accessible submit button with loading state.
#[component]
pub fn SubmitButton(
    /// Button text content.
    #[prop(into)]
    label: String,
    /// Whether the button is in a loading/pending state.
    pending: Signal<bool>,
) -> impl IntoView {
    view! {
        <button
            type="submit"
            class="btn btn-primary"
            class:loading=move || pending.get()
            disabled=move || pending.get()
            aria-busy=move || if pending.get() { "true" } else { "false" }
        >
            {label}
        </button>
    }
}
```

Apply to:
- `#verifyReferralBtn` — pending when `verify_referral` action is in-flight
- `#registerBtn` — pending when `register_user` action is in-flight

---

## 13.12 — Implementation: Language Attribute Fix

### 13.12.1 — Problem

The current PHP version has `<html lang="de">` but all UI text is in English. This causes screen readers to use German pronunciation rules for English text.

### 13.12.2 — Fix in `src/app.rs`

The Leptos `shell()` function already has `<html lang="en">` — verify this is correct and has not been changed:

```rust
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            // ...
        </html>
    }
}
```

No code change needed in Leptos — just verify. This is a **fix** relative to the PHP version.

---

## 13.13 — Implementation: Skip Navigation Link

### 13.13.1 — Problem

The registration page has a back button and potentially other header elements before the main form content. WCAG 2.4.1 (Bypass Blocks) requires a mechanism to skip repeated content.

### 13.13.2 — Implementation

Add a visually-hidden skip link as the first focusable element on the page, which jumps focus to the main form content.

**In `src/pages/register.rs`:**

```rust
view! {
    <ToastProvider>
        <StepAnnouncer current_step=current_step.into() />

        // Skip navigation link — first focusable element
        <a href="#main-form" class="skip-link sr-only"
           // Becomes visible on :focus for keyboard users
           style:position="absolute"
        >
            "Skip to registration form"
        </a>

        <div class="container large_font">
            <div class="container_left">
                // ... phone mockup ...
            </div>
            <div class="container_right">
                <div class="container_inner">
                    <div class="top_head_area">
                        // ... back button ...
                    </div>

                    // Target for skip link
                    <div class="center_area" id="main-form" tabindex="-1">
                        // ... form steps ...
                    </div>
                </div>
            </div>
        </div>
    </ToastProvider>
}
```

### 13.13.3 — CSS for Skip Link

```css
/* In accessibility.css */
.skip-link {
    /* Visually hidden by default (inherits sr-only) */
    /* Becomes visible and moves on-screen when focused */
}

.skip-link:focus {
    position: fixed !important;
    top: 10px;
    left: 10px;
    width: auto !important;
    height: auto !important;
    padding: 12px 24px;
    margin: 0 !important;
    overflow: visible !important;
    clip: auto !important;
    white-space: normal !important;
    background: var(--Blue, #0069FF);
    color: var(--White-primary, #FFF);
    font-size: 1rem;
    font-weight: 700;
    border-radius: 8px;
    z-index: 10000;
    text-decoration: none;
}
```

---

## 13.14 — Implementation: Keyboard Navigation Audit

### 13.14.1 — Expected Tab Order

The tab order must follow the visual reading order within each step. Hidden steps must not receive focus.

**Step 1 (Referral):**

| Tab # | Element | Notes |
|-------|---------|-------|
| 1 | Skip link (only visible on focus) | Jumps to `#main-form` |
| 2 | Back button | `<a href="login.php">` |
| 3 | Referral code input | Auto-focused on step enter |
| 4 | "Verify Code" button | Submit |
| 5 | "Click here" link (default code) | In step footer |

**Step 1b (Default Referral):**

| Tab # | Element | Notes |
|-------|---------|-------|
| 1 | Skip link | |
| 2 | Back button | Goes back to step 1 |
| 3 | "Use This Code" button | Auto-focused on step enter |

**Step 2 (Registration Form):**

| Tab # | Element | Notes |
|-------|---------|-------|
| 1 | Skip link | |
| 2 | Back button | Goes back to step 1 |
| 3 | Email input | Auto-focused on step enter |
| 4 | Username input | |
| 5 | Password input | |
| 6 | Password toggle button | `<button>` now |
| 7 | Confirm password input | |
| 8 | Confirm password toggle button | |
| 9 | Privacy policy checkbox | |
| 10 | Privacy policy link ("Privacy Policy") | Opens in new tab |
| 11 | EULA checkbox | |
| 12 | EULA link | Opens in new tab |
| 13 | "Create Account" button | Submit |
| 14 | "Login here" link | In footer text |

**Step 3 (Success):**

| Tab # | Element | Notes |
|-------|---------|-------|
| 1 | Skip link | |
| 2 | "Continue to Login" link | Auto-focused on step enter |

### 13.14.2 — Ensuring Hidden Steps Don't Receive Focus

Steps that are not `.active` must not contain focusable elements. The existing CSS hides inactive steps with `display: none` (via removing the `.active` class). Verify that the Leptos implementation uses conditional rendering or `class:active` that results in `display: none` for inactive steps.

If using Leptos `<Show>` components:

```rust
// Option A: <Show> — elements not in DOM when hidden (no focus trap risk)
<Show when=move || current_step.get() == 1>
    <div class="form-step active" id="referralStep" data-step="1">
        // ...
    </div>
</Show>
```

If using CSS class toggling (matching the PHP approach):

```rust
// Option B: CSS class toggle — ensure .form-step:not(.active) has display: none
<div
    class="form-step"
    class:active=move || current_step.get() == 1
    id="referralStep"
    data-step="1"
    // Add inert attribute when not active to prevent focus
    attr:inert=move || if current_step.get() != 1 { Some("") } else { None }
>
    // ...
</div>
```

The `inert` HTML attribute (supported in all modern browsers) is the most robust way to prevent focus inside hidden steps when using CSS visibility toggling. It makes the entire subtree non-interactive and invisible to assistive technology.

### 13.14.3 — Enter Key Behaviour

- **Inside referral form:** Enter submits the referral form → triggers "Verify Code"
- **Inside registration form:** Enter submits the registration form → triggers "Create Account"
- **On password toggle button:** Enter (or Space) toggles visibility

These are handled natively by `<form>` + `<button type="submit">` and `<button type="button">`. No extra key handlers needed.

### 13.14.4 — Escape Key Behaviour

If any modal/overlay is open (e.g. future additions), Escape should close it. For the current registration flow, no modals exist — Escape is a no-op.

---

## 13.15 — Implementation: External Link Accessibility

### 13.15.1 — Problem

The "Privacy Policy" and "EULA" links open in a new tab (`target="_blank"`). Users should be warned that a new window will open (WCAG 3.2.5 guidance).

### 13.15.2 — Implementation

Add visually-hidden text and `rel="noopener noreferrer"` to external links:

```rust
<a href="https://peerapp.de/privacy.html" target="_blank" rel="noopener noreferrer">
    "Privacy Policy"
    <span class="sr-only">" (opens in new tab)"</span>
</a>

<a href="https://peerapp.de/EULA.html" target="_blank" rel="noopener noreferrer">
    "End User License Agreement (EULA)"
    <span class="sr-only">" (opens in new tab)"</span>
</a>
```

---

## 13.16 — Implementation: Error Summary Pattern

### 13.16.1 — Approach

When the registration form is submitted with multiple errors, in addition to inline field errors and the step announcer message, display an error summary at the top of the form that links to each invalid field. This is a WCAG best practice for complex forms.

### 13.16.2 — Implementation

```rust
/// Error summary component.
///
/// Rendered at the top of the registration form when submission fails.
/// Lists all current errors as anchor links to the corresponding fields.
#[component]
pub fn ErrorSummary(
    errors: Signal<Vec<(String, String)>>,  // Vec of (field_id, error_message)
) -> impl IntoView {
    view! {
        <Show when=move || !errors.get().is_empty()>
            <div
                class="error-warning"
                role="alert"
                aria-label="Form errors"
                tabindex="-1"
                id="error-summary"
            >
                <p>"Please correct the following errors:"</p>
                <ul>
                    <For
                        each=move || errors.get()
                        key=|(id, _)| id.clone()
                        children=move |(field_id, msg)| {
                            view! {
                                <li>
                                    <a href=format!("#{}", field_id)>
                                        {msg}
                                    </a>
                                </li>
                            }
                        }
                    />
                </ul>
            </div>
        </Show>
    }
}
```

### 13.16.3 — Trigger on Failed Submission

In the registration form submit handler:

```rust
// Collect all current errors
let mut all_errors = Vec::new();
if let Some(msg) = email_error.get() {
    all_errors.push(("email".to_string(), msg));
}
if let Some(msg) = username_error.get() {
    all_errors.push(("username".to_string(), msg));
}
if let Some(msg) = password_error.get() {
    all_errors.push(("password".to_string(), msg));
}
if let Some(msg) = confirm_error.get() {
    all_errors.push(("confirmPassword".to_string(), msg));
}
if let Some(msg) = checkbox_error.get() {
    all_errors.push(("readPrivacy".to_string(), msg));
}

form_errors.set(all_errors);

// Focus the error summary
focus_element("error-summary");
```

---

## 13.17 — Automated Testing: axe-core Integration

### 13.17.1 — Install Dependencies

```bash
cd peer-web
npm init -y  # if no package.json exists
npm install -D @axe-core/playwright playwright
npx playwright install chromium
```

### 13.17.2 — Create `tests/a11y/axe-registration.spec.ts`

```typescript
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const BASE_URL = 'http://localhost:3000';

test.describe('Registration page accessibility', () => {
    test('Step 1 (Referral) has no critical or serious axe violations', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);
        await page.waitForSelector('#referralStep.active');

        const results = await new AxeBuilder({ page })
            .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
            .analyze();

        // Log violations for debugging
        if (results.violations.length > 0) {
            console.log('Violations:', JSON.stringify(results.violations, null, 2));
        }

        const criticalOrSerious = results.violations.filter(
            v => v.impact === 'critical' || v.impact === 'serious'
        );

        expect(criticalOrSerious).toHaveLength(0);
    });

    test('Step 2 (Registration Form) has no critical or serious axe violations', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Navigate to step 2 by submitting a valid referral
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        const results = await new AxeBuilder({ page })
            .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
            .analyze();

        const criticalOrSerious = results.violations.filter(
            v => v.impact === 'critical' || v.impact === 'serious'
        );

        expect(criticalOrSerious).toHaveLength(0);
    });

    test('Step 2 with validation errors has no critical or serious axe violations', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Navigate to step 2
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        // Submit empty form to trigger all validation errors
        await page.click('#registerBtn');
        await page.waitForTimeout(200); // Wait for validation messages

        const results = await new AxeBuilder({ page })
            .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
            .analyze();

        const criticalOrSerious = results.violations.filter(
            v => v.impact === 'critical' || v.impact === 'serious'
        );

        expect(criticalOrSerious).toHaveLength(0);
    });

    test('Step 3 (Success) has no critical or serious axe violations', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Complete full registration flow against mock backend
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        await page.fill('#email', 'test@example.com');
        await page.fill('#username', 'testuser');
        await page.fill('#password', 'StrongPass1!');
        await page.fill('#confirmPassword', 'StrongPass1!');
        await page.check('#readPrivacy');
        await page.check('#agreementEULA');
        await page.click('#registerBtn');
        await page.waitForSelector('#successStep.active');

        const results = await new AxeBuilder({ page })
            .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
            .analyze();

        const criticalOrSerious = results.violations.filter(
            v => v.impact === 'critical' || v.impact === 'serious'
        );

        expect(criticalOrSerious).toHaveLength(0);
    });
});
```

### 13.17.3 — Create `tests/a11y/playwright.config.ts`

```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: '.',
    testMatch: '**/*.spec.ts',
    timeout: 30000,
    use: {
        baseURL: 'http://localhost:3000',
        headless: true,
    },
    webServer: {
        // Assumes cargo leptos watch is already running
        // Or start it here:
        // command: 'cargo leptos watch',
        // cwd: '..',
        url: 'http://localhost:3000',
        reuseExistingServer: true,
    },
});
```

### 13.17.4 — Run

```bash
cd peer-web
npx playwright test tests/a11y/
```

---

## 13.18 — Automated Testing: Keyboard Navigation

### 13.18.1 — Create `tests/a11y/keyboard-navigation.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

const BASE_URL = 'http://localhost:3000';

test.describe('Keyboard navigation', () => {
    test('Tab order on Step 1 follows expected sequence', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);
        await page.waitForSelector('#referralStep.active');

        // Tab from the start of the page
        await page.keyboard.press('Tab'); // Skip link
        await page.keyboard.press('Tab'); // Back button
        const backBtn = page.locator('#backBtn');
        await expect(backBtn).toBeFocused();

        await page.keyboard.press('Tab'); // Referral input
        const referralInput = page.locator('#referralCode');
        await expect(referralInput).toBeFocused();

        await page.keyboard.press('Tab'); // Verify button
        const verifyBtn = page.locator('#verifyReferralBtn');
        await expect(verifyBtn).toBeFocused();

        await page.keyboard.press('Tab'); // "Click here" link
        const defaultCodeLink = page.locator('#useDefaultCodeBtn');
        await expect(defaultCodeLink).toBeFocused();
    });

    test('Tab order on Step 2 follows expected sequence', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Navigate to step 2
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        // Focus should be on email input (auto-focused)
        const emailInput = page.locator('#email');
        await expect(emailInput).toBeFocused();

        await page.keyboard.press('Tab'); // Username
        await expect(page.locator('#username')).toBeFocused();

        await page.keyboard.press('Tab'); // Password
        await expect(page.locator('#password')).toBeFocused();

        await page.keyboard.press('Tab'); // Password toggle
        await expect(page.locator('.toggle-passwordBtn-icon').first()).toBeFocused();

        await page.keyboard.press('Tab'); // Confirm password
        await expect(page.locator('#confirmPassword')).toBeFocused();

        await page.keyboard.press('Tab'); // Confirm toggle
        await expect(page.locator('.toggle-passwordBtn-icon').nth(1)).toBeFocused();

        await page.keyboard.press('Tab'); // Privacy checkbox
        await expect(page.locator('#readPrivacy')).toBeFocused();

        await page.keyboard.press('Tab'); // Privacy link
        await page.keyboard.press('Tab'); // EULA checkbox
        await expect(page.locator('#agreementEULA')).toBeFocused();

        await page.keyboard.press('Tab'); // EULA link
        await page.keyboard.press('Tab'); // Create Account button
        await expect(page.locator('#registerBtn')).toBeFocused();
    });

    test('Enter submits referral form', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');

        // Press Enter while referral input is focused
        await page.locator('#referralCode').press('Enter');

        // Should transition to step 2
        await page.waitForSelector('#registrationStep.active');
    });

    test('Enter submits registration form', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Navigate to step 2
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        // Fill form
        await page.fill('#email', 'test@example.com');
        await page.fill('#username', 'testuser');
        await page.fill('#password', 'StrongPass1!');
        await page.fill('#confirmPassword', 'StrongPass1!');
        await page.check('#readPrivacy');
        await page.check('#agreementEULA');

        // Press Enter on last input
        await page.locator('#registerBtn').press('Enter');

        // Should transition to step 3
        await page.waitForSelector('#successStep.active');
    });

    test('Space and Enter toggle password visibility', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Navigate to step 2
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        // Fill password
        await page.fill('#password', 'TestPass1');

        // Tab to toggle button and press Space
        const toggleBtn = page.locator('.toggle-passwordBtn-icon').first();
        await toggleBtn.focus();
        await toggleBtn.press('Space');

        // Password input should now be type="text"
        await expect(page.locator('#password')).toHaveAttribute('type', 'text');

        // Press Enter to toggle back
        await toggleBtn.press('Enter');
        await expect(page.locator('#password')).toHaveAttribute('type', 'password');
    });

    test('Hidden steps do not receive focus via Tab', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);
        await page.waitForSelector('#referralStep.active');

        // Tab through all elements — count that we never land on step 2/3 elements
        let hitStep2Element = false;
        for (let i = 0; i < 20; i++) {
            await page.keyboard.press('Tab');
            const focused = await page.evaluate(() => document.activeElement?.id);
            if (['email', 'username', 'password', 'confirmPassword', 'registerBtn'].includes(focused || '')) {
                hitStep2Element = true;
                break;
            }
        }

        expect(hitStep2Element).toBe(false);
    });

    test('Focus moves to first input on step transition', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Navigate to step 2
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        // Email should be focused
        await expect(page.locator('#email')).toBeFocused();
    });

    test('Focus moves to first error on failed submission', async ({ page }) => {
        await page.goto(`${BASE_URL}/register`);

        // Navigate to step 2
        await page.fill('#referralCode', '85d5f836-b1f5-4c4e-9381-1b058e13df93');
        await page.click('#verifyReferralBtn');
        await page.waitForSelector('#registrationStep.active');

        // Submit empty form
        await page.click('#registerBtn');
        await page.waitForTimeout(200);

        // First invalid field (email) should be focused
        await expect(page.locator('#email')).toBeFocused();
    });
});
```

---

## 13.19 — Manual Testing: VoiceOver Walkthrough

This section is a checklist for manual testing with macOS VoiceOver (⌘+F5). Each item must be verified by a human tester.

### 13.19.1 — Step 1 (Referral Code)

| # | Action | Expected VoiceOver Announcement |
|---|--------|-------------------------------|
| 1 | Navigate to `/register` | Announces page title: "Peer Network - Register" |
| 2 | VO reads the heading | "Welcome to peer!, heading level 2" |
| 3 | Tab to referral input | "Referral code, edit text. Enter your referral code. Enter the referral code provided to you" (label + placeholder + help text via aria-describedby) |
| 4 | Type invalid code "abc" | On blur/input: "Hmm… that referral code doesn't seem to work." (via aria-live polite region) |
| 5 | Type valid UUID | Validation message clears; no error announced |
| 6 | Tab to "Verify Code" button | "Verify Code, button" |
| 7 | Press Enter on button | "Now on step 2: Registration Form" (from step announcer) |
| 8 | Verification in progress | "Verify Code, button, busy" (via aria-busy) |

### 13.19.2 — Step 2 (Registration Form)

| # | Action | Expected VoiceOver Announcement |
|---|--------|-------------------------------|
| 1 | Step loads | Step announcer: "Now on step 2: Registration Form" |
| 2 | Focus on email input | "Email address, edit text. Enter your email. Enter a valid email address" |
| 3 | Enter invalid email "a@" | "Please enter a valid email address" (validation message) |
| 4 | Tab to username | "Username, edit text. Choose a username. Username must be 5-20 characters, starting with a letter" |
| 5 | Tab to password | "Password, secure edit text. Create a strong password. Password must meet all security requirements" |
| 6 | Type password → strength changes | "Password strength: Weak" → "Password strength: Good" (via aria-live region) |
| 7 | Tab to password toggle | "Show password, toggle button, not pressed" |
| 8 | Press Space on toggle | "Hide password, toggle button, pressed" |
| 9 | Tab to confirm password | "Confirm password, secure edit text" |
| 10 | Type mismatching password | "Passwords do not match" |
| 11 | Tab to Privacy checkbox | "I agree to the Privacy Policy (opens in new tab), unchecked, checkbox" |
| 12 | Tab to EULA checkbox | "I agree to the End User License Agreement (EULA) (opens in new tab), unchecked, checkbox" |
| 13 | Submit with errors | Step announcer: "Please correct the errors in the form"; Focus moves to first error field |
| 14 | Submit valid form | Step announcer: "Registration successful! Welcome to peer!" |

### 13.19.3 — Step 3 (Success)

| # | Action | Expected VoiceOver Announcement |
|---|--------|-------------------------------|
| 1 | Step loads | Step announcer: "Now on step 3: Registration Complete" |
| 2 | VO reads heading | "Welcome to peer!, heading level 2" |
| 3 | Tab to login link | "Continue to Login, link" |

### 13.19.4 — Toast Notifications

| # | Action | Expected VoiceOver Announcement |
|---|--------|-------------------------------|
| 1 | Success toast appears | Immediately reads: "Registration successful! Please check your email to verify your account." (via role="alert" + aria-live="assertive") |
| 2 | Error toast appears | Immediately reads the error message |
| 3 | Toast auto-dismisses | No announcement on dismiss |

---

## 13.20 — Colour Contrast Verification

### 13.20.1 — Minimum Requirements

WCAG 2.1 AA requires:
- **Normal text:** 4.5:1 contrast ratio
- **Large text (≥ 18pt or ≥ 14pt bold):** 3:1 contrast ratio
- **UI components and graphical objects:** 3:1 contrast ratio

### 13.20.2 — Contrast Audit Table

| Element | Foreground | Background | Ratio | Passes AA? | Notes |
|---------|-----------|------------|-------|------------|-------|
| Body text | `#FFF` | `#323232` | 10.9:1 | Yes | |
| Placeholder text | `rgba(255,255,255,0.50)` | `hsla(0,0%,19%,1)` | ~3.3:1 | No (for normal text) | Placeholders should not be sole labels — fixed by adding `<label>` elements |
| Validation error text | `#FF3B3B` | `#323232` | 4.3:1 | Borderline | Large text (clamp min 1.5rem = 24px) → 3:1 threshold → **passes** |
| Validation success text | `#AAFF67` | `#323232` | 8.6:1 | Yes | |
| Button text (primary) | `rgba(255,255,255,0.50)` | `#0069FF` + overlay | ~2.7:1 | No | **Fix needed:** change `.btn-primary` color to `#FFF` |
| Button text (hover) | `#1EBCFB` | transparent | N/A | N/A | Inherits container bg |
| Toast success | `#000` on `#AAFF67` | — | 12.5:1 | Yes | |
| Toast error | `#FFF` on `#FF3B3B` | — | 4.6:1 | Yes | |
| Link text | `#FFF` | `#323232` | 10.9:1 | Yes | |
| Strength meter "Very weak" | `#FF3B3B` | `rgba(255,255,255,0.50)` | N/A | N/A | Decorative; backed by text label |

### 13.20.3 — Contrast Fix: Primary Button Text

The `.btn-primary` has `color: var(--White-secondary)` which is `rgba(255, 255, 255, 0.50)` — semi-transparent white on a blue background. This fails the 4.5:1 contrast requirement for normal text.

**Fix in `accessibility.css`:**

```css
/* Fix: primary button text contrast (WCAG 1.4.3) */
.btn-primary {
    color: var(--White-primary, #FFF);
}

.btn-primary:disabled {
    color: var(--White-secondary); /* Keep dimmed look for disabled state */
    /* Disabled elements are exempt from WCAG contrast requirements */
}
```

---

## 13.21 — File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `src/style/accessibility.css` | **Create** | Focus indicators, skip link styles, button contrast fix |
| `src/components/step_announcer.rs` | **Create** | `StepAnnouncer` component + `StepAnnouncerContext` |
| `src/components/mod.rs` | **Edit** | Add `pub mod step_announcer;` |
| `src/utils/focus.rs` | **Create** | `focus_first_interactive()`, `focus_first_error()` |
| `src/utils/mod.rs` | **Edit** | Add `pub mod focus;` |
| `src/components/referral_step.rs` | **Edit** | Add `<label>`, `aria-invalid`, improve `aria-describedby` |
| `src/components/registration_form.rs` | **Edit** | Add `<label>` elements, `aria-invalid` on all inputs, `<button>` toggle, error summary, `aria-busy` on submit button |
| `src/components/password_strength.rs` | **Edit** | Add `role="meter"`, `aria-valuenow/text`, `aria-live` for announcements, `aria-hidden` on segments |
| `src/pages/register.rs` | **Edit** | Add skip link, `StepAnnouncer`, `inert` on inactive steps, wire focus management |
| `src/app.rs` | **Edit** | Import `accessibility.css`, verify `lang="en"` |
| `tests/a11y/axe-registration.spec.ts` | **Create** | axe-core automated audit (4 tests) |
| `tests/a11y/keyboard-navigation.spec.ts` | **Create** | Keyboard navigation tests (8 tests) |
| `tests/a11y/playwright.config.ts` | **Create** | Playwright config for a11y tests |

---

## 13.22 — SSR Considerations

| Concern | Approach |
|---------|----------|
| `aria-invalid` initial state | SSR renders `aria-invalid="false"` on all inputs — correct, no errors on initial load |
| Step announcer | SSR renders the `div#step-announcer` with empty text — correct, no announcement on page load |
| `inert` attribute | SSR renders `inert` on inactive steps — correct, prevents flash of focusable content before hydration |
| Skip link | SSR renders the skip link in the DOM — correct, immediately available |
| Focus management (`focus_first_interactive`) | Uses `request_animation_frame` which is client-only — no-op on server |
| `<label>` elements | SSR renders labels in the DOM — screen readers get them immediately without waiting for hydration |
| Password toggle `<button>` | SSR renders as `<button>` with `aria-label` — accessible before JS hydrates |

---

## 13.23 — Edge Cases

| Edge Case | Handling |
|-----------|----------|
| JS not loaded / hydration pending | All ARIA attributes are in the SSR HTML; `inert` prevents interaction with hidden steps; form has `novalidate` so browser validation doesn't conflict |
| Very fast step transitions | `focus_first_interactive` uses `request_animation_frame` to defer until DOM is updated; step announcer always reflects latest `current_step` |
| Multiple validation errors simultaneously | Error summary lists all errors; `aria-invalid` is set independently on each field; step announcer gives a generic "correct the errors" message |
| Password manager auto-fill | `autocomplete` attributes are correct (`email`, `username`, `new-password`); labels exist for password manager field detection |
| iOS VoiceOver (mobile Safari) | `role="alert"` and `aria-live` work on iOS; `inert` attribute supported since iOS 15.4; skip link works on iOS with "Links" rotor |
| Browser zoom to 200% | Layout uses `clamp()` and `vw` units; tested at 1440px and 375px — should scale gracefully. Verify no content clipping at 200% zoom |

---

## 13.24 — WCAG 2.1 AA Compliance Checklist

Cross-reference of all relevant WCAG success criteria and their status after this step:

| SC | Name | Level | Status | Implementation |
|----|------|-------|--------|----------------|
| 1.1.1 | Non-text Content | A | Pass | All icons have `aria-hidden="true"`; images have `alt` text |
| 1.3.1 | Info and Relationships | A | **Fixed** | Added `<label>` elements for all inputs (was missing) |
| 1.3.5 | Identify Input Purpose | AA | Pass | `autocomplete` attributes on email, username, password fields |
| 1.4.3 | Contrast (Minimum) | AA | **Fixed** | Button text contrast corrected |
| 1.4.11 | Non-text Contrast | AA | Pass | Focus indicators meet 3:1 ratio |
| 2.1.1 | Keyboard | A | **Fixed** | Password toggle is now `<button>`, all interactive elements focusable |
| 2.4.1 | Bypass Blocks | A | **Fixed** | Skip link added |
| 2.4.3 | Focus Order | A | Pass | Tab order follows visual order; hidden steps don't receive focus |
| 2.4.6 | Headings and Labels | AA | Pass | Each step has an `<h2>` heading; all inputs have labels |
| 2.4.7 | Focus Visible | AA | **Fixed** | Custom focus indicators added (was `outline: none`) |
| 3.1.1 | Language of Page | A | **Fixed** | `lang="en"` (was `lang="de"` in PHP) |
| 3.3.1 | Error Identification | A | Pass | Inline validation messages identify the error; `aria-invalid` on fields |
| 3.3.2 | Labels or Instructions | A | Pass | Labels, placeholders, and `sr-only` help text on all fields |
| 3.3.3 | Error Suggestion | AA | Pass | Validation messages suggest how to fix the error |
| 4.1.2 | Name, Role, Value | A | **Fixed** | `aria-invalid` on all inputs; toggle buttons have correct role/label |
| 4.1.3 | Status Messages | AA | **Fixed** | Password strength announced via `aria-live`; toasts use `role="alert"` |

---

## 13.25 — Dependency Graph

```
Step 6 (Referral UI)
Step 7 (Referral Verification)
Step 8 (Registration Form)
Step 9 (Registration Submission)
Step 10 (Success Confirmation)
Step 11 (Navigation & Back Button)
Step 12 (Toast Notifications)
    │
    ▼
Step 13 (Accessibility)  ◄── YOU ARE HERE
    │
    ├──► Step 14 (CSS & Visual Parity) — uses the accessibility.css additions
    └──► Step 15 (E2E Tests) — includes a11y test suite
```

---

## 13.26 — Definition of Done

- [ ] Every `<input>` has an associated `<label>` (visually hidden via `sr-only`)
- [ ] Every `<input>` has a reactive `aria-invalid` attribute reflecting its error state
- [ ] Every `<input>` has `aria-describedby` linking to its validation message and help text
- [ ] Password visibility toggles are `<button>` elements with `aria-label`, `aria-pressed`, `aria-controls`
- [ ] Custom focus indicators visible on all interactive elements (`:focus-visible` / `:focus-within`)
- [ ] `.btn-primary` text colour meets 4.5:1 contrast ratio
- [ ] Skip navigation link present, hidden by default, visible on focus, targets `#main-form`
- [ ] `StepAnnouncer` component renders `aria-live="polite"` region and announces step transitions
- [ ] Custom messages announced on: referral verified, form validation failure, registration success
- [ ] `focus_first_interactive()` called on every step transition
- [ ] `focus_first_error()` called on form submission failure
- [ ] Inactive steps have `inert` attribute (or are removed from DOM via `<Show>`)
- [ ] Password strength meter has `role="meter"`, `aria-valuenow`, `aria-valuetext`
- [ ] Password strength changes announced via `aria-live` region
- [ ] `aria-busy="true"` on submit buttons while async actions are pending
- [ ] External links (Privacy Policy, EULA) have `rel="noopener noreferrer"` and sr-only "(opens in new tab)" text
- [ ] Error summary component renders on failed form submission with links to each error
- [ ] `<html lang="en">` verified in Leptos shell
- [ ] `accessibility.css` imported in `app.rs`
- [ ] `@media (prefers-reduced-motion: reduce)` covers all new transitions
- [ ] `@media (prefers-contrast: high)` covers focus indicators
- [ ] `cargo leptos build` compiles with no errors
- [ ] axe-core audit passes: 0 critical/serious violations across all 3 steps + error state
- [ ] Keyboard navigation tests pass: all 8 tests in `keyboard-navigation.spec.ts`
- [ ] Manual VoiceOver walkthrough completed for all 3 steps + toasts (section 13.19)
- [ ] Colour contrast verified for all text/background combinations (section 13.20)
