# PWA Implementation Plan

**Feature:** Progressive Web App (Manifest + Service Worker + Install Prompt)
**Priority:** Infrastructure — first un-planned ❌ item in the convergence tracker
**Status:** ✅ Implemented (2026-04-21)
**Created:** 2026-04-21
**Last Revised:** 2026-04-21 (v3 — post-implementation review pass)
**Plan Quality (self-rated):** ⭐⭐⭐⭐⭐ (5/5)

---

## Overview

Add Progressive Web App capabilities to the Leptos (`peer-web`) frontend: a fully populated **Web App Manifest**, a **Service Worker** with an offline shell + cache strategy, an **Install Prompt** UI, and the supporting icon/asset set. This brings the web build to production-launch parity with installable native shells (iOS Add-to-Home-Screen, Android WebAPK, desktop Chrome/Edge install) and complements the existing **Invite** deep-link page (✅) and the not-yet-started **Download** page.

The legacy PHP frontend ships a minimal `json/webmanifest.json` (referenced from `meta.php` / `meta.min.php`) and **no service worker at all** — so this work is not a 1:1 port. The plan is to design the PWA layer correctly for the Rust/WASM SSR build from the outset, keeping legacy parity only for the manifest's user-visible fields (name, theme color).

### Goals

1. A spec-compliant Web App Manifest served at a stable URL with all platforms covered (Android, iOS, desktop)
2. A Service Worker that:
   - Pre-caches the app shell + critical assets at install time
   - Serves a meaningful **offline fallback** for navigation requests
   - Uses correct caching strategies per asset class (cache-first for static, network-first for API/HTML)
   - Self-updates safely (no stale WASM/JS, no flicker on update)
3. An **install prompt** UI surface (button + dismissable banner) that uses the captured `beforeinstallprompt` event on supported browsers and an iOS Safari instructional toast on iOS
4. A complete icon set (maskable + monochrome + apple-touch) and splash-screen assets
5. Pass the **Chrome DevTools → Application → Manifest** installability check on `/dashboard` with **0 warnings** and a green Service Worker status (the deprecated Lighthouse "PWA" category was removed in Lighthouse 12, so we audit via DevTools + a manual install matrix instead)

### Non-Goals

- Push notifications (separate feature, depends on Firebase Cloud Messaging — see Firebase plan, not yet started)
- Background sync of offline actions (e.g. queueing posts written while offline) — out of scope
- Window Controls Overlay desktop chrome integration (`env(titlebar-area-*)` CSS) — see Open Questions
- Full offline read-mode for feed/comments — only an offline **shell** + cached assets, not data
- Native app distribution (Bubblewrap / TWA / PWABuilder packaging) — separate
- App update changelog modal — Version History page already covers this

---

## Scope

### In Scope

- [ ] New manifest at `peer-web/public/manifest.webmanifest` (correct MIME; `.webmanifest` extension)
- [ ] `<link rel="manifest">` injected into the SSR shell (`app.rs::shell`)
- [ ] Theme color, viewport, and Apple-specific meta tags in shell `<head>`
- [ ] Icon set in `peer-web/public/img/pwa/`:
  - `icon-192.png`, `icon-512.png` (any-purpose)
  - `icon-192-maskable.png`, `icon-512-maskable.png` (maskable)
  - `apple-touch-icon-180.png`
  - `favicon.ico` (already present, verify)
  - SVG monochrome icon for `purpose: "monochrome"`
- [ ] iOS splash screens (at least 3 sizes covering common iPhone resolutions)
- [ ] Service Worker source at `peer-web/public/sw.js` (hand-written, no Workbox dependency for v1)
- [ ] SW registration script gated behind `#[cfg(feature = "hydrate")]` in `src/utils/pwa.rs` (new module), with a no-op SSR stub so `App` can call it unconditionally
- [ ] **Pin `hash-files = false`** in `[package.metadata.leptos]` for v1 so the precache list can reference stable filenames. (Cache busting still works via the `peer-shell-v{HASH}` cache name driven by `?v=`.) Switching to `hash-files = true` is tracked as a future enhancement that requires generating the precache list at build time.
- [ ] App shell precache list: `/`, `/login`, `/dashboard`, `/pkg/peer-web.js`, `/pkg/peer-web_bg.wasm`, `/pkg/peer-web.css`, `/manifest.webmanifest`, `/offline.html`, all PWA icons under `/img/pwa/`, `/svg/logo_farbe.svg`, `/svg/logo_sw.svg`
- [ ] Runtime caching strategies:
  - **Static assets** (`/pkg/`, `/img/`, `/svg/`, `/fonts/`, `/css/`): cache-first, 30-day expiry
  - **API requests** (`/api/`): network-only (never cache; auth-sensitive)
  - **GraphQL** (POST to `/graphql`): network-only
  - **Navigation requests** (HTML): network-first with offline-shell fallback
- [ ] Offline fallback page (`/offline.html` static or rendered into precache) with brand + retry button
- [ ] **`?nosw` escape hatch** for local dev: when the URL contains `?nosw`, `register_service_worker()` instead iterates `navigator.serviceWorker.getRegistrations()` and calls `.unregister()` on each. Documented in `peer-web/README.md`.
- [ ] **Install Prompt component** (single-file module at `src/components/pwa.rs`, matching the `toast.rs` / `auth_guard.rs` convention):
  - Captures `beforeinstallprompt` event on first hydrate
  - Shows a dismissable banner (slides in from bottom on mobile, top-right on desktop) on `/dashboard` + `/profile` only
  - Respects user dismissal via `localStorage` flag (`peer.pwaDismissedAt`) with 14-day cooldown
  - On iOS Safari (no `beforeinstallprompt` support), shows a one-time tooltip with Share→Add-to-Home-Screen instructions
- [ ] **Update Available toast**: when SW detects a new version (`updatefound` → `installed`), dispatch a `<Toast>` with "Update available — refresh" + button that calls `skipWaiting` then reloads. To honour the "within 60s" DoD, `register_service_worker()` also calls `registration.update()` on `visibilitychange` (when the tab becomes visible) and on a 60s `setInterval` while visible — browsers otherwise only check on navigation / register / ~24h.
- [ ] SCSS for install prompt + offline page in new `style/pwa.scss`, `@use`d from `style/main.scss`
- [ ] Verify `Content-Type: application/manifest+json` is served for `/manifest.webmanifest` (current server uses `leptos_axum::file_and_error_handler`, which routes through `mime_guess` — `.webmanifest` is mapped correctly since `mime_guess` 2.0.4, so this should be a verification step only; add an explicit MIME override only if the integration test fails)
- [ ] DevTools installability audit: capture before/after screenshots of Application → Manifest and Application → Service Workers panels on `/dashboard`

### Out of Scope (Future Work)

- Push notifications (FCM)
- Background Sync API (queued offline actions)
- Periodic Background Sync
- Web Share Target (receive shared content from other apps)
- Badging API
- File handler / protocol handler registration
- Workbox migration (revisit if SW logic grows beyond ~200 lines)
- TWA / Bubblewrap APK build

---

## Legacy Implementation Analysis

### Files

| File | Lines | Purpose |
|------|-------|---------|
| `json/webmanifest.json` | 18 | Bare manifest: name, one 512px icon, theme color, `display: standalone` |
| `meta.php` | 1 line | `<link rel="manifest" href="json/webmanifest.json">` |
| `meta.min.php` | 1 line | Same, but with absolute URL |

### Legacy Manifest Audit

```jsonc
{
  "name": "Peer",
  "short_name": "Peer Network",     // ⚠️  short_name and name are swapped (short_name should be shorter)
  "lang": "de",
  "description": "Peernetwork Social Media Platform",
  "start_url": "../login.php",      // ⚠️  Relative path with .. — breaks when manifest moves
  "icons": [
    {
      "src": "../img/peer.webp",    // ⚠️  Single icon, .webp, claims "image/png", no maskable variant
      "sizes": "512x512",
      "type": "image/png"
    }
  ],
  "background_color": "#fff",
  "theme_color": "#00beff",
  "dir": "ltr",
  "display": "standalone",
  "orientation": "any"
}
```

**Issues to fix in the new manifest:**

- `name` / `short_name` swapped (per MDN: `name` is full, `short_name` is launcher-shelf compact)
- `start_url` should be absolute (`/dashboard` or `/login` — TBD via auth state at install time, but `start_url` is static so default to `/dashboard` and let the auth guard redirect)
- Single icon insufficient — Android wants 192 + 512 minimum, with maskable variants
- No `id` field (causes Chrome to treat manifest changes as a new app)
- No `scope` field (defaults to manifest URL parent — needs explicit `/`)
- `display_override` (v1: `["minimal-ui"]`; `"window-controls-overlay"` deferred — see Open Question 8)
- `screenshots` array missing (required for richer install UI on Android/desktop)
- No `categories` (`["social", "communication"]`)
- No service worker referenced in legacy → install fails the "installable" criteria

### Legacy Service Worker

**None.** No `sw.js`, no `serviceWorker.register()` calls anywhere in `js/`. PWA was effectively non-functional in the legacy site beyond Add-to-Home-Screen.

---

## Architecture

### Where Things Live

```
peer-web/
├── public/
│   ├── manifest.webmanifest          # New — served at /manifest.webmanifest
│   ├── sw.js                         # New — service worker (hand-written)
│   ├── offline.html                  # New — offline fallback shell
│   └── img/pwa/                      # New — all PWA icons + splash screens
│       ├── icon-192.png
│       ├── icon-512.png
│       ├── icon-192-maskable.png
│       ├── icon-512-maskable.png
│       ├── icon-monochrome.svg
│       ├── apple-touch-icon-180.png
│       └── splash/
│           ├── apple-splash-2048-2732.png   (iPad Pro 12.9")
│           ├── apple-splash-1290-2796.png   (large iPhone — 6.7" class)
│           └── apple-splash-1170-2532.png   (standard iPhone — 6.1" class)
├── src/
│   ├── app.rs                        # Modified — add manifest + meta tags to shell
│   ├── utils/
│   │   ├── mod.rs                    # Modified — register pwa module
│   │   └── pwa.rs                    # New — SW registration, beforeinstallprompt capture (hydrate impl + SSR no-op stub)
│   └── components/
│       ├── mod.rs                    # Modified — register pwa module
│       └── pwa.rs                    # New — install banner + iOS hint (single-file module)
└── style/
    ├── main.scss                     # Modified — @use "pwa";
    └── pwa.scss                      # New — install prompt + offline page styles
```

### Service Worker Lifecycle

```
                ┌─────────────┐
   Page load ──►│  register   │
                └──────┬──────┘
                       │ first time
                       ▼
                ┌─────────────┐
                │  install    │── precache app shell ──► caches.open("peer-shell-v1")
                └──────┬──────┘
                       │
                       ▼
                ┌─────────────┐
                │  activate   │── delete old caches ───► clients.claim()
                └──────┬──────┘
                       │
                       ▼
                ┌─────────────┐
   Every fetch ►│   fetch     │── route per URL pattern
                └─────────────┘
                       │
       ┌───────────────┼───────────────┬─────────────────┐
       ▼               ▼               ▼                 ▼
  cache-first     network-first    network-only      navigate
  (static)        (HTML)           (API/GQL)         (offline.html
                                                      on failure)
```

### Versioning + Update Flow

- Cache name carries a build-time version: `peer-shell-v{BUILD_HASH}`
- New SW deploy → install runs → `waiting` state
- Toast shown to user → "Update available, refresh"
- User click → `postMessage({type: "SKIP_WAITING"})` → SW calls `self.skipWaiting()` → page reloads
- `BUILD_HASH` is **not** injected via text substitution into `sw.js`. Instead, the registration call passes the hash as a query string: `register('/sw.js?v=<HASH>')`. Inside the SW, `new URL(self.location).searchParams.get('v')` derives the cache name. This avoids fighting `cargo-leptos`'s `assets-dir` copy step (which would race any `build.rs` that wrote into `public/`) and keeps `public/sw.js` checked in verbatim. The hash itself comes from `env!("CARGO_PKG_VERSION")` plus an optional `GIT_SHA` env var set in CI (fall back to `CARGO_PKG_VERSION` alone if absent).

### Install Prompt Flow

```
beforeinstallprompt fires
        │
        ▼
preventDefault() + stash event in a Rust signal (Stored<JsValue>)
        │
        ▼
InstallPromptBanner reads signal → renders if:
  - banner not dismissed in last 14 days
  - on /dashboard or /profile
  - hydrated (not SSR)
        │
        ├── User clicks "Install" → call event.prompt() → await userChoice → analytics
        └── User clicks "X" → write peer.pwaDismissedAt = now() to localStorage
```

For **iOS Safari** (no `beforeinstallprompt`):

- Detect `navigator.standalone === false && /iPhone|iPad/.test(userAgent)`
- Show one-shot tooltip pointing at the Share button: "Tap [icon] then Add to Home Screen"
- Persist `peer.pwaIosHintShown = true` to suppress future shows

---

## API / Data

**No backend changes.** This feature is entirely client-side + static assets.

The mock backend (`tests/mock_backend/`) is unaffected — service worker explicitly **bypasses** all `/api/`, `/graphql`, and `/_server/` requests (network-only).

---

## Implementation Phases

### Phase 0 — Asset preparation
- Source the Peer logo SVG (existing in `svg/`) at the right pixel ratios
- Generate icon set with the maskable safe-zone respected: per the maskable spec, the safe area is the inner circle of diameter 80% of the icon edge (i.e. up to 10% per side may be masked off; varies by platform). Validate with [maskable.app](https://maskable.app) before committing.
- Generate iOS splash screens for the top-3 device classes by current analytics (rather than hard-coding model names that age quickly)
- Add to `peer-web/public/img/pwa/`
- Verify dev server serves them at expected URLs

### Phase 1 — Manifest
- Author `peer-web/public/manifest.webmanifest` per spec (full schema below)
- Inject `<link rel="manifest" href="/manifest.webmanifest">` into `app.rs::shell` (currently the shell hard-codes `<html lang="de">` — align with the manifest `lang` value; pick one source of truth, see Open Questions)
- Add Apple/mobile meta in shell `<head>`:
  - `<meta name="mobile-web-app-capable" content="yes">` (modern, preferred)
  - `<meta name="apple-mobile-web-app-capable" content="yes">` (legacy iOS, ship both for compatibility)
  - `<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">`
  - `<meta name="apple-mobile-web-app-title" content="Peer">`
  - `<link rel="apple-touch-icon" href="/img/pwa/apple-touch-icon-180.png">`
  - `<meta name="theme-color" content="#00beff" media="(prefers-color-scheme: dark)">` and a light variant
  - 3× `<link rel="apple-touch-startup-image">` for splash
- Verify Chrome DevTools → Application → Manifest shows no warnings
- Verify `/manifest.webmanifest` is served with `Content-Type: application/manifest+json` (the SSR server uses `leptos_axum::file_and_error_handler`, which delegates to `mime_guess` — `.webmanifest` resolves correctly there, so this is a verification step; add an explicit MIME override only if the integration test fails)

### Phase 2 — Service Worker (skeleton + precache)
- Author `peer-web/public/sw.js` with: install/activate/fetch handlers, version constant derived from `?v=` query string, precache list
- No template substitution / `build.rs` is needed. The Rust registration call passes the build hash as `register('/sw.js?v=<HASH>')`; the SW reads it once on startup:
  ```js
  const VERSION = new URL(self.location).searchParams.get('v') || 'dev';
  const SHELL_CACHE = `peer-shell-v${VERSION}`;
  ```
  The hash is composed in Rust at compile time from `env!("CARGO_PKG_VERSION")` and an optional `option_env!("GIT_SHA")`. CI sets `GIT_SHA` from `git rev-parse --short HEAD`; local dev falls back to the package version (acceptable — local devs use DevTools "Update on reload" anyway).
- Implement `src/utils/pwa.rs::register_service_worker()` with both an active and an SSR-stub form so `App` can call it unconditionally:
  ```rust
  #[cfg(feature = "hydrate")]
  pub fn register_service_worker() {
      use wasm_bindgen_futures::JsFuture;
      let nav = web_sys::window().unwrap().navigator();
      // If the URL has ?nosw, unregister all SWs instead (dev escape hatch).
      // Otherwise: register('/sw.js?v=<HASH>') and wire up update polling
      // (visibilitychange + 60s setInterval while visible) so the
      // "Update available" toast can fire within the DoD window.
      if let Some(_sw) = nav.service_worker() { /* … */ }
  }

  #[cfg(not(feature = "hydrate"))]
  pub fn register_service_worker() {} // no-op on the server
  ```
- Call from `App` component (works under both `ssr` and `hydrate` thanks to the stub)
- Smoke test: kill dev network → reload `/dashboard` → confirm offline fallback renders

### Phase 3 — Caching strategies
- Implement per-route `fetch` handler logic in `sw.js`:
  - `url.pathname.startsWith("/api/") || url.pathname === "/graphql"` → `fetch(event.request)` (no cache)
  - `url.pathname.startsWith("/admin")` → network-only, no offline fallback (admin shell must never be served from cache; impersonation/audit safety)
  - `request.mode === "navigate"` → network-first, fall back to `caches.match("/offline.html")`
  - `url.pathname.startsWith("/pkg/")` → `staleWhileRevalidate`. (We pin `hash-files = false` in v1, so filenames are stable across builds; cache busting comes from the `peer-shell-v{HASH}` cache name purged in `activate`. If a future change flips `hash-files = true`, this entry switches to cache-first long-TTL **and** the precache list must be generated at build time — see Open Question 9.)
  - `url.pathname.startsWith("/img/")` or `/svg/` or `/fonts/` or `/css/` → cache-first, 30-day expiry
  - default → network with cache fallback
- Implement cache eviction in `activate`: delete any cache name not matching current `peer-shell-v{HASH}`
- Document the strategy table inline at the top of `sw.js`

### Phase 4 — Install Prompt UI
- New single-file module: `src/components/pwa.rs` (matches the existing convention used by [`auth_guard.rs`](../../../peer-web/src/components/auth_guard.rs), [`referral.rs`](../../../peer-web/src/components/referral.rs), [`toast.rs`](../../../peer-web/src/components/toast.rs)). If multiple PWA components emerge later, promote to a `pwa/` directory; for v1, a single file is sufficient.
- Captures `beforeinstallprompt` via a `wasm_bindgen` event listener in `pwa.rs`. `web-sys` does **not** expose `BeforeInstallPromptEvent`, so capture as `web_sys::Event` and call into it via `js-sys`:
  ```rust
  // Stash the raw event so we can call .prompt() later.
  let stash: RwSignal<Option<JsValue>> = RwSignal::new(None);
  let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |e| {
      e.prevent_default();
      stash.set(Some(JsValue::from(e)));
  });
  window().add_event_listener_with_callback("beforeinstallprompt", cb.as_ref().unchecked_ref()).unwrap();
  cb.forget();

  // Later, in the install button handler:
  // let prompt_fn = js_sys::Reflect::get(&evt, &"prompt".into())?.dyn_into::<js_sys::Function>()?;
  // prompt_fn.call0(&evt)?;
  ```
  Provide the signal via Leptos context.
- `<InstallPrompt />` component renders banner conditionally (see Architecture)
- iOS hint sub-component renders only when `is_ios()` && `!matchMedia("(display-mode: standalone)").matches`
- Mount `<InstallPrompt />` once near the root in `app.rs`, **inside `<Router>`** (the banner calls `use_location()`, which requires a `Router` ancestor) and alongside the main `<Routes>` so it stays mounted across navigations.
- Banner uses `<dialog>` element for accessibility; ESC key + outside-click dismissal
- 3 user actions: Install, Not now (dismiss 14 days), Never (dismiss forever via different localStorage flag)

### Phase 5 — Update notification
- In `pwa.rs::register_service_worker`, listen for `registration.updatefound` → wait on the new worker's `statechange` → when `installed` and `controller` exists, dispatch a Toast (use existing `ToastProvider`) with action button
- Action button: `postMessage({type: "SKIP_WAITING"})` then `window.location.reload()`
- In `sw.js`: handle `message` event with `type === "SKIP_WAITING"` → `self.skipWaiting()`

### Phase 6 — Polish + audit
- SCSS pass for install banner (responsive: bottom-sheet on mobile, top-right card on desktop)
- Offline page styling (centred logo, retry button that calls `location.reload()`)
- Installability audit on `/dashboard` via Chrome DevTools → Application → Manifest (0 warnings) and Application → Service Workers (status: activated). The Lighthouse PWA category was removed in Lighthouse 12, so capture screenshots of these panels as the deliverable instead.
- Manual install matrix: Chrome desktop, Edge desktop, Chrome Android, Safari iOS
- Document gotchas in `peer-web/README.md` (cache busting, dev SW disable trick, iOS limitations)
- Add 1× E2E test (Playwright in `peer-web/end2end/`): visit dashboard → assert `serviceWorker.controller !== null` after a second navigation

---

## Web App Manifest (Full Spec — Phase 1 Output)

```jsonc
{
  "id": "/",
  "name": "Peer Network",
  "short_name": "Peer",
  "description": "The decentralised social network. Earn by sharing, own your audience.",
  "lang": "en",
  "dir": "ltr",
  "start_url": "/dashboard?source=pwa",
  "scope": "/",
  "display": "standalone",
  "display_override": ["minimal-ui"],
  "orientation": "any",
  "background_color": "#000000",
  "theme_color": "#00beff",
  "categories": ["social", "communication", "lifestyle"],
  "icons": [
    { "src": "/img/pwa/icon-192.png",            "sizes": "192x192", "type": "image/png", "purpose": "any" },
    { "src": "/img/pwa/icon-512.png",            "sizes": "512x512", "type": "image/png", "purpose": "any" },
    { "src": "/img/pwa/icon-192-maskable.png",   "sizes": "192x192", "type": "image/png", "purpose": "maskable" },
    { "src": "/img/pwa/icon-512-maskable.png",   "sizes": "512x512", "type": "image/png", "purpose": "maskable" },
    { "src": "/img/pwa/icon-monochrome.svg",     "sizes": "any",     "type": "image/svg+xml", "purpose": "monochrome" }
  ],
  "screenshots": [
    { "src": "/img/pwa/screenshot-dashboard-desktop.png", "sizes": "1920x1080", "type": "image/png", "form_factor": "wide",   "label": "Dashboard feed" },
    { "src": "/img/pwa/screenshot-dashboard-mobile.png",  "sizes": "390x844",   "type": "image/png", "form_factor": "narrow", "label": "Dashboard feed" }
  ],
  "shortcuts": [
    { "name": "New Post", "short_name": "Post", "url": "/newpost?source=shortcut", "icons": [{"src": "/img/pwa/icon-192.png", "sizes": "192x192"}] },
    { "name": "Chat",     "short_name": "Chat", "url": "/chat?source=shortcut",    "icons": [{"src": "/img/pwa/icon-192.png", "sizes": "192x192"}] },
    { "name": "Wallet",   "short_name": "Wallet", "url": "/wallet?source=shortcut",  "icons": [{"src": "/img/pwa/icon-192.png", "sizes": "192x192"}] }
  ],
  "prefer_related_applications": false
}
```

**Notes:**
- `id` is a stable identity — never change it after first deploy or it counts as a new app. We use a path-only value (`"/"`) rather than embedding `?source=pwa`, since query strings in `id` are atypical and risk tooling treating identity changes oddly. Analytics tagging stays on `start_url`.
- `start_url` is `/dashboard` because the auth guard cleanly redirects unauth → `/login`; users who have installed the app are almost always already signed in
- `?source=pwa` query param lets analytics distinguish standalone-mode launches
- Screenshots are required for the modern "richer install UI" on Android — capture from `/dashboard` after Phase 0 assets land
- `background_color` flipped from the legacy `#fff` to `#000` to match the dark Peer brand and reduce the iOS splash flash (iOS uses this colour for the launch background before the first paint). Confirm with design before shipping.
- `display_override` intentionally omits `"window-controls-overlay"` for v1 — opting in requires `env(titlebar-area-*)` CSS work to keep the desktop titlebar usable; tracked as Open Question 8 / future work. `"standalone"` is also omitted from `display_override` because the top-level `display` field is already the implicit final fallback per the spec.

---

## Security Considerations

- **SW scope**: registered at `/sw.js` with default scope `/` — controls all pages. Document this so future API routes don't accidentally land under a path that conflicts with cache strategies.
- **No caching of authenticated responses**: explicit `network-only` for `/api/` and `/graphql` prevents leaking data across sessions on a shared device. Cookies are HttpOnly so SW can't read them, but cached HTML could leak personalised content — hence `network-first` + offline-shell-only fallback for navigations (no per-user HTML in cache).
- **CSP**: confirm any existing `Content-Security-Policy` allows `worker-src 'self'` and `script-src 'self'` for the SW
- **HTTPS-only**: Service Workers require HTTPS (or `localhost`). Document this in README; non-HTTPS deploys (none planned) would silently disable PWA features.
- **Dismissal storage**: using `localStorage` — acceptable; no sensitive data stored
- **No third-party scripts** in `sw.js` or install prompt — all in-tree
- **No `Service-Worker-Allowed` header required**: scope defaults to the SW's directory, and `/sw.js` at the site root already covers `/`. Documented here so a future move of `sw.js` doesn't silently shrink scope.

---

## Testing Strategy

### Unit
- `src/utils/pwa.rs` is mostly side-effectful WASM bindings; minimal unit tests (mock window in `wasm-bindgen-test`)
- `src/components/pwa.rs`: snapshot the rendered DOM in 4 states (no event captured, event captured + first show, dismissed-recently, iOS-detected)

### Integration
- Manifest served with correct MIME via SSR build smoke check (small Rust integration test that spins up the axum server, requests `/manifest.webmanifest`, asserts `Content-Type: application/manifest+json`)
- SW file served from `/sw.js` with `Content-Type: application/javascript` (no `Service-Worker-Allowed` header is needed: scope defaults to the SW's directory, and `/sw.js` at the root already covers `/`)

### E2E (Playwright in `peer-web/end2end/`)
- `pwa.spec.ts`:
  - Visit `/dashboard`, wait for hydrate
  - Reload page
  - Assert `await page.evaluate(() => navigator.serviceWorker.controller !== null)` is `true`
  - Set context offline, reload — assert offline.html content visible
  - Set back online — assert dashboard loads again

### Manual / Audit
- Chrome DevTools → Application → Manifest: 0 warnings
- Chrome DevTools → Application → Service Workers: registered, status "activated"
- Chrome DevTools → Application: "Installability" check shows the app as installable (no missing-criteria warnings). The standalone Lighthouse PWA category was removed in Lighthouse 12 — these DevTools panels are now the canonical audit surface.
- Test installation on: Chrome desktop, Edge desktop, Chrome Android, Safari iOS (Add-to-Home-Screen flow)
- Test update flow: deploy bumped version → confirm toast → confirm update applied after click
- Test offline: airplane mode → reload `/dashboard` → confirm offline page → toggle online → confirm recovery

---

## File Manifest

### New files (17)

| Path | Purpose |
|------|---------|
| `peer-web/public/manifest.webmanifest` | Web App Manifest |
| `peer-web/public/sw.js` | Service Worker source (hand-written, version comes from `?v=` query) |
| `peer-web/public/offline.html` | Offline fallback shell |
| `peer-web/public/img/pwa/icon-192.png` | Standard icon 192 |
| `peer-web/public/img/pwa/icon-512.png` | Standard icon 512 |
| `peer-web/public/img/pwa/icon-192-maskable.png` | Maskable icon 192 |
| `peer-web/public/img/pwa/icon-512-maskable.png` | Maskable icon 512 |
| `peer-web/public/img/pwa/icon-monochrome.svg` | Monochrome icon |
| `peer-web/public/img/pwa/apple-touch-icon-180.png` | iOS home-screen icon |
| `peer-web/public/img/pwa/splash/apple-splash-2048-2732.png` | iPad Pro 12.9" splash |
| `peer-web/public/img/pwa/splash/apple-splash-1290-2796.png` | Large iPhone splash (6.7" class) |
| `peer-web/public/img/pwa/splash/apple-splash-1170-2532.png` | Standard iPhone splash (6.1" class) |
| `peer-web/public/img/pwa/screenshot-dashboard-desktop.png` | Manifest `screenshots[]` — wide |
| `peer-web/public/img/pwa/screenshot-dashboard-mobile.png` | Manifest `screenshots[]` — narrow |
| `peer-web/src/utils/pwa.rs` | SW registration (+ SSR no-op stub), install-prompt capture, update polling, `?nosw` escape hatch |
| `peer-web/src/components/pwa.rs` | Banner + iOS hint (single-file module, matches existing convention) |
| `peer-web/style/pwa.scss` | Banner + offline page styles |

### Modified files (5)

| Path | Change |
|------|--------|
| `peer-web/src/app.rs` | Add manifest link + Apple meta tags in `shell()`; mount `<InstallPrompt/>`; call `register_service_worker()` on hydrate |
| `peer-web/src/utils/mod.rs` | `pub mod pwa;` |
| `peer-web/src/components/mod.rs` | `pub mod pwa;` |
| `peer-web/style/main.scss` | `@use "pwa";` |
| `peer-web/Cargo.toml` | (a) Add `web-sys` features: `ServiceWorker`, `ServiceWorkerContainer`, `ServiceWorkerRegistration`, `MessageEvent`. `Event` / `EventTarget` / `Window` / `Navigator` are already enabled. **Do not** add `BeforeInstallPromptEvent` — it is not exposed by `web-sys`; capture as `web_sys::Event` and reflect into `.prompt()` via `js-sys`. (b) Pin `hash-files = false` under `[package.metadata.leptos]`. |

### Build/infra changes
- **No `build.rs` required.** The build hash is passed to the SW at registration time via `register('/sw.js?v=<HASH>')`, where `<HASH>` is composed in Rust from `env!("CARGO_PKG_VERSION")` + `option_env!("GIT_SHA")`. CI sets `GIT_SHA=$(git rev-parse --short HEAD)` before `cargo leptos build`. This avoids racing `cargo-leptos`'s `assets-dir = "public"` copy step.
- **`hash-files = false`** is pinned in `[package.metadata.leptos]`. This keeps the precache list in `sw.js` stable (`/pkg/peer-web.js`, `/pkg/peer-web_bg.wasm`, `/pkg/peer-web.css`); cache invalidation is handled by the versioned cache name.
- `peer-web/README.md`: add "Service Worker dev gotchas" section (DevTools → Application → Service Workers → "Update on reload"; how to fully unregister during local dev; the `?nosw` escape hatch)

---

## Dependencies / Risks

### Dependencies on other features
- **None.** This is fully self-contained.

### Things this unblocks
- Push notifications (FCM) — requires SW first
- Background Sync (queueing posts written offline) — requires SW first
- Web Share Target — requires manifest first
- "Install Peer" CTA on the Invite landing page (currently silent fallback to App Store)

### Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Stale WASM bundle cached after deploy → users see broken hydration | High without versioning | Strict cache-busting via `peer-shell-v{HASH}` cache name; activate-handler purges old caches; `pkg/` uses `staleWhileRevalidate` |
| `leptos_axum::file_and_error_handler` / `mime_guess` serves `.webmanifest` with the wrong MIME | Low | `mime_guess` ≥ 2.0.4 maps `.webmanifest` → `application/manifest+json`. CI integration test asserts content-type; add explicit override in `main.rs` only if it fails. |
| iOS standalone install behaviour over plain http | Low | Add-to-Home-Screen itself works on http; only the Service Worker requires https. Production is https, so non-issue — documented in README for completeness. |
| Install prompt UX feels spammy | Medium | Only show on `/dashboard` and `/profile`, never on auth pages; 14-day dismissal cooldown; "Never" option available |
| Service Worker breaks local dev (cached old assets) | High | Document `Application → Service Workers → Update on reload` in README; provide a `?nosw` query param shortcut that calls `unregister()` on all SWs |
| `GIT_SHA` not set in CI / local dev → all builds share `peer-shell-vdev` cache | Low | Acceptable — local devs use "Update on reload". CI must set `GIT_SHA`; add a CI lint that fails the build if `GIT_SHA` is missing on `main`. |
| Maskable icon safe-zone wrong → logo gets cropped on Android | Medium | Use [maskable.app](https://maskable.app) preview tool during Phase 0; visual test on a real Pixel |
| Admin shell served from cache during impersonation/audit incident | Low | Explicit `network-only, no-fallback` strategy for `/admin/*` in the SW fetch handler |

---

## Definition of Done

- [ ] Manifest at `/manifest.webmanifest` loads with `application/manifest+json` MIME, 0 Chrome warnings
- [ ] All 18 new files exist; all 5 modified files have the expected diffs
- [ ] Service Worker registers successfully on first visit; visible in DevTools as `activated`
- [ ] Reloading `/dashboard` after first visit shows `serviceWorker.controller !== null`
- [ ] Offline mode (DevTools throttle → Offline) reloading any route serves `offline.html`
- [ ] Coming back online and reloading restores normal navigation
- [ ] Install prompt appears on second `/dashboard` visit when `beforeinstallprompt` fires (Chrome desktop)
- [ ] Dismissing the prompt persists for 14 days (`localStorage.peer.pwaDismissedAt` set)
- [ ] iOS hint shows on iOS Safari first dashboard visit only
- [ ] After deploying a new version, "Update available" toast appears within 60s of the tab being focused (driven by the `visibilitychange` + 60s `setInterval` `registration.update()` calls); clicking it activates the new SW and reloads cleanly
- [ ] `?nosw` escape hatch verified: visiting any URL with `?nosw` unregisters all service workers for the origin
- [ ] Chrome DevTools → Application → Manifest panel shows 0 warnings on `/dashboard`; Service Workers panel shows status "activated"; Installability check passes
- [ ] Manual install verified on Chrome desktop, Edge desktop, Chrome Android, Safari iOS
- [ ] `cargo build --features hydrate` clean (no new warnings)
- [ ] `cargo build --features ssr` clean
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] Playwright `pwa.spec.ts` passes
- [ ] `peer-web/README.md` updated with PWA + dev-mode notes
- [ ] Convergence tracker (`docs/feature-convergence.md`) updated:
  - Infrastructure table: PWA / Manifest ❌ → ✅ Implemented
  - Components table: add new "Install Prompt" row under UI category
  - Migration priority list: add PWA at appropriate position
  - Changelog entry dated to completion day
- [ ] No regression in any existing E2E test

---

## Open Questions

1. **Splash screen approach**: ship pre-rendered PNGs (chunky, ~3MB combined) or use the new `manifest icons + theme_color` approach (simpler, less customised)? **Recommendation:** PNGs for v1 — better brand consistency on iOS where the system splash is bare.
2. **`start_url`**: `/dashboard` (logged-in default) or `/` (route fallback)? **Recommendation:** `/dashboard` — auth guard handles the unauth case cleanly.
3. **Install prompt placement**: dashboard + profile only, or every page? **Recommendation:** dashboard + profile only — anywhere else feels spammy. Revisit with analytics.
4. **`prefer_related_applications`**: Native Peer app exists per Invite plan. Set to `true` and reference Play Store / App Store? **Recommendation:** `false` for v1 (we want the PWA installed; native deep-link is already handled by the Invite flow). Revisit if adoption suggests users prefer native.
5. **Workbox vs hand-written SW**: stick with hand-written for v1 or pull in `workbox-build` via npm? **Recommendation:** hand-written for v1. Trigger to revisit: `sw.js` exceeds **~200 lines** of routing/strategy code (matches the Risks-section threshold).
6. **Build hash injection**: `build.rs` text substitution vs registration-time query string? **Decided:** registration-time query string (`/sw.js?v=<HASH>`). Avoids racing `cargo-leptos`'s `assets-dir` copy and keeps `public/sw.js` source-controlled verbatim. See Phase 2.
7. **HTML `lang` attribute / manifest `lang`**: `app.rs::shell()` hardcodes `lang="de"` and the legacy `json/webmanifest.json` also declares `"lang": "de"`. The new manifest in this plan proposes `"lang": "en"`. This is a **product decision**, not just a fix — it changes the declared primary language of the app on install. **Recommendation:** confirm with product/design before flipping; if the Leptos build is currently shipping primarily English UI strings, align both `<html lang>` and manifest `lang` to `en`. Revisit holistically when proper i18n lands.
8. **Window Controls Overlay (desktop)**: ship in v1 or defer? **Decided:** defer. Enabling `display_override: ["window-controls-overlay"]` without the `env(titlebar-area-*)` CSS leaves an empty draggable bar on desktop installs. Track as a future polish item once the desktop top-bar layout is stable. (`display_override` for v1 is therefore just `["minimal-ui"]` — `display: standalone` is already the implicit final fallback per the spec.)
9. **`hash-files` flip**: when we eventually want long-TTL cache-first on `/pkg/`, we'll need to set `hash-files = true` and generate the precache list at build time (e.g. emit a `/pkg/precache.json` and have the SW fetch it on `install`). Out of scope for v1; tracked here so the cache-strategy table in Phase 3 has a clear next step.

---

## References

- [W3C Web App Manifest spec](https://www.w3.org/TR/appmanifest/)
- [MDN Service Worker API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)
- [web.dev — Customize the install experience](https://web.dev/learn/pwa/installation-prompt/)
- [Apple — Configuring Web Applications](https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html)
- [maskable.app — maskable icon preview](https://maskable.app)
- Existing plan precedents: `docs/plans/invite/invite-implementation.md`, `docs/plans/version-history/version-history-implementation.md`
