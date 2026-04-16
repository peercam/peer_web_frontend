use async_graphql::{Enum, ID, SimpleObject};
use serde::{Deserialize, Serialize};

use super::comment::Comment;
use super::post::Post;
use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Status of a moderation ticket.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum ModerationStatus {
    WaitingForReview,
    Hidden,
    Restored,
    Illegal,
}

impl ModerationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WaitingForReview => "waiting_for_review",
            Self::Hidden => "hidden",
            Self::Restored => "restored",
            Self::Illegal => "illegal",
        }
    }
}

/// Type of content being moderated.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum ModerationContentType {
    Post,
    Comment,
    User,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Aggregate counts of moderation tickets by status.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ModerationStats {
    #[graphql(name = "AmountAwaitingReview")]
    pub amount_awaiting_review: i32,
    #[graphql(name = "AmountHidden")]
    pub amount_hidden: i32,
    #[graphql(name = "AmountRestored")]
    pub amount_restored: i32,
    #[graphql(name = "AmountIllegal")]
    pub amount_illegal: i32,
}

/// Response for `moderationStats` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ModerationStatsResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ModerationStats>,
}

/// Basic user info for reporters, moderators, and user-type moderation targets.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BasicUserInfo {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: String,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
    pub updatedat: Option<String>,
}

/// The reported content, only one field is non-null depending on `targettype`.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TargetContent {
    pub post: Option<Post>,
    pub comment: Option<Comment>,
    pub user: Option<BasicUserInfo>,
}

/// A single moderation ticket.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ModerationItem {
    #[graphql(name = "moderationTicketId")]
    pub moderation_ticket_id: ID,
    #[graphql(name = "targetContentId")]
    pub target_content_id: ID,
    pub targettype: String,
    pub reportscount: i32,
    pub status: String,
    pub createdat: String,
    pub targetcontent: TargetContent,
    pub reporters: Vec<BasicUserInfo>,
    #[graphql(name = "moderatedBy")]
    pub moderated_by: Option<BasicUserInfo>,
}

/// Response for `moderationItems` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ModerationItemListResponse {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Vec<ModerationItem>,
}
