use async_graphql::{Enum, ID, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

#[derive(Enum, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentVisibilityStatus {
    #[graphql(name = "NORMAL")]
    #[default]
    Normal,
    #[graphql(name = "HIDDEN")]
    Hidden,
    #[graphql(name = "ILLEGAL")]
    Illegal,
}

#[derive(Enum, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentFilterType {
    #[graphql(name = "MYGRANDMALIKES")]
    #[default]
    Mygrandmalikes,
    #[graphql(name = "MYGRANDMAHATES")]
    Mygrandmahates,
}

#[derive(Enum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingType {
    #[graphql(name = "INTROONBOARDING")]
    IntroOnboarding,
}

// ============================================================================
// Profile types (getProfile response)
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ProfileGql {
    pub id: ID,
    pub username: String,
    pub status: i32,
    pub slug: i32,
    pub img: Option<String>,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    #[graphql(name = "iFollowThisUser")]
    pub i_follow_this_user: bool,
    #[graphql(name = "thisUserFollowsMe")]
    pub this_user_follows_me: bool,
    pub isreported: bool,
    pub amountposts: i32,
    pub amounttrending: i32,
    pub amountfollowed: i32,
    pub amountfollower: i32,
    pub amountfriends: i32,
    pub amountblocked: i32,
    pub amountreports: i32,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ProfileInfoResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ProfileGql>,
}

// ============================================================================
// ProfileUser types (follow lists)
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ProfileUserGql {
    pub userid: ID,
    pub username: String,
    pub slug: i32,
    pub img: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    pub isfollowed: bool,
    pub isfollowing: bool,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FollowRelationsGql {
    pub followers: Vec<ProfileUserGql>,
    pub following: Vec<ProfileUserGql>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FollowRelationsResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<FollowRelationsGql>,
}

// ============================================================================
// BasicUserInfo types (friends list)
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BasicUserInfoGql {
    pub userid: ID,
    pub img: Option<String>,
    pub username: String,
    pub slug: i32,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    pub updatedat: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FriendsResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Vec<BasicUserInfoGql>,
}

// ============================================================================
// Follow status toggle response
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct FollowStatusResponseGql {
    pub meta: DefaultResponse,
    pub isfollowing: bool,
}

// ============================================================================
// Search / list user types
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct SearchUserResult {
    pub id: ID,
    pub username: String,
    pub slug: i32,
    pub img: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct SearchUserResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<SearchUserResult>>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserListItem {
    pub id: ID,
    pub username: String,
    pub status: i32,
    pub slug: i32,
    pub img: Option<String>,
    pub biography: Option<String>,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    pub createdat: Option<String>,
    pub updatedat: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserListResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<UserListItem>>,
}

// ============================================================================
// Blocked users types
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BlockedUserGql {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: i32,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: bool,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: ContentVisibilityStatus,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: bool,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BlockedUsersGql {
    #[graphql(name = "iBlocked")]
    pub i_blocked: Vec<BlockedUserGql>,
    #[graphql(name = "blockedBy")]
    pub blocked_by: Vec<BlockedUserGql>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct BlockedUsersResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<BlockedUsersGql>,
}

// ============================================================================
// User info types (getUserInfo)
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesGql {
    #[graphql(name = "contentFilteringSeverityLevel")]
    pub content_filtering_severity_level: Option<ContentFilterType>,
    #[graphql(name = "onboardingsWereShown")]
    pub onboardings_were_shown: Vec<OnboardingType>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserInfoGql {
    pub userid: ID,
    pub liquidity: f64,
    pub amountposts: i32,
    pub amountreports: i32,
    pub amountblocked: i32,
    pub amountfollower: i32,
    pub amountfollowed: i32,
    pub amountfriends: i32,
    pub invited: Option<ID>,
    pub updatedat: Option<String>,
    #[graphql(name = "userPreferences")]
    pub user_preferences: Option<UserPreferencesGql>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserInfoResponseGql {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<UserInfoGql>,
}

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesInput {
    pub content_filtering_severity_level: Option<ContentFilterType>,
    pub shown_onboardings: Option<Vec<OnboardingType>>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesResponseGql {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<UserPreferencesPayloadGql>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserPreferencesPayloadGql {
    #[graphql(name = "contentFilteringSeverityLevel")]
    pub content_filtering_severity_level: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct UpdateResponseGql {
    pub status: String,
    #[graphql(name = "ResponseCode")]
    pub response_code: Option<String>,
}

impl UpdateResponseGql {
    pub fn success(code: &str) -> Self {
        Self {
            status: "success".to_string(),
            response_code: Some(code.to_string()),
        }
    }

    pub fn error(code: &str) -> Self {
        Self {
            status: "error".to_string(),
            response_code: Some(code.to_string()),
        }
    }
}

// ============================================================================
// Referral types
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ReferralInfoResponseGql {
    pub meta: DefaultResponse,
    #[graphql(name = "referralUuid")]
    pub referral_uuid: Option<ID>,
    #[graphql(name = "referralLink")]
    pub referral_link: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ReferralUsersGql {
    #[graphql(name = "invitedBy")]
    pub invited_by: Option<ProfileUserGql>,
    #[graphql(name = "iInvited")]
    pub i_invited: Vec<ProfileUserGql>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ReferralListResponseGql {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: ReferralUsersGql,
}

// ============================================================================
// getUser response (matches GET_USER_QUERY)
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetUserResult {
    pub id: ID,
    pub username: String,
    pub slug: i32,
    pub img: Option<String>,
    pub biography: Option<String>,
    #[graphql(name = "amountFollowers")]
    pub amount_followers: i32,
    #[graphql(name = "amountFollowing")]
    pub amount_following: i32,
    #[graphql(name = "amountPeers")]
    pub amount_peers: i32,
    #[graphql(name = "userPreferences")]
    pub user_preferences: Option<UserPreferencesGql>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetUserResponseGql {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<GetUserResult>,
}

// ============================================================================
// Conversion helpers
// ============================================================================

use crate::state::ContentVisibilityState;

pub fn convert_visibility(state: ContentVisibilityState) -> ContentVisibilityStatus {
    match state {
        ContentVisibilityState::Normal => ContentVisibilityStatus::Normal,
        ContentVisibilityState::Hidden => ContentVisibilityStatus::Hidden,
        ContentVisibilityState::Illegal => ContentVisibilityStatus::Illegal,
    }
}
