<p align="center">
  <img src="public/img/brand/PeerLogoBlack.svg#gh-light-mode-only" alt="Peer Network logo" width="200">
  <img src="public/img/brand/PeerLogoWhite.svg#gh-dark-mode-only" alt="Peer Network logo" width="200">
</p>

# Peer Network — Leptos Frontend SPIKE!

Modern Leptos (Rust/WASM) rewrite-spike of the Peer Network web frontend, to test how one could replace the legacy PHP/JavaScript stack.

## Features Implemented

- ✅ **Registration** — Multi-step flow: referral → email → password → confirmation
- ✅ **Login / Authentication** — Email/password, remember-me, JWT tokens, auto-login
- ✅ **Auth Infrastructure** — HttpOnly cookies, proactive token refresh, 401 interceptor
- ✅ **Protected Routes** — AuthGuard component with redirect preservation
- ✅ **Toast Notifications** — User feedback system
- ✅ **Accessibility** — Screen reader support, keyboard navigation
- ✅ **Progressive Web App** — Installable manifest, offline shell, update-available toast, iOS Add-to-Home-Screen hint ([plan](../docs/plans/pwa/pwa-implementation.md))

## Project Structure

```
src/
├── api/           # GraphQL client, server functions, auth
├── components/    # Reusable UI components
├── hooks/         # Custom hooks (proactive refresh, etc.)
├── models/        # Data types (user, auth, etc.)
├── pages/         # Page components (login, register, etc.)
├── state/         # Global state (AuthContext)
└── utils/         # Helpers (cookies, tokens, validation)
```

## Development

```bash
cargo leptos watch
```

### Environment Variables

```sh
# Required: GraphQL backend endpoint
export GRAPHQL_ENDPOINT="https://api.peer.network/graphql"

# Production mode (enables Secure cookie flag)
export LEPTOS_ENV="production"

# Download proxy (`/download` route)
# Comma-separated host allow-list of upstream CDN hosts the proxy may fetch from.
# The endpoint ONLY accepts `https://` URLs whose host is in this list — this
# closes the SSRF hole in the legacy `download.php`.
export DOWNLOAD_ALLOWED_HOSTS="media.peer.network,cdn.peer.network"

# Optional: hard byte cap on proxied downloads (default 256 MiB).
export DOWNLOAD_MAX_BYTES="268435456"

# Optional: total request deadline in seconds (default 300).
export DOWNLOAD_TIMEOUT_SECS="300"
```

> **Operational note on `/download`:** this endpoint streams bytes from an
> allow-listed upstream to anonymous clients. It **must be rate-limited at the
> reverse proxy / WAF layer** (nginx `limit_req`, Cloudflare, etc.) before
> production deployment — there is no application-level rate limit.

## Prerequisites

1. `rustup toolchain install nightly --allow-downgrade` — Rust nightly
2. `rustup target add wasm32-unknown-unknown` — WASM compilation target
3. `cargo install cargo-leptos --locked` — Leptos build tool
4. `npm install -g sass` — Dart Sass for stylesheets
5. `npm install` in `end2end/` — Playwright for E2E tests

## Building for Release

```bash
cargo leptos build --release
```

Outputs:
- Server binary: `target/server/release/peer-web`
- Static assets: `target/site/`

## Testing

```bash
# Unit tests
cargo test

# E2E tests
cargo leptos end-to-end
```

## Deployment

Copy the server binary and `site/` directory to your server:

```text
peer-web          # Server binary
site/             # Static assets
```

Required environment variables:

```sh
export LEPTOS_OUTPUT_NAME="peer-web"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="0.0.0.0:3000"
export GRAPHQL_ENDPOINT="https://api.peer.network/graphql"
export LEPTOS_ENV="production"
```

## Documentation

- [Feature Convergence Tracker](../docs/feature-convergence.md) — Migration progress
- [Login Implementation](../docs/plans/login/login-auth-implementation.md) — Auth details
- [Leptos Rewrite Study](../docs/leptos-rewrite-study.md) — Architecture decisions
- [PWA Implementation](../docs/plans/pwa/pwa-implementation.md) — Manifest, service worker, install prompt

## Service Worker — Dev Gotchas

The service worker (`public/sw.js`) is registered on hydrate by
`src/utils/pwa.rs` at `/sw.js?v=<BUILD_HASH>`. The cache name keys off that
query string (`peer-shell-v{HASH}`), so a fresh `cargo leptos watch` run
against the same `CARGO_PKG_VERSION` **reuses** the previous cache. A few
practical tips:

- **Update on reload** — In Chrome DevTools, open *Application → Service
  Workers* and tick **“Update on reload”**. Every page reload then forces
  a fresh SW install + activate cycle, which is what you want while
  iterating on frontend code.
- **Full unregister via `?nosw`** — Append `?nosw` to any URL (e.g.
  `http://localhost:3000/dashboard?nosw`). The registration module calls
  `getRegistrations()` and unregisters each one instead of registering a
  new worker. Reload once more to confirm `navigator.serviceWorker.controller`
  is `null`.
- **`GIT_SHA`** — CI sets `GIT_SHA=$(git rev-parse --short HEAD)` before
  `cargo leptos build` so every deploy gets its own cache bucket. Local
  dev falls back to `CARGO_PKG_VERSION`; combined with “Update on reload”
  this is fine.
- **iOS limitations** — iOS Safari does **not** fire `beforeinstallprompt`;
  the install path is the system **Share → Add to Home Screen** flow. The
  `IosHint` component shows a one-shot nudge on iPhone/iPad UAs when the
  site isn’t already running standalone.
- **HTTPS required** — Service workers are disabled outside of `localhost`
  and `https://`. Production is HTTPS, but any staging/dev deploy over
  plain HTTP will silently have no SW + no install prompt.
- **Auth-sensitive routes are never cached** — `/api/*`, `/graphql`, and
  `/admin/*` bypass the cache entirely. If you add a new authenticated
  route, route it through those prefixes or extend the network-only list
  in `sw.js`.

## License

See [LICENSE](LICENSE) for details.

<picture>
    <source srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_Solid_White.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg" alt="Leptos Logo">
</picture>
