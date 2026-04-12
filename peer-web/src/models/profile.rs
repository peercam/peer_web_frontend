//! Profile-related types.
//!
//! These structs mirror the GraphQL types used for user profiles:
//! - `Profile` (getProfile query response)
//! - `ProfileUser` (user in follow lists)
//! - `FollowRelations` (listFollowRelations response)
//! - `BasicUserInfo` (listFriends response)

use serde::{Deserialize, Serialize};

use super::common::DefaultResponse;

/// Content visibility status for profiles and posts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContentVisibilityStatus {
    #[default]
    Normal,
    Hidden,
    Illegal,
}

/// Full profile data from `getProfile` query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub username: String,
    pub status: i32,
    pub slug: i32,
    #[serde(default)]
    pub img: Option<String>,
    #[serde(default)]
    pub biography: Option<String>,
    #[serde(default)]
    pub visibility_status: ContentVisibilityStatus,
    #[serde(default)]
    pub is_hidden_for_users: bool,
    #[serde(default)]
    pub has_active_reports: bool,
    /// Do I follow this profile?
    #[serde(default, rename = "iFollowThisUser")]
    pub i_follow_this_user: bool,
    /// Does this profile follow me?
    #[serde(default, rename = "thisUserFollowsMe")]
    pub this_user_follows_me: bool,
    /// Have I reported this user?
    #[serde(default)]
    pub isreported: bool,
    #[serde(default)]
    pub amountposts: i32,
    #[serde(default)]
    pub amounttrending: i32,
    /// Number of users this profile follows.
    #[serde(default)]
    pub amountfollowed: i32,
    /// Number of followers.
    #[serde(default)]
    pub amountfollower: i32,
    /// Number of mutual follows (peers/friends).
    #[serde(default)]
    pub amountfriends: i32,
    #[serde(default)]
    pub amountblocked: i32,
    #[serde(default)]
    pub amountreports: i32,
}

impl Profile {
    /// Check if this profile is hidden for the current user.
    pub fn is_hidden(&self) -> bool {
        self.visibility_status == ContentVisibilityStatus::Hidden || self.is_hidden_for_users
    }

    /// Check if this profile is marked as illegal.
    pub fn is_illegal(&self) -> bool {
        self.visibility_status == ContentVisibilityStatus::Illegal
    }

    /// Get avatar URL with fallback.
    pub fn avatar_url(&self) -> &str {
        self.img.as_deref().unwrap_or("/svg/noname.svg")
    }

    /// Format the slug for display (e.g., "#12345").
    pub fn display_slug(&self) -> String {
        format!("#{}", self.slug)
    }
}

/// Profile API response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
    pub meta: DefaultResponse,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<Profile>,
}

impl ProfileResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// User in a follow list (follower/following).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUser {
    pub userid: String,
    pub username: String,
    pub slug: i32,
    #[serde(default)]
    pub img: Option<String>,
    #[serde(default)]
    pub visibility_status: ContentVisibilityStatus,
    #[serde(default)]
    pub is_hidden_for_users: bool,
    #[serde(default)]
    pub has_active_reports: bool,
    /// Do I follow this person?
    #[serde(default)]
    pub isfollowed: bool,
    /// Does this person follow me?
    #[serde(default)]
    pub isfollowing: bool,
}

impl ProfileUser {
    /// Get avatar URL with fallback.
    pub fn avatar_url(&self) -> &str {
        self.img.as_deref().unwrap_or("/svg/noname.svg")
    }

    /// Format the slug for display.
    pub fn display_slug(&self) -> String {
        format!("#{}", self.slug)
    }
}

/// Follow relations response (followers + following).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRelations {
    #[serde(default)]
    pub followers: Vec<ProfileUser>,
    #[serde(default)]
    pub following: Vec<ProfileUser>,
}

/// Wrapper for listFollowRelations response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRelationsResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub counter: i32,
    #[serde(rename = "affectedRows")]
    pub affected_rows: Option<FollowRelations>,
}

impl FollowRelationsResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Basic user info (for friends/peers list).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicUserInfo {
    pub userid: String,
    #[serde(default)]
    pub img: Option<String>,
    pub username: String,
    pub slug: i32,
    #[serde(default)]
    pub biography: Option<String>,
    #[serde(default)]
    pub visibility_status: ContentVisibilityStatus,
    #[serde(default)]
    pub is_hidden_for_users: bool,
    #[serde(default)]
    pub has_active_reports: bool,
}

impl BasicUserInfo {
    /// Get avatar URL with fallback.
    pub fn avatar_url(&self) -> &str {
        self.img.as_deref().unwrap_or("/svg/noname.svg")
    }

    /// Format the slug for display.
    pub fn display_slug(&self) -> String {
        format!("#{}", self.slug)
    }
}

/// Wrapper for listFriends response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendsResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub counter: i32,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Vec<BasicUserInfo>,
}

impl FriendsResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Toggle follow status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowStatusResponse {
    pub meta: DefaultResponse,
    /// New follow state after toggle.
    #[serde(default)]
    pub isfollowing: bool,
}

impl FollowStatusResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Generic mutation response (block, report, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericMutationResponse {
    pub meta: DefaultResponse,
}

impl GenericMutationResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Tab type for the relations modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelationsTab {
    #[default]
    Followers,
    Following,
    Peers,
}

impl RelationsTab {
    pub fn label(&self) -> &'static str {
        match self {
            RelationsTab::Followers => "Followers",
            RelationsTab::Following => "Following",
            RelationsTab::Peers => "Peers",
        }
    }
}
