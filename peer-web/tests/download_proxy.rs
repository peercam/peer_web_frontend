//! Integration tests for the `/download` force-download media proxy.
//!
//! These tests spin up the `download_handler` behind a real Axum server on a
//! random loopback port and drive it with `reqwest`. No env-var mutation —
//! each test constructs its own `DownloadConfig` via the test-only
//! constructors on `DownloadConfig`.
//!
//! Happy-path / streaming cases rely on `DownloadConfig::for_test_http` so
//! that the upstream mock can speak plain HTTP on a loopback port. Production
//! code paths are still HTTPS-only; the flag is `#[doc(hidden)]` and unreachable
//! from `from_env()`.
//!
//! Run with: `cargo test --features ssr --no-default-features --test download_proxy`

#![cfg(feature = "ssr")]

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use peer_web::server::download::{download_handler, DownloadConfig};
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// Proxy harness
// ---------------------------------------------------------------------------

async fn start_download_router(cfg: DownloadConfig) -> SocketAddr {
    let app = Router::new().route("/download", get(download_handler).with_state(cfg));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    addr
}

fn build_download_url(addr: SocketAddr, file: &str) -> reqwest::Url {
    let mut u = reqwest::Url::parse(&format!("http://{addr}/download")).unwrap();
    u.query_pairs_mut().append_pair("file", file);
    u
}

fn build_download_url_no_file(addr: SocketAddr) -> reqwest::Url {
    reqwest::Url::parse(&format!("http://{addr}/download")).unwrap()
}

fn mock_hosts() -> HashSet<String> {
    // `127.0.0.1` is the literal string `reqwest::Url::host_str()` returns for
    // loopback. No other host is added — any misconfiguration surfaces as a
    // test failure rather than a silent internet call.
    let mut h = HashSet::new();
    h.insert("127.0.0.1".to_string());
    h
}

/// Production-equivalent config (HTTPS-only). Used by the validation tests.
fn test_cfg() -> DownloadConfig {
    DownloadConfig::for_test(mock_hosts(), 256 * 1024, Duration::from_secs(2))
}

/// Streaming-pipeline config: accepts `http://` so the mock upstream can serve
/// plain HTTP on a loopback port. Production `from_env()` never sets this flag.
fn test_cfg_http(max_bytes: u64, timeout: Duration) -> DownloadConfig {
    DownloadConfig::for_test_http(mock_hosts(), max_bytes, timeout)
}

// ---------------------------------------------------------------------------
// Upstream mock
// ---------------------------------------------------------------------------

/// Spin up a tiny upstream server exposing the fixture routes used across the
/// streaming tests. Returns the `SocketAddr` so each test can embed it into
/// the `?file=` query.
async fn start_mock_upstream() -> SocketAddr {
    async fn ok_bytes(Path(n): Path<u64>) -> Response {
        let body = vec![0x41u8; n as usize];
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::from(body))
            .unwrap()
    }

    /// Advertises a small `Content-Length` then streams many more bytes. Used
    /// to exercise the mid-stream size cap.
    async fn lying() -> Response {
        use futures_util::stream;
        // Advertise 1 KiB, actually emit 512 KiB in 1 KiB chunks.
        let chunks = (0..512).map(|_| {
            Ok::<_, std::io::Error>(bytes::Bytes::from(vec![0x42u8; 1024]))
        });
        let s = stream::iter(chunks);
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_LENGTH, 1024)
            .body(Body::from_stream(s))
            .unwrap()
    }

    /// Advertises a `Content-Length` larger than any test cap — should be
    /// rejected up-front with `413`, never streamed beyond the first chunk.
    /// We actually emit the bytes because hyper overrides the `Content-Length`
    /// header for empty bodies; sending real bytes keeps the advertised length
    /// honest from the proxy's perspective.
    async fn too_big() -> Response {
        let body = vec![0x43u8; 2 * 1024 * 1024]; // 2 MiB
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::from(body))
            .unwrap()
    }

    async fn server_error() -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "boom").into_response()
    }

    /// Never responds — the proxy's total timeout should fire.
    async fn slow() -> Response {
        tokio::time::sleep(Duration::from_secs(30)).await;
        (StatusCode::OK, "late").into_response()
    }

    let app = Router::new()
        .route("/ok/{n}", get(ok_bytes))
        .route("/lying.bin", get(lying))
        .route("/too-big.bin", get(too_big))
        .route("/500", get(server_error))
        .route("/slow", get(slow));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    addr
}

// ---------------------------------------------------------------------------
// Validation-path tests (HTTPS-only config)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn rejects_non_https_scheme() {
    let addr = start_download_router(test_cfg()).await;
    let resp = reqwest::Client::new()
        .get(build_download_url(addr, "http://127.0.0.1/ok.jpg"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_forbidden_host() {
    let addr = start_download_router(test_cfg()).await;
    let resp = reqwest::Client::new()
        .get(build_download_url(addr, "https://evil.example.com/x.jpg"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn rejects_userinfo_in_url() {
    let addr = start_download_router(test_cfg()).await;
    let resp = reqwest::Client::new()
        .get(build_download_url(addr, "https://u:p@127.0.0.1/ok.jpg"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_missing_file_param() {
    let addr = start_download_router(test_cfg()).await;
    let resp = reqwest::Client::new()
        .get(build_download_url_no_file(addr))
        .send()
        .await
        .unwrap();
    // Missing required `file=` → axum's Query extractor rejects with 400.
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_file_scheme() {
    let addr = start_download_router(test_cfg()).await;
    let resp = reqwest::Client::new()
        .get(build_download_url(addr, "file:///etc/passwd"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upstream_unreachable_returns_bad_gateway_or_timeout() {
    // Host allow-listed, scheme valid — but port 1 is closed on loopback.
    let addr = start_download_router(test_cfg()).await;
    let resp = reqwest::Client::new()
        .get(build_download_url(addr, "https://127.0.0.1:1/ok.jpg"))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    assert!(
        status == StatusCode::BAD_GATEWAY || status == StatusCode::GATEWAY_TIMEOUT,
        "expected 502/504, got {status}"
    );
}

#[tokio::test]
async fn error_responses_are_plain_text() {
    let addr = start_download_router(test_cfg()).await;
    let resp = reqwest::Client::new()
        .get(build_download_url(addr, "https://evil.example.com/x.jpg"))
        .send()
        .await
        .unwrap();
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        ct.starts_with("text/plain"),
        "error responses should be text/plain, got: {ct}"
    );
}

// ---------------------------------------------------------------------------
// Streaming-pipeline tests (HTTP-enabled config against a local mock)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn happy_path_streams_bytes_and_shapes_headers() {
    let upstream = start_mock_upstream().await;
    let proxy = start_download_router(test_cfg_http(256 * 1024, Duration::from_secs(5))).await;

    let file_url = format!("http://{upstream}/ok/2048");
    let resp = reqwest::Client::new()
        .get(build_download_url(proxy, &file_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let headers = resp.headers().clone();
    assert_eq!(
        headers.get(header::CONTENT_TYPE).unwrap(),
        "application/octet-stream",
        "upstream image/jpeg MUST NOT be reflected"
    );
    assert_eq!(
        headers.get(header::CACHE_CONTROL).unwrap(),
        "private, no-store",
    );
    assert_eq!(
        headers.get("x-content-type-options").unwrap(),
        "nosniff",
    );
    let cd = headers
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cd.starts_with("attachment;"), "bad CD: {cd}");
    // Filename is the last path segment ("2048") — sanitisation preserves it.
    assert!(cd.contains(r#"filename="2048""#), "bad CD: {cd}");
    // Upstream advertised Content-Length must be forwarded for progress UI.
    assert_eq!(
        headers
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok()),
        Some(2048),
    );

    let body = resp.bytes().await.unwrap();
    assert_eq!(body.len(), 2048);
    assert!(body.iter().all(|b| *b == 0x41));
}

#[tokio::test]
async fn rejects_upstream_with_advertised_length_over_cap() {
    let upstream = start_mock_upstream().await;
    // 1 MiB cap, upstream advertises 2 MiB.
    let proxy = start_download_router(test_cfg_http(1024 * 1024, Duration::from_secs(5))).await;

    let file_url = format!("http://{upstream}/too-big.bin");
    let resp = reqwest::Client::new()
        .get(build_download_url(proxy, &file_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn aborts_stream_when_upstream_lies_about_content_length() {
    let upstream = start_mock_upstream().await;
    // Cap the proxy at 64 KiB — the mock advertises 1 KiB but sends 512 KiB,
    // so the size cap should trip mid-stream.
    let proxy = start_download_router(test_cfg_http(64 * 1024, Duration::from_secs(5))).await;

    let file_url = format!("http://{upstream}/lying.bin");
    let resp = reqwest::Client::new()
        .get(build_download_url(proxy, &file_url))
        .send()
        .await
        .unwrap();

    // Initial status is 200 (headers flushed before the stream realised it's
    // being lied to), but the body read MUST be truncated. We allow either an
    // I/O error during body read or a body that happens to end cleanly but is
    // strictly less than the 512 KiB the upstream wanted to send.
    assert_eq!(resp.status(), StatusCode::OK);
    let outcome = resp.bytes().await;
    match outcome {
        Ok(body) => {
            assert!(
                body.len() < 512 * 1024,
                "expected truncated body, got full {} bytes",
                body.len()
            );
        }
        Err(_) => {
            // Connection closed mid-stream — also acceptable and actually the
            // more common axum behaviour when the body stream yields Err.
        }
    }
}

#[tokio::test]
async fn upstream_5xx_maps_to_bad_gateway() {
    let upstream = start_mock_upstream().await;
    let proxy = start_download_router(test_cfg_http(256 * 1024, Duration::from_secs(5))).await;

    let file_url = format!("http://{upstream}/500");
    let resp = reqwest::Client::new()
        .get(build_download_url(proxy, &file_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn slow_upstream_hits_gateway_timeout() {
    let upstream = start_mock_upstream().await;
    // 500 ms total deadline — mock sleeps for 30 s, so timeout must fire.
    let proxy = start_download_router(test_cfg_http(256 * 1024, Duration::from_millis(500))).await;

    let file_url = format!("http://{upstream}/slow");
    let resp = reqwest::Client::new()
        .get(build_download_url(proxy, &file_url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::GATEWAY_TIMEOUT);
}

#[tokio::test]
async fn name_override_is_sanitised_into_content_disposition() {
    let upstream = start_mock_upstream().await;
    let proxy = start_download_router(test_cfg_http(256 * 1024, Duration::from_secs(5))).await;

    let file_url = format!("http://{upstream}/ok/16");
    let mut url = build_download_url(proxy, &file_url);
    // Path-separators and control chars MUST be stripped.
    url.query_pairs_mut().append_pair("name", "../../etc/passwd");
    let resp = reqwest::Client::new().get(url).send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let cd = resp
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(cd.contains(r#"filename="etcpasswd""#), "bad CD: {cd}");
    // Must never leak path separators into the header, even inside filename*=.
    assert!(!cd.contains('/'), "CD contained raw slash: {cd}");
    assert!(!cd.contains('\\'), "CD contained raw backslash: {cd}");
}
