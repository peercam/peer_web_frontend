use async_graphql::{Enum, ID, SimpleObject};
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

/// Comment action type (only COMMENT is used).
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum CommentType {
    Comment,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Simplified user embedded in comment responses.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CommentUser {
    pub id: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    #[graphql(name = "isfollowed")]
    pub isfollowed: bool,
    #[graphql(name = "isfollowing")]
    pub isfollowing: bool,
}

/// Comment data returned in all comment responses.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Comment {
    pub commentid: ID,
    pub userid: ID,
    pub postid: ID,
    pub parentid: Option<ID>,
    pub content: String,
    pub createdat: String,
    pub amountlikes: i32,
    pub amountreplies: i32,
    pub isliked: bool,
    pub user: CommentUser,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
}

/// Response for `listComments` and `listChildComments` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CommentListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Comment>>,
}

/// Response for `createComment` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateCommentResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Comment>>,
}
