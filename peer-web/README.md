<picture>
    <source srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_Solid_White.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg" alt="Leptos Logo">
</picture>

# Peer Network — Leptos Frontend

Modern Leptos (Rust/WASM) rewrite of the Peer Network web frontend, replacing the legacy PHP/JavaScript stack.

## Features Implemented

- ✅ **Registration** — Multi-step flow: referral → email → password → confirmation
- ✅ **Login / Authentication** — Email/password, remember-me, JWT tokens, auto-login
- ✅ **Auth Infrastructure** — HttpOnly cookies, proactive token refresh, 401 interceptor
- ✅ **Protected Routes** — AuthGuard component with redirect preservation
- ✅ **Toast Notifications** — User feedback system
- ✅ **Accessibility** — Screen reader support, keyboard navigation

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
```

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

## License

See [LICENSE](LICENSE) for details.
