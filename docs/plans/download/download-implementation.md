# Download Endpoint Implementation Plan

**Feature:** Download (Force-Download Media Proxy)
**Priority:** #18 — last remaining ❌ Not Started entry on the convergence tracker
**Status:** 📋 Planned (blocked on Open Question #0 — tracker scope reconciliation)
**Created:** 2026-04-21
**Revised:** 2026-04-21 (self-review pass)
**Estimated Quality:** ⭐⭐⭐⭐ (4/5)

---

## ⚠️ Scope Reconciliation (read first)

[`docs/feature-convergence.md`](../../feature-convergence.md) describes the Download row as *"App download page"* (i.e. a marketing landing page for the mobile app). That is **not** what [`download.php`](../../../download.php) actually does — the legacy file is an 18-line remote-URL force-download proxy invoked by `forceDownload()` in [`js/global.js`](../../../js/global.js). This plan models the PHP file's real behaviour (media proxy) because the PHP file is what exists in the repo and what the convergence tracker links to.

Before Phase 1 starts, **Open Question #0 must be resolved** (see Open Questions section): either

- (a) the tracker description is wrong → correct it when promoting ❌ → ✅, *or*
- (b) the stakeholder actually wants a marketing landing page → `download.php` is orphaned legacy, this plan builds the wrong thing, and a separate `plans/download-landing/` plan is needed.

Until that is answered, treat this document as scoped to the media-proxy interpretation.

---

## Overview

The legacy `download.php` is a tiny **server-side file proxy** that streams an arbitrary remote URL back to the browser with `Content-Disposition: attachment`, forcing a download instead of inline display. It is invoked by the JS helper `forceDownload(url)` in `js/global.js`, which sends the user to:

```
/download.php?file=<encoded-url>
```

This plan ports that capability to the Leptos / Axum SSR server as a **secure, hardened proxy route** at `/download`, plus a small WASM helper so callers (post media menus, audio/video players, etc.) can invoke it the same way `forceDownload()` was used in the legacy frontend.

The legacy implementation is **dangerously naïve** (open URL proxy → SSRF, open redirect, traffic amplification, unbounded memory if `readfile` buffers). The Leptos rewrite **MUST NOT replicate that behaviour verbatim**; this plan tightens it with an allow-list, streaming, size cap, and timeout.

### Goals

1. Functional parity with `download.php` for legitimate use cases (force-download a Peer-hosted media URL).
2. Security hardening — close the SSRF / open-proxy holes that exist in the PHP version.
3. Streaming, bounded-memory, bounded-time behaviour.
4. Reusable WASM client helper (`force_download(url)`) for posts/chat/wallet UIs that need to offer a "Save to device" action.
5. SSR-only route — no Leptos page, no UI chrome, no auth requirement (matches legacy: anonymous downloads work).

---

## Scope

### In Scope

- [ ] Axum route `GET /download?file=<url>[&name=<filename>]` mounted in `src/main.rs`
- [ ] Server-side `download_handler` module under `src/server/download.rs` (new `server` module if needed)
- [ ] **Allow-list** of acceptable upstream hosts (configurable via env var `DOWNLOAD_ALLOWED_HOSTS`, comma-separated; defaults defined in code)
- [ ] **URL validation:** scheme must be `https`, host must be allow-listed, no userinfo, no fragments
- [ ] **Streaming response** via `reqwest::Response::bytes_stream()` → `axum::body::Body::from_stream` (no buffering)
- [ ] **Size cap** (default 256 MiB, env `DOWNLOAD_MAX_BYTES`) — abort the stream with HTTP 413 if exceeded
- [ ] **Timeout** on the upstream request (default 30s connect / 5min total, env `DOWNLOAD_TIMEOUT_SECS`)
- [ ] **Filename sanitisation** — derive from URL path's last segment OR `?name=` override; strip path separators, control chars, leading dots; fall back to `download.bin`
- [ ] **Content-Type:** always force `application/octet-stream` on the response — the endpoint only ever returns attachments, so echoing upstream MIME adds risk (header injection, MIME confusion) with zero benefit. Upstream `Content-Type` is read only to log / inspect, never reflected.
- [ ] **Content-Disposition: attachment** with RFC 5987 encoded `filename*=UTF-8''…` for non-ASCII names
- [ ] **`X-Content-Type-Options: nosniff`** on every response
- [ ] **Cache headers:** `Cache-Control: private, no-store` (matches legacy `must-revalidate` intent + avoids polluting CDNs with proxied content). Legacy `Pragma: public` is intentionally **dropped** — it is an HTTP/1.0 artefact, contradicts `no-store`, and confuses some intermediaries.
- [ ] **Forward upstream `Content-Length`** when present and `<= max_bytes()` so the browser can render a progress bar.
- [ ] **Error responses** with plain-text bodies and stable status codes:
  - `400` — missing/invalid `file` param
  - `403` — host not allow-listed
  - `413` — upstream content too large
  - `502` — upstream unreachable / non-2xx
  - `504` — upstream timeout
- [ ] WASM helper `peer_web::utils::download::force_download(url: &str, suggested_name: Option<&str>)` that builds `/download?file=…[&name=…]` and triggers navigation (matches legacy `forceDownload`). **Gated behind a resolved consumer** — see Open Question #5; we do not ship a helper with zero callers.
- [ ] **Audit logging** at `tracing::info!` on every completed request: `host`, `status`, `bytes_streamed`, `duration_ms`, `request_id`
- [ ] Unit tests for URL validation + filename sanitisation
- [ ] Integration test (server-only) using `axum::Router::oneshot` against a local mock upstream (`mockito` or hand-rolled `tokio::net::TcpListener`)
- [ ] Documentation entry in `docs/feature-convergence.md` (promote ❌ → ✅)

### Out of Scope (Future Work)

- Authentication / per-user download tokens (current product behaviour is anonymous)
- Range-request / resumable downloads (`Accept-Ranges: bytes`)
- Signed download URLs with expiry (would require backend coordination)
- On-the-fly transcoding / format conversion
- Rate limiting (handled at reverse-proxy / WAF layer)
- Replacing direct CDN links elsewhere in the app — only callers that explicitly want **force-attachment** behaviour should route through `/download`

---

## Legacy Implementation Analysis

### Files

| File | Purpose | Lines |
|------|---------|-------|
| [`download.php`](../../../download.php) | Reads `$_GET['file']`, sets attachment headers, calls `readfile($url)` | 18 |
| [`js/global.js`](../../../js/global.js#L1490) | `forceDownload(url)` helper that navigates to `/download.php?file=…` | 5 |

### Legacy behaviour

```php
header("Content-Type: application/octet-stream");
header("Content-Disposition: attachment; filename=\"$filename\"");
header("Content-Transfer-Encoding: binary");
header("Cache-Control: must-revalidate");
header("Pragma: public");
readfile($fileUrl);
```

`$filename = basename($fileUrl)` — extracted from the URL path with no sanitisation.
`readfile()` — buffers the entire upstream response in PHP memory (the `php.ini` `memory_limit` is the only ceiling).

### Known security issues in the legacy version (must NOT be carried over)

| Issue | Impact | Mitigation in new design |
|-------|--------|-------------------------|
| **No host allow-list** | SSRF: `?file=http://169.254.169.254/…` exfiltrates cloud metadata; `?file=http://internal-svc/…` reaches private network | Allow-list of public CDN hosts; reject anything else with `403` |
| **No scheme check** | `?file=file:///etc/passwd` reads local files; `?file=gopher://…` SSRF amplification | Force `https://` only |
| **`basename()` on user input** | Filename injection via `;`, newlines, or non-ASCII can break headers / smuggle responses | Aggressive sanitisation + RFC 5987 encoding |
| **Unbounded memory** (`readfile`) | Single request can OOM the server | Streamed body + hard byte cap |
| **No timeout** | Slowloris upstream hangs PHP-FPM workers | Per-request connect + read timeouts |
| **Open redirect via headers** | Upstream `Location:` would be followed silently | Disable redirect following on the `reqwest::Client`, OR cap at 1 same-host redirect |

### Caller in legacy frontend

[`js/global.js:1490`](../../../js/global.js#L1490):

```js
function forceDownload(url) {
  const baseUrl = `${location.protocol}//${location.host}/`;
  window.location.href =
    baseUrl + "download.php?file=" + encodeURIComponent(url);
}
```

The only call site ([`js/global.js:620`](../../../js/global.js#L620)) is **commented out**, so the helper is currently dormant in the legacy app. The endpoint nonetheless exists and is reachable.

**ROI framing.** ~430 LOC for an endpoint with zero live callers is defensible *only* if at least one consumer is identified up front. Phase 6 is therefore gated on Open Question #5 being answered with a concrete caller (post media menu is the most likely candidate). If no consumer is identified, ship Phases 1–5 (endpoint + tests + docs) and defer the WASM helper until the first consuming feature plan needs it.

---

## Architecture in peer-web

### Module layout

```
peer-web/src/
├── main.rs                 ← register .route("/download", get(download_handler))
├── server/
│   ├── mod.rs              ← #[cfg(feature = "ssr")] pub mod download;
│   └── download.rs         ← handler, validation, streaming
└── utils/
    ├── mod.rs              ← pub mod download;
    └── download.rs         ← #[cfg(feature = "hydrate")] force_download() helper
```

A new top-level `server/` module is introduced (currently no peer-web routes live outside `leptos_axum::LeptosRoutes`). Keeping it as a sibling to `api/` makes intent clear: `api/` is GraphQL-over-HTTP from the WASM client; `server/` is purely server-resident HTTP routes.

### Configuration injection (not `OnceCell`)

Config is read from env **once** in `main` and passed to the handler as typed axum state. This is deliberately **not** a lazy `OnceCell`: process-wide statics that read env at first access are test-hostile (the first test to touch them wins, every subsequent test sees stale values). State injection keeps production behaviour identical while letting tests construct a fresh `DownloadConfig` per case.

```rust
#[derive(Clone)]
pub struct DownloadConfig {
    pub allowed_hosts: std::sync::Arc<std::collections::HashSet<String>>,
    pub max_bytes: u64,
    pub timeout: std::time::Duration,
    pub client: reqwest::Client,
}

impl DownloadConfig {
    pub fn from_env() -> Self {
        let allowed_hosts = std::env::var("DOWNLOAD_ALLOWED_HOSTS")
            .unwrap_or_else(|_| "media.peer.network,cdn.peer.network".to_string())
            .split(',')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        let max_bytes = std::env::var("DOWNLOAD_MAX_BYTES")
            .ok().and_then(|s| s.parse().ok()).unwrap_or(256 * 1024 * 1024);
        let timeout_secs = std::env::var("DOWNLOAD_TIMEOUT_SECS")
            .ok().and_then(|s| s.parse().ok()).unwrap_or(300);
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(std::time::Duration::from_secs(30))
            // NOTE: total deadline, not idle timeout — see "Timeout trade-off" below.
            .timeout(std::time::Duration::from_secs(timeout_secs))
            // TODO(dns-rebind): install a custom resolver rejecting RFC1918 / loopback /
            // link-local / CGNAT addresses when allowed_hosts ever includes third-party hosts.
            .build()
            .expect("reqwest client build");
        Self {
            allowed_hosts: std::sync::Arc::new(allowed_hosts),
            max_bytes,
            timeout: std::time::Duration::from_secs(timeout_secs),
            client,
        }
    }
}
```

Both the default host list and the env var should be revisited once the actual production CDN hostname(s) are confirmed (Open Question #1).

### Route registration

In `peer-web/src/main.rs`:

```rust
use peer_web::server::download::{download_handler, DownloadConfig};

let download_config = DownloadConfig::from_env();

let app = Router::new()
    .route(
        "/download",
        axum::routing::get(download_handler).with_state(download_config),
    )
    .leptos_routes(&leptos_options, routes, /* ... */)
    .fallback(leptos_axum::file_and_error_handler(shell))
    .with_state(leptos_options);
```

The `/download` route is mounted **before** `leptos_routes` so it cannot be shadowed by a future Leptos page at the same path. Per-route `.with_state(download_config)` keeps `DownloadConfig` isolated from `LeptosOptions` (the two states have different types; axum's `FromRef` is not required here).

### Handler signature

```rust
#[derive(serde::Deserialize)]
pub struct DownloadQuery {
    pub file: String,
    #[serde(default)]
    pub name: Option<String>,
}

pub async fn download_handler(
    axum::extract::State(cfg): axum::extract::State<DownloadConfig>,
    axum::extract::Query(q): axum::extract::Query<DownloadQuery>,
) -> Result<axum::response::Response, DownloadError> { /* ... */ }
```

`DownloadError` implements `IntoResponse` → maps to the status codes listed above; logs at `warn` for client errors and `error` for upstream / timeout.

### Filename derivation

Precedence:

1. Sanitised `?name=` query override, if non-empty after sanitisation.
2. Last non-empty path segment of the (already-validated) URL, sanitised.
3. Fallback to `"download.bin"`.

```rust
fn derive_filename(url: &reqwest::Url, name_override: Option<&str>) -> String {
    if let Some(raw) = name_override.map(str::trim).filter(|s| !s.is_empty()) {
        let cleaned = sanitise_filename(raw);
        if cleaned != "download.bin" { return cleaned; }
    }
    let from_path = url
        .path_segments()
        .and_then(|segs| segs.rev().find(|s| !s.is_empty()))
        .unwrap_or("");
    sanitise_filename(from_path)
}
```

Query string and fragment are ignored because `reqwest::Url::path_segments()` returns path-only segments.

### Streaming pipeline

```rust
let upstream = cfg.client.get(validated_url).send().await
    .map_err(|e| if e.is_timeout() { DownloadError::GatewayTimeout } else { DownloadError::BadGateway })?;

if !upstream.status().is_success() {
    return Err(DownloadError::BadGateway);
}

// Sanity-check advertised Content-Length up front.
let advertised_len = upstream.content_length();
if let Some(len) = advertised_len {
    if len > cfg.max_bytes { return Err(DownloadError::TooLarge); }
}

let max = cfg.max_bytes;
let counted = upstream
    .bytes_stream()
    .scan(0u64, move |acc, chunk| {
        let result = match chunk {
            Ok(bytes) => {
                *acc = acc.saturating_add(bytes.len() as u64);
                if *acc > max {
                    // Distinct ErrorKind so logs can tell "we aborted" from "upstream died".
                    // MSRV: ErrorKind::QuotaExceeded stabilised in Rust 1.85 (Feb 2025).
                    // Workspace MSRV is assumed ≥ 1.85; if a toolchain downgrade is ever
                    // required, substitute ErrorKind::Other with a "size cap exceeded" message
                    // and match on the message in log formatting.
                    Err(std::io::Error::new(std::io::ErrorKind::QuotaExceeded, "size cap"))
                } else {
                    Ok(bytes)
                }
            }
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)),
        };
        std::future::ready(Some(result))
    });

let body = axum::body::Body::from_stream(counted);

// `X-Content-Type-Options` is not exported as a constant by `http::header`; use HeaderName.
const X_CONTENT_TYPE_OPTIONS: axum::http::HeaderName =
    axum::http::HeaderName::from_static("x-content-type-options");

let mut builder = axum::response::Response::builder()
    .status(200)
    // Always octet-stream — the response is an attachment, upstream MIME is never reflected.
    .header(axum::http::header::CONTENT_TYPE, "application/octet-stream")
    .header(axum::http::header::CONTENT_DISPOSITION, content_disposition(&filename))
    .header(axum::http::header::CACHE_CONTROL, "private, no-store")
    .header(X_CONTENT_TYPE_OPTIONS, "nosniff");

// Forward Content-Length when known-safe so the browser can show a progress bar.
if let Some(len) = advertised_len {
    builder = builder.header(axum::http::header::CONTENT_LENGTH, len);
}

Ok(builder.body(body)?)
```

The size cap mid-stream uses `scan` so the first chunk that crosses the threshold yields an `io::Error`, which `axum` will surface as a connection close mid-response (the client sees a truncated download). This matches the conservative behaviour we want when a malicious upstream advertises a small `Content-Length` then sends gigabytes — we close immediately rather than buffer. The distinct `ErrorKind::QuotaExceeded` vs `ErrorKind::BrokenPipe` lets the surrounding `tracing` instrumentation log "cap exceeded" and "upstream disconnect" differently without parsing error strings.

### Filename sanitisation

Order of operations (documented because future edits to the order silently change test outcomes):

1. Drop control chars, path separators, NUL. Truncate to 200 chars.
2. Collapse runs of dots (`"....bin"` → `".bin"`).
3. Strip leading dots (`".bin"` → `"bin"`, `".."` → `""`).
4. Fall back to `"download.bin"` when empty.

```rust
fn sanitise_filename(raw: &str) -> String {
    // Step 1: filter.
    let filtered: String = raw
        .chars()
        .filter(|c| !c.is_control() && *c != '/' && *c != '\\' && *c != '\0')
        .take(200)
        .collect();
    // Step 2: collapse dot-runs so `..................bin` cannot survive.
    let mut collapsed = String::with_capacity(filtered.len());
    let mut prev_dot = false;
    for c in filtered.chars() {
        let is_dot = c == '.';
        if !(is_dot && prev_dot) { collapsed.push(c); }
        prev_dot = is_dot;
    }
    // Step 3 + 4: strip leading dots, fall back on empty.
    let cleaned = collapsed.trim_start_matches('.').trim();
    if cleaned.is_empty() { "download.bin".into() } else { cleaned.into() }
}

fn content_disposition(name: &str) -> String {
    // ASCII-safe quoted form + RFC 5987 UTF-8 form for full-fidelity name.
    let ascii: String = name.chars()
        .map(|c| if c.is_ascii() && c != '"' && c != '\\' { c } else { '_' })
        .collect();
    let encoded = percent_encoding::utf8_percent_encode(name, percent_encoding::NON_ALPHANUMERIC);
    format!(r#"attachment; filename="{ascii}"; filename*=UTF-8''{encoded}"#)
}
```

Add `percent-encoding = "2"` as a **direct** dependency (gated under `ssr`). It is already pulled in transitively via `reqwest`, but relying on a transitive dep for our public API surface is brittle — a `reqwest` minor bump could drop it.

### URL validation

```rust
fn validate_url(raw: &str, allowed_hosts: &std::collections::HashSet<String>)
    -> Result<reqwest::Url, DownloadError>
{
    let url = reqwest::Url::parse(raw).map_err(|_| DownloadError::BadRequest)?;
    if url.scheme() != "https" { return Err(DownloadError::BadRequest); }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(DownloadError::BadRequest);
    }
    let host = url.host_str().ok_or(DownloadError::BadRequest)?.to_ascii_lowercase();
    if !allowed_hosts.contains(&host) {
        return Err(DownloadError::Forbidden);
    }
    Ok(url)
}
```

### WASM client helper

`peer-web/src/utils/download.rs`:

```rust
#[cfg(feature = "hydrate")]
pub fn force_download(url: &str, suggested_name: Option<&str>) {
    let Some(window) = web_sys::window() else { return };
    let location = window.location();
    let origin = location.origin().unwrap_or_default();
    // `encode_uri_component` always returns a JS string; `.as_string()` is infallible in practice.
    let encoded_url = js_sys::encode_uri_component(url).as_string().unwrap_or_default();
    let mut href = format!("{origin}/download?file={encoded_url}");
    if let Some(name) = suggested_name {
        let encoded_name = js_sys::encode_uri_component(name).as_string().unwrap_or_default();
        href.push_str("&name=");
        href.push_str(&encoded_name);
    }
    let _ = location.set_href(&href);
}

#[cfg(not(feature = "hydrate"))]
pub fn force_download(_url: &str, _suggested_name: Option<&str>) {
    // SSR no-op
}
```

This mirrors the legacy `forceDownload(url)` semantics while adding an optional `name` override for callers that already know the desired filename (e.g. "post-12345.mp4").

---

## Implementation Phases

### Phase 0 — Scope reconciliation (blocker)
- [ ] Resolve Open Question #0: is `download.php` really the thing being ported, or does the convergence tracker's "App download page" description describe different intended work?
- [ ] If tracker is wrong: update the Download row description in the same commit that promotes ❌ → ✅.
- [ ] If tracker is right: archive this plan, open `docs/plans/download-landing/` instead.

### Phase 1 — Module skeleton & route wiring
- [ ] Create `src/server/download.rs` (gated `#[cfg(feature = "ssr")]`)
- [ ] Add `#[cfg(feature = "ssr")] pub mod server { pub mod download; }` (or equivalent) to `src/lib.rs` — single gate, no duplicate in a `server/mod.rs`.
- [ ] Define `DownloadConfig`, `DownloadQuery`, `DownloadError`, `IntoResponse` impl
- [ ] Wire `.route("/download", get(download_handler).with_state(DownloadConfig::from_env()))` in `src/main.rs`
- [ ] Verify `cargo build --features ssr` passes with a stub handler returning `501 Not Implemented`

### Phase 2 — URL validation + allow-list
- [ ] Implement `validate_url()`
- [ ] Implement `allowed_hosts()` with env override + sane defaults
- [ ] Unit tests:
  - `https://media.peer.network/x.jpg` → ok
  - `http://media.peer.network/x.jpg` → BadRequest (scheme)
  - `https://evil.example.com/x.jpg` → Forbidden
  - `https://user:pass@media.peer.network/x.jpg` → BadRequest
  - `file:///etc/passwd` → BadRequest
  - `https://169.254.169.254/latest/meta-data/` → Forbidden
  - empty string → BadRequest

### Phase 3 — Filename derivation + sanitisation
- [ ] Implement `derive_filename()`, `sanitise_filename()`, `content_disposition()`
- [ ] Unit tests for `sanitise_filename`:
  - `"clean.jpg"` → `"clean.jpg"`
  - `"../../etc/passwd"` → `"etcpasswd"` (slashes removed → `"....etcpasswd"`; dot-run collapse → `".etcpasswd"`; leading-dot strip → `"etcpasswd"`)
  - `"....bin"` → `"bin"` (dot-run collapse → `".bin"`; leading-dot strip → `"bin"`)
  - `"a\nb\rc.txt"` → `"abc.txt"` (control chars removed)
  - `""` → `"download.bin"`
  - `".hidden"` → `"hidden"`
  - 300-char input → truncated to 200
  - Non-ASCII name (e.g. `"fotó.jpg"`) → header contains both `filename="foto_.jpg"`-style ASCII fallback and `filename*=UTF-8''fot%C3%B3.jpg`
- [ ] Unit tests for `derive_filename`:
  - URL `https://cdn/a/b/file.mp4`, no override → `"file.mp4"`
  - URL `https://cdn/a/b/`, no override → `"b"` (trailing slash ignored)
  - URL `https://cdn/`, no override → `"download.bin"`
  - URL `https://cdn/x.bin`, override `Some("custom.zip")` → `"custom.zip"`
  - URL `https://cdn/x.bin`, override `Some("  ")` → `"x.bin"` (whitespace-only override ignored)
  - URL `https://cdn/x.bin?q=1#frag`, no override → `"x.bin"` (query/fragment ignored)

### Phase 4 — Streaming proxy
- [ ] Build the `reqwest::Client` inside `DownloadConfig::from_env()` (no redirect, configured timeouts)
- [ ] Implement the `bytes_stream` + `scan`-based size cap
- [ ] Forward upstream `Content-Length` when present and within cap
- [ ] **Do not** forward upstream `Content-Type` — response is always `application/octet-stream`
- [ ] Apply cache-control, content-disposition, X-Content-Type-Options headers
- [ ] Map errors to `DownloadError` variants (distinguish `timeout` from `connection failure`)

### Phase 5 — Integration test
- [ ] Test harness: spin up a tiny `axum` upstream that serves `/ok.jpg` (200, image/jpeg, fixed bytes), `/big.bin` (200, advertised Content-Length > cap), `/lying.bin` (200, advertises small Content-Length then streams past cap), `/slow` (drips bytes past timeout), `/500` (500)
- [ ] Each test constructs a fresh `DownloadConfig` with the mock's `127.0.0.1:<port>` in `allowed_hosts` — **no env-var mutation, no `OnceCell` reset needed**
- [ ] **Note:** `allowed_hosts` compares the parsed `url::Host` string; for `127.0.0.1` this is the literal `"127.0.0.1"`. Integration tests must either add that literal to the allow-list or bind the mock to a resolvable loopback name via `/etc/hosts` (not recommended in CI).
- [ ] Cases:
  - Happy path → 200, body matches, headers correct (incl. forwarded `Content-Length`)
  - Forbidden host → 403
  - Bad scheme (`http://`, `file://`) → 400
  - Userinfo in URL → 400
  - Upstream 500 → 502
  - Oversize advertised → 413 (no body streamed)
  - Oversize streamed (lies about Content-Length) → connection closes mid-stream, partial body, log line records `cap_exceeded=true`
  - Timeout → 504

### Phase 6 — WASM client helper (gated on Open Question #5)
- [ ] **Precondition:** at least one concrete consumer identified in Open Question #5. If none, skip Phase 6 entirely and defer to the first consuming feature plan.
- [ ] Implement `utils/download.rs` with `#[cfg]` gates
- [ ] Add `pub mod download;` to `utils/mod.rs`
- [ ] Wire the identified consumer's "Save" action to `force_download()`

### Phase 7 — Documentation
- [ ] Update `docs/feature-convergence.md`:
  - Pages table: Download row ❌ → ✅, link to this doc
  - Summary: ✅ 11 → 12, ❌ 1 → 0; convergence ~82% → ~87%
  - Migration Priority: mark #18 complete
  - Changelog entry
- [ ] Update `docs/leptos-rewrite-study.md` Download row: `No` → `Yes`
- [ ] Add a brief security note to `peer-web/README.md` describing `DOWNLOAD_ALLOWED_HOSTS`

---

## File Manifest

| Path | New / Modified | Approx. Lines | Notes |
|------|----------------|---------------|-------|
| `peer-web/src/server/download.rs` | New | ~240 | handler, config, validation, sanitisation, streaming |
| `peer-web/src/utils/download.rs` | New (Phase 6 only) | ~30 | WASM `force_download()` helper |
| `peer-web/src/utils/mod.rs` | Modified (Phase 6 only) | +1 | register module |
| `peer-web/src/lib.rs` | Modified | +3 | `#[cfg(feature = "ssr")] pub mod server { pub mod download; }` — gate lives here, not in a separate `server/mod.rs` |
| `peer-web/src/main.rs` | Modified | +3 | construct `DownloadConfig`, register route with state |
| `peer-web/Cargo.toml` | Modified | +1 | `percent-encoding = "2"` (direct dep, ssr feature) |
| `peer-web/tests/download_proxy.rs` | New | ~200 | integration tests with mock upstream |
| `docs/feature-convergence.md` | Modified | +~10 | promotion + changelog + row description correction (per Open Question #0) |
| `docs/leptos-rewrite-study.md` | Modified | +1 | flip Download row |
| `peer-web/README.md` | Modified | +~10 | env var + rate-limit documentation |

**Total new code:** ~470 lines (server: 240, helper: 30, tests: 200). Helper + utils/mod.rs changes (~31 LOC) are conditional on Open Question #5.

---

## Configuration

| Env var | Default | Purpose |
|---------|---------|---------|
| `DOWNLOAD_ALLOWED_HOSTS` | `media.peer.network,cdn.peer.network` (placeholder — confirm) | Comma-separated host allow-list |
| `DOWNLOAD_MAX_BYTES` | `268435456` (256 MiB) | Hard cap on streamed bytes |
| `DOWNLOAD_TIMEOUT_SECS` | `300` | Total request deadline |

All three are read **once at server start** via `DownloadConfig::from_env()` and injected as axum state — restart required to change. Tests construct `DownloadConfig` directly without touching env vars.

---

## Security Considerations

1. **SSRF closed by allow-list.** No request can reach an arbitrary host. The allow-list is host-only (not URL-pattern) because `reqwest::Url` parses & re-serialises the URL, eliminating path/query smuggling.
2. **HTTPS enforced.** Prevents downgrade attacks and `file://` / `gopher://` abuse.
3. **No redirect following.** A compromised upstream cannot redirect us into the private network; if the upstream legitimately needs redirects, the client should fetch the final URL itself.
4. **No userinfo in URL.** Avoids leaking credentials in server logs and avoids confusing host parsing.
5. **Bounded memory.** `bytes_stream()` + `axum::body::Body::from_stream` never materialises the full file in RAM.
6. **Bounded time.** Connect + total timeouts prevent slowloris-style worker exhaustion.
7. **Bounded size.** Hard byte cap protects bandwidth budget and prevents traffic amplification.
8. **Cache headers.** `private, no-store` prevents intermediary caches from holding proxied user content.
9. **Filename headers.** RFC 5987 + ASCII-safe fallback prevents header injection via newlines / quotes / non-ASCII.
10. **Forced `application/octet-stream`.** Upstream `Content-Type` is never reflected, killing an entire class of MIME-confusion and header-injection vectors. `X-Content-Type-Options: nosniff` prevents browser sniffing from re-inferring an active type.
11. **No auth required** — matches legacy behaviour; the upstream URL is already a public CDN URL by definition (it had to be reachable from `download.php`'s server context, which had no special creds either). If we ever serve **private** content this way, this plan needs an auth layer added.
12. **Audit trail.** Every request is logged via `tracing::info!` with host, status, bytes streamed, and duration — essential for abuse forensics on a proxy endpoint.

### Known limitations (documented, not mitigated in v1)

- **DNS rebinding.** Allow-list matches the hostname string; we trust DNS. A malicious authoritative resolver for an allow-listed host could return an RFC1918 / loopback / link-local address at fetch time and reach internal services. **Acceptable for v1** because the allow-list contains only first-party CDN hosts we operate. If the allow-list ever includes third-party hosts, add a custom `reqwest` resolver that rejects private/loopback/link-local/CGNAT IPs post-resolution.
- **Rate limiting.** Not implemented in the app — **`/download` MUST be rate-limited at the reverse proxy / WAF** before production deployment. This is called out in the README.
- **Total-vs-idle timeout trade-off.** `reqwest`'s `timeout()` is a *total* deadline. A legitimate large file over a slow mobile link can hit the deadline before finishing. The default of 300s × 256 MiB implies a ≥875 KiB/s floor, which is reasonable for a CDN but not for end-user uploads over cellular. If this becomes a real complaint, switch to per-chunk idle-timeout via `tokio::time::timeout` wrapped around each `bytes_stream` poll.

OWASP Top-10 mapping: A10 (SSRF) — primary mitigation; A05 (Security Misconfiguration) — env-driven allow-list with safe defaults, fail-closed on malformed env var (logged loudly at startup); A03 (Injection) — filename header sanitisation + forced response `Content-Type`.

---

## Testing Strategy

### Unit (in `src/server/download.rs`, `#[cfg(test)]`)
- URL validation matrix (8 cases above)
- Filename sanitisation matrix (7 cases above)
- `content_disposition()` formatting

### Integration (`peer-web/tests/download_proxy.rs`)
- Spin up a mock upstream on a random `127.0.0.1:0` port
- Inject its origin into `DOWNLOAD_ALLOWED_HOSTS` for the test process
- Drive the real `download_handler` via `axum::Router::oneshot`
- Verify status, headers, and body for each scenario in Phase 5

### Manual smoke
- Build with `cargo build --features ssr`
- Run dev server, hit `/download?file=https://<allowed>/test.jpg` from a browser, confirm Save dialog and file integrity

### What we do **not** test
- Real-world CDN behaviour (out of scope for unit/integration; covered by manual smoke before promoting)
- Browser behaviour for various MIME types (browser concern, not server concern)

---

## Definition of Done

- [ ] All Phase 1–7 checkboxes ticked
- [ ] `cargo build --features ssr` clean
- [ ] `cargo build --features hydrate --target wasm32-unknown-unknown` clean
- [ ] `cargo clippy --all-targets --features ssr -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] All new unit + integration tests passing (≥ 15 cases)
- [ ] Manual smoke confirms a real download from an allow-listed host succeeds with the correct filename
- [ ] Manual smoke confirms a non-allow-listed host returns `403`
- [ ] `docs/feature-convergence.md` updated; Download row promoted to ✅
- [ ] `docs/leptos-rewrite-study.md` Download row updated
- [ ] No regressions in existing `cargo test` suite

---

## Open Questions

0. **Scope reconciliation (blocker).** Does "Download" in the convergence tracker mean the media-proxy that `download.php` actually implements, or the "App download page" its row description claims? This plan assumes the former. **Must be answered before Phase 1.** See the Scope Reconciliation section at the top.
1. **Real CDN host(s).** The legacy code does not hard-code a host — `download.php` accepts any URL the caller passes. What are the actual production media hosts (`media.peer.network`? `cdn.peer.network`? S3 bucket?) that need to be in the default allow-list? **Decision blocker for Phase 2 unit-test fixtures and the README documentation.**
2. **Auth requirement?** Legacy is anonymous. Confirm we are **not** about to start serving private/paid content via this endpoint; if we are, add a server-fn auth check + per-user rate limit before shipping.
3. **Range requests.** Current plan returns a single non-resumable response. Is that acceptable for large video downloads (where users may want to pause/resume)? If not, add `Accept-Ranges: bytes` + range proxying as a Phase 8.
4. ~~**Audit logging.**~~ **Decided:** yes. Included in Scope above — `tracing::info!` with `host`, `status`, `bytes_streamed`, `duration_ms`, `request_id` on every request.
5. **Helper rollout (blocks Phase 6).** Phase 6 ships a WASM `force_download()` helper. ~430 LOC for a dormant endpoint is defensible only if a concrete consumer is identified up front — the legacy caller is commented out, so we must not replicate its dormancy. Candidates: post media context menu (view-post), audio player, video player, wallet receipt PDF. **Name at least one consumer and wire it in Phase 6, or defer Phase 6 to the first consuming feature plan.**
6. **Rate-limit ownership.** Plan assumes the reverse proxy / WAF handles rate limiting. Confirm which layer (nginx? Cloudflare? traefik middleware?) owns this and that a rule for `/download` is added **before** the feature ships. The README note (Phase 7) will state this as a hard requirement.

---

## Convergence Tracker Impact (preview)

After completion this plan promotes the only remaining ❌ entry, taking the tracker to:

| Status | Before | After |
|--------|--------|-------|
| ✅ Implemented | 11 | 12 |
| 🟡 Near-Complete | 6 | 6 |
| 🚧 In Progress | 2 | 2 |
| ❌ Not Started | 1 | 0 |

**Caveat:** these numbers assume Open Question #0 resolves as *"tracker description is wrong, `download.php` = media proxy"*. If it resolves the other way (stakeholder wants a marketing landing page), this plan does **not** close the ❌ — it partially solves an unrelated need — and the count above is incorrect.

Headline convergence percentage is tracked in [`docs/feature-convergence.md`](../../feature-convergence.md) using its weighted calculation; this plan does not restate the figure to avoid drift. Pages-table convergence reaches **100% Started** for the first time (subject to the same caveat).
