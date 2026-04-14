# ADR: Rewrite Mock Backend in Rust

**Status:** Proposed  
**Date:** 2026-04-14  
**Authors:** —  
**Supersedes:** `tests/mock_backend/` (Node.js + Express + graphql-http)

---

## Context

The current mock backend (`tests/mock_backend/`) is a ~200-line Node.js service built with Express and `graphql-http`. It exists to give the Leptos frontend a local GraphQL endpoint for offline development and end-to-end tests. Today it only covers three registration-related mutations (`verifyReferralString`, `register`, `verifyAccount`) with in-memory state and a `/reset` endpoint for test isolation.

Limitations of the current implementation:

1. **Language mismatch.** The frontend rewrite targets Rust (Leptos + Axum, edition 2024, nightly toolchain). Keeping the mock in Node.js forces contributors to maintain a separate runtime, dependency tree, and mental model.
2. **Narrow coverage.** The real backend exposes five GraphQL schemas (guest, authenticated, admin, moderator, bridge) across 10+ domains (auth, profiles, posts, comments, wallet, tokenomics, ads, shop, moderation, admin). The current mock only covers pre-auth registration — any new feature work requires bolting on more hand-rolled JS resolvers.
3. **No type safety across the boundary.** The GraphQL schema in `schema.graphql` is only validated at runtime. There is no compile-time contract between the mock's response shapes and the Leptos request types.
4. **Test ergonomics.** The smoke tests are a bespoke assert harness (`test.js`) instead of a standard test framework, making CI integration and failure reporting weaker than necessary.

## Decision

Rewrite the mock backend as a Rust crate using **async-graphql** on **Axum**, adopting modern, idiomatic Rust (edition 2024). The new crate will live at `tests/mock_backend/` (replacing the Node.js code) and be runnable both as a standalone binary and as an in-process test fixture.

### Technology choices

| Concern | Choice | Rationale |
|---------|--------|-----------|
| GraphQL library | [async-graphql](https://github.com/async-graphql/async-graphql) 8.x | Code-first schema from Rust types; unions, subscriptions, field guards. Dominant Rust GQL crate. |
| HTTP framework | [Axum](https://docs.rs/axum) 0.8 | Already used by the Leptos SSR server — zero new framework to learn. |
| Async runtime | [Tokio](https://tokio.rs/) 1.x | Standard. Required by Axum and async-graphql. |
| Serialisation | serde + serde_json | Already workspace deps. |
| UUID generation | uuid 1.x (v4) | Already a dev-dependency. |
| Test framework | Built-in `#[tokio::test]` + `axum::test` / `tower::ServiceExt` | First-class in the Rust ecosystem; no external runner needed. |
| CORS | tower-http `CorsLayer` | Already a dep in the main crate's SSR feature. |

### Crate layout

```
tests/mock_backend/
├── Cargo.toml
├── src/
│   ├── main.rs            # binary entry: parse CLI flags, bind port, serve
│   ├── lib.rs             # pub fn app() → Router — importable by integration tests
│   ├── schema/
│   │   ├── mod.rs          # build_schema() → async_graphql::Schema<Query, Mutation, EmptySubscription>
│   │   ├── query.rs        # Query root (_health)
│   │   └── mutation/
│   │       ├── mod.rs
│   │       ├── auth.rs     # register, verifyAccount, login, refreshToken, logout, …
│   │       ├── referral.rs # verifyReferralString
│   │       ├── post.rs     # createPost, likePost, …
│   │       ├── wallet.rs   # transferTokens, getBalance, …
│   │       └── ...         # one file per backend domain
│   ├── state.rs            # SharedState: Arc<RwLock<MockState>>
│   ├── types/
│   │   ├── mod.rs
│   │   ├── response.rs     # DefaultResponse, ResponseCode enum
│   │   ├── user.rs         # User, ReferralUser, Profile, …
│   │   ├── post.rs         # Post, Comment, …
│   │   └── wallet.rs       # Balance, Transaction, …
│   ├── seed.rs             # deterministic seed data (referral codes, test users, …)
│   └── error.rs            # MockError → async_graphql::Error conversion
└── tests/
    └── integration.rs      # reqwest / tower::ServiceExt tests against app()
```

### Key design decisions

#### 1. Code-first schema via async-graphql derive macros

```rust
#[derive(SimpleObject)]
pub struct ReferralResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: String,
    pub affected_rows: Option<Vec<ReferralUser>>,
    pub meta: DefaultResponse,
}
```

The schema is the Rust types. No `.graphql` file to keep in sync — `async-graphql` generates the SDL at startup, and we can snapshot-test it to catch accidental breakage.

#### 2. Shared state behind `Arc<RwLock<_>>`

```rust
pub struct MockState {
    pub known_referrals: HashSet<Uuid>,
    pub registered_emails: HashSet<String>,
    pub verified_users: HashSet<Uuid>,
    pub users: HashMap<Uuid, User>,
    pub posts: Vec<Post>,
    pub wallets: HashMap<Uuid, Balance>,
    // …extend per domain
}
```

State is injected into the Axum router via `Extension(shared_state)` and into async-graphql via `Schema::data()`. A `POST /reset` endpoint reconstructs `MockState::default()` for test isolation — same pattern as today, but type-safe.

#### 3. In-process test fixture (no port binding)

Integration tests import `mock_backend::app()` and drive it with `tower::ServiceExt::oneshot`, avoiding port conflicts and startup races:

```rust
#[tokio::test]
async fn register_success() {
    let app = mock_backend::app();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/graphql")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"query":"mutation { register(input: { ... }) { status } }"}"#))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // …assert on JSON body
}
```

#### 4. Domain-per-module resolver organisation

Each backend domain (auth, posts, wallet, etc.) gets its own module under `schema/mutation/`. Resolver functions receive `&Context<'_>` and extract `MockState` from context data. This keeps individual files small and makes it easy for multiple contributors to extend coverage without merge conflicts.

#### 5. Deterministic seed data

`seed.rs` provides a `MockState::default()` with pre-populated referral codes, test users, sample posts, and wallet balances. All UUIDs and values are hardcoded constants (`const` or `once_cell::sync::Lazy`) so tests can assert against known values without fragile regex matching.

#### 6. Feature-flag for the E2E test harness

The Leptos E2E tests (`peer-web/end2end/`) can depend on `mock_backend` as a dev-dependency and spin it up in a `#[fixture]`, removing the need for a separate `npm start` step in CI.

```toml
# peer-web/Cargo.toml
[dev-dependencies]
mock_backend = { path = "../../tests/mock_backend" }
```

### Incremental coverage plan

| Phase | Domains | Mutations / Queries |
|-------|---------|---------------------|
| **0 — Parity** | Auth (registration) | `verifyReferralString`, `register`, `verifyAccount` |
| **1 — Login flows** | Auth (session) | `login`, `refreshToken`, `logout`, `deleteAccount`, `forgotPassword`, `resetPassword`, `changePassword` |
| **2 — User profiles** | Users & Profiles | `getProfile`, `editProfile`, `searchUser`, `followUser`, `blockUser`, `getUserPreferences`, … |
| **3 — Posts** | Posts & Content | `createPost`, `listPosts`, `getPost`, `likePost`, `savePost`, `reportPost`, upload stub, … |
| **4 — Social** | Comments, Chat | `addComment`, `replyComment`, `likeComment`, chat stubs |
| **5 — Economy** | Wallet, Tokenomics, Shop, Ads | `transferTokens`, `getBalance`, `mintTokens`, shop/ad mutations |
| **6 — Admin** | Moderation, Admin | Moderation tickets, admin queries |

Phase 0 is the MVP and directly replaces the current Node.js mock. Subsequent phases are driven by whichever Leptos page is being built next.

## Consequences

### Positive

- **Single toolchain.** `cargo build` / `cargo test` for everything — no `npm install`, no Node.js runtime in CI.
- **Compile-time guarantees.** Response types shared (or at least structurally mirrored) between mock and frontend; the compiler catches shape mismatches before tests even run.
- **Faster feedback loop.** In-process tower tests execute in microseconds vs. the current approach of spawning a server and hitting it over TCP.
- **Extensibility.** Adding a new domain is: create a module, define the types, register the resolver — the async-graphql `MergedObject` macro wires it in.
- **Idiomatic Rust.** `Result`-based error handling, `#[derive]`-driven boilerplate elimination, `clippy` and `rustfmt` enforced — consistent with the Leptos codebase.

### Negative / Risks

- **Up-front effort.** Porting the existing resolvers is straightforward, but setting up the project skeleton and CI integration is a one-time cost.
- **async-graphql learning curve.** Contributors unfamiliar with the crate need to learn its derive-macro conventions — mitigated by good module structure and existing documentation on the crate's site.
- **Schema drift risk.** The mock's schema may diverge from the real backend. Mitigate by periodically diffing the mock's exported SDL against the backend schema files listed in [docs/backend_api/README.md](backend_api/README.md).

### Neutral

- The Node.js mock can remain in-tree on a legacy branch until Phase 0 reaches full parity and CI is updated.

## Alternatives Considered

| Alternative | Why rejected |
|-------------|-------------|
| **Keep Node.js, just extend it** | Perpetuates the dual-runtime problem; no compile-time type safety. |
| **Use a schema-first Rust crate (juniper)** | juniper still requires a separate `.graphql` file and lacks async-graphql's ergonomics (merged objects, field guards, built-in SDL export). |
| **Generate mocks from the real backend schema** | The real backend is not open-source to this repo; we would need a schema introspection step that adds CI complexity and a network dependency. |
| **WireMock-style HTTP-level stubs** | Loses GraphQL type-level validation; fragile string matching instead of structured resolvers. |

## References

- Current mock: [`tests/mock_backend/`](../tests/mock_backend/)
- Backend API docs: [`docs/backend_api/`](backend_api/)
- Leptos rewrite study: [docs/leptos-rewrite-study.md](leptos-rewrite-study.md)
- async-graphql book: <https://async-graphql.github.io/async-graphql/en/>
- Axum docs: <https://docs.rs/axum/0.8>
