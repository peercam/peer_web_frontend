//! Moderation types for the admin dashboard.
//!
//! These structs mirror the GraphQL types returned by the moderation
//! queries: `moderationStats`, `moderationItems`, and `performModeration`.

use serde::{Deserialize, Serialize};

use super::common::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Valid moderation actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationAction {
    Hidden,
    Restored,
    Illegal,
}

impl std::fmt::Display for ModerationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModerationAction::Hidden => write!(f, "hidden"),
            ModerationAction::Restored => write!(f, "restored"),
            ModerationAction::Illegal => write!(f, "illegal"),
        }
    }
}

// ============================================================================
// Stat types
// ============================================================================

/// Aggregate counts of moderation tickets by status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationStats {
    #[serde(rename = "AmountAwaitingReview")]
    pub amount_awaiting_review: i32,
    #[serde(rename = "AmountHidden")]
    pub amount_hidden: i32,
    #[serde(rename = "AmountRestored")]
    pub amount_restored: i32,
    #[serde(rename = "AmountIllegal")]
    pub amount_illegal: i32,
}

/// Response for the `moderationStats` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationStatsResponse {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: Option<String>,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<ModerationStats>,
    pub meta: Option<DefaultResponse>,
}

// ============================================================================
// Item types
// ============================================================================

/// Basic user info for reporters, moderators, and user-type targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicUserInfo {
    pub userid: Option<String>,
    pub img: Option<String>,
    pub username: Option<String>,
    pub slug: Option<String>,
    pub biography: Option<String>,
    #[serde(rename = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[serde(rename = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    #[serde(rename = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
    pub updatedat: Option<String>,
}

/// Post data nested inside a moderation item's target content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationPost {
    pub id: Option<String>,
    pub contenttype: Option<String>,
    pub title: Option<String>,
    pub media: Option<String>,
    pub cover: Option<String>,
    pub mediadescription: Option<String>,
    pub createdat: Option<String>,
    #[serde(rename = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[serde(rename = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    pub amountreports: Option<i32>,
    pub amountlikes: Option<i32>,
    pub amountviews: Option<i32>,
    pub amountcomments: Option<i32>,
    pub amountdislikes: Option<i32>,
    pub amounttrending: Option<f64>,
    pub isliked: Option<bool>,
    pub isviewed: Option<bool>,
    pub isreported: Option<bool>,
    pub isdisliked: Option<bool>,
    pub issaved: Option<bool>,
    pub tags: Option<String>,
    pub url: Option<String>,
    pub user: Option<ModerationPostUser>,
}

/// User info nested inside a post/comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationPostUser {
    pub id: Option<String>,
    pub username: Option<String>,
    pub slug: Option<String>,
    pub img: Option<String>,
    #[serde(rename = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[serde(rename = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    pub isfollowed: Option<bool>,
    pub isfollowing: Option<bool>,
    pub isreported: Option<bool>,
    pub isfriend: Option<bool>,
}

/// Comment data nested inside a moderation item's target content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationComment {
    pub commentid: Option<String>,
    pub userid: Option<String>,
    pub postid: Option<String>,
    pub parentid: Option<String>,
    pub content: Option<String>,
    pub createdat: Option<String>,
    #[serde(rename = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[serde(rename = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    pub amountlikes: Option<i32>,
    pub amountreplies: Option<i32>,
    pub amountreports: Option<i32>,
    pub isreported: Option<bool>,
    pub isliked: Option<bool>,
    pub user: Option<ModerationPostUser>,
}

/// The reported content — only one field is populated depending on `targettype`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetContent {
    pub post: Option<ModerationPost>,
    pub comment: Option<ModerationComment>,
    pub user: Option<BasicUserInfo>,
}

/// A single moderation ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationItem {
    #[serde(rename = "moderationTicketId")]
    pub moderation_ticket_id: String,
    #[serde(rename = "targetContentId")]
    pub target_content_id: String,
    pub targettype: String,
    pub reportscount: i32,
    pub status: String,
    pub createdat: String,
    pub targetcontent: TargetContent,
    pub reporters: Vec<BasicUserInfo>,
    #[serde(rename = "moderatedBy")]
    pub moderated_by: Option<BasicUserInfo>,
}

/// Response for the `moderationItems` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerationItemListResponse {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: Option<String>,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Vec<ModerationItem>,
    pub meta: Option<DefaultResponse>,
}
