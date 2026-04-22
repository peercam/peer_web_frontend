//! Force-download media proxy (replacement for legacy `download.php`).
//!
//! Exposes `GET /download?file=<url>[&name=<filename>]` which streams the given
//! remote URL back to the browser with `Content-Disposition: attachment`. The
//! legacy PHP implementation was an open URL proxy; this port hardens it with
//!
//! * an HTTPS-only, host allow-list URL validator (no SSRF, no open proxy),
//! * bounded memory via streaming,
//! * bounded time via connect + total timeouts,
//! * bounded size via a hard byte cap enforced mid-stream,
//! * filename sanitisation + RFC 5987 encoded `Content-Disposition`, and
//! * forced `application/octet-stream` response type + `nosniff`.
//!
//! See `docs/plans/download/download-implementation.md` for the full rationale.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderName, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::StreamExt;
use serde::Deserialize;

const DEFAULT_MAX_BYTES: u64 = 256 * 1024 * 1024;
const DEFAULT_TIMEOUT_SECS: u64 = 300;
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_ALLOWED_HOSTS: &str = "media.peer.network,cdn.peer.network";
const FILENAME_MAX_LEN: usize = 200;
const FALLBACK_FILENAME: &str = "download.bin";

const X_CONTENT_TYPE_OPTIONS: HeaderName = HeaderName::from_static("x-content-type-options");

/// Axum handler state: parsed env config + a pre-built `reqwest::Client`.
#[derive(Clone)]
pub struct DownloadConfig {
    pub allowed_hosts: Arc<HashSet<String>>,
    pub max_bytes: u64,
    /// Total deadline passed to `reqwest::Client::timeout()`. Stored so future
    /// code paths (e.g. per-chunk idle timeouts) can observe the budget.
    pub timeout: Duration,
    pub client: reqwest::Client,
    /// Test hook: when `true`, `validate_url` accepts `http://` as well as
    /// `https://`. Always `false` in production — the only way to flip it is
    /// via `DownloadConfig::for_test_http`. This is the minimum surface needed
    /// to drive the streaming pipeline from integration tests without a
    /// self-signed rustls harness.
    #[doc(hidden)]
    pub allow_http_for_tests: bool,
}

impl DownloadConfig {
    /// Build a config from env vars, falling back to safe defaults.
    ///
    /// Intentionally not a `OnceCell` — tests need to construct fresh configs
    /// without mutating process-wide state.
    pub fn from_env() -> Self {
        let allowed_hosts: HashSet<String> = std::env::var("DOWNLOAD_ALLOWED_HOSTS")
            .unwrap_or_else(|_| DEFAULT_ALLOWED_HOSTS.to_string())
            .split(',')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let max_bytes = std::env::var("DOWNLOAD_MAX_BYTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_BYTES);

        let timeout_secs = std::env::var("DOWNLOAD_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        let timeout = Duration::from_secs(timeout_secs);

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS))
            .timeout(timeout)
            .build()
            .expect("reqwest client build");

        Self {
            allowed_hosts: Arc::new(allowed_hosts),
            max_bytes,
            timeout,
            client,
            allow_http_for_tests: false,
        }
    }

    /// Test-only constructor (HTTPS-only, same validation as production).
    #[doc(hidden)]
    pub fn for_test(allowed_hosts: HashSet<String>, max_bytes: u64, timeout: Duration) -> Self {
        Self::build_for_test(allowed_hosts, max_bytes, timeout, false)
    }

    /// Test-only constructor that additionally accepts `http://` URLs. Used
    /// by the streaming integration tests to avoid standing up a self-signed
    /// HTTPS mock. Never call from production code.
    #[doc(hidden)]
    pub fn for_test_http(
        allowed_hosts: HashSet<String>,
        max_bytes: u64,
        timeout: Duration,
    ) -> Self {
        Self::build_for_test(allowed_hosts, max_bytes, timeout, true)
    }

    fn build_for_test(
        allowed_hosts: HashSet<String>,
        max_bytes: u64,
        timeout: Duration,
        allow_http: bool,
    ) -> Self {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(timeout)
            .build()
            .expect("reqwest client build");
        Self {
            allowed_hosts: Arc::new(allowed_hosts),
            max_bytes,
            timeout,
            client,
            allow_http_for_tests: allow_http,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub file: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug)]
pub enum DownloadError {
    BadRequest(&'static str),
    Forbidden,
    TooLarge,
    BadGateway,
    GatewayTimeout,
    Internal,
}

impl IntoResponse for DownloadError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            DownloadError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            DownloadError::Forbidden => (StatusCode::FORBIDDEN, "host not allow-listed"),
            DownloadError::TooLarge => {
                (StatusCode::PAYLOAD_TOO_LARGE, "upstream content too large")
            }
            DownloadError::BadGateway => (StatusCode::BAD_GATEWAY, "upstream error"),
            DownloadError::GatewayTimeout => (StatusCode::GATEWAY_TIMEOUT, "upstream timeout"),
            DownloadError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error"),
        };
        (status, body).into_response()
    }
}

/// Drop-guarded audit state carried inside the stream's `scan` accumulator.
///
/// Logging at stream completion (rather than at response-build time) is the
/// only way to capture the actual `bytes_streamed` total from this handler —
/// once we return a `Response`, axum owns the body and we never see it again.
/// Because `scan`'s state is dropped when the stream is dropped (either after
/// normal completion, client disconnect, or the cap-exceeded error), `Drop`
/// fires exactly once per request and gives us a reliable audit line.
struct AuditState {
    bytes: u64,
    max_bytes: u64,
    host: String,
    filename: String,
    started: Instant,
    cap_exceeded: bool,
    aborted_upstream: bool,
}

impl Drop for AuditState {
    fn drop(&mut self) {
        leptos::logging::log!(
            "download: complete host={} filename={} bytes_streamed={} max_bytes={} \
             duration_ms={} cap_exceeded={} upstream_aborted={}",
            self.host,
            self.filename,
            self.bytes,
            self.max_bytes,
            self.started.elapsed().as_millis(),
            self.cap_exceeded,
            self.aborted_upstream,
        );
    }
}

/// Main handler.
pub async fn download_handler(
    State(cfg): State<DownloadConfig>,
    Query(q): Query<DownloadQuery>,
) -> Result<Response, DownloadError> {
    let started = Instant::now();

    let url = validate_url(&q.file, &cfg.allowed_hosts, cfg.allow_http_for_tests)?;
    let host = url.host_str().unwrap_or("?").to_string();
    let filename = derive_filename(&url, q.name.as_deref());

    let upstream = match cfg.client.get(url.clone()).send().await {
        Ok(r) => r,
        Err(e) => {
            leptos::logging::log!(
                "download: upstream error host={} err={} duration_ms={}",
                host,
                e,
                started.elapsed().as_millis()
            );
            return Err(if e.is_timeout() {
                DownloadError::GatewayTimeout
            } else {
                DownloadError::BadGateway
            });
        }
    };

    if !upstream.status().is_success() {
        leptos::logging::log!(
            "download: upstream non-2xx host={} status={} duration_ms={}",
            host,
            upstream.status().as_u16(),
            started.elapsed().as_millis()
        );
        return Err(DownloadError::BadGateway);
    }

    let advertised_len = upstream.content_length();
    if let Some(len) = advertised_len
        && len > cfg.max_bytes
    {
        leptos::logging::log!(
            "download: advertised too large host={} len={} cap={}",
            host,
            len,
            cfg.max_bytes
        );
        return Err(DownloadError::TooLarge);
    }

    leptos::logging::log!(
        "download: streaming host={} filename={} advertised_len={:?} setup_ms={}",
        host,
        filename,
        advertised_len,
        started.elapsed().as_millis()
    );

    let audit = AuditState {
        bytes: 0,
        max_bytes: cfg.max_bytes,
        host,
        filename: filename.clone(),
        started,
        cap_exceeded: false,
        aborted_upstream: false,
    };

    let stream = upstream.bytes_stream().scan(audit, |state, chunk| {
        let result: Result<bytes::Bytes, std::io::Error> = match chunk {
            Ok(b) => {
                state.bytes = state.bytes.saturating_add(b.len() as u64);
                if state.bytes > state.max_bytes {
                    state.cap_exceeded = true;
                    // We use `std::io::Error::other` (which wraps
                    // `ErrorKind::Other`) rather than the more specific
                    // `ErrorKind::QuotaExceeded` because the latter only
                    // stabilised in Rust 1.85; `Other` keeps the code portable
                    // to slightly older toolchains. The `cap_exceeded` flag on
                    // `AuditState` carries the semantic distinction into the
                    // completion log.
                    Err(std::io::Error::other("size cap exceeded"))
                } else {
                    Ok(b)
                }
            }
            Err(e) => {
                state.aborted_upstream = true;
                Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e))
            }
        };
        std::future::ready(Some(result))
    });

    let body = Body::from_stream(stream);

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, content_disposition(&filename))
        .header(header::CACHE_CONTROL, "private, no-store")
        .header(X_CONTENT_TYPE_OPTIONS, "nosniff");

    if let Some(len) = advertised_len {
        builder = builder.header(header::CONTENT_LENGTH, len);
    }

    builder.body(body).map_err(|_| DownloadError::Internal)
}

// ---------------------------------------------------------------------------
// URL validation
// ---------------------------------------------------------------------------

fn validate_url(
    raw: &str,
    allowed_hosts: &HashSet<String>,
    allow_http: bool,
) -> Result<reqwest::Url, DownloadError> {
    if raw.is_empty() {
        return Err(DownloadError::BadRequest("missing file parameter"));
    }
    let url = reqwest::Url::parse(raw).map_err(|_| DownloadError::BadRequest("invalid url"))?;
    let scheme_ok = url.scheme() == "https" || (allow_http && url.scheme() == "http");
    if !scheme_ok {
        return Err(DownloadError::BadRequest("scheme must be https"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(DownloadError::BadRequest("userinfo not allowed"));
    }
    let host = url
        .host_str()
        .ok_or(DownloadError::BadRequest("missing host"))?
        .to_ascii_lowercase();
    if !allowed_hosts.contains(&host) {
        return Err(DownloadError::Forbidden);
    }
    Ok(url)
}

// ---------------------------------------------------------------------------
// Filename handling
// ---------------------------------------------------------------------------

/// Sanitisation outcome — a `Result` instead of a magic sentinel lets callers
/// distinguish "input was clean and just happens to equal `download.bin`" from
/// "input was empty / all stripped away and we had to fall back".
enum SanitisedName {
    Clean(String),
    Fallback,
}

impl SanitisedName {
    fn into_string(self) -> String {
        match self {
            SanitisedName::Clean(s) => s,
            SanitisedName::Fallback => FALLBACK_FILENAME.to_string(),
        }
    }
}

fn derive_filename(url: &reqwest::Url, name_override: Option<&str>) -> String {
    if let Some(raw) = name_override.map(str::trim).filter(|s| !s.is_empty())
        && let SanitisedName::Clean(name) = sanitise_filename_checked(raw)
    {
        return name;
    }
    let from_path = url
        .path_segments()
        .and_then(|segs| segs.rev().find(|s| !s.is_empty()))
        .unwrap_or("");
    sanitise_filename_checked(from_path).into_string()
}

#[cfg(test)]
fn sanitise_filename(raw: &str) -> String {
    sanitise_filename_checked(raw).into_string()
}

fn sanitise_filename_checked(raw: &str) -> SanitisedName {
    // Step 1: drop control chars, path separators, NUL; cap length.
    let filtered: String = raw
        .chars()
        .filter(|c| !c.is_control() && *c != '/' && *c != '\\' && *c != '\0')
        .take(FILENAME_MAX_LEN)
        .collect();

    // Step 2: collapse runs of dots so `...bin` cannot survive.
    let mut collapsed = String::with_capacity(filtered.len());
    let mut prev_dot = false;
    for c in filtered.chars() {
        let is_dot = c == '.';
        if !(is_dot && prev_dot) {
            collapsed.push(c);
        }
        prev_dot = is_dot;
    }

    // Step 3 + 4: strip leading dots, fall back on empty.
    let cleaned = collapsed.trim_start_matches('.').trim();
    if cleaned.is_empty() {
        SanitisedName::Fallback
    } else {
        SanitisedName::Clean(cleaned.to_string())
    }
}

fn content_disposition(name: &str) -> String {
    // ASCII-safe quoted form + RFC 5987 UTF-8 form for full-fidelity name.
    let ascii: String = name
        .chars()
        .map(|c| {
            if c.is_ascii() && c != '"' && c != '\\' && !c.is_control() {
                c
            } else {
                '_'
            }
        })
        .collect();
    let encoded =
        percent_encoding::utf8_percent_encode(name, percent_encoding::NON_ALPHANUMERIC).to_string();
    format!(r#"attachment; filename="{ascii}"; filename*=UTF-8''{encoded}"#)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn hosts(list: &[&str]) -> HashSet<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    // ---- validate_url ----

    #[test]
    fn validate_url_accepts_allowed_https_host() {
        let h = hosts(&["media.peer.network"]);
        assert!(validate_url("https://media.peer.network/x.jpg", &h, false).is_ok());
    }

    #[test]
    fn validate_url_rejects_http_scheme() {
        let h = hosts(&["media.peer.network"]);
        assert!(matches!(
            validate_url("http://media.peer.network/x.jpg", &h, false),
            Err(DownloadError::BadRequest(_))
        ));
    }

    #[test]
    fn validate_url_accepts_http_when_test_flag_set() {
        let h = hosts(&["127.0.0.1"]);
        assert!(validate_url("http://127.0.0.1/x.jpg", &h, true).is_ok());
    }

    #[test]
    fn validate_url_rejects_non_allow_listed_host() {
        let h = hosts(&["media.peer.network"]);
        assert!(matches!(
            validate_url("https://evil.example.com/x.jpg", &h, false),
            Err(DownloadError::Forbidden)
        ));
    }

    #[test]
    fn validate_url_rejects_userinfo() {
        let h = hosts(&["media.peer.network"]);
        assert!(matches!(
            validate_url("https://user:pass@media.peer.network/x.jpg", &h, false),
            Err(DownloadError::BadRequest(_))
        ));
    }

    #[test]
    fn validate_url_rejects_file_scheme() {
        let h = hosts(&["media.peer.network"]);
        assert!(matches!(
            validate_url("file:///etc/passwd", &h, false),
            Err(DownloadError::BadRequest(_))
        ));
    }

    #[test]
    fn validate_url_rejects_file_scheme_even_with_http_flag() {
        // `allow_http_for_tests` must only widen to `http`, not to arbitrary
        // schemes — `file://` stays blocked.
        let h = hosts(&["media.peer.network"]);
        assert!(matches!(
            validate_url("file:///etc/passwd", &h, true),
            Err(DownloadError::BadRequest(_))
        ));
    }

    #[test]
    fn validate_url_rejects_metadata_ip_when_not_allow_listed() {
        let h = hosts(&["media.peer.network"]);
        assert!(matches!(
            validate_url("https://169.254.169.254/latest/meta-data/", &h, false),
            Err(DownloadError::Forbidden)
        ));
    }

    #[test]
    fn validate_url_rejects_empty_string() {
        let h = hosts(&["media.peer.network"]);
        assert!(matches!(
            validate_url("", &h, false),
            Err(DownloadError::BadRequest(_))
        ));
    }

    #[test]
    fn validate_url_lowercases_host() {
        let h = hosts(&["media.peer.network"]);
        assert!(validate_url("https://MEDIA.PEER.NETWORK/x.jpg", &h, false).is_ok());
    }

    // ---- sanitise_filename ----

    #[test]
    fn sanitise_clean_name() {
        assert_eq!(sanitise_filename("clean.jpg"), "clean.jpg");
    }

    #[test]
    fn sanitise_strips_path_traversal() {
        // slashes removed → "....etcpasswd"; dot-run collapse → ".etcpasswd";
        // leading-dot strip → "etcpasswd".
        assert_eq!(sanitise_filename("../../etc/passwd"), "etcpasswd");
    }

    #[test]
    fn sanitise_collapses_and_strips_dots() {
        assert_eq!(sanitise_filename("....bin"), "bin");
    }

    #[test]
    fn sanitise_removes_control_chars() {
        assert_eq!(sanitise_filename("a\nb\rc.txt"), "abc.txt");
    }

    #[test]
    fn sanitise_empty_falls_back() {
        assert_eq!(sanitise_filename(""), FALLBACK_FILENAME);
    }

    #[test]
    fn sanitise_hidden_loses_leading_dot() {
        assert_eq!(sanitise_filename(".hidden"), "hidden");
    }

    #[test]
    fn sanitise_truncates_long_names() {
        let input = "a".repeat(300);
        assert_eq!(sanitise_filename(&input).len(), FILENAME_MAX_LEN);
    }

    #[test]
    fn sanitise_dot_heavy_input_falls_back() {
        // 300 dots \u2192 truncated to 200 \u2192 collapsed to 1 \u2192 leading-dot-stripped
        // to empty \u2192 fallback. Locks in the "purely-dot input cannot smuggle a
        // hidden-file name" invariant.
        let input = ".".repeat(300);
        assert_eq!(sanitise_filename(&input), FALLBACK_FILENAME);
    }

    // ---- derive_filename ----

    #[test]
    fn derive_from_url_path() {
        let u = reqwest::Url::parse("https://cdn/a/b/file.mp4").unwrap();
        assert_eq!(derive_filename(&u, None), "file.mp4");
    }

    #[test]
    fn derive_trailing_slash_uses_last_non_empty_segment() {
        let u = reqwest::Url::parse("https://cdn/a/b/").unwrap();
        assert_eq!(derive_filename(&u, None), "b");
    }

    #[test]
    fn derive_root_falls_back() {
        let u = reqwest::Url::parse("https://cdn/").unwrap();
        assert_eq!(derive_filename(&u, None), FALLBACK_FILENAME);
    }

    #[test]
    fn derive_override_wins() {
        let u = reqwest::Url::parse("https://cdn/x.bin").unwrap();
        assert_eq!(derive_filename(&u, Some("custom.zip")), "custom.zip");
    }

    #[test]
    fn derive_blank_override_ignored() {
        let u = reqwest::Url::parse("https://cdn/x.bin").unwrap();
        assert_eq!(derive_filename(&u, Some("   ")), "x.bin");
    }

    #[test]
    fn derive_ignores_query_and_fragment() {
        let u = reqwest::Url::parse("https://cdn/x.bin?q=1#frag").unwrap();
        assert_eq!(derive_filename(&u, None), "x.bin");
    }

    #[test]
    fn derive_literal_download_bin_override_is_respected() {
        // Regression: earlier implementation used the string "download.bin" as
        // a "sanitisation failed" sentinel and silently fell through to the
        // URL path when a caller explicitly asked for that name. With the
        // `Result`-style `sanitise_filename_checked`, a clean override wins
        // even if it happens to equal the fallback.
        let u = reqwest::Url::parse("https://cdn/something-else.mp4").unwrap();
        assert_eq!(derive_filename(&u, Some("download.bin")), "download.bin");
    }

    // ---- content_disposition ----

    #[test]
    fn content_disposition_ascii() {
        let cd = content_disposition("file.jpg");
        assert!(cd.contains(r#"filename="file.jpg""#));
        // `percent_encoding::NON_ALPHANUMERIC` also encodes `.` → `%2E`.
        assert!(cd.contains("filename*=UTF-8''file%2Ejpg"));
    }

    #[test]
    fn content_disposition_non_ascii_encodes() {
        let cd = content_disposition("fotó.jpg");
        // Non-ASCII replaced with underscore in the ASCII fallback.
        assert!(cd.contains(r#"filename="fot_.jpg""#));
        // RFC 5987 UTF-8 form carries the original (dot is also percent-encoded
        // by `NON_ALPHANUMERIC` — that is valid per RFC 5987).
        assert!(cd.contains("filename*=UTF-8''fot%C3%B3%2Ejpg"));
    }

    #[test]
    fn content_disposition_escapes_quotes() {
        let cd = content_disposition("a\"b.bin");
        // The quote is replaced by underscore in the ASCII fallback so the header stays syntactically valid.
        assert!(cd.contains(r#"filename="a_b.bin""#));
    }
}
