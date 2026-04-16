use async_graphql::{ID, InputObject, SimpleObject};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::moderation::BasicUserInfo;
use super::registration::DefaultResponse;
use super::user::ContentVisibilityStatus;

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Extended user type for admin search results.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdminUser {
    pub id: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    pub biography: Option<String>,
    pub status: Option<i32>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
    // Admin-only fields:
    pub email: Option<String>,
    pub verified: Option<i32>,
    pub roles_mask: Option<i32>,
    pub ip: Option<String>,
    pub liquidity: Option<Decimal>,
    pub situation: Option<String>,
}

/// Response for `listUsersAdminV2` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdminUserListResponse {
    pub meta: DefaultResponse,
    pub status: String,
    pub counter: i32,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AdminUser>>,
}

/// A single follow relationship for the friendship graph.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AllUserInfo {
    pub followerid: Option<ID>,
    pub followername: Option<String>,
    pub followedid: Option<ID>,
    pub followedname: Option<String>,
}

/// Response for `allfriends` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AllUserFriends {
    pub meta: DefaultResponse,
    pub status: String,
    pub counter: i32,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AllUserInfo>>,
}

/// A subcomment (reply) in the admin post comments view.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostSubCommentsData {
    pub commentid: Option<ID>,
    pub userid: Option<ID>,
    pub postid: Option<ID>,
    pub parentid: Option<ID>,
    pub content: Option<String>,
    pub createdat: Option<String>,
    pub amountlikes: Option<Decimal>,
    pub amountreplies: Option<Decimal>,
    pub isliked: Option<bool>,
    pub user: Option<BasicUserInfo>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
}

/// A top-level comment in the admin post comments view (with subcomments).
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostCommentsData {
    pub commentid: Option<ID>,
    pub userid: Option<ID>,
    pub postid: Option<ID>,
    pub parentid: Option<ID>,
    pub content: Option<String>,
    pub createdat: Option<String>,
    pub amountlikes: Option<Decimal>,
    pub isliked: Option<bool>,
    pub user: Option<BasicUserInfo>,
    pub subcomments: Option<Vec<PostSubCommentsData>>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
}

/// Response for `postcomments` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostCommentsResponse {
    pub meta: DefaultResponse,
    pub status: String,
    pub counter: i32,
    #[graphql(name = "ResponseCode")]
    #[graphql(deprecation = "Use meta.ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<PostCommentsData>>,
}

/// Input for `generateLeaderboard` query.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct LeaderboardParamsInput {
    pub start_date: String,
    pub end_date: String,
    #[graphql(name = "leaderboardUsersCount")]
    pub leaderboard_users_count: i32,
}

/// Response for `generateLeaderboard` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct LeaderboardResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "leaderboardResultLink")]
    pub leaderboard_result_link: Option<String>,
}
