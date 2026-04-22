# Cargo Conventions

Run cargo commands across all of these dimensions (separate invocations as needed):

- `--workspace --all-targets --all-features` — full feature set
- `--workspace --all-targets --no-default-features` — minimal feature set

Do NOT combine `--all-features` with `--no-default-features` for this repo:
the `peer-web` crate has mutually-exclusive `ssr` and `hydrate` features.
Enabling both pulls js-sys/wasm-bindgen shims that panic in native tests
(e.g. `components::success_step`, `utils::token::test_token_expiry_extraction`).

Applies to `cargo clippy`, `cargo test`, `cargo build`, etc.

# Terminal

- Never pipe to `tail`, `grep`, `head` — the harness paginates output automatically.
- `tee` to a temp file is allowed for later analysis.
