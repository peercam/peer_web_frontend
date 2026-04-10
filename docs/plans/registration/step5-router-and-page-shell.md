# Step 5 — Router & Page Shell

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Set up the Leptos router with the `/register` route and a minimal `RegisterPage` component that renders the multi-step container shell — matching the HTML structure from `register.php`. Import the existing CSS so the page is styled. Confirm the page is server-side rendered (not a blank WASM-only shell).

---

## 5.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Step 1 complete | `cd peer-web && cargo leptos build` | Compiles without errors |
| Step 2 complete | `cargo check` | All models and module skeleton compile |
| Leptos dev server works | `cargo leptos watch` | Serves at `http://localhost:3000` |
| Register page stub exists | Check `src/pages/register.rs` | Contains placeholder `RegisterPage` component |
| CSS file available | Check `css/login-register.css` exists in repo root | File present |

---

## 5.2 — Architecture Overview

Step 5 establishes the routing and page-level component structure that Steps 6–11 will build on. The `RegisterPage` component owns the multi-step state and renders the outer container. Each step's inner content will be filled in by subsequent steps.

```
┌──────────────────────────────────────────────────────────┐
│  src/app.rs  — <Router>                                  │
│                                                          │
│   <Routes>                                               │
│     <Route path="/"          view=HomePage />            │
│     <Route path="/register"  view=RegisterPage />        │
│   </Routes>                                              │
└─────────────────────┬────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────────────────┐
│  src/pages/register.rs  — RegisterPage                   │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Reactive state:                                   │  │
│  │    current_step: RwSignal<u8>  (1, 2, or 3)       │  │
│  │    referral_code: RwSignal<String>                 │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │  HTML structure (mirrors register.php):            │  │
│  │                                                    │  │
│  │  div.container.large_font                          │  │
│  │  ├── div.container_left   (phone mockup + logo)    │  │
│  │  └── div.container_right                           │  │
│  │      └── div.container_inner                       │  │
│  │          ├── div.top_head_area  (back button)      │  │
│  │          ├── div.center_area                       │  │
│  │          │   ├── div.form-step[data-step=1]        │  │
│  │          │   ├── div.form-step[data-step=1b]       │  │
│  │          │   ├── div.form-step[data-step=2]        │  │
│  │          │   └── div.form-step[data-step=3]        │  │
│  │          └── div.footer_area                       │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Mirror the `register.php` HTML structure exactly | Enables reuse of existing `login-register.css` without modifications; ensures visual parity (Step 14) is straightforward |
| Use `current_step: RwSignal<u8>` to control step visibility | Simple reactive state; toggling the `.active` CSS class per step matches the existing JS pattern |
| Read `?ref=` query parameter on mount | Matches current PHP/JS behaviour where visiting `/register?ref=UUID` prefills the referral input |
| SSR the full HTML | Leptos SSR ensures `/register` is crawlable and renders immediately; WASM hydrates for interactivity |
| Placeholder content in each step `div` | Steps 6–10 will replace placeholders with real form components; this step only verifies the shell |
| Import CSS via `<Stylesheet>` in the shell component | Leptos `<Stylesheet>` emits a `<link>` tag in the `<head>` during SSR |

---

## 5.3 — Copy Static Assets into the Leptos Project

The Leptos project serves static files from `peer-web/public/` (or `peer-web/style/` for CSS). Copy the necessary assets so they are accessible at dev time.

### 5.3.1 — CSS

```bash
# From the repo root
cp css/login-register.css peer-web/style/login-register.css
```

### 5.3.2 — Fonts

```bash
mkdir -p peer-web/public/fonts
cp -r fonts/font-poppins peer-web/public/fonts/font-poppins
cp -r fonts/peer-icon-font peer-web/public/fonts/peer-icon-font
```

### 5.3.3 — Images and SVGs

```bash
mkdir -p peer-web/public/img peer-web/public/svg
cp img/register.webp peer-web/public/img/register.webp
cp svg/logo_sw.svg peer-web/public/svg/logo_sw.svg
cp svg/logo_farbe.svg peer-web/public/svg/logo_farbe.svg
cp svg/blueglow.svg peer-web/public/svg/blueglow.svg
cp svg/blueglow1.svg peer-web/public/svg/blueglow1.svg
```

> **Note:** These are copies, not symlinks, so the Leptos project is self-contained. When visual parity work begins (Step 14), these will be reconciled.

### 5.3.4 — Update font paths in the copied CSS

The copied `login-register.css` references fonts and SVGs using _relative_ paths (`../fonts/...`, `../svg/...`). Since the Leptos project serves static assets from `/`, update the CSS background URLs:

```bash
# Inside peer-web/style/login-register.css, update paths:
sed -i '' "s|url('../svg/|url('/svg/|g" peer-web/style/login-register.css
sed -i '' "s|url('../fonts/|url('/fonts/|g" peer-web/style/login-register.css
sed -i '' "s|url('../img/|url('/img/|g" peer-web/style/login-register.css
```

---

## 5.4 — Configure Leptos to Serve the CSS

### 5.4.1 — Update `Cargo.toml` style settings

Ensure `cargo-leptos` knows about the stylesheet. In `peer-web/Cargo.toml`, under the `[package.metadata.leptos]` section:

```toml
[package.metadata.leptos]
# ... existing settings ...

# Additional CSS files to include (login-register.css will be served as a static asset)
# The main style file is compiled by cargo-leptos:
style-file = "style/main.scss"

# Static assets directory:
assets-dir = "public"
```

> **Note:** `login-register.css` is a plain CSS file, not SCSS, so we serve it as a static asset from `public/` or reference it from the `style/` directory. The cleanest approach is to `@import` it from `style/main.scss`.

### 5.4.2 — Import login-register.css from the main stylesheet

In `peer-web/style/main.scss`, add at the top:

```scss
/* Import the registration/login styles from the legacy PHP app */
@import "login-register.css";

/* Import Poppins font */
@import url("/fonts/font-poppins/stylesheet.css");

/* Import Peer icon font */
@import url("/fonts/peer-icon-font/css/peer-network.css");

/* ... existing Leptos default styles below ... */
```

This ensures `cargo leptos watch` bundles the registration CSS into the output.

---

## 5.5 — Update `src/app.rs` — Router Configuration

Replace the default router in `src/app.rs` with one that includes the `/register` route. This file is the root of the Leptos component tree.

### Full `src/app.rs`

```rust
//! Root application component and router configuration.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_router::components::{Route, Router, Routes};

use crate::error_template::{AppError, ErrorTemplate};
use crate::pages::register::RegisterPage;

/// Shell function providing `<head>` metadata for SSR.
///
/// This is called by the Axum server to wrap the rendered HTML.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

/// Root application component.
///
/// Sets up the router and global providers (meta, stylesheets).
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/peer-web.css" />

        <Router>
            <main>
                <Routes fallback=|| {
                    let mut outside_errors = Errors::default();
                    outside_errors.insert_with_default_key(AppError::NotFound);
                    view! { <ErrorTemplate outside_errors /> }.into_view()
                }>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/register") view=RegisterPage />
                </Routes>
            </main>
        </Router>
    }
}

/// Minimal home page (placeholder).
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome to Peer"</h1>
        <a href="/register">"Create an Account"</a>
    }
}
```

### Key changes from the default scaffold

| Change | Why |
|--------|-----|
| Added `use crate::pages::register::RegisterPage;` | Imports the registration page component |
| Added `<Route path=path!("/register") view=RegisterPage />` | Maps the `/register` URL to our component |
| Added `<Stylesheet>` for the bundled CSS | Ensures `login-register.css` styles are loaded |
| Kept `<MetaTags />` and `provide_meta_context()` | Enables per-page `<Title>` and `<Meta>` via `leptos_meta` |

---

## 5.6 — Update `src/pages/register.rs` — Page Shell Component

Replace the Step 2 stub with the full page shell. This mirrors the HTML structure of `register.php` exactly, using placeholder text where Steps 6–10 will add real content.

### Full `src/pages/register.rs`

```rust
//! Registration page — multi-step shell component.
//!
//! This component renders the full page layout for `/register`:
//! - Left panel: phone mockup with image and animated logo
//! - Right panel: multi-step form container
//!
//! The actual form content for each step is a placeholder here.
//! Steps 6–10 will replace each placeholder with real components.
//!
//! ## Step flow
//!
//! 1. Referral code entry (step 1) / default referral (step 1b)
//! 2. Registration form (step 2)
//! 3. Success confirmation (step 3)

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_query_map;

/// Registration step identifier.
///
/// Tracks which step of the multi-step form is currently visible.
/// The CSS class `.active` is applied to the matching `.form-step` div.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegStep {
    /// Step 1: Enter referral code
    Referral,
    /// Step 1b: Show default referral code
    DefaultReferral,
    /// Step 2: Registration form (email, username, password)
    Register,
    /// Step 3: Success confirmation
    Success,
}

impl RegStep {
    /// Returns the `data-step` attribute value matching register.php.
    pub fn data_step(&self) -> &'static str {
        match self {
            Self::Referral => "1",
            Self::DefaultReferral => "1b",
            Self::Register => "2",
            Self::Success => "3",
        }
    }

    /// Returns true if the back button should be visible for this step.
    pub fn show_back_button(&self) -> bool {
        match self {
            Self::Referral | Self::DefaultReferral | Self::Register => true,
            Self::Success => false,
        }
    }

    /// Returns the previous step for back-button navigation.
    /// `None` means navigate to `/login` (external).
    pub fn previous(&self) -> Option<RegStep> {
        match self {
            Self::Referral => None,           // Back → /login
            Self::DefaultReferral => Some(Self::Referral),
            Self::Register => Some(Self::Referral),
            Self::Success => None,            // No back button shown
        }
    }
}

/// The main registration page component.
///
/// Renders the full registration page layout with:
/// - Phone mockup (left panel)
/// - Multi-step form container (right panel)
/// - Reactive step transitions controlled by `current_step`
///
/// ## Query parameters
///
/// - `?ref=<UUID>` — Pre-fills the referral code input (read on mount)
///
/// ## Signals (will be expanded in later steps)
///
/// - `current_step: RwSignal<RegStep>` — Controls which form step is visible
/// - `referral_code: RwSignal<String>` — Referral code entered or pre-filled
#[component]
pub fn RegisterPage() -> impl IntoView {
    // ── Reactive state ──────────────────────────────────────────────────
    let current_step = RwSignal::new(RegStep::Referral);
    let referral_code = RwSignal::new(String::new());

    // ── Read ?ref= query parameter on mount ─────────────────────────────
    let query = use_query_map();
    Effect::new(move |_| {
        if let Some(ref_code) = query.read().get("ref") {
            if !ref_code.is_empty() {
                referral_code.set(ref_code.to_string());
            }
        }
    });

    // ── Helper: CSS class for a form step ───────────────────────────────
    // Returns "form-step active" if this step is current, else "form-step"
    let step_class = move |step: RegStep| {
        move || {
            if current_step.get() == step {
                "form-step active"
            } else {
                "form-step"
            }
        }
    };

    // ── Back-button handler ─────────────────────────────────────────────
    let on_back = move |_| {
        let step = current_step.get();
        match step.previous() {
            Some(prev) => current_step.set(prev),
            None => {
                // Navigate to /login — using window.location for now
                // (will be replaced with leptos_router navigate in Step 11)
                #[cfg(feature = "hydrate")]
                {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/login");
                    }
                }
            }
        }
    };

    // ── View ────────────────────────────────────────────────────────────
    view! {
        <Title text="Peer Network - Register" />
        <Meta name="description" content="Create your Peer Network account. Join the blockchain-based social network." />

        <div class="container large_font">
            // ── Left panel: phone mockup ────────────────────────────────
            <div class="container_left">
                <div class="phone">
                    <div class="screen">
                        <img
                            src="/img/register.webp"
                            alt="Register preview"
                            width="612"
                            height="612"
                        />
                    </div>
                    <div class="home-button">
                        <img
                            src="/svg/logo_sw.svg"
                            alt="Peer Logo"
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

            // ── Right panel: form container ─────────────────────────────
            <div class="container_right">
                <div class="container_inner">
                    // ── Top area: back button ───────────────────────────
                    <div class="top_head_area">
                        <Show when=move || current_step.get().show_back_button()>
                            <a
                                class="btn btn-secondary back-btn"
                                href="#"
                                on:click=on_back
                            >
                                <span aria-hidden="true">
                                    <i class="peer-icon medium_font peer-icon-arrow-left"></i>
                                </span>
                                "Back"
                            </a>
                        </Show>
                    </div>

                    // ── Center area: form steps ─────────────────────────
                    <div class="center_area">

                        // Step 1: Referral Code
                        <div
                            class=step_class(RegStep::Referral)
                            data-step="1"
                            id="referralStep"
                        >
                            <div class="step-header">
                                <h2 class="x_large_font">
                                    "Welcome to " <strong>"peer!"</strong>
                                </h2>
                                <p class="large_font">
                                    "One quick step left! Enter your referral code to complete registration."
                                </p>
                            </div>
                            // Referral form placeholder — Step 6 will replace this
                            <p class="medium_font">"[Referral code form — Step 6]"</p>
                        </div>

                        // Step 1b: Default Referral Code
                        <div
                            class=step_class(RegStep::DefaultReferral)
                            data-step="1b"
                            id="defaultReferralStep"
                        >
                            <div class="step-header">
                                <h2 class="x_large_font">"Claim Your Invitation"</h2>
                                <p class="large_font">
                                    "Earning starts the moment you enter this magic code"
                                </p>
                            </div>
                            // Default referral display placeholder — Step 6 will replace this
                            <p class="medium_font">"[Default referral code — Step 6]"</p>
                        </div>

                        // Step 2: Registration Form
                        <div
                            class=step_class(RegStep::Register)
                            data-step="2"
                            id="registrationStep"
                        >
                            <div class="step-header">
                                <h2 class="x_large_font">"Register"</h2>
                                <p class="large_font">
                                    "Create your account in few seconds and start earning on your favorite content."
                                </p>
                            </div>
                            // Registration form placeholder — Step 8 will replace this
                            <p class="medium_font">"[Registration form — Step 8]"</p>
                        </div>

                        // Step 3: Success
                        <div
                            class=step_class(RegStep::Success)
                            data-step="3"
                            id="successStep"
                        >
                            <div class="success-message">
                                <div class="step-header">
                                    <span class="icon" aria-hidden="true">
                                        <i class="peer-icon peer-icon-good-tick-circle"></i>
                                    </span>
                                    <h2 class="x_large_font">
                                        "Welcome to " <strong>"peer!"</strong>
                                    </h2>
                                    <p class="large_font">
                                        "Your account is ready! Start exploring and earn your first token today."
                                    </p>
                                </div>
                                // Login link placeholder — Step 10 will replace this
                                <p class="medium_font">"[Continue to Login — Step 10]"</p>
                            </div>
                        </div>
                    </div>

                    // ── Footer area ─────────────────────────────────────
                    <div class="footer_area medium_font">
                        <p class="version version-number"></p>
                    </div>
                </div>
            </div>
        </div>
    }
}
```

### What this mirrors from `register.php`

| PHP element | Leptos equivalent | Notes |
|-------------|-------------------|-------|
| `<div class="container large_font">` | `<div class="container large_font">` | Identical class names |
| `div.container_left > .phone > .screen > img` | Same nesting | Image path changed to `/img/register.webp` (served from `public/`) |
| `div.container_right > .container_inner` | Same nesting | Holds back button + form steps + footer |
| `#backBtn` as `<a href="login.php">` | `<a on:click=on_back>` | Reactive — navigates based on current step |
| `div.form-step.active` toggled by JS | `class=step_class(RegStep::*)` | Reactive CSS class; `.active` toggled by `current_step` signal |
| PHP cache-busting `?<?php echo filemtime(...)?>` | Not needed; `cargo-leptos` hashes CSS filenames | Asset hashing is handled by the build tool |

---

## 5.7 — Update `src/pages/mod.rs`

Ensure the `RegStep` enum is also exported for use in later steps:

```rust
//! Page components (one per route).

pub mod register;

pub use register::{RegisterPage, RegStep};
```

---

## 5.8 — Add Dependencies

### 5.8.1 — Update `Cargo.toml`

Ensure the following dependencies are present:

```toml
[dependencies]
leptos = { version = "0.7", features = ["nightly"] }
leptos_meta = { version = "0.7" }
leptos_router = { version = "0.7" }
# ... other existing deps ...

[dependencies.web-sys]
version = "0.3"
features = ["Window", "Location"]
optional = true

[features]
hydrate = ["leptos/hydrate", "dep:web-sys"]
ssr = ["leptos/ssr", "leptos_meta/ssr", "leptos_router/ssr"]
```

> **Note:** `web-sys` is needed for the back-button handler's `window.location.set_href("/login")` fallback. It's gated behind the `hydrate` feature so it's only compiled into the WASM bundle.

---

## 5.9 — Verify SSR Output

A key requirement of this step is confirming the page is **server-side rendered** — the initial HTML response must contain the full page structure, not just a blank `<body>` that WASM populates.

### 5.9.1 — Check SSR with `curl`

```bash
curl -s http://localhost:3000/register | head -80
```

**Expected output should contain:**

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>Peer Network - Register</title>
    <!-- ... stylesheets ... -->
</head>
<body>
    <!-- ... -->
    <div class="container large_font">
        <div class="container_left">
            <div class="phone">
                <div class="screen">
                    <img src="/img/register.webp" .../>
                </div>
                <!-- ... -->
            </div>
            <!-- ... -->
        </div>
        <div class="container_right">
            <div class="container_inner">
                <div class="top_head_area">
                    <a class="btn btn-secondary back-btn" ...>Back</a>
                </div>
                <div class="center_area">
                    <div class="form-step active" data-step="1" id="referralStep">
                        <!-- ... content ... -->
                    </div>
                    <div class="form-step" data-step="1b" ...>
                        <!-- ... -->
                    </div>
                    <!-- ... -->
                </div>
            </div>
        </div>
    </div>
</body>
</html>
```

**Key assertions:**

1. The `<title>` is "Peer Network - Register" (set via `<Title>`)
2. The `.container` div is present in the raw HTML (not injected by JS)
3. Step 1 (`data-step="1"`) has class `form-step active`
4. Steps 1b, 2, 3 have class `form-step` (no `active`)
5. The back button `<a>` with class `back-btn` is present

### 5.9.2 — Confirm SSR is not a blank shell

```bash
# Count lines containing "form-step" — should be 4 (one per step)
curl -s http://localhost:3000/register | grep -c "form-step"
# Expected: 4

# Confirm "container_left" is in the HTML (phone mockup rendered server-side)
curl -s http://localhost:3000/register | grep -c "container_left"
# Expected: 1
```

---

## 5.10 — Verify Query Parameter Pre-fill

### 5.10.1 — Manual browser test

Navigate to:

```
http://localhost:3000/register?ref=85d5f836-b1f5-4c4e-9381-1b058e13df93
```

Open the browser console and verify:

```javascript
// After hydration, the referral_code signal should be set.
// (In later steps, this will auto-fill the input field.)
// For now, just confirm no console errors on load.
```

**Expected:** The page loads without errors. The `?ref=` parameter is read (verified by adding a temporary `log!()` in the `Effect` or by inspecting the component in the Leptos devtools extension).

### 5.10.2 — Test with no query parameter

Navigate to:

```
http://localhost:3000/register
```

**Expected:** The page loads normally. Step 1 (referral) is shown. No errors in console.

---

## 5.11 — Unit Tests

Create tests for the `RegStep` enum logic in `src/pages/register.rs`. Append to the file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ── RegStep::data_step ──────────────────────────────────────────────

    #[test]
    fn test_data_step_values() {
        assert_eq!(RegStep::Referral.data_step(), "1");
        assert_eq!(RegStep::DefaultReferral.data_step(), "1b");
        assert_eq!(RegStep::Register.data_step(), "2");
        assert_eq!(RegStep::Success.data_step(), "3");
    }

    // ── RegStep::show_back_button ───────────────────────────────────────

    #[test]
    fn test_back_button_visible_on_step_1() {
        assert!(RegStep::Referral.show_back_button());
    }

    #[test]
    fn test_back_button_visible_on_step_1b() {
        assert!(RegStep::DefaultReferral.show_back_button());
    }

    #[test]
    fn test_back_button_visible_on_step_2() {
        assert!(RegStep::Register.show_back_button());
    }

    #[test]
    fn test_back_button_hidden_on_step_3() {
        assert!(!RegStep::Success.show_back_button());
    }

    // ── RegStep::previous ───────────────────────────────────────────────

    #[test]
    fn test_previous_from_referral_is_none() {
        // Back from step 1 → navigate to /login (external)
        assert_eq!(RegStep::Referral.previous(), None);
    }

    #[test]
    fn test_previous_from_default_referral() {
        assert_eq!(RegStep::DefaultReferral.previous(), Some(RegStep::Referral));
    }

    #[test]
    fn test_previous_from_register() {
        assert_eq!(RegStep::Register.previous(), Some(RegStep::Referral));
    }

    #[test]
    fn test_previous_from_success_is_none() {
        assert_eq!(RegStep::Success.previous(), None);
    }
}
```

### Run the tests

```bash
cd peer-web
cargo test --lib -- pages::register::tests
```

**Expected:** All 8 tests pass:

```
test pages::register::tests::test_data_step_values ... ok
test pages::register::tests::test_back_button_visible_on_step_1 ... ok
test pages::register::tests::test_back_button_visible_on_step_1b ... ok
test pages::register::tests::test_back_button_visible_on_step_2 ... ok
test pages::register::tests::test_back_button_hidden_on_step_3 ... ok
test pages::register::tests::test_previous_from_referral_is_none ... ok
test pages::register::tests::test_previous_from_default_referral ... ok
test pages::register::tests::test_previous_from_register ... ok
test pages::register::tests::test_previous_from_success_is_none ... ok
```

---

## 5.12 — Commands to Verify

### 5.12.1 — Compilation check

```bash
cd peer-web
cargo check --features ssr
cargo check --features hydrate
```

**Expected:** Both feature-gated compilations succeed with no errors. The `web-sys` import only compiles under `hydrate`.

### 5.12.2 — Run unit tests

```bash
cargo test --lib -- pages::register
```

**Expected:** All `RegStep` tests pass.

### 5.12.3 — Dev server smoke test

```bash
cargo leptos watch
```

Then in another terminal:

```bash
# 1. Page renders
curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/register
# Expected: 200

# 2. SSR content present
curl -s http://localhost:3000/register | grep -q "container large_font" && echo "OK: container present" || echo "FAIL"
# Expected: "OK: container present"

# 3. Step 1 is active
curl -s http://localhost:3000/register | grep -q 'form-step active' && echo "OK: active step present" || echo "FAIL"
# Expected: "OK: active step present"

# 4. Title is set
curl -s http://localhost:3000/register | grep -q "Peer Network - Register" && echo "OK: title set" || echo "FAIL"
# Expected: "OK: title set"

# 5. Phone mockup image tag present
curl -s http://localhost:3000/register | grep -q "register.webp" && echo "OK: image present" || echo "FAIL"
# Expected: "OK: image present"
```

### 5.12.4 — Browser verification

Open `http://localhost:3000/register` in a browser:

| Check | Expected |
|-------|----------|
| Page displays without blank flash | Container loads immediately (SSR), then hydrates |
| Left panel shows phone mockup with image | `register.webp` visible inside phone frame |
| Animated gradient background on left panel | CSS animation working |
| Right panel shows "Welcome to peer!" heading | Step 1 content visible |
| Only step 1 is visible | Steps 1b, 2, 3 are hidden (`display: none`) |
| Back button says "Back" with arrow icon | Peer icon font rendering |
| No console errors | Check DevTools console |

---

## 5.13 — Files Changed Summary

| File | Action | Purpose |
|------|--------|---------|
| `src/app.rs` | **Update** | Add `/register` route, import `RegisterPage` |
| `src/pages/register.rs` | **Update** | Replace stub with full page shell component + `RegStep` enum |
| `src/pages/mod.rs` | **Update** | Export `RegStep` alongside `RegisterPage` |
| `style/login-register.css` | **Create** (copy) | Registration page styles from PHP app |
| `style/main.scss` | **Update** | `@import` the login-register CSS and font stylesheets |
| `public/fonts/` | **Create** (copy) | Poppins and Peer icon fonts |
| `public/img/register.webp` | **Create** (copy) | Phone mockup image |
| `public/svg/*.svg` | **Create** (copy) | Logo and background glow SVGs |
| `Cargo.toml` | **Update** | Add `web-sys` optional dependency with `Window` + `Location` features |

---

## 5.14 — Common Issues & Solutions

| Issue | Cause | Solution |
|-------|-------|----------|
| Page is blank, content appears after JS loads | SSR not working; app is client-only rendering | Confirm `shell()` is used in the Axum setup (`src/main.rs`); run with `cargo leptos watch` not `trunk serve` |
| CSS not applied / unstyled page | `login-register.css` not bundled | Check `style/main.scss` has the `@import`; verify `cargo leptos watch` output mentions the CSS file |
| Peer icons show as empty boxes | Icon font files not found | Check `public/fonts/peer-icon-font/` exists and CSS `@font-face` paths are correct (no `../` prefix) |
| Phone mockup image missing | `register.webp` not in `public/img/` | Copy with `cp img/register.webp peer-web/public/img/register.webp` |
| `RwSignal` not found | Wrong Leptos import | Use `use leptos::prelude::*;` which re-exports `RwSignal` |
| `use_query_map` not found | Missing `leptos_router` import | Add `use leptos_router::hooks::use_query_map;` |
| `web_sys::window()` error during SSR | `web-sys` called on server | Ensure the `window.location` code is inside `#[cfg(feature = "hydrate")]` |
| Gradient animation stutters | CSS `will-change` property | Already present in `login-register.css`; ensure no conflicting resets |
| `path!` macro not recognized | `leptos_router` version mismatch | Use `leptos_router = "0.7"` or match your Leptos version; older versions use `path="/register"` string syntax |
| WASM hydration mismatch warning | SSR HTML differs from hydrated output | Ensure `step_class` returns the same initial value on both server and client (Step 1 = `Referral` is the default) |

---

## 5.15 — What Comes Next

With the router and page shell in place, the following steps can now proceed:

| Next step | What it adds | Depends on |
|-----------|-------------|------------|
| **Step 6** — Referral Code Entry UI | Replaces the Step 1 placeholder with the referral input form, client-side UUID validation, and the "Don't have a code?" link | Step 5 (this step) |
| **Step 7** — Referral Verification | Wires the "Verify Code" button to `verify_referral` server function | Steps 4 + 6 |
| **Step 8** — Registration Form UI | Replaces the Step 2 placeholder with email/username/password fields and client-side validation | Step 5 (this step) |
| **Step 11** — Navigation & Back Button | Enhances the back-button with browser history integration and focus management | Steps 6 + 8 + 10 |

> **Note:** Steps 6 and 8 can be worked on **in parallel** since they modify different step `div`s within the shell established here.
