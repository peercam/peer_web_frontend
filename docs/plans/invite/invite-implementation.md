# Invite Page Implementation Plan

**Feature:** Invite (Deep-Link Landing Page)  
**Priority:** #12a (first of the remaining "Not Started" pages)  
**Status:** ✅ Complete  
**Created:** 2026-04-14  
**Completed:** 2026-04-14

---

## Overview

Implement the Invite page for the Leptos frontend. This is a lightweight landing page accessed via a shareable referral link (e.g. `https://peer.network/invite?referralUuid=abc-123`). Its purpose is to:

1. Attempt to open the **native Peer app** via a `peer://` deep link
2. If the app is not installed, **redirect** to the appropriate app store (Android/iOS) or desktop registration page
3. Provide a manual **"Click Here"** fallback button
4. Store the referral UUID in `localStorage` for later use during registration

This page is the missing piece between the Referral Board (✅ complete — users copy/share their invite link) and the Registration flow (✅ complete — accepts `?referralUuid=` query params).

### Goals

1. Full parity with legacy `invite.php` + `js/invite.js` behaviour
2. Platform detection (Android / iOS / Desktop)
3. Deep-link attempt with automatic fallback
4. Manual fallback button for when auto-redirect fails
5. Referral UUID persistence in `localStorage`
6. Copy referral deep link to clipboard before redirecting to app store (mobile)
7. Minimal, no-chrome UI (no sidebar, no header — just a redirect landing page)

---

## Scope

### In Scope

- [x] Invite page (`/invite` route)
- [x] Read `?referralUuid=` query parameter
- [x] Platform detection (Android / iOS / Desktop)
- [x] Deep-link attempt: `peer://invite/{referralUuid}`
- [x] Auto-redirect fallback after 1.5s timeout:
  - Android → Google Play Store listing (commented out, matching legacy)
  - iOS → Apple App Store listing (commented out, matching legacy)
  - Desktop → `/register?referralUuid={uuid}`
- [x] Manual "Click Here" button with same fallback logic
- [x] Clipboard copy of deep-link/referral UUID before redirect (mobile)
- [x] `localStorage` persistence of `referralUuid`
- [x] Page `<Title>` tag
- [x] Minimal styling (centered text + button)

### Out of Scope (Future Work)

- Universal Links / App Links (requires server `.well-known/` config)
- Smart App Banner (`<meta name="apple-itunes-app">`)
- QR code display
- Referral code validation against backend (registration handles this)
- Analytics / tracking of invite link clicks
- Custom branded splash screen

---

## Legacy Implementation Analysis

### Files

| File | Purpose | Lines |
|------|---------|-------|
| `invite.php` | HTML shell: config div with `data-host`/`data-media-host`, "Having trouble?" text + "Click Here" link | ~25 |
| `js/invite.js` | Platform detection, deep-link attempt, auto-redirect timeout, clipboard copy, manual click handler | ~85 |

### Layout Structure

The legacy invite page is intentionally minimal — no header, no sidebar, no footer. Just a bare page with a single line of text and a link:

```
┌────────────────────────────────────────────┐
│                                            │
│  Having trouble opening the app?           │
│  Click Here to continue manually.          │
│                                            │
└────────────────────────────────────────────┘
```

### Key Behaviours

1. **Query Parameter Extraction**
   - Reads `?referralUuid=` from the URL
   - Constructs multiple URLs from it:
     - `deepLink`: `peer://invite/{uuid}`
     - `desktopFallback`: `{origin}/register.php?referralUuid={uuid}`
     - `androidFallback`: Google Play Store URL (`id=eu.peernetwork.app`)
     - `iosFallback`: App Store URL (`id6744612499`)

2. **localStorage Persistence**
   - Immediately stores `referralUuid` in `localStorage` on page load
   - This survives app store redirects — the registration page can pick it up later

3. **Auto-Open Flow (on DOMContentLoaded)**
   - Calls `openApp()` which:
     1. Sets a `visibilitychange` listener to cancel the fallback timeout if the app opens (browser goes hidden)
     2. Sets `window.location = deepLink` to trigger the native app
     3. Sets a 1500ms timeout for fallback:
        - Android: (commented out in legacy — no auto-redirect to store)
        - iOS: (commented out in legacy — no auto-redirect to store)
        - Desktop: redirects to `/register.php?referralUuid={uuid}`

4. **Manual "Click Here" Button**
   - Prevents default link action
   - Copies the deep link (mobile) or referral UUID (desktop) to clipboard
   - Then redirects:
     - Android → Play Store
     - iOS → App Store
     - Desktop → `/register?referralUuid={uuid}`

5. **Clipboard Copy**
   - Uses `navigator.clipboard.writeText()` (primary)
   - Legacy textarea fallback for older browsers
   - On mobile: copies the `peer://` deep link
   - On desktop: copies the raw `referralUuid`
   - Redirect happens in the callback after copy

### Key Observation

The legacy auto-redirect to app stores is **commented out** for Android and iOS. Only the desktop fallback (→ registration page) is active. The manual button still does the store redirect. The Leptos implementation should replicate this exact behaviour and note it for future activation.

---

## Leptos Implementation

### Architecture Overview

This is a simple single-page component — no sub-components needed, no API calls, no auth requirement.

```
pages/invite.rs            ← Page component (public, no auth guard)
style/invite.scss          ← Minimal page styles
```

### Page Component (`src/pages/invite.rs`)

```rust
//! Invite page — deep-link landing for referral links.
//!
//! Accessed via `/invite?referralUuid=...`. Attempts to open the native
//! Peer app, then falls back to the appropriate app store or registration page.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

/// App store URLs.
#[cfg(feature = "hydrate")]
const ANDROID_STORE: &str = "https://play.google.com/store/apps/details?id=eu.peernetwork.app";
#[cfg(feature = "hydrate")]
const IOS_STORE: &str = "https://apps.apple.com/app/peer-network/id6744612499";

/// Platform detected from user agent.
#[cfg(feature = "hydrate")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Android,
    Ios,
    Desktop,
}

/// Invite page — deep-link redirect landing.
#[component]
pub fn InvitePage() -> impl IntoView {
    let query = use_query_map();

    let referral_uuid = Memo::new(move |_| {
        query.get().get("referralUuid").unwrap_or_default()
    });

    // Client-side deep link logic (hydrate-only)
    #[cfg(feature = "hydrate")]
    {
        use gloo_timers::callback::Timeout;
        use std::cell::Cell;
        use std::rc::Rc;
        use wasm_bindgen::{closure::Closure, JsCast};

        Effect::new(move |_| {
            let uuid = referral_uuid.get();
            if uuid.is_empty() { return; }
            let Some(window) = web_sys::window() else { return; };

            // Store in localStorage for later registration pickup
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("referralUuid", &uuid);
            }

            let platform = detect_platform(&window);
            let deep_link = format!("peer://invite/{}", uuid);
            let _ = window.location().set_href(&deep_link);

            // Rc<Cell<Option<Timeout>>> so the visibilitychange closure
            // can cancel by taking the value (Fn, not FnOnce).
            let uuid_for_fallback = uuid.clone();
            let timeout_handle: Rc<Cell<Option<Timeout>>> = Rc::new(Cell::new(None));
            let timeout_handle_clone = timeout_handle.clone();

            let fallback = Timeout::new(1_500, move || {
                let Some(w) = web_sys::window() else { return; };
                match platform {
                    Platform::Android | Platform::Ios => {} // legacy: commented out
                    Platform::Desktop => {
                        let origin = w.location().origin().unwrap_or_default();
                        let url = format!("{}/register?referralUuid={}", origin, uuid_for_fallback);
                        let _ = w.location().set_href(&url);
                    }
                }
            });
            timeout_handle.set(Some(fallback));

            if let Some(document) = window.document() {
                let closure = Closure::<dyn Fn()>::new(move || {
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if doc.hidden() {
                            let _ = timeout_handle_clone.take(); // drop cancels the timer
                        }
                    }
                });
                let _ = document.add_event_listener_with_callback(
                    "visibilitychange", closure.as_ref().unchecked_ref(),
                );
                closure.forget();
            }
        });
    }

    // Manual fallback click handler (hydrate-only)
    #[cfg(feature = "hydrate")]
    let on_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        let uuid = referral_uuid.get();
        if uuid.is_empty() { return; }
        let Some(window) = web_sys::window() else { return; };
        let platform = detect_platform(&window);
        let deep_link = format!("peer://invite/{}", uuid);
        let origin = window.location().origin().unwrap_or_default();

        // Copy deep link (mobile) or UUID (desktop) to clipboard
        let text = match platform {
            Platform::Android | Platform::Ios => &deep_link,
            Platform::Desktop => &uuid,
        };
        let _ = window.navigator().clipboard().write_text(text);

        match platform {
            Platform::Android => { let _ = window.location().set_href(ANDROID_STORE); }
            Platform::Ios     => { let _ = window.location().set_href(IOS_STORE); }
            Platform::Desktop => {
                let url = format!("{}/register?referralUuid={}", origin, uuid);
                let _ = window.location().set_href(&url);
            }
        }
    };

    // SSR: no-op click handler (captures referral_uuid to suppress warnings)
    #[cfg(not(feature = "hydrate"))]
    let on_click = move |_ev: leptos::ev::MouseEvent| { let _ = &referral_uuid; };

    view! {
        <Title text="PeerNetwork Invite"/>
        <div class="invite-page">
            <p class="invite-prompt">
                "Having trouble opening the app?"
                <a href="#" class="invite-link" on:click=on_click>"Click Here"</a>
                " to continue manually."
            </p>
        </div>
    }
}

/// Detect platform from user agent.
#[cfg(feature = "hydrate")]
fn detect_platform(window: &web_sys::Window) -> Platform {
    let ua = window.navigator().user_agent().unwrap_or_default().to_lowercase();
    if ua.contains("android") { Platform::Android }
    else if ua.contains("iphone") || ua.contains("ipad") || ua.contains("ipod") { Platform::Ios }
    else { Platform::Desktop }
}
```

### Implementation Notes

Key differences from the initial design sketch (above was updated to match):

1. **All client-side logic is `#[cfg(feature = "hydrate")]` gated** — constants, `Platform` enum, `Effect`, click handler, and `detect_platform()` are only compiled for WASM. The SSR build gets a no-op click handler that captures `referral_uuid` to suppress unused-variable warnings.

2. **`Rc<Cell<Option<Timeout>>>` pattern** — `gloo_timers::Timeout::cancel()` consumes `self`, making a closure that calls it `FnOnce`. Since `wasm_bindgen::Closure<dyn Fn()>` requires `Fn`, we wrap the timeout in `Rc<Cell<Option<Timeout>>>` and use `.take()` to drop/cancel it from inside the `visibilitychange` listener.

3. **`detect_platform` takes `&web_sys::Window`** — avoids repeated `web_sys::window()` calls; the caller already has a window reference.

4. **No separate helper functions** — `store_referral_uuid`, `attempt_deep_link`, `manual_redirect`, and `copy_to_clipboard` from the design were inlined into the `Effect` and click handler for simplicity (single-use code in a ~185-line file).

5. **No legacy clipboard textarea fallback** — the legacy code has a `document.execCommand('copy')` fallback for older browsers. The Leptos version uses only `navigator.clipboard.writeText()`, which is supported by all browsers that can run WASM.

### SSR Considerations

- The initial HTML renders only the "Having trouble?" text + link (good for SEO / link previews)
- All deep-link logic runs after hydration via `Effect::new`
- No server functions or API calls needed
- No auth guard (public page — anyone with the link can visit)
- **Implementation note:** All client-only code (`Platform` enum, constants, `detect_platform()`, `Effect`, `on_click`) is wrapped in `#[cfg(feature = "hydrate")]` blocks. The SSR build provides a no-op `on_click` fallback that captures `referral_uuid` to satisfy the borrow checker.

---

## SCSS Styling (`style/invite.scss`)

Minimal styling — centered content on a blank page:

```scss
.invite-page {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 20px;
  background: var(--bg, #0a0a0a);
}

.invite-prompt {
  font-size: 1rem;
  color: var(--white-opacity-70, rgba(255, 255, 255, 0.7));
  text-align: center;
  line-height: 1.6;
}

.invite-link {
  color: var(--accent, #6c63ff);
  text-decoration: none;
  font-weight: 600;
  margin-left: 4px;

  &:hover {
    text-decoration: underline;
  }
}
```

---

## Routing

Add to `src/app.rs`:

```rust
<Route path=StaticSegment("invite") view=InvitePage/>
```

Add to `src/pages/mod.rs`:

```rust
pub mod invite;
pub use invite::InvitePage;
```

Update the `use` import in `src/app.rs`:

```rust
use crate::pages::{..., InvitePage, ...};
```

---

## File Manifest

### New Files

| File | Purpose | Lines |
|------|---------|-------|
| `src/pages/invite.rs` | Page component + platform detection + deep-link logic | 185 |
| `style/invite.scss` | Minimal page styles | 26 |

### Modified Files

| File | Change |
|------|--------|
| `src/app.rs` | Add `/invite` route, import `InvitePage` |
| `src/pages/mod.rs` | Add `invite` module + re-export |
| `style/main.scss` | Import `invite.scss` |

---

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| `leptos_router` (query params) | ✅ Available | `use_query_map()` — same pattern as registration |
| `web_sys` (location, localStorage, clipboard, visibility) | ✅ Available | Already in `Cargo.toml` |
| `gloo_timers` | ✅ Available | Already used in the project for timeouts |
| `wasm_bindgen` (closures) | ✅ Available | Already in `Cargo.toml` |
| Registration flow | ✅ Complete | Accepts `?referralUuid=` and reads `localStorage` |
| Referral Board | ✅ Complete | Generates the invite links that point to this page |

---

## Existing Code to Reuse

- **`use_query_map()`** — Same pattern as `register.rs` for reading `?referralUuid=`
- **`localStorage` referral UUID** — Registration page already reads `referralUuid` from `localStorage` (see `register.rs` line 239)
- **Clipboard API** — Same pattern as `ReferralHeader` component (clipboard copy)

---

## Implementation Phases

### Phase 1: Page Component & Routing ✅
1. Created `src/pages/invite.rs` with the `InvitePage` component
2. Added `/invite` route in `app.rs`
3. Registered module in `pages/mod.rs`

### Phase 2: Client-Side Logic ✅
1. Implemented `detect_platform()` with user-agent detection
2. Inline localStorage persistence in `Effect`
3. Deep-link attempt with `Rc<Cell<Option<Timeout>>>` + visibility listener
4. Manual redirect in `#[cfg(feature = "hydrate")]` click handler
5. Clipboard copy via `navigator.clipboard.writeText()`

### Phase 3: Styling ✅
1. Created `style/invite.scss` with minimal centered layout
2. Imported in `main.scss`

### Phase 4: Build Verification ✅
1. `cargo check --features hydrate` — no errors or warnings from `invite.rs`
2. `cargo check --features ssr` — no errors or warnings from `invite.rs`
3. Pre-existing errors in other modules (unrelated) confirmed unchanged

---

## Edge Cases

| Scenario | Behaviour |
|----------|-----------|
| No `?referralUuid=` param | Page renders but no deep-link attempt, no redirect |
| Empty `referralUuid` value | Same as missing — no action |
| App installed (mobile) | Deep link opens app, visibility change cancels fallback |
| App not installed (mobile) | Deep link fails silently, user clicks "Click Here" → store |
| Desktop browser | Deep link fails silently, auto-redirect to `/register` after 1.5s |
| Browser blocks `clipboard.writeText` | Best-effort — no error shown (matching legacy behaviour) |
| User navigates back from store | `referralUuid` persisted in localStorage, available on next visit |

---

## Security Considerations

- `referralUuid` is a UUID string — validated by the registration backend, not this page
- No user input is rendered as HTML (no XSS vector)
- Deep link uses a custom scheme (`peer://`) — cannot be exploited for open redirects
- App store URLs are hardcoded constants — not derived from user input
- `localStorage` is same-origin — safe for storing the referral UUID

---

## Connection to Referral Ecosystem

```
┌─────────────────┐     shares link      ┌─────────────┐
│  Referral Board  │ ──────────────────→ │  Invite Page │
│  (✅ complete)   │                     │  (✅ complete)│
│                  │                     │              │
│  getReferralInfo │                     │  /invite?    │
│  → referralLink  │                     │  referralUuid│
└─────────────────┘                     └──────┬───────┘
                                               │
                         deep link attempt     │ fallback
                         peer://invite/{uuid}  │
                                               ▼
                                        ┌─────────────┐
                                        │  Register    │
                                        │  (✅ complete)│
                                        │              │
                                        │  ?referralUuid│
                                        │  +localStorage│
                                        └─────────────┘
```

---

## Open Questions

1. **~~Enable auto-redirect to stores?~~** — Resolved: kept disabled for v1 (parity with legacy). The `Platform::Android | Platform::Ios => {}` match arms are no-ops. Trivial to enable later by adding `set_href` calls.

2. **~~Missing UUID handling~~** — Resolved: page renders static text but takes no action when `referralUuid` is empty. The `Effect` and click handler both early-return on empty UUID.

3. **~~Web Share API~~** — Resolved: out of scope for v1. The referral board's copy button is the primary sharing mechanism.

---

## Appendix: Implementation Deviations

The following deviations from the planned code were made during implementation. All are intentional improvements.

| # | Area | Plan | Actual | Rationale |
|---|------|------|--------|----------|
| 1 | Helper functions | Separate `store_referral_uuid()`, `attempt_deep_link()`, `manual_redirect()`, `copy_to_clipboard()` fns | All logic inlined into `Effect::new` and `on_click` closures | Avoids passing `web_sys::Window` around for a ~185-line single-purpose page |
| 2 | `#[cfg]` gating | Not mentioned — plan assumed client-only code would "just work" | `Platform`, constants, `detect_platform()`, `Effect` body, `on_click` all wrapped in `#[cfg(feature = "hydrate")]`; SSR gets a no-op click handler | Required for clean builds on both `--features hydrate` and `--features ssr` |
| 3 | `detect_platform()` signature | `fn detect_platform() -> Platform` (grabs its own `web_sys::window()`) | `fn detect_platform(window: &web_sys::Window) -> Platform` (takes reference) | Avoids redundant `web_sys::window()` call; callers already have a window reference |
| 4 | iOS user-agent detection | Case-sensitive: `ua.contains("iPad")`, `ua.contains("iPhone")`, `ua.contains("iPod")` | Normalises to lowercase first: `ua.to_lowercase().contains("ipad")` etc. | More robust — user agents are not guaranteed to preserve casing |
| 5 | Timeout cancellation | `fallback.cancel()` called directly in `visibilitychange` closure | `Rc<Cell<Option<Timeout>>>` — visibility closure takes and drops the timeout | `gloo_timers::Timeout` is consumed on fire; wrapping in `Rc<Cell<Option<_>>>` allows the closure to take ownership without moving the timeout |
| 6 | Legacy clipboard fallback | Plan mentioned only `navigator.clipboard.writeText()` (matching plan's `copy_to_clipboard()`) | Same — no `<textarea>` fallback | Acceptable: Leptos/WASM requires a modern browser that supports the Clipboard API |
