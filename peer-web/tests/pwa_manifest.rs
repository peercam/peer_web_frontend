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

    assert_eq!(parsed["id"], "/", "stable identity — do not change after launch");
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
