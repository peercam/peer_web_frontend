# Step 14 — CSS & Visual Parity

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Achieve pixel-level visual parity between the Leptos registration page (`/register`) and the existing PHP registration page (`register.php`). Every element — fonts, icons, colours, spacing, animations, responsive breakpoints, and decorative assets — must render identically so that a user cannot distinguish the two implementations.

---

## 14.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Steps 6–12 complete | `cargo leptos build` | Compiles; referral, registration form, success, navigation, and toast components all render |
| Step 13 (a11y) complete or in-progress | `axe-core` audit | No critical violations — CSS work may adjust markup so both steps can run in parallel |
| Mock backend running | `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` | Returns `{"data":{"_health":true}}` |
| PHP version accessible | Open `register.php` in browser (e.g. via `php -S localhost:8080`) | Renders the full 3-step registration flow for visual comparison |
| Screenshot tooling installed | `npx backstopjs --version` or `npx percy --version` | Returns a version number (see §14.11 for setup) |

---

## 14.2 — Architecture Overview

```
peer-web/
├── public/                          ← cargo-leptos copies everything here to target/site/
│   ├── fonts/
│   │   ├── font-poppins/            ← symlink or copy of ../../fonts/font-poppins/
│   │   │   ├── Poppins-Regular.woff2
│   │   │   ├── Poppins-Bold.woff2
│   │   │   ├── ... (all weights)
│   │   │   └── stylesheet.css
│   │   └── peer-icon-font/          ← symlink or copy of ../../fonts/peer-icon-font/
│   │       ├── css/
│   │       │   └── peer-network.css
│   │       └── font/
│   │           ├── peer-network.woff2
│   │           └── ...
│   ├── img/
│   │   └── register.webp            ← phone mockup image
│   ├── svg/
│   │   ├── logo_sw.svg              ← black/white logo (phone home button)
│   │   ├── logo_farbe.svg           ← colour logo (floating on gradient)
│   │   ├── blueglow.svg             ← background decoration (container)
│   │   └── blueglow1.svg            ← background decoration (container_right)
│   └── favicon.ico
├── style/
│   └── main.scss                    ← Leptos entry stylesheet
└── src/
    └── pages/
        └── register.rs              ← Leptos view! markup with correct CSS classes
```

### Strategy

The approach is **import first, adapt minimally**:

1. **Copy static assets** (fonts, images, SVGs) into `public/` so they are served by cargo-leptos
2. **Import `login-register.css` verbatim** as a starting point — no rewrites
3. **Fix asset paths** inside the imported CSS (relative path adjustments for the new directory structure)
4. **Map every CSS class** from `register.php` onto the corresponding Leptos `view!` element
5. **Verify at every responsive breakpoint** — fix any differences caused by markup order changes
6. **Add the 4 animations** (`gradient-animation`, `tock`, `fadeIn`, `spin`) — they require no JS changes
7. **Automated screenshot regression** to catch regressions going forward

---

## 14.3 — Sub-task Breakdown

| # | Sub-task | Section |
|---|----------|---------|
| 14.3.1 | Copy font assets into `public/fonts/` | §14.4 |
| 14.3.2 | Copy image & SVG assets into `public/img/` and `public/svg/` | §14.4 |
| 14.3.3 | Import font stylesheets into Leptos | §14.5 |
| 14.3.4 | Import `login-register.css` with path adjustments | §14.6 |
| 14.3.5 | Map CSS classes to Leptos `view!` markup (all 3 steps + step 1b) | §14.7 |
| 14.3.6 | Verify & fix animations | §14.8 |
| 14.3.7 | Verify & fix responsive breakpoints | §14.9 |
| 14.3.8 | Verify password strength meter colours | §14.10 |
| 14.3.9 | Set up automated screenshot comparison | §14.11 |
| 14.3.10 | Final visual QA at 375px, 768px, 1440px | §14.12 |

---

## 14.4 — Asset Provisioning

### 14.4.1 — Font Files

The Poppins font family and the Peer icon font must be available under `public/` so that cargo-leptos serves them from `target/site/`.

**Option A — Symlinks (development, recommended):**

```bash
cd peer-web/public
mkdir -p fonts
ln -s ../../../fonts/font-poppins fonts/font-poppins
ln -s ../../../fonts/peer-icon-font fonts/peer-icon-font
```

**Option B — Copy (CI / production):**

```bash
cp -r ../../fonts/font-poppins peer-web/public/fonts/font-poppins
cp -r ../../fonts/peer-icon-font peer-web/public/fonts/peer-icon-font
```

> **Note:** Symlinks keep a single source of truth and avoid drift. The CI build script should use `cp -r` since symlinks may not survive Docker layers.

### 14.4.2 — Images & SVGs

```bash
cd peer-web/public
mkdir -p img svg

# Phone mockup image
cp ../../img/register.webp img/register.webp

# SVGs used by the registration page
cp ../../svg/logo_sw.svg svg/logo_sw.svg
cp ../../svg/logo_farbe.svg svg/logo_farbe.svg
cp ../../svg/blueglow.svg svg/blueglow.svg
cp ../../svg/blueglow1.svg svg/blueglow1.svg
```

### 14.4.3 — Verification

After provisioning, the directory should look like:

```
public/
├── favicon.ico
├── fonts/
│   ├── font-poppins/        ← 90 font files + stylesheet.css
│   └── peer-icon-font/
│       ├── css/peer-network.css
│       └── font/peer-network.{eot,svg,ttf,woff,woff2}
├── img/
│   └── register.webp
└── svg/
    ├── blueglow.svg
    ├── blueglow1.svg
    ├── logo_farbe.svg
    └── logo_sw.svg
```

**Test:**

```bash
cargo leptos build
ls target/site/fonts/font-poppins/Poppins-Regular.woff2  # should exist
ls target/site/fonts/peer-icon-font/font/peer-network.woff2  # should exist
ls target/site/img/register.webp  # should exist
ls target/site/svg/blueglow.svg  # should exist
```

---

## 14.5 — Font Stylesheet Imports

### 14.5.1 — Poppins Font

The Poppins stylesheet (`fonts/font-poppins/stylesheet.css`) uses relative `url(...)` paths like `url('Poppins-Regular.woff2')`. Since the stylesheet lives next to the font files inside `public/fonts/font-poppins/`, these paths will resolve correctly when served from `target/site/`.

Add the `<link>` tag to the Leptos shell in `src/app.rs`:

```rust
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>

                // Font stylesheets
                <link rel="stylesheet" href="/fonts/font-poppins/stylesheet.css"/>
                <link rel="stylesheet" href="/fonts/peer-icon-font/css/peer-network.css"/>

                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
```

### 14.5.2 — Peer Icon Font

`fonts/peer-icon-font/css/peer-network.css` references the font files via `url('../font/peer-network.woff2?48334736')`. Since `css/` and `font/` are siblings under `public/fonts/peer-icon-font/`, the relative paths resolve correctly.

**Verification:** After `cargo leptos build`, open DevTools → Network → filter by "font" and confirm both `Poppins-*.woff2` and `peer-network.woff2` load with HTTP 200.

---

## 14.6 — Import `login-register.css`

### 14.6.1 — Copy the CSS

```bash
cp css/login-register.css peer-web/style/login-register.css
```

### 14.6.2 — Path Adjustments

The original CSS references assets using `../svg/` and `../img/` paths relative to the `css/` folder. In the Leptos project, the compiled CSS is served from `/pkg/peer-web.css`, so relative paths like `../svg/` would resolve to `/svg/` — which is correct because `target/site/svg/` is where cargo-leptos copies the SVGs.

However, the `style/` source folder is compiled by cargo-leptos (via dart-sass & Lightning CSS), and `url()` references are resolved relative to the output location. We need to use **absolute paths** instead:

```css
/* BEFORE (in original login-register.css) */
background: url('../svg/blueglow.svg') no-repeat top right;
/* ... */
background: url('../svg/blueglow1.svg') no-repeat bottom left;

/* AFTER (in style/login-register.css) */
background: url('/svg/blueglow.svg') no-repeat top right;
/* ... */
background: url('/svg/blueglow1.svg') no-repeat bottom left;
```

There are exactly **2 `url()` references** in `login-register.css` that need updating:

| Line | Original | Updated |
|------|----------|---------|
| `.container` | `url('../svg/blueglow.svg')` | `url('/svg/blueglow.svg')` |
| `.container_right` | `url('../svg/blueglow1.svg')` | `url('/svg/blueglow1.svg')` |

### 14.6.3 — Import from `main.scss`

Update `style/main.scss` to import the registration CSS:

```scss
@import 'login-register.css';

body {
    font-family: "Poppins", sans-serif;
    text-align: center;
}
```

> **Important:** Remove the duplicate `body` rule from `main.scss` — the rule in `login-register.css` (`font-family: "Poppins", sans-serif; line-height: 1.3; min-height: 100vh; margin: 0;`) is the authoritative one. The existing `main.scss` body rule sets `text-align: center` which would conflict.

**Updated `style/main.scss`:**

```scss
@import 'login-register.css';
```

That's it — `login-register.css` already sets all body styles.

### 14.6.4 — Verification

```bash
cargo leptos build
# Inspect the built CSS:
grep 'blueglow' target/site/pkg/peer-web.css
# Should show: url(/svg/blueglow.svg) and url(/svg/blueglow1.svg)
```

---

## 14.7 — CSS Class Mapping to Leptos Markup

This is the core of visual parity: every CSS class applied in `register.php` must appear on the corresponding Leptos element. Below is the complete class mapping for every element in the registration page.

### 14.7.1 — Outer Container & Left Panel

**PHP (`register.php`):**
```html
<div class="container large_font">
    <div class="container_left">
        <div class="phone">
            <div class="screen">
                <img src="img/register.webp" alt="Login" width="612" height="612">
            </div>
            <div class="home-button">
                <img src="svg/logo_sw.svg" alt="PeerLogo" width="96" height="96">
            </div>
        </div>
        <img class="logo" src="svg/logo_farbe.svg" alt="Peer logo" width="96" height="96" />
    </div>
    <div class="container_right">
        <div class="container_inner">
            ...
        </div>
    </div>
</div>
```

**Leptos (`src/pages/register.rs`):**
```rust
view! {
    <div class="container large_font">
        <div class="container_left">
            <div class="phone">
                <div class="screen">
                    <img src="/img/register.webp" alt="Login" width="612" height="612"/>
                </div>
                <div class="home-button">
                    <img src="/svg/logo_sw.svg" alt="PeerLogo" width="96" height="96"/>
                </div>
            </div>
            <img class="logo" src="/svg/logo_farbe.svg" alt="Peer logo" width="96" height="96"/>
        </div>
        <div class="container_right">
            <div class="container_inner">
                // ... top_head_area, center_area, footer_area
            </div>
        </div>
    </div>
}
```

> **Key difference:** Asset paths use absolute `/img/...` and `/svg/...` (Leptos serves from site root), not relative `img/...`.

### 14.7.2 — Top Head Area (Back Button)

**PHP:**
```html
<div class="top_head_area">
    <a href="login.php" class="btn btn-secondary back-btn" id="backBtn">
        <span aria-hidden="true"><i class="peer-icon medium_font peer-icon-arrow-left"></i></span>
        Back
    </a>
</div>
```

**Leptos:**
```rust
<div class="top_head_area">
    <a href="/login" class="btn btn-secondary back-btn" id="backBtn"
       on:click=handle_back>
        <span aria-hidden="true">
            <i class="peer-icon medium_font peer-icon-arrow-left"></i>
        </span>
        "Back"
    </a>
</div>
```

**Classes to verify:** `top_head_area`, `btn`, `btn-secondary`, `back-btn`, `peer-icon`, `medium_font`, `peer-icon-arrow-left`

### 14.7.3 — Step 1: Referral Code Entry

**Critical classes:**

| Element | Classes |
|---------|---------|
| Step container | `form-step active` (dynamically toggled via signal) |
| Header wrapper | `step-header` |
| Title | `x_large_font` |
| Subtitle | `large_font` |
| Input group | `input-group` |
| Input field wrapper | `input-field` + dynamic `valid` / `invalid` |
| Icon span | `input-icon` |
| Icon `<i>` | `peer-icon peer-icon-referral` |
| Validation icon span | `validation-icon` + dynamic `show` |
| Validation icon `<i>` | `peer-icon peer-icon-tick-circle` |
| Validation message | `validation-message medium_font` + dynamic `valid` |
| SR-only help | `sr-only` |
| Submit button | `btn btn-primary` |
| Step footer | `step-footer medium_font` |

**Dynamic class toggling in Leptos** (example for `input-field`):

```rust
<div
    class=move || {
        let mut c = String::from("input-field");
        match validation_state.get() {
            ValidationState::Valid => c.push_str(" valid"),
            ValidationState::Invalid => c.push_str(" invalid"),
            ValidationState::Neutral => {}
        }
        c
    }
>
```

### 14.7.4 — Step 1b: Default Referral Code

**Critical classes:**

| Element | Classes |
|---------|---------|
| Step container | `form-step` (add `active` when shown) |
| Referral display | `referral-code-display medium_font` |
| Icon | `input-icon` → `peer-icon peer-icon-referral` |
| Button | `btn btn-primary` |

### 14.7.5 — Step 2: Registration Form

**Critical classes (in addition to shared ones from §14.7.3):**

| Element | Classes |
|---------|---------|
| Email icon | `peer-icon peer-icon-envelope` |
| Username icon | `peer-icon peer-icon-user` |
| Password icon | `peer-icon peer-icon-lock` |
| Password toggle | `toggle-passwordBtn-icon` → `peer-icon peer-icon-eye-close` / `peer-icon-eye-open` |
| Password strength wrapper | `password-strength` + `none` (hidden initially) |
| Strength labels | `strength-labels medium_font` |
| Each label span | `strength-text` + `very-weak`/`weak`/`improvement`/`good`/`excellent` + dynamic `active` |
| Strength meter | `strength-meter` |
| Strength fill | `strength-fill` + dynamic `weak`/`weak2`/`medium`/`strong`/`excellent` |
| Each segment | `strength-segment segment-weak` / `segment-weak2` / `segment-medium` / `segment-strong` / `segment-excellent` |
| Requirements list | `strength-requirements medium_font` with `role="list"` |
| Each req `<li>` | dynamic `met` class |
| Confirm password toggle | `toggle-passwordBtn-icon` → `peer-icon peer-icon-eye-close` |
| Checkbox wrapper | `checkbox-field` → `checkbox-wrapper` |
| Checkbox label | `checkbox-label medium_font` |
| Checkbox validation | `validation-message medium_font` |
| Submit button | `btn btn-primary` |
| Already registered | `already_register medium_font` |

### 14.7.6 — Step 3: Success

**Critical classes:**

| Element | Classes |
|---------|---------|
| Step container | `form-step` + dynamic `active` |
| Success wrapper | `success-message` |
| Header | `step-header` (centred via `.success-message .step-header { text-align: center }`) |
| Icon span | `icon` |
| Icon `<i>` | `peer-icon peer-icon-good-tick-circle` (green via `.success-message .icon { color: var(--Green-Accent) }`) |
| Title | `x_large_font` |
| Subtitle | `large_font` |
| Login button | `btn btn-primary` (as an `<a>` tag) |

### 14.7.7 — Footer

**PHP:**
```html
<div class="footer_area medium_font">
    <p class="version version-number"></p>
</div>
```

**Leptos:**
```rust
<div class="footer_area medium_font">
    <p class="version version-number">{version_string}</p>
</div>
```

### 14.7.8 — Utility Classes

These utility classes appear throughout and must be mapped correctly:

| Class | Purpose | CSS Rule |
|-------|---------|----------|
| `none` | `display: none` | General hide |
| `show` | `display: block` | General show |
| `loading` | Disables pointer events, shows spinner pseudo-element | Applied to button during async requests |
| `sr-only` | Visually hidden, screen-reader only | `position: absolute; width: 1px; height: 1px; ...` |

---

## 14.8 — Animations

There are **4 keyframe animations** in `login-register.css` that must work identically in the Leptos version:

### 14.8.1 — `gradient-animation` (Left Panel Background)

```css
@keyframes gradient-animation {
    0%   { background-position: 0% 0%; }
    50%  { background-position: 100% 100%; }
    100% { background-position: 0% 0%; }
}
```

Applied to `.container_left` via `animation: gradient-animation 10s ease infinite`. This is a pure CSS animation — **no Leptos code needed**. It will work automatically as long as `.container_left` has the gradient background.

**Verify:** The left panel gradient shifts smoothly in a loop.

### 14.8.2 — `tock` (Floating Logo)

```css
@keyframes tock {
    0%   { transform: translateX(0) rotate(0deg); }
    50%  { transform: translateX(195%) rotate(360deg); }
    100% { transform: translateX(0) rotate(0deg); }
}
```

Applied to `.container_left .logo` via `animation: tock 6s ease-out 2.14s forwards`. The `will-change: transform` property is also set.

**Verify:** The colour Peer logo on the left panel bounces right and rotates, then returns, starting ~2s after page load.

### 14.8.3 — `fadeIn` (Step Transitions)

```css
@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to   { opacity: 1; transform: translateY(0); }
}
```

Applied to `.form-step.active` via `animation: fadeIn 0.3s ease-in`. This fires whenever a step becomes active.

**Leptos consideration:** The `active` class is toggled reactively. When the signal changes, Leptos updates the class attribute and the browser triggers the animation. **No special handling needed** — CSS animations replay on class change.

**Verify:** Transitioning between steps shows a subtle fade-up animation.

### 14.8.4 — `spin` (Loading Spinner)

```css
@keyframes spin {
    0%   { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}
```

Applied to `.loading::after` pseudo-element. Added to buttons during async server calls.

**Leptos implementation:** Toggle the `loading` class on the button when an action is pending:

```rust
<button
    class=move || {
        let mut c = String::from("btn btn-primary");
        if action.pending().get() {
            c.push_str(" loading");
        }
        c
    }
    disabled=move || action.pending().get()
>
    "Create Account"
</button>
```

**Verify:** While a server function is in-flight, the button shows a spinning white circle and disables pointer events.

---

## 14.9 — Responsive Breakpoint Verification

The CSS defines **6 responsive breakpoints**. Each must be verified for both the PHP and Leptos versions.

### 14.9.1 — Breakpoint Matrix

| Breakpoint | Key Changes | Verify |
|------------|-------------|--------|
| `≤ 3000px` | Grid 50/50, reduced font sizes, input line-height 70px, button line-height 70px | Layout splits evenly |
| `≤ 2000px` | `max-width: 600px` on `container_inner`, smaller back button, input line-height 60px | Form area narrows |
| `≤ 1200px` | Grid 40/60, step-header margin 1.2rem, input line-height 50px | Left panel shrinks |
| `≤ 1024px` | Grid 35/65, phone min-width 270px, smaller back button | Phone mockup downsizes |
| `≤ 980px` | Grid 10/90, phone hidden, left panel loses border-radius, fixed font sizes (28/16/14px) | Phone disappears, near-fullscreen form |
| `≤ 600px` | Strength requirements font 10px, gap 3px | Tiny text for password hints |
| `≤ 500px` | Grid 5/95, reduced padding, logo resized | Near-edge-to-edge on mobile |

### 14.9.2 — Test Procedure

For each breakpoint, resize the browser (or use DevTools responsive mode) and verify:

1. **Grid proportions** — left panel and form panel match PHP version
2. **Font sizes** — headings, body text, and button text scale identically
3. **Input sizing** — field height (line-height) and padding match
4. **Phone mockup** — visible/hidden at correct breakpoint, sizing correct
5. **Password strength meter** — segment widths and gap sizing correct
6. **Checkbox sizing** — checkbox dimensions and checkmark sizing correct
7. **Back button** — line-height and padding adjustments correct

### 14.9.3 — Common Pitfalls

| Issue | Cause | Fix |
|-------|-------|-----|
| Font sizes don't match | `clamp()` values depend on viewport width — ensure no conflicting `font-size` from `main.scss` | Remove any body `font-size` override from `main.scss` |
| Phone mockup position off | The `.phone` has `position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%)` — ensure no parent has unexpected `position: relative` | Check the Leptos `<main>` wrapper doesn't add positioning |
| Inputs taller in Leptos | The Leptos `<Router>` / `<main>` wrapper may add padding | Ensure `<main>` has no default styles; or bypass it for the register route |
| Background glow not visible | `background-size: contain` on `.container` and `.container_right` — verify SVG files are loading | Check network tab for 404s on `blueglow.svg` / `blueglow1.svg` |

---

## 14.10 — Password Strength Meter Colours

The strength meter is a critical visual element with 5 states. Each state highlights different segments with specific colours.

### 14.10.1 — State → Segment Colour Matrix

| Strength State | `.strength-fill` class | `segment-weak` | `segment-weak2` | `segment-medium` | `segment-strong` | `segment-excellent` |
|----------------|------------------------|-----------------|------------------|-------------------|-------------------|---------------------|
| Very Weak | `weak` | `var(--Red-Accent)` | default | default | default | default |
| Weak | `weak2` | `var(--Red-Accent)` | `var(--Red-Accent)` | default | default | default |
| Needs Improvement | `medium` | `#F7931A` | `#F7931A` | `#F7931A` | default | default |
| Good | `strong` | `var(--Green-Accent)` | `var(--Green-Accent)` | `var(--Green-Accent)` | `var(--Green-Accent)` | default |
| Excellent | `excellent` | `var(--Green-Accent)` | `var(--Green-Accent)` | `var(--Green-Accent)` | `var(--Green-Accent)` | `var(--Green-Accent)` |

Default segment colour: `var(--White-secondary)` (`rgba(255, 255, 255, 0.50)`)

### 14.10.2 — Strength Label Colours

The `.strength-labels` container shows one label at a time. Each label has a colour:

| Label | Class | Colour |
|-------|-------|--------|
| Very weak | `.very-weak` | `var(--Red-Accent)` (#FF3B3B) |
| Weak | `.weak` | `var(--Red-Accent)` (#FF3B3B) |
| Needs improvement | `.improvement` | `#F7931A` (orange) |
| Good | `.good` | `var(--Green-Accent)` (#AAFF67) |
| Excellent | `.excellent` | `var(--Green-Accent)` (#AAFF67) |

Only the label with the `.active` class is visible (opacity 1). All others are hidden.

### 14.10.3 — Leptos Implementation

```rust
// Derive the strength class name from the password score
let strength_class = Memo::new(move |_| {
    match password_score.get() {
        0 => "",
        1 => "weak",
        2 => "weak2",
        3 => "medium",
        4 => "strong",
        5 => "excellent",
        _ => "",
    }
});

// In the view:
<div class=move || format!("strength-fill {}", strength_class.get())>
    <span class="strength-segment segment-weak"></span>
    <span class="strength-segment segment-weak2"></span>
    <span class="strength-segment segment-medium"></span>
    <span class="strength-segment segment-strong"></span>
    <span class="strength-segment segment-excellent"></span>
</div>
```

**Verify:** Type passwords of increasing complexity and confirm:
1. `a` → 1 red segment, "Very weak" label in red
2. `ab` → 2 red segments, "Weak" label in red
3. `Abcd1` → 3 orange segments, "Needs improvement" in orange
4. `Abcd1234` → 4 green segments, "Good" in green
5. `Abcd1234!@` → 5 green segments, "Excellent" in green

---

## 14.11 — Automated Screenshot Comparison

### 14.11.1 — Tool Choice

Use **BackstopJS** for visual regression testing. It captures screenshots at specified viewports and compares them pixel-by-pixel.

### 14.11.2 — Setup

```bash
cd peer-web/end2end
npm install backstopjs --save-dev
npx backstop init
```

### 14.11.3 — BackstopJS Configuration

Create/update `peer-web/end2end/backstop.json`:

```json
{
  "id": "registration-visual-parity",
  "viewports": [
    { "label": "mobile", "width": 375, "height": 812 },
    { "label": "tablet", "width": 768, "height": 1024 },
    { "label": "desktop-medium", "width": 1440, "height": 900 },
    { "label": "desktop-large", "width": 2560, "height": 1440 }
  ],
  "scenarios": [
    {
      "label": "Step 1 — Referral Code Entry",
      "url": "http://localhost:3000/register",
      "delay": 1000,
      "misMatchThreshold": 0.1,
      "selectors": [".container"]
    },
    {
      "label": "Step 1 — Referral Code Valid",
      "url": "http://localhost:3000/register",
      "delay": 500,
      "onReadyScript": "fill-valid-referral.js",
      "misMatchThreshold": 0.1,
      "selectors": [".container_right"]
    },
    {
      "label": "Step 1b — Default Referral Code",
      "url": "http://localhost:3000/register",
      "delay": 500,
      "onReadyScript": "show-default-referral.js",
      "misMatchThreshold": 0.1,
      "selectors": [".container_right"]
    },
    {
      "label": "Step 2 — Registration Form (Empty)",
      "url": "http://localhost:3000/register",
      "delay": 500,
      "onReadyScript": "navigate-to-step2.js",
      "misMatchThreshold": 0.1,
      "selectors": [".container_right"]
    },
    {
      "label": "Step 2 — Registration Form (Validated)",
      "url": "http://localhost:3000/register",
      "delay": 500,
      "onReadyScript": "fill-registration-form.js",
      "misMatchThreshold": 0.1,
      "selectors": [".container_right"]
    },
    {
      "label": "Step 2 — Password Strength Meter (All States)",
      "url": "http://localhost:3000/register",
      "delay": 500,
      "onReadyScript": "navigate-to-step2.js",
      "misMatchThreshold": 0.1,
      "selectors": [".password-strength"]
    },
    {
      "label": "Step 3 — Success Screen",
      "url": "http://localhost:3000/register",
      "delay": 500,
      "onReadyScript": "navigate-to-step3.js",
      "misMatchThreshold": 0.1,
      "selectors": [".container_right"]
    }
  ],
  "paths": {
    "bitmaps_reference": "backstop_data/bitmaps_reference",
    "bitmaps_test": "backstop_data/bitmaps_test",
    "engine_scripts": "backstop_data/engine_scripts",
    "html_report": "backstop_data/html_report"
  },
  "engine": "playwright",
  "engineOptions": {
    "browser": "chromium"
  },
  "report": ["browser"],
  "debug": false
}
```

### 14.11.4 — Reference Capture Workflow

1. **Capture PHP reference screenshots** by temporarily pointing BackstopJS at the PHP dev server:

```bash
# Start PHP server
cd /path/to/peer_web_frontend
php -S localhost:8080 &

# Update backstop.json URLs to http://localhost:8080/register.php
# Then capture references:
cd peer-web/end2end
npx backstop reference
```

2. **Switch URLs back** to `http://localhost:3000/register` (the Leptos version)

3. **Run comparison:**

```bash
npx backstop test
```

4. **Review report:** Opens an HTML report showing side-by-side diffs. Any pixel difference above the `misMatchThreshold` (0.1%) is flagged.

### 14.11.5 — CI Integration

Add to the E2E test script (step 15):

```bash
# In CI pipeline, after cargo leptos build and starting the server:
cd end2end && npx backstop test --config=backstop.json
```

---

## 14.12 — Final Visual QA Checklist

Perform this checklist manually at **375px**, **768px**, and **1440px** viewport widths. Compare the Leptos version side-by-side with the PHP version.

### 14.12.1 — Global

| # | Check | Pass? |
|---|-------|-------|
| 1 | `Poppins` font loads and renders (check DevTools → Computed → `font-family`) | ☐ |
| 2 | Peer icon font glyphs render (not empty boxes or fallback text) | ☐ |
| 3 | `color-scheme: dark` is active (`:root` custom properties) | ☐ |
| 4 | Background glow SVGs visible on container and container_right | ☐ |
| 5 | Left panel gradient animates (shifts diagonally over 10s) | ☐ |
| 6 | Colour logo bounces/rotates on left panel after ~2s | ☐ |
| 7 | Phone mockup visible at ≥ 980px, hidden below | ☐ |
| 8 | Phone mockup image (`register.webp`) loads inside phone screen | ☐ |
| 9 | B/W logo renders inside phone home button | ☐ |

### 14.12.2 — Step 1: Referral

| # | Check | Pass? |
|---|-------|-------|
| 10 | Step header "Welcome to **peer!**" matches font weight (400 for "Welcome to", 700 italic for "peer!") | ☐ |
| 11 | Input field has rounded pill shape (`border-radius: 75px`) | ☐ |
| 12 | Referral icon (gift box) renders in blue (`var(--Blue)`) | ☐ |
| 13 | Valid input → green border (`var(--Green-Accent)`) + tick icon visible | ☐ |
| 14 | Invalid input → red border (`var(--Red-Accent)`) + red validation message | ☐ |
| 15 | "Verify Code" button matches primary style (blue gradient, white text) | ☐ |
| 16 | "Don't have a code?" footer text is centred, secondary colour | ☐ |
| 17 | Back button has secondary outline style with arrow-left icon | ☐ |

### 14.12.3 — Step 1b: Default Referral

| # | Check | Pass? |
|---|-------|-------|
| 18 | Referral code display has dark background pill shape | ☐ |
| 19 | Referral icon renders beside the UUID text | ☐ |
| 20 | "Use This Code" button matches primary style | ☐ |

### 14.12.4 — Step 2: Registration Form

| # | Check | Pass? |
|---|-------|-------|
| 21 | Email, username, password, confirm-password fields all have pill shape | ☐ |
| 22 | Each field has correct icon (envelope, user, lock, lock) | ☐ |
| 23 | Password toggle icon (eye-close) is visible, clickable, toggles to eye-open | ☐ |
| 24 | Password strength meter hidden initially, appears after typing | ☐ |
| 25 | Strength segments transition through 5 colour states (see §14.10) | ☐ |
| 26 | Strength requirements list ("Min. 8 chars, 1 lowercase, 1 uppercase, 1 number") | ☐ |
| 27 | Met requirements turn green (`var(--Green-Accent)`) | ☐ |
| 28 | Checkboxes custom-styled (no native appearance, white border, fill on check) | ☐ |
| 29 | Privacy Policy and EULA links are underlined, open in new tab | ☐ |
| 30 | "Create Account" button matches primary style | ☐ |
| 31 | "Already registered? Login here" centred, secondary colour | ☐ |

### 14.12.5 — Step 3: Success

| # | Check | Pass? |
|---|-------|-------|
| 32 | Large green tick icon (`peer-icon-good-tick-circle`) renders | ☐ |
| 33 | Icon is ~12rem and `var(--Green-Accent)` coloured | ☐ |
| 34 | "Welcome to **peer!**" centred, same font styling as step 1 | ☐ |
| 35 | "Continue to Login" button matches primary style | ☐ |

### 14.12.6 — Interactions & Transitions

| # | Check | Pass? |
|---|-------|-------|
| 36 | Step transition plays `fadeIn` animation (0.3s fade-up) | ☐ |
| 37 | Button hover state: primary buttons turn `var(--Hover)` blue | ☐ |
| 38 | Button hover state: secondary (back) button fills white with black text | ☐ |
| 39 | Disabled button shows `cursor: not-allowed`, text becomes secondary colour | ☐ |
| 40 | Loading state: button shows spinning circle, disables pointer events | ☐ |
| 41 | Toast notification slides in from right, auto-dismisses after 3s | ☐ |
| 42 | Toast types: success (green bg), error (red bg), info (dark bg) | ☐ |
| 43 | `prefers-reduced-motion: reduce` disables all animations | ☐ |
| 44 | `prefers-contrast: high` increases input border width to 3px | ☐ |

### 14.12.7 — Icon Glyph Inventory

Verify every Peer icon font glyph used in the registration flow renders correctly:

| Icon Class | Used Where | Expected Glyph |
|------------|-----------|-----------------|
| `peer-icon-arrow-left` | Back button | Left arrow |
| `peer-icon-referral` | Referral input, default code display | Gift/referral box |
| `peer-icon-tick-circle` | Validation success | Circled tick |
| `peer-icon-envelope` | Email input | Envelope |
| `peer-icon-user` | Username input | Person silhouette |
| `peer-icon-lock` | Password & confirm password inputs | Padlock |
| `peer-icon-eye-close` | Password toggle (hidden) | Closed eye |
| `peer-icon-eye-open` | Password toggle (visible) | Open eye |
| `peer-icon-good-tick-circle` | Success screen | Large circled tick |

---

## 14.13 — SSR Considerations

| Concern | Approach |
|---------|----------|
| CSS delivered before JS | `login-register.css` is compiled into `peer-web.css` via `style-file` in `Cargo.toml` and `<Stylesheet>` — loads in `<head>` before hydration, so the page renders styled immediately |
| Font loading flash (FOIT/FOUT) | Both font stylesheets use `font-display: swap` — text renders immediately with fallback, then repaints when fonts load (same as PHP version) |
| First-paint layout shift | All layout CSS is in the critical path (loaded via `<link>` in `<head>`) — no CLS expected |
| Dynamic classes on SSR | Leptos evaluates reactive closures on the server for initial render — step 1 will have `class="form-step active"` in the SSR HTML |
| `prefers-reduced-motion` | Media query in CSS — works identically on server-rendered and hydrated pages |
| Animation on SSR | CSS animations specified in stylesheets run as soon as the browser parses the CSS — same behaviour as PHP |

---

## 14.14 — File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `public/fonts/font-poppins/` | **Create (symlink or copy)** | Poppins font files + `stylesheet.css` |
| `public/fonts/peer-icon-font/` | **Create (symlink or copy)** | Peer icon font files + `peer-network.css` |
| `public/img/register.webp` | **Copy** | Phone mockup image |
| `public/svg/logo_sw.svg` | **Copy** | B/W logo for phone home button |
| `public/svg/logo_farbe.svg` | **Copy** | Colour logo for floating animation |
| `public/svg/blueglow.svg` | **Copy** | Background decoration (main container) |
| `public/svg/blueglow1.svg` | **Copy** | Background decoration (right panel) |
| `style/login-register.css` | **Create (copy + edit)** | Registration CSS with 2 `url()` path fixes |
| `style/main.scss` | **Edit** | Replace body rule with `@import 'login-register.css'` |
| `src/app.rs` | **Edit** | Add `<link>` tags for Poppins and peer-icon-font stylesheets; change `lang="en"` to `lang="de"` |
| `src/pages/register.rs` | **Edit** | Ensure all CSS classes from §14.7 are correctly applied to every element |
| `end2end/backstop.json` | **Create** | BackstopJS configuration for visual regression testing |
| `end2end/package.json` | **Edit** | Add `backstopjs` dev dependency |

---

## 14.15 — Testing

### 14.15.1 — Automated: BackstopJS

```bash
# 1. Start both servers
php -S localhost:8080 &                    # PHP reference
cd peer-web && cargo leptos watch &        # Leptos under test

# 2. Capture PHP references (one-time)
cd end2end
# (temporarily set URLs to localhost:8080/register.php)
npx backstop reference

# 3. Switch URLs back to localhost:3000/register
npx backstop test

# 4. Review
# Opens HTML report with side-by-side comparisons
```

**Pass criteria:** All 7 scenarios × 4 viewports = 28 comparisons pass with < 0.1% pixel difference.

### 14.15.2 — Manual: Side-by-Side

1. Open PHP version at `localhost:8080/register.php` in one browser window
2. Open Leptos version at `localhost:3000/register` in another
3. Walk through the QA checklist in §14.12 at each viewport width
4. Record any differences and fix

### 14.15.3 — Automated: Font Loading

```bash
# Verify fonts are served with correct MIME types
curl -sI http://localhost:3000/fonts/font-poppins/Poppins-Regular.woff2 | grep content-type
# Expected: content-type: font/woff2

curl -sI http://localhost:3000/fonts/peer-icon-font/font/peer-network.woff2 | grep content-type
# Expected: content-type: font/woff2
```

### 14.15.4 — Automated: Asset 404 Check

```bash
# Verify all referenced assets return 200
for asset in \
    /fonts/font-poppins/stylesheet.css \
    /fonts/peer-icon-font/css/peer-network.css \
    /img/register.webp \
    /svg/logo_sw.svg \
    /svg/logo_farbe.svg \
    /svg/blueglow.svg \
    /svg/blueglow1.svg; do
    status=$(curl -sI "http://localhost:3000${asset}" | head -1 | awk '{print $2}')
    echo "${asset}: ${status}"
done
# All should print 200
```

### 14.15.5 — Unit Test: CSS Import

```bash
cargo leptos build
# Confirm login-register.css content is present in the compiled output:
grep -c 'gradient-animation' target/site/pkg/peer-web.css
# Expected: at least 1 (the @keyframes rule)

grep -c 'container_left' target/site/pkg/peer-web.css
# Expected: multiple matches
```

---

## 14.16 — Definition of Done

Step 14 is complete when **all** of the following are true:

1. `cargo leptos build` succeeds without warnings related to missing assets
2. All font files serve with HTTP 200 and correct MIME types
3. All SVG and image assets serve with HTTP 200
4. BackstopJS comparison passes at 375px, 768px, 1440px, and 2560px (< 0.1% diff)
5. All 44 items in the visual QA checklist (§14.12) pass
6. All 9 icon glyphs in §14.12.7 render correctly
7. Password strength meter transitions through all 5 colour states correctly
8. `prefers-reduced-motion` and `prefers-contrast: high` media queries work
9. No console errors related to missing fonts, CSS, or assets in DevTools
