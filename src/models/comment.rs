//! Comment-related types for view post functionality.
//!
//! These structs mirror the GraphQL types used for comments:
//! - `Comment` (query response)
//! - `CommentUser` (embedded user in comments)
//! - `CommentListResponse` (paginated comment list)

use serde::{Deserialize, Serialize};

use super::common::DefaultResponse;

/// Simplified user embedded in comments.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentUser {
    pub id: String,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub img: Option<String>,
    #[serde(default)]
    pub isfollowed: bool,
    #[serde(default)]
    pub isfollowing: bool,
}

/// Comment data from API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub commentid: String,
    pub userid: String,
    pub postid: String,
    #[serde(default)]
    pub parentid: Option<String>,
    pub content: String,
    pub createdat: String,
    #[serde(default)]
    pub amountlikes: i32,
    #[serde(default)]
    pub amountreplies: i32,
    #[serde(default)]
    pub isliked: bool,
    pub user: CommentUser,
}

/// Comment list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows", default)]
    pub affected_rows: Vec<Comment>,
}

impl CommentListResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Create comment response (single comment returned).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows", default)]
    pub affected_rows: Vec<Comment>,
}

impl CreateCommentResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }

    /// Extract the created comment.
    pub fn comment(&self) -> Option<&Comment> {
        self.affected_rows.first()
    }
}

/// Like comment response.
///
/// Backend returns a flat-response envelope where `status` is lowercase
/// but `ResponseCode` / `ResponseMessage` are PascalCase. `rename_all =
/// "PascalCase"` would also uppercase `status` — override it explicitly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LikeCommentResponse {
    #[serde(rename = "status")]
    pub status: String,
    pub response_code: String,
    pub response_message: String,
}

impl LikeCommentResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}
