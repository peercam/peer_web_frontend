# Terminal command rules (peer_web_frontend)

- NEVER append `tail`, `head`, `grep`, etc. to commands. The harness paginates output automatically.
- `tee` to a temp file is allowed for later analysis.
- Canonical Leptos build: `cd peer-web && cargo leptos build` (see AGENTS.md).
- SSR sanity check: `cd peer-web && cargo build --features ssr`.
