//! Profile API server functions.
//!
//! Provides server functions for fetching user profiles, managing follow
//! relationships, and performing profile-related actions.

use leptos::prelude::*;
use serde::Serialize;

use crate::models::post::{ContentFilterType, PostFilterType, PostListResponse, PostSortType};
use crate::models::profile::{
    FollowRelationsResponse, FollowStatusResponse, FriendsResponse, GenericMutationResponse,
    Profile,
};

/// Variables for the getProfile query.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetProfileVars {
    #[serde(skip_serializing_if = "Option::is_none")]
    userid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_by: Option<String>,
}

/// Variables for the listFollowRelations query.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListFollowRelationsVars {
    #[serde(skip_serializing_if = "Option::is_none")]
    userid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_by: Option<String>,
    offset: i32,
    limit: i32,
}

/// Variables for the listFriends query.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListFriendsVars {
    #[serde(skip_serializing_if = "Option::is_none")]
    userid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_by: Option<String>,
    offset: i32,
    limit: i32,
}

/// Variables for the listUserPosts query.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListUserPostsVars {
    userid: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    filter_by: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_by: Option<String>,
    sort_by: String,
    offset: i32,
    limit: i32,
}

/// Variables for toggle and report mutations.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct UserIdVars {
    userid: String,
}

/// Fetch a user's profile.
///
/// If `user_id` is None, fetches the current user's own profile.
///
/// # Arguments
///
/// * `user_id` - Optional user ID; if None, fetches current user's profile
/// * `content_filter_by` - Optional content filtering level
#[server(GetProfile, "/api")]
pub async fn get_profile(
    user_id: Option<String>,
    content_filter_by: Option<ContentFilterType>,
) -> Result<Profile, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{GET_PROFILE_QUERY, GetProfileData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = GetProfileVars {
        userid: user_id,
        content_filter_by: content_filter_by.map(|c| match c {
            ContentFilterType::Mygrandmalikes => "MYGRANDMALIKES".to_string(),
            ContentFilterType::Mygrandmahates => "MYGRANDMAHATES".to_string(),
        }),
    };

    let data: GetProfileData = query(GET_PROFILE_QUERY, vars, Some(&token)).await?;

    data.get_profile
        .affected_rows
        .ok_or_else(|| ServerFnError::new("Profile not found"))
}

/// Fetch followers and following for a user.
///
/// # Arguments
///
/// * `user_id` - Optional user ID; if None, fetches current user's relations
/// * `offset` - Pagination offset
/// * `limit` - Number of records to fetch
#[server(ListFollowRelations, "/api")]
pub async fn list_follow_relations(
    user_id: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<FollowRelationsResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{LIST_FOLLOW_RELATIONS_QUERY, ListFollowRelationsData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ListFollowRelationsVars {
        userid: user_id,
        content_filter_by: None,
        offset,
        limit,
    };

    let data: ListFollowRelationsData =
        query(LIST_FOLLOW_RELATIONS_QUERY, vars, Some(&token)).await?;

    Ok(data.list_follow_relations)
}

/// Fetch mutual follows (peers/friends) for a user.
///
/// # Arguments
///
/// * `user_id` - Optional user ID; if None, fetches current user's friends
/// * `offset` - Pagination offset
/// * `limit` - Number of records to fetch
#[server(ListFriends, "/api")]
pub async fn list_friends(
    user_id: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<FriendsResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{LIST_FRIENDS_QUERY, ListFriendsData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ListFriendsVars {
        userid: user_id,
        content_filter_by: None,
        offset,
        limit,
    };

    let data: ListFriendsData = query(LIST_FRIENDS_QUERY, vars, Some(&token)).await?;

    Ok(data.list_friends)
}

/// Fetch posts for a specific user (profile posts feed).
///
/// # Arguments
///
/// * `user_id` - The user ID to fetch posts for
/// * `filter_by` - Content type filters
/// * `content_filter_by` - Content filtering level
/// * `sort_by` - Sort order
/// * `offset` - Pagination offset
/// * `limit` - Number of posts to fetch
#[server(ListUserPosts, "/api")]
pub async fn list_user_posts(
    user_id: String,
    filter_by: Vec<PostFilterType>,
    content_filter_by: Option<ContentFilterType>,
    sort_by: PostSortType,
    offset: i32,
    limit: i32,
) -> Result<PostListResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{LIST_USER_POSTS_QUERY, ListPostsData, query};

    let token = get_access_token_from_cookies().await.ok();

    let vars = ListUserPostsVars {
        userid: user_id,
        filter_by: filter_by.iter().map(|f| f.to_string()).collect(),
        content_filter_by: content_filter_by.map(|c| match c {
            ContentFilterType::Mygrandmalikes => "MYGRANDMALIKES".to_string(),
            ContentFilterType::Mygrandmahates => "MYGRANDMAHATES".to_string(),
        }),
        sort_by: sort_by.to_string(),
        offset,
        limit,
    };

    let data: ListPostsData = query(LIST_USER_POSTS_QUERY, vars, token.as_deref()).await?;

    Ok(data.list_posts)
}

/// Toggle follow status for a user.
///
/// If currently following, this will unfollow. If not following, this will follow.
///
/// # Arguments
///
/// * `user_id` - The user ID to follow/unfollow
#[server(ToggleFollow, "/api")]
pub async fn toggle_follow(user_id: String) -> Result<FollowStatusResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{TOGGLE_FOLLOW_MUTATION, ToggleFollowData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = UserIdVars { userid: user_id };

    let data: ToggleFollowData = mutate(TOGGLE_FOLLOW_MUTATION, vars, Some(&token)).await?;

    Ok(data.toggle_user_follow_status)
}

/// Toggle block status for a user.
///
/// If currently blocked, this will unblock. If not blocked, this will block.
///
/// # Arguments
///
/// * `user_id` - The user ID to block/unblock
#[server(ToggleBlock, "/api")]
pub async fn toggle_block(user_id: String) -> Result<GenericMutationResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{TOGGLE_BLOCK_MUTATION, ToggleBlockData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = UserIdVars { userid: user_id };

    let data: ToggleBlockData = mutate(TOGGLE_BLOCK_MUTATION, vars, Some(&token)).await?;

    Ok(data.toggle_block_user_status)
}

/// Report a user.
///
/// # Arguments
///
/// * `user_id` - The user ID to report
#[server(ReportUser, "/api")]
pub async fn report_user(user_id: String) -> Result<GenericMutationResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{REPORT_USER_MUTATION, ReportUserData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = UserIdVars { userid: user_id };

    let data: ReportUserData = mutate(REPORT_USER_MUTATION, vars, Some(&token)).await?;

    Ok(data.report_user)
}

/// Fetch biography text from the media server.
///
/// Biographies are stored as text files on the media server. This function
/// fetches the file content and returns it as a string.
///
/// # Arguments
///
/// * `bio_path` - Path to the biography file on the media server
#[server(FetchBiography, "/api")]
pub async fn fetch_biography(bio_path: String) -> Result<String, ServerFnError> {
    let media_host =
        std::env::var("MEDIA_HOST").unwrap_or_else(|_| "https://media.getpeer.eu".to_string());

    // Ensure the path doesn't start with a slash to avoid double slashes
    let clean_path = bio_path.trim_start_matches('/');
    let url = format!("{}/{}", media_host, clean_path);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch biography: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new("Biography not available"));
    }

    response
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read biography: {}", e)))
}
