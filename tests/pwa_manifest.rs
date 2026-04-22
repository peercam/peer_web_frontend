//! Verifies the PWA manifest story that the plan promises:
//!
//! - `peer-web/public/manifest.webmanifest` exists and parses as JSON
//! - `mime_guess` (the crate `leptos_axum::file_and_error_handler` delegates
//!   to when serving static files) resolves the `.webmanifest` extension to
//!   `application/manifest+json`
//! - Key identity fields in the manifest match the plan (`id`, `start_url`,
//!   maskable icon coverage) so the install criteria stay green.

use std::path::PathBuf;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("public")
        .join("manifest.webmanifest")
}

#[test]
fn manifest_file_is_present_and_valid_json() {
    let path = manifest_path();
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).expect("manifest.webmanifest must be valid JSON");

    assert_eq!(
        parsed["id"], "/",
        "stable identity — do not change after launch"
    );
    assert_eq!(parsed["scope"], "/");
    assert!(
        parsed["start_url"]
            .as_str()
            .is_some_and(|s| s.starts_with("/dashboard")),
        "start_url should target /dashboard",
    );

    let icons = parsed["icons"].as_array().expect("icons[] required");
    let has_maskable = icons.iter().any(|i| i["purpose"] == "maskable");
    let has_monochrome = icons.iter().any(|i| i["purpose"] == "monochrome");
    assert!(has_maskable, "Android needs at least one maskable icon");
    assert!(has_monochrome, "monochrome icon is part of the v1 plan");
}

#[test]
fn webmanifest_extension_resolves_to_application_manifest_json() {
    // `leptos_axum::file_and_error_handler` serves static files through
    // `tower_http::services::ServeDir`, which itself uses `mime_guess` to
    // pick a `Content-Type`. `.webmanifest` was added to `mime_guess` in
    // 2.0.4 — this test guards against a future dep downgrade.
    let mime = mime_guess::from_ext("webmanifest").first_or_octet_stream();
    assert_eq!(mime.essence_str(), "application/manifest+json");
}

// Mirrors `end2end/tests/pwa.spec.ts` — "service worker registers" and
// "offline navigation falls back to the offline shell". The Playwright
// suite exercises these in the browser; here we guard the *static
// prerequisites* so a future refactor can't silently drop the files the
// SW depends on.
#[test]
fn service_worker_file_is_present_and_registers_core_routes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("public")
        .join("sw.js");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    // The SW must declare a versioned cache name (cache-busting story in
    // docs/plans/pwa/pwa-implementation.md) and reference the offline
    // fallback shell used by the "offline navigation" E2E test.
    assert!(
        raw.contains("peer-shell-v"),
        "service worker must use a versioned cache name (peer-shell-v…)"
    );
    assert!(
        raw.contains("/offline.html"),
        "service worker must register /offline.html as the nav fallback"
    );
}

#[test]
fn offline_shell_is_present_and_self_contained() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("public")
        .join("offline.html");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    // The E2E `offline navigation falls back to the offline shell` test
    // asserts that the rendered `<h1>` matches /offline/i. Guard that
    // contract at the asset level so the fallback stays meaningful.
    let lower = raw.to_lowercase();
    assert!(
        lower.contains("<h1") && lower.contains("offline"),
        "offline.html must include an <h1> heading mentioning 'offline'"
    );
}

#[test]
fn html_extension_resolves_to_text_html() {
    // `service-worker.js` serves the cached `/offline.html` in response
    // to a failed navigation; the browser treats it as an HTML document
    // because ServeDir hands it off to `mime_guess`.
    let mime = mime_guess::from_ext("html").first_or_octet_stream();
    assert_eq!(mime.essence_str(), "text/html");
    let mime = mime_guess::from_ext("js").first_or_octet_stream();
    assert!(mime.essence_str().contains("javascript"));
}
