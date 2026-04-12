//! Posts API server functions.
//!
//! Provides server functions for fetching posts, ads, and performing
//! post actions (like, dislike, save, view).

use leptos::prelude::*;
use serde::Serialize;

use crate::models::post::{
    AdListResponse, ContentFilterType, PostActionType, PostFilterType, PostListResponse,
    PostSortType, UserSearchResult,
};

/// Variables for the listPosts query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListPostsVars {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    filter_by: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_by: Option<String>,
    sort_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    offset: i32,
    limit: i32,
}

/// Variables for the listAdvertisementPosts query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListAdPostsVars {
    #[serde(skip_serializing_if = "Option::is_none")]
    content_filter_by: Option<String>,
    offset: i32,
    limit: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
}

/// Variables for the resolvePostAction mutation.
#[derive(Debug, Serialize)]
struct PostActionVars {
    postid: String,
    action: String,
}

/// Variables for the searchUser query.
#[derive(Debug, Serialize)]
struct SearchUserVars {
    username: String,
    offset: i32,
    limit: i32,
}

/// Variables for the getUser query.
#[derive(Debug, Serialize)]
struct GetUserVars {
    id: String,
}

/// Fetch paginated posts with filters.
///
/// # Arguments
///
/// * `filter_by` - Content type and feed filters
/// * `content_filter_by` - Content filtering severity level
/// * `sort_by` - Sort order
/// * `title` - Title search query
/// * `tag` - Tag filter
/// * `offset` - Pagination offset
/// * `limit` - Number of posts to fetch
#[server(ListPosts, "/api")]
pub async fn list_posts(
    filter_by: Vec<PostFilterType>,
    content_filter_by: Option<ContentFilterType>,
    sort_by: PostSortType,
    title: Option<String>,
    tag: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<PostListResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, ListPostsData, LIST_POSTS_QUERY};

    let token = get_access_token_from_cookies().await.ok();

    let vars = ListPostsVars {
        filter_by: filter_by.iter().map(|f| f.to_string()).collect(),
        content_filter_by: content_filter_by.map(|c| match c {
            ContentFilterType::Mygrandmalikes => "MYGRANDMALIKES".to_string(),
            ContentFilterType::Mygrandmahates => "MYGRANDMAHATES".to_string(),
        }),
        sort_by: sort_by.to_string(),
        title: title.filter(|s| !s.is_empty()),
        tag: tag.filter(|s| !s.is_empty()),
        offset,
        limit,
    };

    let data: ListPostsData = query(LIST_POSTS_QUERY, vars, token.as_deref()).await?;
    Ok(data.list_posts)
}

/// Fetch advertisement posts.
#[server(ListAdPosts, "/api")]
pub async fn list_ad_posts(
    content_filter_by: Option<ContentFilterType>,
    offset: i32,
    limit: i32,
    title: Option<String>,
    tag: Option<String>,
) -> Result<AdListResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, ListAdPostsData, LIST_AD_POSTS_QUERY};

    let token = get_access_token_from_cookies().await.ok();

    let vars = ListAdPostsVars {
        content_filter_by: content_filter_by.map(|c| match c {
            ContentFilterType::Mygrandmalikes => "MYGRANDMALIKES".to_string(),
            ContentFilterType::Mygrandmahates => "MYGRANDMAHATES".to_string(),
        }),
        offset,
        limit,
        title: title.filter(|s| !s.is_empty()),
        tag: tag.filter(|s| !s.is_empty()),
    };

    let data: ListAdPostsData = query(LIST_AD_POSTS_QUERY, vars, token.as_deref()).await?;
    Ok(data.list_advertisement_posts)
}

/// Perform an action on a post (like, dislike, save, view, etc.).
#[server(PostAction, "/api")]
pub async fn post_action(post_id: String, action: PostActionType) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, PostActionData, POST_ACTION_MUTATION};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = PostActionVars {
        postid: post_id,
        action: match action {
            PostActionType::View => "VIEW",
            PostActionType::Like => "LIKE",
            PostActionType::Dislike => "DISLIKE",
            PostActionType::Save => "SAVE",
            PostActionType::Report => "REPORT",
            PostActionType::Unlike => "UNLIKE",
            PostActionType::Undislike => "UNDISLIKE",
            PostActionType::Unsave => "UNSAVE",
        }
        .to_string(),
    };

    let _data: PostActionData = mutate(POST_ACTION_MUTATION, vars, Some(&token)).await?;
    Ok(())
}

/// Search users by username.
#[server(SearchUsers, "/api")]
pub async fn search_users(username: String, limit: i32) -> Result<Vec<UserSearchResult>, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, SearchUserData, SEARCH_USERS_QUERY};

    let token = get_access_token_from_cookies().await.ok();

    let vars = SearchUserVars {
        username,
        offset: 0,
        limit,
    };

    let data: SearchUserData = query(SEARCH_USERS_QUERY, vars, token.as_deref()).await?;
    Ok(data.search_user.affected_rows)
}

/// Get user info by ID.
#[server(GetUserInfo, "/api")]
pub async fn get_user_info(user_id: String) -> Result<crate::models::user::UserInfo, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, GetUserData, GET_USER_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = GetUserVars { id: user_id };

    let data: GetUserData = query(GET_USER_QUERY, vars, Some(&token)).await?;

    data.get_user
        .affected_rows
        .ok_or_else(|| ServerFnError::new("User not found"))
}
