# Step 1 — Scaffold Leptos Project

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Initialize a `cargo-leptos` project in a new `peer-web/` directory alongside the existing PHP app, configure it for SSR + CSR, point it at the mock backend, and verify that it compiles, serves, and passes default tests.

---

## 1.1 — Prerequisites

Before starting, confirm every prerequisite from the master plan:

| Requirement | Verification command | Expected |
|-------------|---------------------|----------|
| Rust toolchain (stable) | `rustup show active-toolchain` | `stable-aarch64-apple-darwin` (or similar stable) |
| `wasm32-unknown-unknown` target | `rustup target list --installed \| grep wasm32` | `wasm32-unknown-unknown` present |
| `cargo-leptos` CLI | `cargo leptos --version` | `0.2.x` or later |
| Node.js (for mock backend) | `node --version` | `v18+` |
| Mock backend operational | `curl -s http://localhost:4000/graphql -X POST -H 'Content-Type: application/json' -d '{"query":"{ _health }"}'` | Returns JSON |

### Install missing tools

```bash
# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# cargo-leptos
cargo install cargo-leptos

# cargo-generate (used by `cargo leptos new`)
cargo install cargo-generate
```

---

## 1.2 — Create the Leptos project

Run from the repository root (`peer_web_frontend/`):

```bash
cargo leptos new peer-web
```

When prompted, select:
- **Template:** `Axum` (not Actix)
- **Nightly Rust:** No (use stable with `--features ssr,hydrate`)
- **Tailwind:** No (we'll use the existing `css/login-register.css`)
- **End-to-end testing:** Yes (Playwright — we'll use this in Step 15)

This creates:

```
peer-web/
├── Cargo.toml
├── src/
│   ├── main.rs          ← Axum server entry point
│   ├── lib.rs           ← Crate root, feature gates
│   ├── app.rs           ← Root <App/> with default router
│   └── error_template.rs← Error UI
├── style/
│   └── main.scss        ← Default styles (we'll replace later)
├── public/              ← Static assets
├── end2end/             ← Playwright config (if selected)
└── .cargo/
    └── config.toml      ← Build target hints
```

---

## 1.3 — Verify initial compilation

Before making any changes, confirm the generated project compiles as-is:

```bash
cd peer-web
cargo leptos build
```

**Expected:** Build succeeds for both SSR (native) and CSR (WASM) targets. Output ends with `Finished` and no errors.

If compilation fails due to nightly features, switch to the stable-compatible template or pin the Leptos version (see §1.4).

---

## 1.4 — Configure `Cargo.toml`

Edit `peer-web/Cargo.toml` to pin Leptos versions, add required dependencies, and configure features.

### Dependencies

```toml
[package]
name = "peer-web"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos = { version = "0.7", features = [] }
leptos_meta = { version = "0.7" }
leptos_router = { version = "0.7" }
leptos_axum = { version = "0.7", optional = true }
leptos_integration_utils = { version = "0.7", optional = true }

# Server-side
axum = { version = "0.8", optional = true }
tokio = { version = "1", features = ["rt-multi-thread", "macros"], optional = true }
tower = { version = "0.5", optional = true }
tower-http = { version = "0.6", features = ["fs", "cors"], optional = true }
http = { version = "1", optional = true }
reqwest = { version = "0.12", features = ["json"], optional = true }

# Shared
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cfg-if = "1"
thiserror = "2"
wasm-bindgen = "=0.2.100"
console_error_panic_hook = "0.1"

[features]
hydrate = ["leptos/hydrate"]
ssr = [
    "dep:axum",
    "dep:tokio",
    "dep:tower",
    "dep:tower-http",
    "dep:http",
    "dep:reqwest",
    "dep:leptos_axum",
    "dep:leptos_integration_utils",
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
]

[package.metadata.leptos]
# Name of the binary target
bin-target = "peer-web"
# The site root
site-root = "target/site"
# The site-pkg directory within site-root
site-pkg-dir = "pkg"
# The port to run the server on during `cargo leptos watch`
site-addr = "127.0.0.1:3000"
# Assets source directory
assets-dir = "public"
# Style file (SCSS or CSS)
style-file = "style/main.scss"
```

### Key decisions

| Choice | Rationale |
|--------|-----------|
| Leptos 0.7 | Latest stable release line; supports stable Rust |
| Axum 0.8 | Current stable Axum; matches leptos_axum expectations |
| `reqwest` for SSR only | GraphQL calls happen server-side via server functions, not in WASM |
| `serde` everywhere | Needed for (de)serializing GraphQL request/response types |
| `tower-http` with `cors` | Allows the Leptos dev server to call the mock backend on port 4000 |
| Port 3000 | Avoids collision with mock backend (4000) and common tools |

---

## 1.5 — Environment configuration

Create `peer-web/.env` for local development:

```env
GRAPHQL_ENDPOINT=http://localhost:4000/graphql
LEPTOS_SITE_ADDR=127.0.0.1:3000
RUST_LOG=info
```

Ensure the Axum server reads this at startup. In `src/main.rs`, confirm (or add):

```rust
use std::env;

// Inside main():
let graphql_endpoint = env::var("GRAPHQL_ENDPOINT")
    .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string());
```

> **Security note:** `.env` must never contain production secrets. Add a `.env.example` with placeholder values and add `.env` to `.gitignore`.

---

## 1.6 — Update `.gitignore`

Add the following to the root `.gitignore` (or create `peer-web/.gitignore`):

```gitignore
# Rust / Leptos build artifacts
/peer-web/target/
/peer-web/.env

# WASM build cache
/peer-web/pkg/
```

---

## 1.7 — Verify default app serves

Start the development server:

```bash
cd peer-web
cargo leptos watch
```

**Expected output (terminal):**

```
Compiling peer-web v0.1.0
    ...
    Finished `release` profile [optimized] target(s)
💿 serving at http://127.0.0.1:3000
```

**Expected behaviour (browser):**

1. Navigate to `http://localhost:3000`.
2. The default Leptos welcome page renders (typically "Welcome to Leptos!" with a counter button).
3. Clicking the counter button increments the count (proves WASM hydration is working).
4. View page source — HTML contains the rendered content (proves SSR is working).

---

## 1.8 — Verify `cargo test` passes

```bash
cd peer-web
cargo test
```

**Expected:** All default tests pass (typically 0 or 1 tests in the generated template). Zero compilation warnings from our code (dependency warnings are acceptable).

---

## 1.9 — Verify environment variable is accessible

Add a temporary smoke test to confirm the env var is reachable from server context.

Create or edit `peer-web/src/main.rs` to log the endpoint at startup:

```rust
println!("GraphQL endpoint: {}", 
    std::env::var("GRAPHQL_ENDPOINT").unwrap_or_else(|_| "(not set)".into()));
```

Run `cargo leptos watch` and confirm the terminal output includes:

```
GraphQL endpoint: http://localhost:4000/graphql
```

Once verified, this println can be left in (it's useful during development) or wrapped behind `RUST_LOG=debug`.

---

## 1.10 — (Optional) Verify mock backend connectivity

If the mock backend from Step 0 is running, do a quick connectivity test from inside the Leptos project. Create a temporary integration test:

```rust
// peer-web/tests/mock_connectivity.rs
#[tokio::test]
async fn mock_backend_is_reachable() {
    let endpoint = std::env::var("GRAPHQL_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4000/graphql".to_string());
    
    let client = reqwest::Client::new();
    let res = client
        .post(&endpoint)
        .json(&serde_json::json!({
            "query": "{ _health }"
        }))
        .send()
        .await
        .expect("Failed to reach mock backend");
    
    assert!(res.status().is_success());
}
```

Run with the mock backend active:

```bash
# Terminal 1: (from repo root)
cd tests/mock_backend && npm start

# Terminal 2:
cd peer-web && cargo test -- mock_connectivity
```

**Expected:** Test passes. This confirms the Leptos project can talk to the mock backend over HTTP.

---

## 1.11 — Directory layout after Step 1

```
peer_web_frontend/               ← existing repo root
├── register.php                 ← existing PHP app (unchanged)
├── js/register/register.js      ← existing JS (unchanged)
├── css/login-register.css       ← existing CSS (unchanged)
├── tests/mock_backend/          ← from Step 0
├── docs/
│   └── plans/
│       ├── registration-leptos-migration.md
│       ├── step0-mock-backend.md
│       └── step1-scaffold-leptos-project.md   ← this document
└── peer-web/                    ← NEW from this step
    ├── Cargo.toml
    ├── .env
    ├── .gitignore
    ├── public/
    ├── style/
    │   └── main.scss
    ├── src/
    │   ├── main.rs
    │   ├── lib.rs
    │   ├── app.rs
    │   └── error_template.rs
    ├── tests/
    │   └── mock_connectivity.rs
    └── end2end/                 ← Playwright (optional scaffold)
```

---

## 1.12 — Implementation checklist

- [ ] **1.12a** Confirm all prerequisites are installed (§1.1).
- [ ] **1.12b** Run `cargo leptos new peer-web` with the Axum template (§1.2).
- [ ] **1.12c** Verify the generated project compiles as-is with `cargo leptos build` (§1.3).
- [ ] **1.12d** Edit `Cargo.toml` — pin Leptos version, add `reqwest`, `serde`, `tower-http` dependencies, and configure `[package.metadata.leptos]` (§1.4).
- [ ] **1.12e** Create `.env` with `GRAPHQL_ENDPOINT` (§1.5).
- [ ] **1.12f** Update `.gitignore` to exclude `target/`, `.env`, `pkg/` (§1.6).
- [ ] **1.12g** Run `cargo leptos watch` — confirm the welcome page renders at `localhost:3000` with working SSR and hydration (§1.7).
- [ ] **1.12h** Run `cargo test` — confirm all tests pass (§1.8).
- [ ] **1.12i** Verify `GRAPHQL_ENDPOINT` env var is accessible from server code (§1.9).
- [ ] **1.12j** (Optional) Run mock connectivity test to confirm Leptos can reach the mock backend (§1.10).

---

## 1.13 — Troubleshooting

| Problem | Likely cause | Fix |
|---------|-------------|-----|
| `cargo leptos new` fails with "command not found" | `cargo-leptos` not installed | `cargo install cargo-leptos` |
| `cargo leptos new` fails with "cargo-generate not found" | Missing generator tool | `cargo install cargo-generate` |
| WASM compilation fails: "target not found" | Missing WASM target | `rustup target add wasm32-unknown-unknown` |
| `error[E0554]: #![feature] may not be used on the stable release channel` | Template requires nightly | Either run `rustup default nightly` or choose a stable-compatible template; Leptos 0.7 supports stable |
| `cargo leptos watch` builds but page is blank | WASM not loading; check browser console | Ensure `site-pkg-dir` in `Cargo.toml` matches the path in the generated HTML |
| Port 3000 already in use | Another process on that port | Change `site-addr` in `Cargo.toml` or kill the conflicting process (`lsof -ti:3000 | xargs kill`) |
| `reqwest` fails to compile on WASM target | `reqwest` included in `hydrate` feature | Ensure `reqwest` is gated behind `ssr` feature only (§1.4) |
| `.env` not loaded | `cargo leptos watch` doesn't auto-load `.env` | Use `dotenv` crate, `source .env` before running, or `env $(cat .env | xargs) cargo leptos watch` |

---

## 1.14 — Design decisions & notes

- **`peer-web/` sits alongside the PHP app.** Both can run simultaneously during the migration. The PHP app on its existing Apache/Nginx port, and Leptos on port 3000. Once a route is migrated (e.g., `/register`), traffic for that route can be proxied to Leptos.
- **Axum over Actix.** Axum is the recommended backend for Leptos 0.7 and has better ecosystem alignment with `tower` middleware. The Leptos team maintains `leptos_axum` as the primary integration.
- **No Tailwind.** The existing project uses hand-written CSS. Introducing Tailwind would create an unnecessary divergence. We'll import the existing stylesheets as-is in Step 14.
- **SCSS support.** The default template includes `main.scss`. We keep it for now (it compiles to CSS automatically via `cargo-leptos`) but will replace its contents in Step 14 when we import the existing styles.
- **Stable Rust.** Leptos 0.7 works on stable Rust, eliminating the need for nightly. This simplifies CI and reduces the chance of breakage from nightly regressions.
- **`reqwest` gated to SSR.** All HTTP calls to the GraphQL backend happen inside server functions, which run on the Axum server (native Rust). The WASM client never makes direct GraphQL calls — it calls server functions instead. This avoids CORS issues and keeps the API endpoint private.
- **No database.** The Leptos app is a frontend-only layer. All data persistence is handled by the Peer backend (via GraphQL). The Axum server's only state is the GraphQL endpoint URL and JWT tokens.
- **`wasm-bindgen` pinned.** Pinning `wasm-bindgen` to an exact version (e.g., `=0.2.100`) avoids subtle build failures when `cargo-leptos` and `wasm-bindgen-cli` disagree on versions. Update this pin deliberately.

---

## 1.15 — What's next

With the scaffolded project compiling and serving, **Step 2** will create the module skeleton (`api/`, `models/`, `components/`, `pages/`, `state/`) and define the Rust types that mirror the GraphQL registration schema.

→ [Back to master plan](registration-leptos-migration.md)
