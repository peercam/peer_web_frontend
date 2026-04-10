# Step 11 — Navigation & Back Button

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Implement the multi-step navigation logic — back button behaviour, step transitions, browser history integration, and focus management. This step unifies the navigation patterns wired incrementally in Steps 6–10 into a single, cohesive system that matches the existing PHP/JS behaviour in `register.php` and `js/register/register.js`.

**Depends on:** Steps 6 (Referral UI), 8 (Registration Form), 10 (Success Confirmation)

---

## 11.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 6 complete | Navigate to `/register` | Referral code input renders on step 1 |
| Step 8 complete | Advance to step 2 | Registration form fields render with validation |
| Step 10 complete | Complete registration flow | Success screen renders on step 3 |
| `current_step` signal exists | Check `src/pages/register.rs` | `current_step: RwSignal<u8>` declared and shared across step components |
| CSS imported | Check build output | `login-register.css` loaded (`.form-step`, `.back-btn`, `.active` classes available) |
| Step announcer exists | Check `src/pages/register.rs` | `aria-live="polite"` region present from Step 10 |
| Dev server runs | `cargo leptos watch` | Compiles and serves at `localhost:3000` |

---

## 11.2 — Architecture Overview

### Current JS Behaviour (reference)

The existing `AccessibleRegistrationForm` class in `js/register/register.js` manages step navigation with these key methods:

| Method | Behaviour |
|--------|-----------|
| `goToStep(n)` | Sets `this.currentStep = n`, calls `showStep()`, updates back button visibility |
| `showStep(stepId)` | Removes `.active` from all `.form-step` divs, adds `.active` to target, focuses first interactive element |
| `goBack(e)` | If `currentStep > 1`, prevents default link behaviour, calls `goToStep(currentStep - 1)` |
| `updateBackButton(show)` | Sets `backBtn.style.display` to `'flex'` or `'none'` |
| `announceStep(msg)` | Updates `#step-announcer` textContent for screen readers |

### Step Map

```
Step 1  ──►  Step 1b  ──►  Step 1 (auto-fill)
  │                          │
  ▼                          ▼
Step 2  ◄───────────────── Step 1
  │
  ▼
Step 3 (success — no back button)
```

| Step | ID | Back Button Behaviour |
|------|----|-----------------------|
| 1 (`referralStep`) | `referralStep` | Visible; navigates to `/login` (full page navigation) |
| 1b (`defaultReferralStep`) | `defaultReferralStep` | Visible; returns to step 1 |
| 2 (`registrationStep`) | `registrationStep` | Visible; returns to step 1 (preserves referral code) |
| 3 (`successStep`) | `successStep` | **Hidden** |

### Leptos Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           RegisterPage Component                             │
│                                                                              │
│  Signals:                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │  current_step: RwSignal<RegistrationStep>                                ││
│  │  show_default_referral: RwSignal<bool>                                   ││
│  │  referral_code: RwSignal<String>  (preserved across back-navigation)     ││
│  │  email: RwSignal<String>          (preserved across back-navigation)     ││
│  │  username: RwSignal<String>       (preserved across back-navigation)     ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  Derived:                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │  show_back_button: Memo<bool>                                            ││
│  │    = current_step != Step3 (success)                                     ││
│  │                                                                          ││
│  │  back_button_href: Memo<Option<String>>                                  ││
│  │    = Step1 → Some("/login"), otherwise → None                            ││
│  │                                                                          ││
│  │  step_announcement: Memo<String>                                         ││
│  │    = step-dependent screen reader text                                   ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  Layout:                                                                     │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │  top_head_area                                                           ││
│  │  ┌──────────────────────────────────────────────────────────────────────┐││
│  │  │  BackButton component                                                │││
│  │  │    visible: show_back_button                                         │││
│  │  │    on_click: go_back()                                               │││
│  │  └──────────────────────────────────────────────────────────────────────┘││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │  center_area                                                             ││
│  │    Step 1  (class:active when current_step == Step1)                     ││
│  │    Step 1b (class:active when current_step == Step1 && show_default)     ││
│  │    Step 2  (class:active when current_step == Step2)                     ││
│  │    Step 3  (class:active when current_step == Step3)                     ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │  step-announcer (aria-live="polite")                                     ││
│  │    {step_announcement}                                                   ││
│  └──────────────────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 11.3 — Step Enum

Replace the raw `u8` step signal with a proper enum for type safety and clarity. This enum is used throughout the registration page.

### Create or update `src/models/registration_step.rs`

```rust
/// Represents the current step in the multi-step registration flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistrationStep {
    /// Step 1: Referral code entry
    Referral,
    /// Step 2: Registration form (email, username, password)
    Registration,
    /// Step 3: Success confirmation
    Success,
}

impl RegistrationStep {
    /// Internal step number for data-step attribute and ordering.
    pub fn number(&self) -> u8 {
        match self {
            Self::Referral => 1,
            Self::Registration => 2,
            Self::Success => 3,
        }
    }

    /// Screen reader announcement text for this step.
    pub fn announcement(&self) -> &'static str {
        match self {
            Self::Referral => "Step 1: Referral Code Entry",
            Self::Registration => "Step 2: Registration Form",
            Self::Success => "Registration successful! Welcome to peer!",
        }
    }

    /// The previous step, if one exists.
    /// Returns `None` for Step 1 (back goes to `/login` instead).
    pub fn previous(&self) -> Option<Self> {
        match self {
            Self::Referral => None,
            Self::Registration => Some(Self::Referral),
            Self::Success => None, // no back from success
        }
    }

    /// Whether the back button should be shown on this step.
    pub fn show_back_button(&self) -> bool {
        !matches!(self, Self::Success)
    }
}
```

Export from `src/models/mod.rs`:

```rust
pub mod registration_step;
```

### Why an enum instead of `u8`

| `u8` approach | Enum approach |
|---------------|---------------|
| `current_step.get() > 1 && current_step.get() < 3` — magic numbers | `step.show_back_button()` — self-documenting |
| Easy to introduce invalid states (`current_step.set(5)`) | Exhaustive `match` — compiler catches missing cases |
| Step-specific logic scattered across `if/else` chains | Behaviour co-located on the enum |

---

## 11.4 — Back Button Component

### Create `src/components/back_button.rs`

The back button is an `<a>` tag (matching the PHP markup) that either navigates to `/login` (on step 1) or calls a callback to go to the previous step.

```rust
use leptos::prelude::*;

/// Back button for multi-step navigation.
///
/// On step 1: renders as a link to `/login` (full page navigation).
/// On step 2: calls `on_back` callback to return to step 1.
/// On step 3: hidden entirely.
#[component]
pub fn BackButton(
    /// Whether the button is currently visible.
    visible: Signal<bool>,
    /// If `Some(url)`, renders as a link to that URL.
    /// If `None`, renders as a button that calls `on_back`.
    href: Signal<Option<String>>,
    /// Callback invoked when back is clicked (only when `href` is `None`).
    on_back: Callback<()>,
) -> impl IntoView {
    view! {
        <a
            href=move || href.get().unwrap_or_default()
            class="btn btn-secondary back-btn"
            id="backBtn"
            style:display=move || if visible.get() { "flex" } else { "none" }
            on:click=move |ev| {
                // If no href (i.e. not step 1), prevent navigation and trigger callback
                if href.get().is_none() {
                    ev.prevent_default();
                    on_back.run(());
                }
            }
        >
            <span aria-hidden="true">
                <i class="peer-icon medium_font peer-icon-arrow-left"></i>
            </span>
            "Back"
        </a>
    }
}
```

### Export from `src/components/mod.rs`

```rust
pub mod back_button;
```

### Design Notes

| Decision | Rationale |
|----------|-----------|
| `<a>` tag, not `<button>` | Matches existing PHP markup (`<a href="login.php" class="btn btn-secondary back-btn">`); CSS relies on `a.back-btn` selector |
| `href` is reactive `Signal<Option<String>>` | On step 1, it's `Some("/login")` for a real navigation; on step 2, it's `None` to trigger the `on_back` callback |
| `style:display` toggle | Matches the JS pattern: `backBtn.style.display = show ? 'flex' : 'none'` |
| `on:click` with `prevent_default` | Only prevents default when back navigates within the form (step 2→1); on step 1, the `<a href="/login">` navigates normally |

---

## 11.5 — Navigation Logic in RegisterPage

### 11.5.1 — Signals and Derived State

Update `src/pages/register.rs` to use the new `RegistrationStep` enum and derived navigation signals:

```rust
use crate::models::registration_step::RegistrationStep;
use crate::components::back_button::BackButton;

#[component]
pub fn RegisterPage() -> impl IntoView {
    // --- Core step state ---
    let current_step = RwSignal::new(RegistrationStep::Referral);
    let show_default_referral = RwSignal::new(false);

    // --- Form data signals (preserved across back-navigation) ---
    let referral_code = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // --- Derived navigation signals ---
    let show_back = Memo::new(move |_| {
        current_step.get().show_back_button()
    });

    let back_href = Memo::new(move |_| {
        match current_step.get() {
            RegistrationStep::Referral => Some("/login".to_string()),
            _ => None,
        }
    });

    let step_announcement = Memo::new(move |_| {
        current_step.get().announcement().to_string()
    });

    // --- Back button handler ---
    let go_back = Callback::new(move |_: ()| {
        if let Some(prev) = current_step.get().previous() {
            current_step.set(prev);
            // show_default_referral resets when going back to step 1
            show_default_referral.set(false);
            // Focus management is handled by the Effect below
        }
    });

    // --- Focus management: focus first interactive element on step change ---
    Effect::new(move |_| {
        let step = current_step.get();
        // Focus the first input/button in the new step after a micro-task
        // (allows DOM to update before querying)
        request_animation_frame(move || {
            focus_first_interactive_in_step(step);
        });
    });

    // ... view! macro with all steps ...
}
```

### 11.5.2 — View Layout

```rust
    view! {
        <div class="container large_font">
            <div class="container_left">
                // ... phone mockup, logo (unchanged from Step 5) ...
            </div>
            <div class="container_right">
                <div class="container_inner">
                    <div class="top_head_area">
                        <BackButton
                            visible=show_back.into()
                            href=back_href.into()
                            on_back=go_back
                        />
                    </div>

                    <div class="center_area">
                        // Step 1: Referral
                        <div
                            class="form-step"
                            id="referralStep"
                            data-step="1"
                            class:active=move || {
                                current_step.get() == RegistrationStep::Referral
                                    && !show_default_referral.get()
                            }
                        >
                            <ReferralStep /* ... props ... */ />
                        </div>

                        // Step 1b: Default Referral
                        <div
                            class="form-step"
                            id="defaultReferralStep"
                            data-step="1b"
                            class:active=move || {
                                current_step.get() == RegistrationStep::Referral
                                    && show_default_referral.get()
                            }
                        >
                            <DefaultReferralView /* ... props ... */ />
                        </div>

                        // Step 2: Registration Form
                        <div
                            class="form-step"
                            id="registrationStep"
                            data-step="2"
                            class:active=move || {
                                current_step.get() == RegistrationStep::Registration
                            }
                        >
                            <RegistrationForm /* ... props ... */ />
                        </div>

                        // Step 3: Success
                        <div
                            class="form-step"
                            id="successStep"
                            data-step="3"
                            class:active=move || {
                                current_step.get() == RegistrationStep::Success
                            }
                        >
                            <SuccessStep email=email.into() />
                        </div>
                    </div>

                    // Screen reader step announcer
                    <div
                        id="step-announcer"
                        class="sr-only"
                        aria-live="polite"
                        aria-atomic="true"
                    >
                        {step_announcement}
                    </div>
                </div>
            </div>
        </div>
    }
```

---

## 11.6 — Focus Management

After each step transition, the first focusable element in the new step must receive focus. This mirrors the existing JS behaviour:

```javascript
// From register.js showStep()
setTimeout(() => {
    const focusable = targetStep.querySelector('input, button, select, textarea');
    focusable?.focus();
}, 100);
```

### Leptos Implementation

```rust
/// Focus the first interactive element (input, button, select, textarea)
/// within the active step container.
fn focus_first_interactive_in_step(step: RegistrationStep) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;

        let step_id = match step {
            RegistrationStep::Referral => "referralStep",
            RegistrationStep::Registration => "registrationStep",
            RegistrationStep::Success => "successStep",
        };

        if let Some(document) = web_sys::window()
            .and_then(|w| w.document())
        {
            if let Some(container) = document.get_element_by_id(step_id) {
                let selector = "input, button, select, textarea, a[href]";
                if let Ok(Some(el)) = container.query_selector(selector) {
                    if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = html_el.focus();
                    }
                }
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = step;
    }
}

/// Wrapper for requestAnimationFrame to delay focus until after DOM update.
fn request_animation_frame(f: impl FnOnce() + 'static) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let closure = Closure::once_into_js(f);
        if let Some(window) = web_sys::window() {
            let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = f;
    }
}
```

### Focus Targets per Step

| Step | First Focusable Element | Why |
|------|-------------------------|-----|
| 1 (Referral) | `<input id="referralCode">` | User needs to enter/edit their referral code |
| 1b (Default Referral) | `<button id="useThisCodeBtn">` | Single action: "Use This Code" |
| 2 (Registration) | `<input id="email">` | First form field |
| 3 (Success) | `<a href="/login">Continue to Login</a>` | Only interactive element; directs user forward |

---

## 11.7 — State Preservation on Back Navigation

A critical requirement: when the user navigates **back** from step 2 to step 1, all form data must be preserved. When they advance to step 2 again, their previously entered email, username, etc. should still be there.

### How It Works

All form field signals (`referral_code`, `email`, `username`, `password`, `confirm_password`) are owned by the parent `RegisterPage` component and passed down as props. Since we use **CSS toggle** (`.active`) rather than conditional rendering (`<Show>`), all step DOM nodes remain mounted at all times. This means:

1. **Input values are bound to `RwSignal`s** — the signal retains its value even when the step div is hidden.
2. **No re-render on back navigation** — toggling `.active` is a CSS-only change; the component tree is untouched.
3. **Validation states persist** — derived signals (e.g., `is_email_valid`) continue to reflect the current signal values.

### Edge Cases

| Scenario | Behaviour |
|----------|-----------|
| Back from step 2 → step 1 | Referral code input still shows the verified code |
| Forward again to step 2 | Email, username, password fields retain their values |
| Back from step 1 → `/login` | Full page navigation; all state is lost (expected) |
| Refresh on any step | `current_step` resets to `Referral`; all signals reset to defaults (matches current PHP behaviour) |
| Back from step 1b → step 1 | Handled by toggling `show_default_referral` to `false`, not by changing `current_step` |

---

## 11.8 — Browser History Integration

### 11.8.1 — Problem Statement

In the current PHP app, the entire registration flow is a single URL (`register.php`). Pressing the browser back button on step 2 leaves the page entirely (navigates to the previous browser history entry). The step state is managed purely in JavaScript memory.

### 11.8.2 — Approach: URL Hash Fragments

To improve UX, we can optionally push step state into URL hash fragments so the browser back button integrates with step navigation:

| Step | URL |
|------|-----|
| 1 | `/register` or `/register#step-1` |
| 2 | `/register#step-2` |
| 3 | `/register#step-3` |

### 11.8.3 — Implementation

```rust
/// Push the current step into the browser history via hash fragment.
fn push_step_to_history(step: RegistrationStep) {
    #[cfg(feature = "hydrate")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(history) = window.history() {
                let hash = format!("#step-{}", step.number());
                let _ = history.push_state_with_url(
                    &wasm_bindgen::JsValue::NULL,
                    "",
                    Some(&hash),
                );
            }
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = step;
    }
}

/// Listen for browser popstate events (back/forward buttons)
/// and update `current_step` accordingly.
fn listen_for_popstate(current_step: RwSignal<RegistrationStep>) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        if let Some(window) = web_sys::window() {
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                if let Some(w) = web_sys::window() {
                    if let Ok(location) = w.location().hash() {
                        let step = match location.as_str() {
                            "#step-1" => RegistrationStep::Referral,
                            "#step-2" => RegistrationStep::Registration,
                            "#step-3" => RegistrationStep::Success,
                            _ => RegistrationStep::Referral,
                        };
                        current_step.set(step);
                    }
                }
            }) as Box<dyn FnMut(_)>);

            let _ = window.add_event_listener_with_callback(
                "popstate",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget(); // leak intentionally — lives for page lifetime
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = current_step;
    }
}
```

### 11.8.4 — Wiring into RegisterPage

```rust
#[component]
pub fn RegisterPage() -> impl IntoView {
    let current_step = RwSignal::new(RegistrationStep::Referral);

    // --- Browser history integration ---
    // On mount: read initial hash and set step accordingly
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(hash) = window.location().hash() {
                    let step = match hash.as_str() {
                        "#step-2" => RegistrationStep::Registration,
                        "#step-3" => RegistrationStep::Success,
                        _ => RegistrationStep::Referral,
                    };
                    current_step.set(step);
                }
            }
        }
    });

    // Listen for browser back/forward
    listen_for_popstate(current_step);

    // Push step changes to history
    Effect::new(move |_| {
        let step = current_step.get();
        push_step_to_history(step);
    });

    // ... rest of the component ...
}
```

### 11.8.5 — Security Note

The `#step-3` hash should **not** be trusted to bypass the registration flow. Even if a user manually navigates to `/register#step-3`, the `SuccessStep` component simply shows a confirmation message with no privileged data. The actual account creation requires server-side validation (Step 9). There is no security risk in displaying the success UI without completing registration — it's just a static message.

---

## 11.9 — Step 1b (Default Referral) Navigation

Step 1b is a sub-step within step 1, not a separate step. It is controlled by the `show_default_referral: RwSignal<bool>` signal rather than `current_step`.

### Navigation Rules

| Action | Result |
|--------|--------|
| Click "Don't have a code?" on step 1 | `show_default_referral.set(true)` — shows step 1b |
| Click "Use This Code" on step 1b | `show_default_referral.set(false)`, auto-fill referral input — returns to step 1 |
| Click Back on step 1b | `show_default_referral.set(false)` — returns to step 1 |
| Click Back on step 1 (not 1b) | Navigate to `/login` (link behaviour, no callback) |

### Back Button Integration

The `go_back` callback needs special handling when `show_default_referral` is active:

```rust
    let go_back = Callback::new(move |_: ()| {
        // If we're on the default referral sub-step, go back to step 1
        if current_step.get() == RegistrationStep::Referral
            && show_default_referral.get()
        {
            show_default_referral.set(false);
            return;
        }

        // Otherwise, go to the previous step
        if let Some(prev) = current_step.get().previous() {
            current_step.set(prev);
            show_default_referral.set(false);
        }
    });
```

However, note that on step 1 (non-1b), the back button `href` is `Some("/login")`, so `on_back` won't fire — the browser navigates normally. The callback is only invoked when `href` is `None` (steps 1b and 2).

### Updated `back_href` Logic

```rust
    let back_href = Memo::new(move |_| {
        match current_step.get() {
            RegistrationStep::Referral if !show_default_referral.get() => {
                // Step 1 (not 1b) — link to login page
                Some("/login".to_string())
            }
            _ => None, // Steps 1b, 2: use callback
        }
    });
```

---

## 11.10 — Transition Animation

The existing CSS handles the fade-in animation:

```css
.form-step {
    display: none;
}

.form-step.active {
    display: block;
    animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to   { opacity: 1; transform: translateY(0); }
}
```

Since we use `class:active` toggling, the animation plays automatically whenever a step gains the `.active` class. No additional Leptos code is needed for the transition effect.

### Animation Plays On

| Transition | Animation? |
|------------|------------|
| Step 1 → Step 2 (forward) | Yes — step 2 gains `.active` |
| Step 2 → Step 1 (back) | Yes — step 1 gains `.active` |
| Step 2 → Step 3 (success) | Yes — step 3 gains `.active` |
| Step 1 → Step 1b | Yes — step 1b gains `.active` |
| Step 1b → Step 1 | Yes — step 1 gains `.active` |

---

## 11.11 — Files to Create / Modify

| File | Action | Description |
|------|--------|-------------|
| `src/models/registration_step.rs` | **Create** | `RegistrationStep` enum with navigation methods |
| `src/models/mod.rs` | **Edit** | Add `pub mod registration_step;` |
| `src/components/back_button.rs` | **Create** | `BackButton` component |
| `src/components/mod.rs` | **Edit** | Add `pub mod back_button;` |
| `src/pages/register.rs` | **Edit** | Replace `RwSignal<u8>` with `RwSignal<RegistrationStep>`, add derived navigation signals, wire `BackButton`, add focus management, add browser history integration |

---

## 11.12 — SSR Considerations

| Concern | Approach |
|---------|----------|
| Back button initial state | SSR renders the back button as `<a href="/login" style="display: flex">` (visible, linking to login) — matches step 1 default |
| `window`, `document`, `history` access | All browser API calls guarded with `#[cfg(feature = "hydrate")]` |
| Focus management | No-op on server; only runs in WASM after hydration |
| `popstate` listener | Only registered in WASM; no impact on SSR |
| Hash fragment reading | Only in WASM `Effect::new` on mount; SSR always renders step 1 |
| All steps in SSR HTML | CSS toggle means all `.form-step` divs are present in SSR output; only the first has `.active` |

### SSR HTML Output (initial load)

```html
<div class="top_head_area">
    <a href="/login" class="btn btn-secondary back-btn" id="backBtn" style="display: flex;">
        <span aria-hidden="true"><i class="peer-icon medium_font peer-icon-arrow-left"></i></span>
        Back
    </a>
</div>
<div class="center_area">
    <div class="form-step active" id="referralStep" data-step="1"><!-- ... --></div>
    <div class="form-step" id="defaultReferralStep" data-step="1b"><!-- ... --></div>
    <div class="form-step" id="registrationStep" data-step="2"><!-- ... --></div>
    <div class="form-step" id="successStep" data-step="3"><!-- ... --></div>
</div>
```

---

## 11.13 — Testing

### 11.13.1 — Unit Tests: `RegistrationStep` Enum

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_numbers() {
        assert_eq!(RegistrationStep::Referral.number(), 1);
        assert_eq!(RegistrationStep::Registration.number(), 2);
        assert_eq!(RegistrationStep::Success.number(), 3);
    }

    #[test]
    fn test_previous_step() {
        assert_eq!(RegistrationStep::Referral.previous(), None);
        assert_eq!(
            RegistrationStep::Registration.previous(),
            Some(RegistrationStep::Referral)
        );
        assert_eq!(RegistrationStep::Success.previous(), None);
    }

    #[test]
    fn test_show_back_button() {
        assert!(RegistrationStep::Referral.show_back_button());
        assert!(RegistrationStep::Registration.show_back_button());
        assert!(!RegistrationStep::Success.show_back_button());
    }

    #[test]
    fn test_announcements() {
        assert!(RegistrationStep::Referral.announcement().contains("Referral"));
        assert!(RegistrationStep::Registration.announcement().contains("Registration"));
        assert!(RegistrationStep::Success.announcement().contains("successful"));
    }
}
```

Run with:

```bash
cargo test --lib -- registration_step
```

### 11.13.2 — SSR Render Tests

```rust
#[cfg(test)]
mod ssr_tests {
    use super::*;
    use leptos::prelude::*;

    #[test]
    fn test_back_button_visible_on_step_1() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let html = leptos::ssr::render_to_string(|| {
                let visible = RwSignal::new(true);
                let href = RwSignal::new(Some("/login".to_string()));
                let on_back = Callback::new(|_: ()| {});
                view! {
                    <BackButton
                        visible=visible.into()
                        href=href.into()
                        on_back=on_back
                    />
                }
            });

            assert!(html.contains("back-btn"), "Missing back-btn class");
            assert!(html.contains("display: flex"), "Back button should be visible");
            assert!(html.contains("/login"), "Should link to /login on step 1");
        });
    }

    #[test]
    fn test_back_button_hidden_on_step_3() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let html = leptos::ssr::render_to_string(|| {
                let visible = RwSignal::new(false);
                let href = RwSignal::new(None::<String>);
                let on_back = Callback::new(|_: ()| {});
                view! {
                    <BackButton
                        visible=visible.into()
                        href=href.into()
                        on_back=on_back
                    />
                }
            });

            assert!(html.contains("display: none"), "Back button should be hidden on step 3");
        });
    }

    #[test]
    fn test_only_step_1_active_on_initial_render() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let html = leptos::ssr::render_to_string(|| {
                let current_step = RwSignal::new(RegistrationStep::Referral);
                let show_default = RwSignal::new(false);
                view! {
                    <div
                        class="form-step"
                        id="referralStep"
                        class:active=move || {
                            current_step.get() == RegistrationStep::Referral
                                && !show_default.get()
                        }
                    >
                        "Step 1"
                    </div>
                    <div
                        class="form-step"
                        id="registrationStep"
                        class:active=move || {
                            current_step.get() == RegistrationStep::Registration
                        }
                    >
                        "Step 2"
                    </div>
                    <div
                        class="form-step"
                        id="successStep"
                        class:active=move || {
                            current_step.get() == RegistrationStep::Success
                        }
                    >
                        "Step 3"
                    </div>
                }
            });

            // referralStep should have .active
            assert!(
                html.contains("form-step active\" id=\"referralStep\"")
                    || html.contains("form-step active\" id=\"referralStep"),
                "Step 1 should be active"
            );
            // Other steps should NOT have .active
            assert!(
                !html.contains("form-step active\" id=\"registrationStep"),
                "Step 2 should not be active"
            );
            assert!(
                !html.contains("form-step active\" id=\"successStep"),
                "Step 3 should not be active"
            );
        });
    }
}
```

Run with:

```bash
cargo test --lib -- ssr_tests
```

### 11.13.3 — Manual Browser Testing Checklist

| # | Test | Steps | Expected |
|---|------|-------|----------|
| 1 | Back button visible on step 1 | Load `/register` | Back button visible with "Back" text and left arrow icon |
| 2 | Step 1 back navigates to login | Click Back on step 1 | Browser navigates to `/login` |
| 3 | Step 2 back returns to step 1 | Verify referral → advance to step 2 → click Back | Step 1 is shown; referral code input still has the verified code |
| 4 | Back button hidden on step 3 | Complete full registration → reach step 3 | Back button is not visible |
| 5 | Focus moves on step forward | Advance from step 1 to step 2 | Email input in step 2 has focus |
| 6 | Focus moves on step back | Go back from step 2 to step 1 | Referral code input has focus |
| 7 | Focus on success step | Reach step 3 | "Continue to Login" link has focus |
| 8 | Step 1b navigation | Click "Don't have a code?" → click Back | Returns to step 1 (not `/login`) |
| 9 | Step 1b "Use This Code" | Click "Don't have a code?" → click "Use This Code" | Returns to step 1 with default code filled in |
| 10 | Screen reader announces step changes | Enable VoiceOver (⌘+F5), navigate through steps | Each transition announces the step name |
| 11 | Browser back button (step 2 → 1) | Advance to step 2, press browser Back | Returns to step 1 (if history integration enabled) |
| 12 | Browser forward button (step 1 → 2) | After test 11, press browser Forward | Returns to step 2 (if history integration enabled) |
| 13 | Direct URL hash access | Navigate to `/register#step-2` | Step 2 is shown (note: no server data, so form will be empty) |
| 14 | Fade-in animation | Navigate between any steps | Each new step fades in with 0.3s animation |
| 15 | Form state preserved | Fill email/username on step 2 → Back → Forward to step 2 | Email and username values are still present |
| 16 | Responsive back button | Test at 375px, 1024px, 1440px, 2000px+ widths | Back button line-height and padding adjust per CSS breakpoints |

### 11.13.4 — Accessibility Tests

| # | Test | Expected |
|---|------|----------|
| 1 | Keyboard-only navigation | Tab reaches Back button; Enter/Space activates it; Shift+Tab from first field reaches Back |
| 2 | Screen reader: back button | VoiceOver reads "Back, link" (step 1) or "Back, link" (step 2, though it acts as a button via `preventDefault`) |
| 3 | Screen reader: step announcer | On each step change, VoiceOver announces the step-specific text from `step_announcement` |
| 4 | Focus order after back | After going back, Tab order starts from the first element of the destination step |
| 5 | No focus trap | Tab past the last element of the active step does not loop — reaches browser chrome (URL bar) |

---

## 11.14 — Edge Cases & Error Handling

| Edge Case | Handling |
|-----------|----------|
| Rapidly clicking Back repeatedly | `current_step` is a simple signal set — multiple rapid sets converge to the final value; no race condition |
| Back on step 1 with `show_default_referral` active | `go_back` checks this flag first; returns to step 1 (not `/login`) |
| User manually edits URL hash to `#step-3` | Success UI displays (no security risk — see 11.8.5) |
| User manually edits URL hash to `#step-99` | Defaults to `RegistrationStep::Referral` in the match |
| `popstate` fires after component is unmounted | The `closure.forget()` leak means the listener persists, but `current_step.set()` on a disposed signal is a no-op in Leptos |
| Browser with JS disabled | SSR HTML shows step 1 with back button linking to `/login`; functional as a static page (no step transitions without JS) |
| `requestAnimationFrame` unavailable | Guarded by `#[cfg(feature = "hydrate")]`; on SSR it's a no-op. In WASM, `rAF` is universally supported |

---

## 11.15 — Dependency Graph

```
Step 6 (Referral UI)
Step 8 (Registration Form)     ──► Step 11 (Navigation & Back Button) ◄── YOU ARE HERE
Step 10 (Success Confirmation)       │
                                     ├──► Step 12 (Toast) — may show toasts on navigation errors
                                     ├──► Step 13 (Accessibility) — audits focus management + announcer
                                     ├──► Step 14 (CSS & Visual Parity) — verifies back button styling
                                     └──► Step 15 (E2E Tests) — tests back navigation, state preservation
```

---

## 11.16 — Definition of Done

- [ ] `RegistrationStep` enum created with `number()`, `announcement()`, `previous()`, `show_back_button()` methods
- [ ] `BackButton` component created and renders matching PHP markup (`<a class="btn btn-secondary back-btn">`)
- [ ] `current_step` signal changed from `RwSignal<u8>` to `RwSignal<RegistrationStep>`
- [ ] Back button on step 1 navigates to `/login`
- [ ] Back button on step 1b returns to step 1
- [ ] Back button on step 2 returns to step 1 with referral code preserved
- [ ] Back button hidden on step 3
- [ ] Focus moves to first interactive element on every step transition
- [ ] `aria-live` announcer updates on every step transition
- [ ] Step 1b ↔ Step 1 toggle works via `show_default_referral` signal
- [ ] All form field values preserved across back/forward navigation
- [ ] Browser history hash fragments pushed on step change
- [ ] Browser back button triggers step regression via `popstate` listener
- [ ] Fade-in animation plays on every step transition (via CSS `.active` class)
- [ ] `cargo leptos build` compiles with no errors
- [ ] `RegistrationStep` unit tests pass (`cargo test`)
- [ ] SSR render tests pass (`cargo test`)
- [ ] Manual browser testing checklist (11.13.3) passes all 16 items
- [ ] Accessibility tests (11.13.4) pass all 5 items
