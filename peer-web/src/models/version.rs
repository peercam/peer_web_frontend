//! Version release data types for the version history page.

use serde::{Deserialize, Serialize};

/// A single changelog entry within a version release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChange {
    pub title: String,
    pub description: Vec<String>,
}

/// An external link associated with a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionLink {
    pub label: String,
    pub href: String,
}

/// A version release containing changelog information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRelease {
    pub id: String,
    pub version: String,
    pub date: String,
    pub changes: Vec<VersionChange>,
    pub links: Vec<VersionLink>,
}
