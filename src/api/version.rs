//! Version history API server function.

use leptos::prelude::*;

use crate::models::version::VersionRelease;

/// Fetch version releases from the static JSON file.
#[server(GetVersionReleases, "/api")]
pub async fn get_version_releases() -> Result<Vec<VersionRelease>, ServerFnError> {
    // After the repo-layout flattening (see docs/adr-repo-layout-promote-peer-web.md)
    // the legacy JSON fixtures live under legacy/assets/json/. The first entry is
    // the new canonical location; the others are kept as fallbacks for any
    // out-of-tree deploy that still mirrors the old paths.
    let paths = [
        "legacy/assets/json/version_releases.json",
        "json/version_releases.json",
        "../json/version_releases.json",
    ];
    let mut content = None;
    for path in &paths {
        if let Ok(data) = tokio::fs::read_to_string(path).await {
            content = Some(data);
            break;
        }
    }
    let content = content.ok_or_else(|| ServerFnError::new("Version releases file not found"))?;
    let releases: Vec<VersionRelease> = serde_json::from_str(&content)
        .map_err(|e| ServerFnError::new(format!("Failed to parse version releases: {}", e)))?;
    Ok(releases)
}
