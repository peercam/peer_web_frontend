//! Advertisement history types.
//!
//! Types for the My Ads page:
//! - `AdHistoryStats` (aggregated campaign statistics)
//! - `Advertisement` (individual ad record)
//! - `AdHistoryResponse` / `AdHistoryResult` (API response wrappers)
//! - `AdvertisementType` / `AdvertisementSort` (enums)

use serde::{Deserialize, Serialize};

use super::common::DefaultResponse;
use super::post::{Post, PostUser};

/// Advertisement type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AdvertisementType {
    Pinned,
    Basic,
}

impl std::fmt::Display for AdvertisementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdvertisementType::Pinned => write!(f, "PINNED"),
            AdvertisementType::Basic => write!(f, "BASIC"),
        }
    }
}

/// Sort order for advertisement history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementSort {
    Newest,
    Oldest,
    BiggestCost,
    SmallestCost,
}

/// Aggregated stats for all user advertisements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdHistoryStats {
    #[serde(default)]
    pub token_spent: f64,
    #[serde(default)]
    pub euro_spent: f64,
    #[serde(default)]
    pub amount_ads: i32,
    #[serde(default)]
    pub gems_earned: f64,
    #[serde(default)]
    pub amount_likes: i32,
    #[serde(default)]
    pub amount_views: i32,
    #[serde(default)]
    pub amount_comments: i32,
    #[serde(default)]
    pub amount_dislikes: i32,
    #[serde(default)]
    pub amount_reports: i32,
}

/// Individual advertisement record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Advertisement {
    pub id: String,
    pub created_at: String,
    #[serde(rename = "type")]
    pub ad_type: AdvertisementType,
    pub timeframe_start: String,
    pub timeframe_end: String,
    #[serde(default)]
    pub total_token_cost: f64,
    #[serde(default)]
    pub total_euro_cost: f64,
    #[serde(default)]
    pub gems_earned: f64,
    #[serde(default)]
    pub amount_likes: i32,
    #[serde(default)]
    pub amount_views: i32,
    #[serde(default)]
    pub amount_comments: i32,
    #[serde(default)]
    pub amount_dislikes: i32,
    #[serde(default)]
    pub amount_reports: i32,
    pub user: PostUser,
    pub post: Post,
}

impl Advertisement {
    /// Check if this advertisement is currently active (end date in the future).
    pub fn is_active(&self) -> bool {
        // Compare ISO date strings — works for ISO 8601 format
        let now = chrono::Utc::now().to_rfc3339();
        self.timeframe_end > now
    }
}

/// Result wrapper inside the ad history response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdHistoryResult {
    #[serde(default)]
    pub stats: AdHistoryStats,
    #[serde(default)]
    pub advertisements: Vec<Advertisement>,
}

/// Ad history API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdHistoryResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub affected_rows: Option<AdHistoryResult>,
}

/// Response from advertisePostPinned mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisePostRow {
    pub id: String,
    pub created_at: String,
    #[serde(rename = "type")]
    pub ad_type: AdvertisementType,
    pub timeframe_start: String,
    pub timeframe_end: String,
    #[serde(default)]
    pub total_token_cost: f64,
    #[serde(default)]
    pub total_euro_cost: f64,
}

/// Response from advertisePostPinned / advertisePostBasic mutations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisePostResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub affected_rows: Vec<AdvertisePostRow>,
}
