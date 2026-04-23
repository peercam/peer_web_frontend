<p align="center">
  <img src="public/img/brand/PeerLogoBlack.svg#gh-light-mode-only" alt="Peer Network logo" width="200">
  <img src="public/img/brand/PeerLogoWhite.svg#gh-dark-mode-only" alt="Peer Network logo" width="200">
</p>

# Peer Network — Leptos Frontend

A Leptos (Rust / WASM) rewrite of the Peer Network web frontend. Full-stack
Axum SSR with client-side hydration, an in-process GraphQL mock backend, and
~90% feature parity with the legacy PHP app.

**Status:** 14 / 21 pages ✅ implemented · 5 🟡 near-complete · 2 🚧 in progress
(see [docs/feature-convergence.md](docs/feature-convergence.md) for the
canonical tracker).

## Tech Stack

- **Frontend:** [Leptos 0.8](https://leptos.dev) (nightly), `leptos_router`, `leptos_meta`
- **Server:** Axum 0.8 SSR + `leptos_axum`, Tokio, `tower-http`
- **Hydration:** `wasm-bindgen`, `web-sys`, `js-sys`, `gloo-timers`
- **Data:** GraphQL over `reqwest`; `serde` / `serde_json`; `chrono`, `rust_decimal`
- **Mock backend:** `async-graphql` v7 + Axum (workspace member, [packages/mock_backend](packages/mock_backend))
- **Styles:** Dart Sass (`style/main.scss`) → Lightning CSS
- **Tests:** unit + integration (`cargo test`), `wasm-bindgen-test` ([tests-wasm](tests-wasm)), Playwright ([end2end](end2end))
- **PWA:** hand-written service worker, manifest, install prompt, iOS Add-to-Home-Screen hint

## Quickstart

Zero-friction local loop, against the bundled Rust mock backend:

```bash
# 1. Install prerequisites (see below) — once.

# 2. In one terminal: run the mock GraphQL backend on :4000
cargo run -p mock_backend

# 3. In another terminal: run the Leptos dev server on :3000
GRAPHQL_ENDPOINT=http://localhost:4000/graphql cargo leptos watch
```

Open <http://localhost:3000>. Sign in with the seeded user
`test@peer.com` / `TestPass123` (full seed data in
[packages/mock_backend/README.md](packages/mock_backend/README.md)).

### Prerequisites

1. `rustup toolchain install nightly --allow-downgrade` — Rust nightly (pinned in [rust-toolchain.toml](rust-toolchain.toml))
2. `rustup target add wasm32-unknown-unknown`
3. `cargo install cargo-leptos --locked`
4. `npm install -g sass` — Dart Sass for stylesheets
5. `(cd end2end && npm install)` — Playwright (only needed for E2E)
6. `cargo install wasm-pack --locked` — only needed for `tests-wasm`

## Project Structure

```
.
├── Cargo.toml              # workspace root + peer-web crate
├── src/
│   ├── api/                # GraphQL client, server functions, auth, download proxy
│   ├── components/         # Reusable UI (layout shell, pwa, modals, post, chat, …)
│   ├── pages/              # 19 page components (login, dashboard, chat, wallet, …)
│   ├── hooks/              # use_infinite_scroll, proactive_refresh, …
│   ├── models/             # User, Auth, Post, Wallet, …
│   ├── state/              # AuthContext, toasts, global signals
│   ├── server/             # Axum router, SSR entry
│   ├── utils/              # Cookies, tokens, validation, PWA registration
│   ├── fixtures/           # Shared test fixtures
│   ├── app.rs              # Top-level <App/> + routes
│   ├── main.rs             # Server binary entry (ssr feature)
│   └── lib.rs              # Hydrate entry (hydrate feature)
├── packages/
│   └── mock_backend/       # async-graphql mock server (266 integration tests)
├── tests/                  # Server-side integration tests (Axum, GraphQL, SSR, proxy)
├── tests-wasm/             # wasm-bindgen-test browser tests (separate crate)
├── end2end/                # Playwright E2E tests + helpers
├── style/                  # SCSS sources (compiled to target/site/pkg/peer-web.css)
├── public/                 # Static assets (manifest.webmanifest, sw.js, img/, …)
├── docs/
│   ├── feature-convergence.md
│   ├── leptos-rewrite-study.md
│   ├── adr-*.md            # Architecture Decision Records
│   ├── backend_api/        # GraphQL backend API reference
│   └── plans/              # Per-feature implementation plans + sprints
├── legacy/                 # Original PHP/JS sources (reference only)
└── AGENTS.md               # Contributor / agent conventions
```

## Configuration

### Required

```sh
# GraphQL backend endpoint (mock or real)
export GRAPHQL_ENDPOINT="https://api.peer.network/graphql"

# Production mode — enables the Secure flag on auth cookies
export LEPTOS_ENV="production"
```

### Download proxy (`/download` route)

The `/download` endpoint is a force-download media proxy that streams bytes
from an allow-listed upstream. It closes the SSRF hole present in the legacy
`download.php` ([implementation notes](docs/plans/download/download-implementation.md)).

```sh
# Comma-separated host allow-list. Only https:// URLs whose host is in
# this list will be fetched.
export DOWNLOAD_ALLOWED_HOSTS="media.peer.network,cdn.peer.network"

# Optional: hard byte cap on proxied downloads (default 256 MiB).
export DOWNLOAD_MAX_BYTES="268435456"

# Optional: total request deadline in seconds (default 300).
export DOWNLOAD_TIMEOUT_SECS="300"
```

> **Operational note:** `/download` serves anonymous clients and **must be
> rate-limited at the reverse-proxy / WAF layer** (nginx `limit_req`,
> Cloudflare, etc.) before production. There is no application-level rate
> limit.

### Deployment

```sh
export LEPTOS_OUTPUT_NAME="peer-web"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="0.0.0.0:3000"
```

## Building for Release

```bash
cargo leptos build --release
```

Outputs:

- `target/server/release/peer-web` — server binary
- `target/site/` — static assets (WASM, JS, CSS, public/)

The WASM bundle is built under the `wasm-release` profile (`opt-level = "z"`,
LTO, single codegen unit, `panic = "abort"`).

For deployment, copy the server binary alongside the `site/` directory and
set the env vars in [Deployment](#deployment) above.

## Testing

Per [AGENTS.md](AGENTS.md), default to running the **workspace** with
**all targets** and **all features** (then again with `--no-default-features`
where relevant).

```bash
# Unit + server-side integration tests (workspace, all targets, all features)
cargo test --workspace --all-targets --all-features

# Mock backend integration suite (266 tests)
cargo test -p mock_backend

# Browser component tests (real DOM, events, reactivity)
(cd tests-wasm && wasm-pack test --headless --chrome)

# End-to-end tests (Playwright)
(cd end2end && npx playwright test)
```

The four test surfaces correspond to the layers documented in
[tests-wasm/README.md](tests-wasm/README.md):

| Layer | Location | Covers |
|------:|----------|--------|
| 1 | `src/**` `#[cfg(test)]` | Pure decision logic |
| 2 | `tests-wasm/` | Real DOM, events, reactivity |
| 3 | `tests/` | SSR HTML, server functions, GraphQL, download proxy |
| 4 | `end2end/` | Routing, cookies, SW, accessibility (axe) |

## Mock Backend

[packages/mock_backend](packages/mock_backend) is an in-process Rust GraphQL
server (`async-graphql` v7 on Axum) that mirrors the production schema across
seven phases — registration, auth/session, users/profiles, posts, social
(comments + chat), economy (wallet/shop/ads), and admin/moderation. It backs
both local development and the integration test suite, with 266 tests
preserving parity with the real backend.

Bring it up with `cargo run -p mock_backend` (binds `:4000`). See its
[README](packages/mock_backend/README.md) for seeded users, referral codes,
and the full operation list.

## Progressive Web App

The PWA layer (manifest, hand-written service worker at
[public/sw.js](public/sw.js), install prompt, update-available toast, iOS
Add-to-Home-Screen hint) is documented in
[docs/plans/pwa/pwa-implementation.md](docs/plans/pwa/pwa-implementation.md).

A few practical dev tips:

- **Update on reload** — In Chrome DevTools, *Application → Service Workers
  → Update on reload*. Forces a fresh install/activate cycle on every reload.
- **Full unregister via `?nosw`** — append `?nosw` to any URL and reload to
  unregister all service workers (e.g. `http://localhost:3000/dashboard?nosw`).
- **HTTPS required** — service workers are disabled outside `localhost` and
  `https://`. Plain-HTTP staging deploys silently have no SW and no install
  prompt.
- **iOS** — Safari does not fire `beforeinstallprompt`; install is the system
  *Share → Add to Home Screen* flow. The `IosHint` component shows a one-shot
  nudge.
- **Auth-sensitive routes are never cached** — `/api/*`, `/graphql`, and
  `/admin/*` bypass the cache. New authenticated routes should route through
  those prefixes or extend the network-only list in `sw.js`.

## Documentation

- [docs/feature-convergence.md](docs/feature-convergence.md) — migration progress (canonical)
- [docs/leptos-rewrite-study.md](docs/leptos-rewrite-study.md) — architecture decisions
- [docs/backend_api/](docs/backend_api/) — GraphQL backend API reference
- [docs/plans/](docs/plans/) — per-feature implementation plans + completion sprints
- [docs/adr-chat-realtime-transport.md](docs/adr-chat-realtime-transport.md)
- [docs/adr-mock-backend-rust-rewrite.md](docs/adr-mock-backend-rust-rewrite.md)
- [docs/adr-repo-layout-promote-peer-web.md](docs/adr-repo-layout-promote-peer-web.md)
- [packages/mock_backend/README.md](packages/mock_backend/README.md)
- [tests-wasm/README.md](tests-wasm/README.md)

## Contributing

Read [AGENTS.md](AGENTS.md) before opening a PR. Two rules in particular:

1. Run cargo commands across the **workspace** with **all targets** and
   **all features**, then again with `--no-default-features` where relevant.
2. Do **not** pipe terminal output through `tail`, `grep`, or `head` in
   automated harnesses — the harness paginates output for you. `tee` to a
   temp file if you need to retain output.

## License

See [LICENSE](LICENSE) for details.

<picture>
    <source srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_Solid_White.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg" alt="Leptos Logo">
</picture>
