# Legacy

This directory is **frozen reference material** from the pre-Rust PHP era of the
Peer web frontend. It is **not deployed**, **not maintained**, and pull requests
that touch it will generally be rejected.

See the architecture decision record that archived this code:
[../docs/adr-repo-layout-promote-peer-web.md](../docs/adr-repo-layout-promote-peer-web.md).

## Contents

- `php/` — the original PHP entry points (`index.php`, `auth.php`, `chat.php`,
  …), the `admin/` console, the `template-parts/` includes, and the `.htaccess`
  rewrite rules. None of this is loaded by the live Rust/Leptos app.
- `assets/` — the original `css/`, `js/`, `img/`, `svg/`, `svgnew/`, `fonts/`,
  `json/`, and `favicon.ico` shipped with the PHP app. Some files (notably
  `assets/json/version_releases.json` and a handful of SVG icons) are still
  consulted by the Rust rewrite for parity; those references are explicit and
  point into this tree on purpose.
- `README.md` (this file — replaces the old root README, which was a one-line
  pointer) and `gitignore` (the old, misnamed root ignore file) are kept here
  as historical artifacts.

## Why is it still in the repository?

Until the Rust rewrite reaches feature parity, the PHP code remains the
canonical reference for UI flows, copy, response codes, admin screens, and
asset shapes. Once parity is reached, a follow-up ADR will propose deleting
this directory in a single atomic commit.

## Original README

The pre-rewrite README was:

> # peer_web_frontend
> This is the repository for the Web frontend code.
> [Contributing Guidelines](https://github.com/peer-network/.github/blob/main/CONTRIBUTING.md)

