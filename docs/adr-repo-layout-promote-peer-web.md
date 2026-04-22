# ADR: Promote `peer-web/` to Repository Root and Archive Legacy PHP and Static Assets

**Status:** Accepted
**Date:** 2026-04-22
**Accepted:** 2026-04-22
**Authors:** —
**Supersedes:** Current dual-stack layout where the legacy PHP frontend lives at the repo root and the Rust/Leptos rewrite is nested under `peer-web/`.

---

## Context

The repository was originally a PHP application (`index.php`, `chat.php`, `dashboard.php`, …) with sibling asset folders (`css/`, `js/`, `img/`, `svg/`, `template-parts/`, `fonts/`, `json/`, `admin/`). The Leptos/Axum rewrite was added later as a self-contained Cargo workspace member at `peer-web/`, with its own `Cargo.toml`, `src/`, `style/`, `public/`, `tests/`, `tests-wasm/`, `end2end/`, and `design/` directories.

That nesting reflected reality at the time the rewrite started: the PHP app was the product, and the Rust crate was a subordinate experiment. That relationship has now inverted:

1. **Active development is in Rust.** Feature work (chat realtime transport, mock backend, posts/wallet/profiles convergence — see [adr-chat-realtime-transport.md](adr-chat-realtime-transport.md), [adr-mock-backend-rust-rewrite.md](adr-mock-backend-rust-rewrite.md), [feature-convergence.md](feature-convergence.md), [leptos-rewrite-study.md](leptos-rewrite-study.md)) targets `peer-web/`. The PHP files are reference material, not a deploy target.
2. **Tooling friction.** Running `cargo`, `trunk`, `wasm-pack`, or `npx playwright` requires `cd peer-web` every time. CI configs, IDE workspaces, and `rust-analyzer` all have to be told the crate root is one level down. Shell history, `just`/`make` aliases, and contributor docs all carry the extra prefix.
3. **Discoverability.** A new contributor cloning the repo sees `index.php`, `auth.php`, `login.php` first and reasonably assumes this is a PHP project. The actual entry point — `peer-web/src/main.rs` / `peer-web/Cargo.toml` — is buried.
4. **Asset duplication risk.** As the Leptos UI grows, there is increasing pressure to copy fonts, SVGs, and JSON fixtures from the root into `peer-web/public/` or `peer-web/style/`. Keeping the PHP tree at the root makes it ambiguous which copy is canonical.
5. **Search and grep noise.** `grep_search`, `rg`, and editor "find in files" results are dominated by legacy PHP/JS that no longer informs current decisions.

Doing nothing means the friction compounds with every new module, and the legacy assets keep masquerading as live code.

## Decision

Restructure the repository so that the Rust/Leptos crate is the root project and all PHP-era code is moved verbatim into a single `legacy/` directory.

### Target layout

```
/                                  # was peer-web/
├── Cargo.toml                     # peer-web crate manifest
├── Cargo.lock
├── rust-toolchain.toml
├── README.md                      # rewritten to describe the Rust app
├── LICENSE
├── AGENTS.md                      # stays at root
├── .gitignore                     # merged
├── .env.example
├── src/                           # Leptos app
├── style/
├── public/
├── tests/                         # Rust integration tests for the peer-web crate
│                                  # (peer-web/tests/* promoted as-is)
├── tests-wasm/                    # separate wasm-bindgen-test crate
├── end2end/                       # Playwright
├── design/
├── docs/                          # ADRs, plans/, backend_api/ — stays at root
├── memories/                      # agent memory — stays at root
├── packages/
│   └── mock_backend/              # Rust mock GraphQL backend (own crate, was tests/mock_backend/)
└── legacy/
    ├── README.md                  # explains what this directory is and is not
    ├── php/                       # *.php files from the old root
    │   ├── index.php
    │   ├── auth.php
    │   ├── chat.php
    │   ├── dashboard.php
    │   ├── …
    │   ├── admin/
    │   └── template-parts/
    └── assets/                    # css/, js/, img/, svg/, svgnew/, fonts/, json/
```

### What moves

**Promoted to root** (everything currently under `peer-web/`):

- `peer-web/Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.env.example`, `LICENSE`, `README.md`
- `peer-web/src/`, `peer-web/style/`, `peer-web/public/`, `peer-web/design/`
- `peer-web/tests/`, `peer-web/tests-wasm/`, `peer-web/end2end/`
- `peer-web/.gitignore` (merged with the root `.gitignore`)
- `peer-web/target/` stays git-ignored at its new location (root `target/`)

**Demoted to `legacy/`**:

- All root `*.php` files (`404.php`, `auth.php`, `cache.php`, `chat.php`, `dashboard.php`, `download.php`, `edit_profile.php`, `forgotpassword.php`, `host.php`, `index.php`, `invite.php`, `login.php`, `meta.min.php`, `meta.php`, `myAds.php`, `newpost.php`, `phpheader.php`, `phpinfo.php`, `post.php`, `profile.php`, `profileSettings.php`, `referralBoard.php`, `register.php`, `version_history.php`, `view-profile.php`, `viewPeerShop.php`, `wallet.php`) → `legacy/php/`
- `admin/` (PHP + its own `css/`, `js/`, `template-parts/`) → `legacy/php/admin/`
- `template-parts/` → `legacy/php/template-parts/`
- `css/`, `js/`, `img/`, `svg/`, `svgnew/`, `fonts/`, `json/` → `legacy/assets/`
- Root `gitignore` (the misnamed file without a leading dot) → `legacy/` for reference, then deleted in a follow-up commit if no tooling depends on it
- Root `README.md` → `legacy/README.md` (in Commit A, before `peer-web/README.md` is promoted in Commit B)

**Stays at root**:

- `docs/` (ADRs, `plans/`, `backend_api/` specs are still authoritative)
- `memories/` (agent memory store)
- `AGENTS.md`
- `test-results/` (Playwright output; git-ignored artifact, not tracked — verify `.gitignore` still covers it after the move)

**Relocated within the new root**:

- `tests/mock_backend/` → `packages/mock_backend/`. The mock backend is its own Cargo crate (it has its own `Cargo.toml`, `src/`, `tests/`, `fixtures/`); nesting a sibling crate inside the host crate's `tests/` directory is misleading and was a holdover from when there was no obvious other home for it. Promoting `peer-web/` to root makes `packages/` the natural place for sibling crates. See the Workspace section below.

### Mechanics

The move must preserve git history. Use `git mv` exclusively, in three commits:

1. **Commit A — `legacy/` first.** `mkdir legacy legacy/php legacy/assets`, then `git mv` each PHP file, `admin/`, `template-parts/`, and each asset folder into its target. Also `git mv README.md legacy/README.md` so the name is free for Commit B. No content changes. This frees the root namespace.
2. **Commit B — relocate the mock backend.** `mkdir packages`, then `git mv tests/mock_backend packages/mock_backend`. This empties the root `tests/` directory (assuming it held nothing else) so Commit C can promote `peer-web/tests/` cleanly. Update path references inside `packages/mock_backend/` only if the crate references siblings via `../` (verify with `rg '\.\./' packages/mock_backend/Cargo.toml`).
3. **Commit C — promote `peer-web/`.** For each entry inside `peer-web/`, `git mv peer-web/<entry> ./<entry>`, then `rmdir peer-web`. One case needs special handling:
   - **`LICENSE`**: identical files are expected; declare the root `LICENSE` canonical and `git rm peer-web/LICENSE` instead of moving (or move-and-overwrite if the texts differ). Confirm with `diff LICENSE peer-web/LICENSE` first.

   `peer-web/Cargo.toml` needs no edits if it contains no `../` or `peer-web/` paths — verify with `rg '\.\./|peer-web/' peer-web/Cargo.toml` before the move. After promotion it becomes a workspace member; see Workspace below.

Doing `legacy/` first avoids the `git mv` collision between the old root `README.md` and `peer-web/README.md`. Doing the mock-backend relocation second avoids any collision on `tests/`.

### Workspace

Introducing `packages/` means the root must become a Cargo workspace (a sibling crate cannot live alongside a single-crate root project without one). The new root `Cargo.toml` gains a `[workspace]` table:

```toml
[workspace]
resolver = "2"
members = [".", "packages/*"]
```

The `peer-web` crate's existing `[package]` / `[dependencies]` tables stay in the root `Cargo.toml` unchanged — a virtual-manifest split is not required and would make `cargo build` from the root ambiguous. If a virtual manifest is preferred later, that is a separate ADR.

`Cargo.lock` is unified at the root (it already is, since the workspace today is single-member). Running `cargo build` from the root builds `peer-web`; `cargo build -p mock_backend` builds the mock backend; `cargo test --workspace` runs both test suites.

### Follow-up edits in the same PR

- **`README.md`**: replace the PHP-era README with the Leptos one (move the old text into `legacy/README.md` if any of it is still useful).
- **`.gitignore`**: union of root and `peer-web/.gitignore`; ensure `target/`, `dist/`, `node_modules/`, `test-results/`, `.env` are covered.
- **`AGENTS.md`**: update any path references that assumed `peer-web/` prefix.
- **`docs/**`**: search-and-replace `peer-web/` path references in ADRs and plans where they refer to source files (not historical context). Historical references inside dated ADRs stay as-is.
- **CI / GitHub Actions** (if present): drop the `working-directory: peer-web` directive; update Playwright and `cargo` invocations.
- **`legacy/README.md`** (new): one paragraph stating the directory is frozen reference material from the pre-Rust app, not deployed, not maintained, and PRs touching it will generally be rejected.

### Non-goals

- **No code rewrite.** This ADR is purely structural. No `.rs`, `.php`, `.css`, or `.js` file content changes beyond path-reference updates and the new `[workspace]` table.
- **No deletion of legacy code.** Archiving — not removing — is the point. A separate future ADR can propose deleting `legacy/` once the Rust app reaches feature parity.
- **No virtual-manifest split.** A `[workspace]` table is added to the root `Cargo.toml`, but the `peer-web` crate's `[package]` stays there too (so `cargo build` at the root still builds the app). Splitting into a virtual manifest with `peer-web/` as its own member directory is a separate decision.
- **No deploy/runtime changes.** Whatever serves the PHP app today (if anything) is unaffected; consumers pointing at the repo root for PHP would need to repath to `legacy/php/`, but this is assumed to be no one.

## Consequences

### Positive

- `cargo build`, `cargo test`, and `trunk serve` all run from the repo root with no `cd`. `wasm-pack test` and `npx playwright test` shorten to `cd tests-wasm` / `cd end2end` instead of the doubled `cd peer-web/tests-wasm`.
- `rust-analyzer` discovers the project automatically; no `linkedProjects` config needed.
- New contributors see Rust code first and read the right README.
- `grep`, `rg`, and editor search default to the active codebase; legacy noise is opt-in by searching inside `legacy/`.
- Path references in docs and tooling shorten (`peer-web/src/foo.rs` → `src/foo.rs`).
- Sets the stage for eventually deleting `legacy/` in one atomic commit.

### Negative / risks

- **Large diff.** The PR will touch hundreds of files (mostly renames). Code review must trust `git log --follow` and the rename detection rather than reading the diff line-by-line. Mitigation: do the move in the two commits described above, with no content changes, so reviewers can verify by running `git diff -M --stat`.
- **External links break.** Any bookmark, blog post, or external doc deep-linking into `peer-web/src/...` on GitHub will 404. Acceptable: the project is pre-1.0 and the rewrite is internal.
- **Stale local branches.** Contributors with in-flight branches will hit rename conflicts on rebase. Mitigation: announce the cut-over, land the move on a quiet day, and document the `git rebase -X find-renames=90%` workaround in the PR description. For history archaeology after the move, `git log --follow <path>` is the canonical recovery tool.
- **Tooling that hard-codes `peer-web/`.** Any unnoticed script, CI job, Dockerfile, or editor config breaks. Mitigation: the pre-merge `rg 'peer-web/'` sweep in the Rollout section is mandatory, not optional.
- **Cross-ADR path drift.** Other ADRs ([adr-mock-backend-rust-rewrite.md](adr-mock-backend-rust-rewrite.md), [adr-chat-realtime-transport.md](adr-chat-realtime-transport.md), [feature-convergence.md](feature-convergence.md), [leptos-rewrite-study.md](leptos-rewrite-study.md)) and anything under `docs/plans/` may reference `peer-web/`-prefixed paths. The same `rg` sweep must update these in the move PR; historical context inside dated ADRs stays as-is.
- **`legacy/` rots faster.** Once moved, the PHP code is even less likely to be updated. This is intentional but worth naming: the directory is a museum, not a library.

## Alternatives considered

1. **Leave the layout as-is.** Lowest churn, but the friction described in Context keeps growing. Rejected.
2. **Delete the PHP code outright.** Cleanest root, but loses the reference value the Rust rewrite still draws on (UI flows, copy, response-code tables in `json/response-codes.json`, admin screens). Premature until feature parity is reached. Rejected for now; revisit after the rewrite ships.
3. **Move PHP into a sibling repository.** Cleanest separation, but adds a second repo to clone, host, and link from docs. Rejected as overkill for archived code.
4. **Cargo workspace with `legacy/` and `web/` as members.** Only useful if `legacy/` had Rust code. It does not. The `packages/`-style workspace adopted here is scoped to actual Rust crates. Rejected as originally framed.
5. **Rename `peer-web/` to `web/` or `app/` instead of promoting it.** Marginally better than today but still requires a `cd`, still hides the active code under a subdirectory, and still leaves the PHP tree masquerading as the project. Rejected.
6. **Keep `mock_backend/` under `tests/`.** Was the status quo. A sibling crate under another crate's `tests/` confuses both `cargo` (the host crate's `tests/*.rs` are integration-test targets) and humans (it reads as a fixture, not a deployable). Rejected; `packages/mock_backend/` is the right home.

## Rollout

1. Open a tracking issue summarising the move and linking this ADR.
2. Land Commit A (`legacy/` move), Commit B (mock-backend relocation to `packages/`), and Commit C (`peer-web/` promotion) as a single PR.
3. In the same PR: add the `[workspace]` table to the new root `Cargo.toml`; update `README.md`, `.gitignore`, `AGENTS.md`, CI configs, and any `peer-web/`-prefixed or `tests/mock_backend/`-prefixed path references in `docs/` (including sibling ADRs and `docs/plans/`).
4. Add `legacy/README.md` describing the freeze.
5. Run a pre-merge sweep: `rg 'peer-web/|tests/mock_backend'` across the repo. Every remaining hit must be either (a) a deliberate historical reference inside a dated ADR, or (b) fixed.
6. Verify locally: `cargo build --workspace`, `cargo test --workspace`, `wasm-pack test --headless --chrome` (run from `tests-wasm/`), `npx playwright test` (from `end2end/`), and a `trunk` / SSR smoke run.
7. Merge. Announce in the project channel with the rebase guidance for open branches.
8. Mark this ADR **Accepted** and date the acceptance.
