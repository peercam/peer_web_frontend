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

// ============================================================================
// Post Creation Types
// ============================================================================

/// Content type for post creation (lowercase serialization).
///
/// Used with the `createPost` mutation which expects lowercase values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CreateContentType {
    #[default]
    Text,
    Image,
    Audio,
    Video,
}

impl std::fmt::Display for CreateContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateContentType::Text => write!(f, "text"),
            CreateContentType::Image => write!(f, "image"),
            CreateContentType::Audio => write!(f, "audio"),
            CreateContentType::Video => write!(f, "video"),
        }
    }
}

impl CreateContentType {
    /// Get the display name for UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            CreateContentType::Text => "Text Post",
            CreateContentType::Image => "Image Post",
            CreateContentType::Audio => "Audio Post",
            CreateContentType::Video => "Video Post",
        }
    }

    /// Get the icon class for this content type.
    pub fn icon_class(&self) -> &'static str {
        match self {
            CreateContentType::Text => "peer-icon-text",
            CreateContentType::Image => "peer-icon-image",
            CreateContentType::Audio => "peer-icon-audio",
            CreateContentType::Video => "peer-icon-video",
        }
    }

    /// Maximum number of media files allowed.
    pub fn max_media(&self) -> usize {
        match self {
            CreateContentType::Image => 5,
            CreateContentType::Video => 2,
            CreateContentType::Audio => 1,
            CreateContentType::Text => 1,
        }
    }

    /// Check if this content type requires media upload.
    pub fn requires_media(&self) -> bool {
        !matches!(self, CreateContentType::Text)
    }
}

/// Input for creating a new post.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostInput {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mediadescription: Option<String>,
    pub contenttype: CreateContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_files: Option<String>,
}

impl CreatePostInput {
    /// Validate the input fields.
    pub fn validate(&self) -> Result<(), String> {
        // Title: 1-63 chars
        if self.title.is_empty() {
            return Err("Title is required".to_string());
        }
        if self.title.len() > 63 {
            return Err("Title must be 63 characters or less".to_string());
        }

        // Description: max 500 chars
        if let Some(ref desc) = self.mediadescription {
            if desc.len() > 500 {
                return Err("Description must be 500 characters or less".to_string());
            }
        }

        // Tags: max 10, valid format
        if let Some(ref tags) = self.tags {
            if tags.len() > 10 {
                return Err("Maximum 10 tags allowed".to_string());
            }
            for tag in tags {
                if !is_valid_tag(tag) {
                    return Err(format!("Invalid tag format: {}", tag));
                }
            }
        }

        Ok(())
    }
}

/// Check if a tag name is valid (alphanumeric/underscores, 2-53 chars).
pub fn is_valid_tag(tag: &str) -> bool {
    let len = tag.len();
    len >= 2 && len <= 53 && tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Post eligibility response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEligibilityResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub eligibility_token: Option<String>,
}

impl PostEligibilityResponse {
    /// Check if eligibility was granted.
    pub fn is_eligible(&self) -> bool {
        self.meta.status == "success" && self.eligibility_token.is_some()
    }

    /// Get the eligibility token if available.
    pub fn token(&self) -> Option<&str> {
        self.eligibility_token.as_deref()
    }
}

/// Upload response from `/upload-post`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadPostResponse {
    pub status: String,
    #[serde(rename = "ResponseCode")]
    pub response_code: String,
    #[serde(default)]
    pub affected_rows: Option<UploadAffectedRows>,
}

/// Affected rows from upload response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadAffectedRows {
    pub uploaded_files: String,
}

impl UploadPostResponse {
    /// Check if upload was successful.
    pub fn is_success(&self) -> bool {
        self.status == "success" && self.response_code == "11515"
    }

    /// Get the uploaded file names.
    pub fn uploaded_files(&self) -> Option<&str> {
        self.affected_rows
            .as_ref()
            .map(|a| a.uploaded_files.as_str())
    }
}

/// Response from createPost mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub affected_rows: Option<CreatedPost>,
}

/// Created post data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedPost {
    pub id: String,
    pub contenttype: String,
    pub title: String,
}

impl CreatePostResponse {
    /// Check if post was created successfully.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }

    /// Get the created post ID.
    pub fn post_id(&self) -> Option<&str> {
        self.affected_rows.as_ref().map(|p| p.id.as_str())
    }
}

/// Tag from search results.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
}

/// Tag search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSearchResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[serde(rename = "affectedRows", default)]
    pub affected_rows: Vec<Tag>,
}

impl TagSearchResponse {
    /// Check if search was successful.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }

    /// Get the tag names.
    pub fn tag_names(&self) -> Vec<String> {
        self.affected_rows.iter().map(|t| t.name.clone()).collect()
    }
}

/// Represents a media file for upload.
#[derive(Debug, Clone)]
pub struct MediaFile {
    /// File name.
    pub name: String,
    /// MIME type.
    pub mime_type: String,
    /// File data as bytes.
    pub data: Vec<u8>,
    /// Preview URL (for displaying in UI).
    pub preview_url: Option<String>,
}

impl MediaFile {
    /// Create a new media file.
    pub fn new(name: String, mime_type: String, data: Vec<u8>) -> Self {
        Self {
            name,
            mime_type,
            data,
            preview_url: None,
        }
    }

    /// Create with a preview URL.
    pub fn with_preview(mut self, url: String) -> Self {
        self.preview_url = Some(url);
        self
    }

    /// Get file size in bytes.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Convert to base64 data URL.
    #[cfg(feature = "hydrate")]
    pub fn to_data_url(&self) -> String {
        use base64::{Engine, engine::general_purpose::STANDARD};
        format!(
            "data:{};base64,{}",
            self.mime_type,
            STANDARD.encode(&self.data)
        )
    }
}
