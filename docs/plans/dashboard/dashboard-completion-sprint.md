# Dashboard — Completion Sprint Plan

**Feature:** Dashboard (#3 in migration priority)
**Status:** 🟡 Mostly Implemented → target ✅ Implemented
**Created:** 2026-04-23
**Parent plan:** [dashboard-implementation.md](dashboard-implementation.md)

---

## Why this sprint

Dashboard has been the longest-standing 🟡 row in the
[feature-convergence tracker](../../feature-convergence.md). The post feed,
filters, sort controls, search bar, sidebars, infinite scroll and ad
interleaving have all landed; what blocks the ✅ promotion is a single
hot-path interaction (post-card click → View Post) plus a handful of test /
documentation gaps.

It's also the **gating dependency** for two adjacent rows:

- View Post (#5) — its [completion sprint Task 10](../view-post/view-post-completion-sprint.md)
  explicitly defers "Dashboard → View Post overlay integration" until
  Dashboard ships the click handler.
- New Post (#7) — the dashboard's `AddPostButton` is the primary entry point
  to the New Post flow; the New Post sprint assumes Dashboard is the launcher.

Closing Dashboard therefore unblocks the next two sprints in the queue with
minimal scope of its own.

---

## Code audit (vs the implementation plan's Known Issues list)

Cross-referencing the 4 issues recorded in
[dashboard-implementation.md § Known Issues](dashboard-implementation.md#known-issues)
against the current tree:

| # | Original Known Issue | Current state (verified 2026-04-23) | Action |
|---|----------------------|--------------------------------------|--------|
| 1 | Missing `search_users` / `spawn_local` imports under `hydrate` in `user_search.rs` | **Resolved.** `src/components/search/user_search.rs:5-9` gates both imports behind `#[cfg(feature = "hydrate")]`; client build passes. | Update parent plan — strike from Known Issues. |
| 2 | `ProfileWidget` returns placeholder data | **Resolved.** `src/components/widgets/profile_widget.rs:14-39` reads `auth.is_authenticated`, calls `get_profile`, and maps to `UserInfo` with placeholder fallback only when unauthenticated. | Update parent plan — strike from Known Issues. |
| 3 | Post click handler is a stub (logs only) | **Open.** `src/components/posts/post_card.rs:39-42` still contains the `// TODO: Open view post modal/overlay` stub and `leptos::logging::log!`. | Sprint Task 1 below. |
| 4 | `LIST_POSTS_QUERY` omits `isreported`, `amounttrending`, `hasActiveReports`, `visibilityStatus`, `isHiddenForUsers` | **Resolved.** `src/api/graphql.rs:381,657-666` shows all five fields are now requested; matching `#[serde(default)]` defenses remain on the model. | Update parent plan — strike from Known Issues. |

Net: 3 of 4 known issues are already fixed silently — they just need to be
reflected in the docs. Only Issue #3 is real engineering work.

### Other gaps surfaced during audit

| # | Gap | Evidence | Action |
|---|-----|----------|--------|
| A | No Playwright spec for the dashboard feed itself. `dashboard.spec.ts` is absent; layout/home/wallet/forgot-password specs only navigate **through** `/dashboard` to reach other URLs. | `end2end/tests/` has no `dashboard.spec.ts`; existing matches treat `/dashboard` as a transit URL. | Sprint Task 4 below. |
| B | The first `<PostCard>` in the feed eagerly fires `post_action(View)` from a top-level `Effect`, regardless of whether the card is on screen. Legacy used IntersectionObserver to mark only visible cards. | `src/components/posts/post_card.rs:29-37` runs view-tracking unconditionally on mount. | Sprint Task 2 below. |
| C | Empty / error states for the feed have no E2E coverage and the empty-state copy hasn't been visually verified against legacy. | Inferred from absence of dashboard spec; visual parity not required for ✅, but a smoke assertion is. | Folded into Task 4. |

---

## Tasks

### Task 1 — Wire post-card click to View Post navigation **(P0, blocker)**

**File:** `src/components/posts/post_card.rs`
**Effort:** S

Replace the TODO stub with a real navigation. Two valid approaches:

1. **Navigation (recommended for v1):** `use_navigate()(format!("/post/{post_id}"), Default::default())`.
   `ViewPostPage` already supports this URL and its close button calls
   `navigate_back()`, so the round-trip behaves like an overlay from the
   user's perspective.
2. **Modal overlay (legacy parity, deferred):** A future iteration may add a
   `show_post_overlay: RwSignal<Option<String>>` in dashboard state and render
   `<ViewPostContent>` conditionally over the feed. This is what
   [view-post sprint Task 10](../view-post/view-post-completion-sprint.md)
   anticipates and is out of scope for this sprint.

Implementation notes:

- `post_id` is already cloned out of `post` at the top of `PostCard`; reuse it.
- Anchor links inside the card (username, tags, sidebar avatar) must keep
  `ev.stop_propagation()` so the wrapping click handler doesn't double-fire —
  audit `post_card.rs` for any internal `<a>` that lacks it.
- Like / dislike / save buttons already call `ev.stop_propagation()`; no
  change needed there.
- Ad cards (`item.is_ad()` true) currently share the same click handler.
  Decide explicitly: either also navigate to `/post/:id` (consistent with
  legacy) or short-circuit and open the advertised URL. Default to navigation
  for parity; record the choice in the task PR description.

**Acceptance:** clicking anywhere on the card body (but not on a button or
inner link) navigates to `/post/{id}`; the browser back button returns to the
feed with scroll position preserved (Leptos router default behaviour — verify,
don't assume).

### Task 2 — Defer view-tracking until card is visible **(P1)**

**File:** `src/components/posts/post_card.rs`
**Effort:** M

Wrap the existing fire-and-forget `post_action(View)` in an
`IntersectionObserver` so that off-screen cards in the initial render batch
don't pre-burn view counts. Pattern is already established in the same file
for the original infinite-scroll observer (see Phase 1 of the parent plan).

Acceptance: open dashboard, scroll past the third card, then check
network: `resolvePostAction` with `action: VIEW` is fired exactly once per
card, only after the card crosses the viewport threshold (~50% visible). No
calls fire for cards that never enter the viewport.

### Task 3 — Reflect resolved Known Issues in the parent plan **(P1, doc only)**

**File:** `docs/plans/dashboard/dashboard-implementation.md`
**Effort:** S

- Strike Known Issues #1, #2, #4 (or move them under a "Resolved" sub-heading
  with the resolving commit / file reference).
- Re-word Known Issue #3 to reference this sprint document.
- Bump status line to "🟡 Mostly Implemented → ✅ Implemented after sprint".
- Add a 2026-04-23 changelog entry summarising the audit.

### Task 4 — Playwright spec: `dashboard.spec.ts` **(P0)**

**File (new):** `end2end/tests/dashboard.spec.ts`
**Effort:** L

Mirror the structure of `end2end/tests/wallet.spec.ts`. Recommended cases:

| # | Case | Asserts |
|---|------|---------|
| T1 | Authed user lands on `/dashboard` and sees the post grid | At least one `.post-card` (or current selector) is visible within the network-idle window |
| T2 | Empty feed renders the empty state | Mock backend returns `affectedRows: []`; copy matches legacy "No posts found" |
| T3 | Content-type filter checkbox persists across reload | Toggle IMAGE off → reload → checkbox is still off (localStorage `selectedContentTypes`) |
| T4 | Feed filter radio persists across reload | Select FOLLOWED → reload → still selected (localStorage `selected-feed`) |
| T5 | Clicking a post card navigates to `/post/:id` | Validates Task 1; also asserts back-button returns to `/dashboard` |
| T6 | User search dropdown shows results | Type into the user-search input; expect a result list within the debounce window |
| T7 | Like button toggles optimistically without page navigation | Click like inside a card; `aria-pressed` (or current state class) flips; URL unchanged (regression guard against Task 1) |

Reuse the `mock-server.ts` helper; add fixtures in `end2end/fixtures/posts/`
if not already present, or piggy-back on the mock-backend Phase 3 seeded data.

### Task 5 — Mock-backend smoke for the feed query **(P2)**

**File:** `tests/server_functions_integration.rs` (or new
`tests/dashboard_integration.rs` if it grows)
**Effort:** M

Mock-backend Phase 3 already implements `listPosts`, `listAdvertisementPosts`,
`searchUser`, and `resolvePostAction` (per
[feature-convergence § Mock Backend](../../feature-convergence.md#mock-backend-rust-rewrite)).
Add a single Rust integration test that exercises the **client server function
→ mock-backend round-trip** for `list_posts` with a non-default
`PostFilterType` and `PostSortBy` (Rust ident: `PostSortType`), asserting the response deserialises into
the full `Post` model including the previously-missing fields
(`isreported`, `amounttrending`, `hasActiveReports`, `visibilityStatus`,
`isHiddenForUsers`). This guards against a future mock-backend regression
silently dropping fields.

### Task 6 — Convergence-tracker promotion **(P0, ships with Task 1+4)**

**Files:** `docs/feature-convergence.md`, `CHANGELOG.md`
**Effort:** S

- Flip Dashboard row from 🟡 to ✅; replace the Notes column gap text
  ("post click … is a TODO stub") with a one-liner summarising the wiring +
  Playwright coverage and a link to this sprint.
- Bump the **Summary** counts (Implemented 15 → 16; Near-Complete 5 → 4) and
  the **Last Updated** date.
- Bump the migration-priority entry (#3) to ✅.
- Add a dated entry under `Added` and `Documentation` in
  [CHANGELOG.md](../../../CHANGELOG.md).

---

## Out of scope (explicitly deferred)

- **Modal-overlay variant of post detail** — captured in
  [view-post-completion-sprint.md Task 10](../view-post/view-post-completion-sprint.md);
  the navigation approach in Task 1 satisfies functional parity and ✅ status
  without requiring it.
- **Real-time post updates / websocket subscriptions** — already deferred per
  Open Question #2 in the parent plan; optimistic updates remain the
  contract.
- **Visual / responsive parity polish** beyond what Task 4 transitively
  exercises — Dashboard uses the shared `SiteShell` and inherits the same
  responsive guarantees as Wallet, which has independent E2E coverage.
- **Profile-widget redesign** — current implementation is functionally
  correct; visual polish is a separate, lower-priority styling pass.

---

## Acceptance gate

Dashboard is promoted to ✅ when **all** of the following hold:

1. `cargo leptos build` and the workspace test suites
   (`cargo test --workspace --all-targets --all-features` and
   `cargo test --workspace --all-targets --no-default-features`) pass clean.
2. `pnpm --dir end2end exec playwright test dashboard.spec.ts` is green
   against the Rust mock backend.
3. Tasks 1–4 and 6 are merged. (Task 5 is recommended but not blocking.)
4. The parent [dashboard-implementation.md](dashboard-implementation.md)
   Known Issues list reflects reality.
5. [feature-convergence.md](../../feature-convergence.md) row, summary
   counts, and migration-priority entry all show ✅.

---

## Effort summary

| Task | Effort | Priority |
|------|--------|----------|
| 1 — Wire click → View Post nav | S | P0 |
| 2 — IntersectionObserver for view tracking | M | P1 |
| 3 — Doc reconciliation in parent plan | S | P1 |
| 4 — `dashboard.spec.ts` (7 cases) | L | P0 |
| 5 — Mock-backend round-trip Rust test | M | P2 |
| 6 — Convergence + CHANGELOG promotion | S | P0 |

P0 work alone is sufficient to flip the row to ✅; P1/P2 tighten the
regression net.
