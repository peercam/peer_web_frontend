//! Post and feed-related types.
//!
//! These structs mirror the GraphQL types used for the dashboard feed:
//! - `Post` (query response)
//! - `PostFilterType` (filter enum)
//! - `PostSortType` (sort enum)
//! - `ContentFilterType` (content filtering severity)
//! - `AdvertisementPost` (promoted posts)

use serde::{Deserialize, Serialize};

use super::common::DefaultResponse;

/// Content type of a post.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentType {
    Image,
    Video,
    Audio,
    Text,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Image => write!(f, "IMAGE"),
            ContentType::Video => write!(f, "VIDEO"),
            ContentType::Audio => write!(f, "AUDIO"),
            ContentType::Text => write!(f, "TEXT"),
        }
    }
}

/// Post filter types for the `filterBy` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PostFilterType {
    Image,
    Video,
    Audio,
    Text,
    Followed,
    Follower,
    Friends,
    Viewed,
}

impl std::fmt::Display for PostFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostFilterType::Image => write!(f, "IMAGE"),
            PostFilterType::Video => write!(f, "VIDEO"),
            PostFilterType::Audio => write!(f, "AUDIO"),
            PostFilterType::Text => write!(f, "TEXT"),
            PostFilterType::Followed => write!(f, "FOLLOWED"),
            PostFilterType::Follower => write!(f, "FOLLOWER"),
            PostFilterType::Friends => write!(f, "FRIENDS"),
            PostFilterType::Viewed => write!(f, "VIEWED"),
        }
    }
}

/// Post sort options for the `sortBy` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum PostSortType {
    #[default]
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

impl std::fmt::Display for PostSortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostSortType::Newest => write!(f, "NEWEST"),
            PostSortType::Trending => write!(f, "TRENDING"),
            PostSortType::Likes => write!(f, "LIKES"),
            PostSortType::Dislikes => write!(f, "DISLIKES"),
            PostSortType::Views => write!(f, "VIEWS"),
            PostSortType::Comments => write!(f, "COMMENTS"),
            PostSortType::ForMe => write!(f, "FOR_ME"),
            PostSortType::Oldest => write!(f, "OLDEST"),
            PostSortType::Follower => write!(f, "FOLLOWER"),
            PostSortType::Followed => write!(f, "FOLLOWED"),
            PostSortType::Relevant => write!(f, "RELEVANT"),
            PostSortType::Friends => write!(f, "FRIENDS"),
        }
    }
}

/// Content filtering severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentFilterType {
    /// Stricter: hides flagged content.
    Mygrandmalikes,
    /// Relaxed: shows more content.
    Mygrandmahates,
}

/// Block list filtering option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IgnoreOption {
    Yes,
    No,
}

/// Post action types for the `resolvePostAction` mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PostActionType {
    View,
    Like,
    Dislike,
    Save,
    Report,
    Unlike,
    Undislike,
    Unsave,
}

/// Simplified user embedded in posts.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostUser {
    pub id: String,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub img: Option<String>,
    #[serde(default)]
    pub isfollowed: bool,
    #[serde(default)]
    pub isfollowing: bool,
    #[serde(default)]
    pub isfriend: bool,
}

/// Post data from API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: String,
    pub contenttype: ContentType,
    pub title: String,
    #[serde(default)]
    pub media: Option<String>,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub mediadescription: Option<String>,
    pub createdat: String,
    #[serde(default)]
    pub amountlikes: i32,
    #[serde(default)]
    pub amountviews: i32,
    #[serde(default)]
    pub amountcomments: i32,
    #[serde(default)]
    pub amountdislikes: i32,
    #[serde(default)]
    pub amounttrending: Option<i32>,
    #[serde(default)]
    pub isliked: bool,
    #[serde(default)]
    pub isviewed: bool,
    #[serde(default)]
    pub isdisliked: bool,
    #[serde(default)]
    pub issaved: bool,
    #[serde(default)]
    pub isreported: bool,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub has_active_reports: Option<bool>,
    #[serde(default)]
    pub visibility_status: Option<String>,
    #[serde(default)]
    pub is_hidden_for_users: Option<bool>,
    pub user: PostUser,
}

/// Advertisement metadata attached to promoted posts.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisementInfo {
    pub advertisementid: String,
    pub advertisementtype: String,
    pub startdate: String,
    pub enddate: String,
}

/// Advertisement post wrapper (from `listAdvertisementPosts`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisementPost {
    pub post: Post,
    pub advertisement: AdvertisementInfo,
}

/// List posts response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows", default)]
    pub affected_rows: Vec<Post>,
}

impl PostListResponse {
    /// Check if the response indicates success.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Advertisement list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows", default)]
    pub affected_rows: Vec<AdvertisementPost>,
}

/// User search result.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSearchResult {
    pub id: String,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub img: Option<String>,
    #[serde(default)]
    pub biography: Option<String>,
}

/// User search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSearchResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows", default)]
    pub affected_rows: Vec<UserSearchResult>,
}

/// Post action response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostActionResponse {
    pub meta: DefaultResponse,
}

/// Unified feed item for rendering (regular post or ad).
#[derive(Debug, Clone)]
pub enum FeedItem {
    Post(Post),
    Ad {
        post: Post,
        ad_info: AdvertisementInfo,
    },
}

impl FeedItem {
    /// Get a reference to the underlying post.
    pub fn post(&self) -> &Post {
        match self {
            FeedItem::Post(p) => p,
            FeedItem::Ad { post, .. } => post,
        }
    }

    /// Check if this is an ad.
    pub fn is_ad(&self) -> bool {
        matches!(self, FeedItem::Ad { .. })
    }

    /// Get the post ID.
    pub fn id(&self) -> &str {
        &self.post().id
    }
}
