# Home / Landing Page Implementation Plan

**Feature:** Home / Landing (`/`)
**Priority:** Next — closes the highest-visibility gap currently in the convergence tracker
**Status:** ❌ Not Started
**Created:** 2026-04-22

---

## Overview

Replace the placeholder `HomePage` component in [peer-web/src/app.rs](../../../peer-web/src/app.rs) with a proper landing route that achieves parity with the legacy `index.php` behaviour and stops shipping bare-HTML "Welcome to Peer" markup as the first thing visitors see.

### What the legacy actually does

The legacy [index.php](../../../index.php) is **three lines**:

```php
<?php
header("Location: dashboard.php");
exit();
```

It is a server-side 302 to `dashboard.php`. `dashboard.php` is wrapped in `checkAuth("unauthorized")`, which itself redirects unauthenticated users to `login.php`. Net effect: visiting `/` ends up at `/dashboard` (if signed in) or `/login` (if not).

### What we ship today

Visiting `/` in the Leptos build renders:

```html
<h1>Welcome to Peer</h1>
<a href="/register">Create an Account</a>
```

— with no shared chrome, no styling, no auth awareness, no service-worker offline shell wrapping, and no `<Title>` parity with the rest of the app. This was documented in the 2026-04-22 changelog entry of [`feature-convergence.md`](../../feature-convergence.md).

### Why this is the next feature

1. **Highest-visibility gap.** Every uncached cold load of the production hostname (and every PWA `start_url` launch — see below) lands here.
2. **Tiny surface area.** No new API calls, no new GraphQL operations, no mock-backend changes, no SCSS module of any meaningful size.
3. **Unblocks the PWA `start_url`.** [public/manifest.webmanifest](../../../peer-web/public/manifest.webmanifest) sets `"start_url": "/"`, so every "Add to Home Screen" launch currently lands on the placeholder. Fixing `/` fixes the installed-app first-run experience for free.
4. **Removes a footgun for E2E tests.** Playwright specs that probe `/` get nondeterministic content depending on whether anyone has wired `<HomePage/>` to anything.

### Goals

1. Functional parity with legacy `index.php`: visiting `/` ends up on `/dashboard` (authed) or `/login` (guest). Note: the guest-redirect message deliberately changes from legacy's `?message=unauthorized` to `?message=mustLogin` so the SPA stays consistent with [`AuthGuard`](../../../peer-web/src/components/auth_guard.rs)'s convention. Both values render valid copy in [`login.rs`](../../../peer-web/src/pages/login.rs).
2. Zero flash of placeholder content during the auth-check window — match the existing pattern in [`AuthGuard`](../../../peer-web/src/components/auth_guard.rs).
3. SSR-safe: server-rendered HTML for `/` should not contain "Welcome to Peer" any more (including the `<Title>`).
4. Preserve incoming `?redirect=…` through to `/login`; replace any incoming `?message=…` with `mustLogin`. Other arbitrary query keys are dropped, mirroring `AuthGuard`'s drop-everything-but-`message`/`redirect` posture. Note: this is **new behaviour**, not inherited — `AuthGuard` builds `redirect=` from `location.pathname`, never from an inbound `?redirect=` query value (see [`auth_guard.rs`](../../../peer-web/src/components/auth_guard.rs) lines 21–40). `HomePage` is the first place in the SPA that does inbound-query passthrough.
5. No regression to the PWA install / offline shell behaviour.

### Non-Goals

- A marketing landing page, hero imagery, screenshots, or feature copy. Legacy never had this at `/`; it was always a redirect.
- Extracting shared `Header` / `Footer` / `Sidebars` components. These are tracked separately in the Components table of the convergence doc and are **not** in scope here. (Confusingly, the 2026-04-22 changelog entry implied that a marketing surface had been "lost"; review of `index.php` shows that was never true.)
- A guest-browsing experience for `/` (e.g. a public feed). Out of scope; tracked for a future plan if/when a product decision is made.
- Localisation / i18n of the landing copy. Nothing to localise once the redirect lands.

---

## Scope

### In Scope

- [ ] Replace the `HomePage` component in [peer-web/src/app.rs](../../../peer-web/src/app.rs)
- [ ] Auth-aware redirect: `/` → `/dashboard` when `is_authenticated` is true
- [ ] Auth-aware redirect: `/` → `/login?message=mustLogin` when `is_session_checked` is true and `is_authenticated` is false
- [ ] Loading sentinel during the `is_session_checked == false` window (visually identical to `AuthGuard`'s `auth-guard-loading` div so QA sees one consistent loader)
- [ ] Preserve incoming `redirect` and `message` query params: if a visitor lands at `/?redirect=%2Fwallet`, the eventual bounce must carry that through (today's `AuthGuard` already does this for `redirect`; we just need to not clobber it)
- [ ] `<Title>` tag — set to `"Peer Network"` for parity with how [`dashboard.php`](../../../dashboard.php) titles itself (`Peer Network - Dashboard`); the `/` redirect is brief enough that the title mostly matters for the SSR HTML and any browser tab that hangs on a slow auth check
- [ ] Update the route registration to keep `/` mapped to the new component
- [ ] Update the [feature-convergence.md](../../feature-convergence.md) tracker (Home/Landing row, Summary counts, Migration Priority, Changelog)

### Out of Scope (Future Work)

- Server-side 302 from `/` (would require an Axum route mounted before `leptos_routes`). The Leptos client-side `<Redirect/>` approach matches how `EditProfileRedirect` is already implemented in `app.rs` and keeps the routing logic colocated with the rest of the SPA. Revisit only if SEO / first-byte-redirect telemetry says we need it.
- Public guest landing experience (see Non-Goals).
- Removing the `auth-guard-loading` div in favour of a branded splash. Tracked separately under future "Layout components" work.

---

## Legacy Implementation Analysis

### Files

| File | Purpose | Lines |
|------|---------|-------|
| [`index.php`](../../../index.php) | Hard 302 to `dashboard.php` | 3 |
| [`dashboard.php`](../../../dashboard.php) | `checkAuth("unauthorized")` then renders dashboard chrome | ~80 (head + body) |
| [`auth.php`](../../../auth.php) (`checkAuth`) | Redirects unauthenticated visitors to `login.php?message=…` | n/a |

### Behavioural Contract

Following the redirect chain end-to-end:

```
GET /                      → 302 /dashboard.php
GET /dashboard.php (guest) → 302 /login.php?message=unauthorized
GET /dashboard.php (auth)  → 200 (renders dashboard)
```

The Leptos port substitutes `?message=mustLogin` for `?message=unauthorized` so the guest bounce matches what [`AuthGuard`](../../../peer-web/src/components/auth_guard.rs) emits everywhere else in the SPA. Both messages are handled by [`login.rs`](../../../peer-web/src/pages/login.rs) and surface user-appropriate copy.

Two consequences worth preserving:

1. The **guest** path lands on the login page, **not** a marketing page. There is no public landing surface to port.
2. The **authed** path lands on the dashboard, **not** a personalised "for you" surface. The dashboard is already implemented in Leptos.

So the entire job of `/` is "be the redirect node that picks the right destination."

---

## Architecture

### Component Structure

```
peer-web/src/app.rs
└── HomePage  (rewritten — no new file)
    ├── reads use_auth()
    ├── reads use_location() to capture incoming query string
    ├── waits on is_session_checked
    └── emits <Redirect path=…/> to /dashboard or /login
```

No new modules, no new files, no new SCSS. This is intentionally a single-component change inside `app.rs`.

### Redirect Decision Table

| `is_session_checked` | `is_authenticated` | Render |
|----------------------|--------------------|--------|
| `false` | — | `<div class="auth-guard-loading" role="status" aria-busy="true"></div>` |
| `true`  | `true`  | `<Redirect path="/dashboard"/>` |
| `true`  | `false` | `<Redirect path="/login?message=mustLogin[&redirect=…]"/>` |

Query-param rules (guest case):

- `message` — always set to `mustLogin`; any incoming `message` is discarded.
- `redirect` — if present on the incoming URL, passed through to the outbound URL. **Encoding contract:** `use_query_map()` / `use_query` returns *decoded* values, so a visitor landing on `/?redirect=%2Fwallet` exposes the value `/wallet` to the component. The helper must **re-encode** before concatenating it into the outbound URL — do not paste the decoded value in raw. Reuse the exact same allow-list `AuthGuard` uses (`A–Z a–z 0–9 - _ . ~ /`, everything else `%XX`) so the two code paths can never drift; lift it into a tiny `pub(crate) fn encode_redirect(&str) -> String` in `state::auth` (or a sibling util module) and call it from both `HomePage` and `AuthGuard`.
- All other incoming query keys — dropped. Keeps the helper trivial; revisit only if a real use-case for arbitrary key passthrough appears.

Query-param rules (authed case): incoming query is dropped. `/dashboard` does not consume any params today, and the legacy 302 likewise dropped them.

### Why Client-Side Redirect

Leptos' `<Redirect/>` runs **after** hydration on the client. The closer precedent to copy from is [`AuthGuard`](../../../peer-web/src/components/auth_guard.rs), which already pairs a `<Show>` on `is_session_checked` with a `<Redirect/>` fallback — exactly the shape `HomePage` needs. (`EditProfileRedirect` in `app.rs` solves a related but simpler problem with `use_navigate()` inside an `Effect`; it has no auth gate and so isn't quite the right template here.)

**Observed SSR behaviour (verified 2026-04-22, `leptos_router` 0.7):** because the outer `<Show when=is_session_checked>` resolves to its *fallback* during SSR (the `check_session` `Resource` is unresolved server-side), the inner `<Redirect/>` never renders and so no `<meta http-equiv="refresh">` is emitted. The SSR shell ships the `auth-guard-loading` sentinel with `<title>Peer Network</title>` and zero placeholder copy. The redirect fires on the client immediately after hydration once the session-check Resource resolves. This is consistent with the rest of the SPA's JS-required posture (see Open Question 3) and is **not** a regression vs the prior placeholder, which also required JS to be useful.

We deliberately do **not** mount an Axum handler at `/` because:

- It would shadow the Leptos route and break SSR for any future use of `/`.
- The auth state lives in HttpOnly cookies that the existing server-fn `check_session` already consults — no benefit to re-implementing that decision in `main.rs`.
- The existing PWA service worker's network-first navigation strategy already works for the SPA route; an Axum 302 would need its own SW handling.

---

## Implementation Plan

### Phase 1 — Component Rewrite (~30 LOC)

**File:** [`peer-web/src/app.rs`](../../../peer-web/src/app.rs)

1. Add imports:
   - `use leptos_router::components::Redirect;`
   - `use leptos_router::hooks::use_location;`
   - `use crate::state::auth::use_auth;`
2. Replace the existing `HomePage` body (currently the `<Title text="Welcome to Peer"/>` + `<h1>Welcome…</h1>` placeholder) with the auth-aware redirect logic per the decision table above. Update the `<Title>` to `"Peer Network"`.
3. Build a small helper inside the component (closure over `use_location`) that reads the incoming query, extracts `redirect` if present, and returns the guest destination URL. Keep it inline — this is a one-shot use, no need for a util module.

**Acceptance for Phase 1:**

- [ ] `cargo build --features ssr` clean
- [ ] `cargo build --features hydrate --target wasm32-unknown-unknown` clean
- [ ] `cargo clippy --all-features -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] Manual smoke (`cargo leptos watch`, with the service worker unregistered or a hard reload to bypass any cached `/` HTML):
  - [ ] Visiting `/` while signed out lands on `/login?message=mustLogin`
  - [ ] Visiting `/` while signed in lands on `/dashboard`
  - [ ] Visiting `/?redirect=%2Fwallet` while signed out lands on `/login?message=mustLogin&redirect=%2Fwallet`
  - [ ] Visiting `/?message=somethingElse` while signed out lands on `/login?message=mustLogin` (incoming `message` is discarded)
  - [ ] No flash of "Welcome to Peer" markup during the session-check window
  - [x] View-source on `/` (hydrated DOM) no longer contains the literal string `Welcome to Peer` (body **or** `<title>`)
  - [x] **Raw SSR check:** `curl -s http://127.0.0.1:3000/ | tee /tmp/home-ssr.html` then verify no `Welcome to Peer` substring anywhere. **Verified 2026-04-22:** body contains only `<div role="status" aria-busy="true" class="auth-guard-loading"></div>`; `<title>` is `Peer Network`. No `http-equiv="refresh"` is emitted (the outer `<Show>` short-circuits to the loading fallback in SSR — see Architecture → Why Client-Side Redirect for the documented outcome). The client-side redirect fires post-hydration as designed.

### Phase 2 — E2E Test Coverage (~40 LOC)

**File:** new `peer-web/end2end/tests/home.spec.ts`

Three Playwright cases, modelled on the existing [`pwa.spec.ts`](../../../peer-web/end2end/tests/pwa.spec.ts):

1. **Guest redirect.** Fresh context (no cookies) → `goto('/')` → expect URL to settle on `/login?message=mustLogin`.
2. **Authed redirect.** Sign in (see fixture note below) using `test@peer.com` / `TestPass123` from the mock backend → `goto('/')` → expect URL to settle on `/dashboard`.
3. **Query preservation.** Fresh context → `goto('/?redirect=%2Fwallet')` → expect URL to settle on `/login?message=mustLogin&redirect=%2Fwallet`.

**Fixture note.** There is no shared login fixture today — [`wallet.spec.ts`](../../../peer-web/end2end/tests/wallet.spec.ts) inlines its own `async function login(page, email, password)` helper (lines 30–34: `goto('/login')` → fill `#loginEmail` / `#loginPassword` → submit). Two acceptable options, in order of preference:

- **(a)** Inline the same 5-line `login()` helper in `home.spec.ts`. Cheapest, matches existing conventions, no cross-spec coupling. **Default choice.**
- **(b)** Extract the helper into `peer-web/end2end/helpers/login.ts` and migrate `wallet.spec.ts` to use it. Cleaner long-term but doubles the diff and pulls an unrelated spec into the PR. Defer to a follow-up unless review specifically asks for it.

**Acceptance for Phase 2:**

- [ ] `npx playwright test home.spec.ts` passes locally against the Rust mock backend
- [ ] Login helper is either inlined per option (a) or, if option (b) is chosen, the `wallet.spec.ts` migration is in the same PR and still green

### Phase 3 — Convergence Tracker Update (~20 LOC of markdown)

**File:** [`docs/feature-convergence.md`](../../feature-convergence.md)

1. Pages table — Home/Landing row: 🚧 In Progress → ✅ Implemented; replace gap notes with one-liner pointing at this plan
2. Summary counts: bump ✅ count by +1 and 🚧 count by −1, then recompute the convergence percentages from the actual table at PR time (do **not** trust the counts quoted at plan-authoring time — re-derive)
3. Migration Priority: insert Home as a completed entry (or fold into the changelog if no number is allocated)
4. Changelog: new dated entry summarising the change, files touched, and pointing at this plan
5. Bump **Last Updated** date

### Phase 4 — Cleanup Verification

- [x] Grep the repo for any remaining hardcoded reference to the placeholder copy (`Welcome to Peer`) and remove if dead. **Verified 2026-04-22:** removed stale `peer-web/end2end/tests/example.spec.ts` (Playwright starter that asserted the old placeholder title/h1; superseded by `home.spec.ts`). Remaining hits (`register.rs` post-signup success message, `home-implementation.md` plan prose) are intentional and unrelated.
- [ ] Confirm the PWA `start_url` (`/`) installs cleanly and lands on the right destination after auth — both with and without an existing session cookie
- [ ] Confirm the offline fallback (`offline.html`) is unaffected — the SW's network-first navigation still serves it when `/` is requested offline

---

## File Manifest

### Modified

| File | Approx. Δ LOC | Purpose |
|------|---------------|---------|
| `peer-web/src/app.rs` | +25 / −6 | Rewrite `HomePage` (mirrors the `<Show>`/`<Show>` shape of `AuthGuard`); current placeholder body is 5 lines (lines 128–133) |
| `peer-web/src/components/auth_guard.rs` | +1 / −10 | Switch the inline percent-encoding loop to the shared `encode_redirect` helper introduced for `HomePage` (see Architecture → Query-param rules) |
| `peer-web/src/state/auth.rs` *(or sibling util module)* | +12 / −0 | New `pub(crate) fn encode_redirect(&str) -> String` shared by `HomePage` and `AuthGuard` |
| `docs/feature-convergence.md` | +20 / −5 | Tracker + changelog update |

### New

| File | Approx. LOC | Purpose |
|------|-------------|---------|
| `peer-web/end2end/tests/home.spec.ts` | ~40 | Three E2E cases |
| `docs/plans/home/home-implementation.md` | this doc | Plan |

### Not Touched

- No new SCSS — the loading sentinel reuses the existing `auth-guard-loading` selector
- No new API modules
- No new mock-backend resolvers — `check_session` is already complete (Phase 1 of the mock-backend rewrite)
- No new components in `src/components/`
- No changes to routing other than the `HomePage` body itself

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Redirect loop if `is_authenticated` flips during the redirect | Low | High | The `<Redirect/>` component is fire-and-forget; once we leave `/` the `HomePage` unmounts. Loop only possible if `/dashboard` itself bounces back to `/`, which `AuthGuard` does not do (it bounces to `/login`). |
| SSR HTML still contains "Welcome to Peer" if hydration races | Low | Medium | The outer `<Show when=is_session_checked>` resolves to the `auth-guard-loading` fallback in SSR (the session-check `Resource` is unresolved server-side), so the placeholder body is never rendered. Verified 2026-04-22 by raw `curl` + grep (Phase 1 acceptance). |
| Lost query params (`message`, `redirect`) | Medium | Medium | Explicit preservation in the redirect-URL builder, covered by Phase 2 E2E case 3. |
| Service worker caches the old "Welcome to Peer" HTML for `/` | Medium | Medium | The SW uses network-first for navigations and bumps its cache key off `BUILD_HASH`; the next deploy invalidates the stale entry. No code change needed, but local QA must hard-reload / unregister the SW or they will see stale markup, and the PR description should call this out. |
| User pastes a deep link to a guest-only page (e.g. `/login`) into `/` somehow | Very Low | Low | Out of scope — `/login` is its own route and is not affected by this plan. |

---

## Open Questions

1. **Should `/` be a server-side 302 instead?** Default answer: no (see "Why Client-Side Redirect"). Revisit only if SEO bots fail to follow the meta-refresh, which we have no evidence of today.
2. **Should we set `<Title>` to anything user-visible during the redirect?** **Resolved (2026-04-22):** shipped as `"Peer Network"`. The page is on screen for ~hundreds of ms; not worth a translation key. The brief title flash from `Peer Network` → `Peer Network - Dashboard` on the authed path was deemed acceptable.
3. **Should we add a `noscript` fallback that does an HTML form-submit redirect?** Probably overkill — every other page in the app requires JS. Tracked as a future hardening task if a no-JS audit is requested.

---

## Definition of Done

- [ ] All Phase 1 acceptance items checked
- [ ] All Phase 2 E2E cases passing in CI (or locally if CI for E2E is not yet wired — see Mock Backend `phase-ci-integration` plan, which is still ❌ Not Started)
- [ ] Phase 3 tracker update merged in the same PR
- [ ] Phase 4 cleanup verification done and noted in PR description
- [ ] No clippy warnings, no fmt drift, both feature flags build clean
- [ ] PR description includes a screenshot or short clip of the before/after redirect behaviour
- [ ] PR description calls out the legacy → SPA banner change (`?message=unauthorized` → `?message=mustLogin`) so QA knows the login-page copy will read "Please log in to access your dashboard." instead of "You do not have access. Please log in to continue." for visitors arriving via `/`
