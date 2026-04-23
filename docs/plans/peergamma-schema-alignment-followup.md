# Peergamma Schema Alignment — Mock Backend & Docs Follow-Up

> **Status:** ✅ Implemented. All six items mirrored in the mock backend, the
> reference docs, and the historical implementation plans (`docs/plans/`).
> `cargo test -p mock_backend --all-targets --all-features` is green.

Tracking commit `0dbdb4d` (`fix(graphql): align query contracts with peergamma backend schema`).

That MR fixed the **frontend** GraphQL query constants and serde wrappers so
they match production peergamma. It intentionally left the **mock backend**
(`packages/mock_backend/`) and the **backend API documentation**
(`docs/backend_api/`) unchanged. This plan closes that gap so all three
sources tell the same story.

## Why this matters

Right now the contract has three sources of truth:

1. **Production peergamma SDL** — the ground truth (out of repo).
2. **Frontend constants** (`src/api/graphql.rs`) — *aligned* by `0dbdb4d`.
3. **Mock backend SDL** (`packages/mock_backend/src/`) — *drifted*. The
   frontend talks to it for tests, fixtures, e2e, and dev offline mode.
4. **Reference docs** (`docs/backend_api/`) — *drifted*. Anyone onboarding
   reads these.

If we don't sync (3) and (4), every contract test that passes against the
mock will still silently disagree with production, which defeats the
purpose of `0dbdb4d`.

## Six fixes to mirror, plus housekeeping

The frontend MR fixed six things. We mirror each one in the mock and
the docs.

| # | Area | Mock backend file(s) | Docs file(s) | Status |
|---|---|---|---|---|
| 1 | `PostSortType` → `PostSortBy` | `src/types/post.rs`, `src/schema/query/posts.rs` | `03-posts-and-content.md`, `api.md` | ✅ |
| 2a | `performShopOrder.tokenAmount: String! → Decimal!` | already `Decimal` in mock — confirm scalar registration | `08-shop.md` (already correct), `api.md` (verified) | ✅ |
| 2b | `performShopOrder.country: Country! → ShopSupportedDeliveryCountry!` | already `ShopSupportedDeliveryCountry` in mock ✓ | already correct ✓ | ✅ |
| 3 | `logout` — drop `refreshToken` arg, read from JWT | `src/schema/mutation/auth.rs`, `tests/auth_session.rs` | `01-authentication-and-account.md`, `api.md` (added `logout: LogoutPayload!` to aggregate SDL) | ✅ |
| 4 | Replace `getUser(id:)` resolver with `getProfile(userid:)` | `src/schema/query/users.rs`, `tests/profiles.rs` | `02-users-and-profiles.md` (already documents `getProfile`); `api.md` clean | ✅ |
| 5 | `unlikeComment` → `likeComment` toggle | `src/schema/mutation/comment.rs`, `tests/comments.rs` | `04-comments.md` (toggle behaviour noted) | ✅ |
| 6 | `LikeCommentResponse.status` serde rename collision | (frontend-only — already done in `0dbdb4d`) | n/a | ✅ |

## Implementation order

Work bottom-up so each step is independently green:

1. **Plan doc** (this file). ✅
2. **Mock backend code changes** — types and resolvers. ✅
3. **Mock backend tests** — update queries/mutations the tests issue. ✅
4. **Frontend `LogoutUser` server fn** — stop sending the no-longer-declared
   `refreshToken` variable (cosmetic, but eliminates a dangling input). ✅
5. **Backend API docs** — line them up with the new SDL shape. ✅
6. **Historical implementation plans** (`docs/plans/dashboard/`,
   `docs/plans/login/`, `docs/plans/view-post/`,
   `docs/plans/mock-backend/phase-4-…`) — sweep stale GraphQL examples so
   future readers see the current schema. ✅
7. **Verify**: `cargo check --workspace --all-targets --all-features`,
   then `cargo test -p mock_backend`, then frontend `cargo check --features ssr`. ✅

## Detailed change list

### 1. `PostSortType` → `PostSortBy`

- `packages/mock_backend/src/types/post.rs`: rename the Rust enum `PostSortType`
  to `PostSortBy`, and **also** annotate `#[graphql(name = "PostSortBy")]`
  on the derive so the GraphQL type name is unambiguous (defensive — async-graphql
  derives the GraphQL name from the Rust ident, but the explicit attribute
  documents intent and protects against future renames).
- `packages/mock_backend/src/schema/query/posts.rs`: update the `use`
  and pattern matches.
- Search for any other references and update.
- Docs: `docs/backend_api/03-posts-and-content.md` and `docs/backend_api/api.md`
  — replace `PostSortType` with `PostSortBy` in both the prose and the SDL
  block (`enum PostSortType { … }` → `enum PostSortBy { … }`).

### 2. `performShopOrder` scalar/enum

Mock already has it right:

- `packages/mock_backend/src/schema/mutation/shop.rs:23` —
  `#[graphql(name = "tokenAmount")] token_amount: Decimal`.
- `packages/mock_backend/src/types/shop.rs:34` — `country: ShopSupportedDeliveryCountry`.

Action: just **verify** by inspecting the generated SDL in a smoke test
(or by reading `cargo run --bin mock_backend -- --print-schema` if such a
flag exists; otherwise rely on the type signatures). No code change.

Docs already correct in `08-shop.md`. Cross-check `api.md` to make sure
no stale `Country` enum or `tokenAmount: String!` is left.

### 3. `logout` — drop `refreshToken`

- `packages/mock_backend/src/schema/mutation/auth.rs`: change `logout`
  signature from `(refresh_token: String) -> LogoutPayload` to
  `(ctx: &Context<'_>) -> LogoutPayload`. Resolve the user via the
  existing `CurrentUser`/JWT context (same pattern as `delete_account`).
  If unauthenticated, return a non-fatal `LogoutPayload` (the real
  backend treats logout as idempotent — no error if no session).
- `packages/mock_backend/tests/auth_session.rs`: rewrite the two
  `logout(refreshToken: "…")` test queries to call `logout { … }`
  with a Bearer token in the Authorization header (the test harness
  already supports this pattern — see other auth tests).
- `src/api/auth.rs` (frontend): drop the `Vars { refresh_token }` struct
  and pass `()` (or empty `serde_json::json!({})`) as variables, since
  `LOGOUT_MUTATION` no longer declares `$refreshToken`.
- Docs: add a `### \`logout\`` section to
  `docs/backend_api/01-authentication-and-account.md` between `refreshToken`
  and `requestPasswordReset`. Document zero arguments, JWT-based identity,
  idempotent semantics, response codes.

### 4. `getUser` → `getProfile`

The mock already exposes `getProfile(userid:)` correctly. The drift is
that it **also** exposes `getUser(id:)` which production does not. We
remove the frontend-specific shim.

- `packages/mock_backend/src/schema/query/users.rs`: delete the
  `async fn get_user(...)` resolver and the `GetUserResponseGql` /
  `GetUserResult` types it builds (or move them out if used elsewhere
  — grep shows they aren't).
- `packages/mock_backend/src/types/user.rs`: drop the GraphQL types only
  used by `get_user`.
- `packages/mock_backend/tests/profiles.rs:293,324`: rewrite the two
  `getUser(id: …) { … }` tests to use `getProfile(userid: …) { meta { … }
  affected_rows { id username slug img biography amountfollower
  amountfollowed amountfriends } }`. Update assertion field names from
  camelCase (`amountFollowers`) to lowercase backend names
  (`amountfollower`, etc.) — those are what the SDL exposes; the
  frontend's camelCase aliases are client-side.
- Docs: `docs/backend_api/api.md` — if it references `getUser` in the
  `Query` block, remove. (`02-users-and-profiles.md` already documents
  `getProfile` correctly.)

### 5. `unlikeComment` → `likeComment` toggle

- `packages/mock_backend/src/schema/mutation/comment.rs`:
  - Change `like_comment` to be a **toggle**: if the
    `(user_id, comment_uuid)` pair is in `comment_likes`, remove it
    (returning `11603` "Comment unliked"); otherwise insert it (returning
    `11603` "Comment liked"). Keep the self-like guard.
  - Delete the `unlike_comment` resolver.
- `packages/mock_backend/tests/comments.rs:418-440`: rewrite
  `test_unlike_comment` to call `likeComment` twice and assert toggle
  behaviour (first call likes, second unlikes). Update the expected
  field name from `data["unlikeComment"]` to `data["likeComment"]`.
- Docs: `docs/backend_api/04-comments.md` — append a paragraph to the
  `### \`likeComment\`` section explaining the toggle semantics, and
  delete any standalone `unlikeComment` reference (none currently exists
  in the per-section docs, but check `api.md`).

### 6. `LikeCommentResponse` PascalCase override

Already fixed in `0dbdb4d` (`src/models/comment.rs`). Nothing further.

## Verification

Per `AGENTS.md`, run all targets / all features / workspace, both with
default features and with `--no-default-features`:

```
cargo check --workspace --all-targets --all-features
cargo check --workspace --all-targets --no-default-features
cargo test  --workspace --all-targets --all-features
cargo test  --workspace --all-targets --no-default-features
```

Spot-check:

- `cargo test -p mock_backend auth_session::` — logout flow.
- `cargo test -p mock_backend profiles::` — `getProfile` parity.
- `cargo test -p mock_backend comments::` — `likeComment` toggle.
- `cargo check --features ssr` (frontend root) — confirms the frontend
  still compiles after dropping the `Vars { refresh_token }` struct.

## Out of scope (parking lot)

These belong to other tracks per the original MR description:

- **Track B** — CI cross-repo schema diff automation.
- **Track C** — Chat ops (`listChats`, `sendChatMessage`, `createChat`,
  `listChatMessages`, `markChatRead`). The mock has these implemented;
  peergamma does not yet. We leave the mock as-is and gate the frontend
  chat code behind a feature/flag in a separate PR.
- **Track D** — AGPL-3.0 licensing decision.
