use async_graphql::{Enum, ID, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;
pub use super::user::ContentFilterType;

// ============================================================================
// Enums
// ============================================================================

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "lowercase")]
pub enum ContentType {
    Image,
    Audio,
    Video,
    Text,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostFilterType {
    Image,
    Audio,
    Video,
    Text,
    Followed,
    Follower,
    Viewed,
    Friends,
}

/// GraphQL exposes this as `PostSortBy` to match the production peergamma SDL
/// (`src/services/post_service.rs:35`). The Rust ident stays `PostSortType`
/// to keep the frontend models, fixtures, and mock state references stable.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(name = "PostSortBy", rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostSortType {
    Newest,
    Trending,
    Likes,
    Dislikes,
    Views,
    Comments,
    ForMe,
    Oldest,
    Follower,
    Followed,
    Relevant,
    Friends,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum IgnoreOption {
    Yes,
    No,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostActionType {
    Like,
    Dislike,
    Report,
    View,
    Share,
    Save,
    Unlike,
    Undislike,
    Unsave,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum PostType {
    Post,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum GetOnly {
    View,
    Like,
    Dislike,
    Commentlike,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

/// User embedded in post responses.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostUser {
    pub id: ID,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
    #[graphql(name = "isfollowed")]
    pub isfollowed: bool,
    #[graphql(name = "isfollowing")]
    pub isfollowing: bool,
    #[graphql(name = "isfriend")]
    pub isfriend: bool,
}

/// Full post type returned in all post responses.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Post {
    pub id: ID,
    pub contenttype: String,
    pub title: String,
    pub media: Option<String>,
    pub cover: Option<String>,
    pub mediadescription: Option<String>,
    pub createdat: String,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    pub amountreports: i32,
    pub amountlikes: i32,
    pub amountviews: i32,
    pub amountcomments: i32,
    pub amountdislikes: i32,
    pub amounttrending: Option<i32>,
    pub isliked: bool,
    pub isviewed: bool,
    pub isreported: bool,
    pub isdisliked: bool,
    pub issaved: bool,
    pub tags: Vec<String>,
    pub url: String,
    pub user: PostUser,
}

/// Response for `listPosts` and `guestListPost` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Post>>,
}

/// Input for `createPost` mutation.
#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct PostInput {
    pub title: String,
    pub mediadescription: Option<String>,
    pub contenttype: ContentType,
    pub media: Option<Vec<String>>,
    pub cover: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    #[graphql(name = "uploadedFiles")]
    pub uploaded_files: Option<String>,
}

/// Response for `createPost` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreatePostResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<CreatedPost>,
}

/// Created post data in the response.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreatedPost {
    pub id: ID,
    pub contenttype: String,
    pub title: String,
}

/// Response for `postEligibility` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostEligibilityResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "eligibilityToken")]
    pub eligibility_token: Option<String>,
}

/// A single tag.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}

/// Response for `searchTags` and `listTags` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TagSearchResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Tag>>,
}

/// Response for `postInteractions` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct PostInteractionResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<PostUser>>,
}
