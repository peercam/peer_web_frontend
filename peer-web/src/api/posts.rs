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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
struct PostActionVars {
    postid: String,
    action: String,
}

/// Variables for the searchUser query.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct SearchUserVars {
    username: String,
    offset: i32,
    limit: i32,
}

/// Variables for the getUser query.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
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
    use crate::api::graphql::{LIST_POSTS_QUERY, ListPostsData, query};

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
    use crate::api::graphql::{LIST_AD_POSTS_QUERY, ListAdPostsData, query};

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
    use crate::api::graphql::{POST_ACTION_MUTATION, PostActionData, mutate};

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
pub async fn search_users(
    username: String,
    limit: i32,
) -> Result<Vec<UserSearchResult>, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{SEARCH_USERS_QUERY, SearchUserData, query};

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
pub async fn get_user_info(
    user_id: String,
) -> Result<crate::models::user::UserInfo, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{GET_USER_QUERY, GetUserData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = GetUserVars { id: user_id };

    let data: GetUserData = query(GET_USER_QUERY, vars, Some(&token)).await?;

    data.get_user
        .affected_rows
        .ok_or_else(|| ServerFnError::new("User not found"))
}

// ============================================================================
// Post Creation Functions
// ============================================================================

/// Variables for the postEligibility query.
#[derive(Debug, Serialize)]
#[allow(dead_code)] // used by #[server] macro expansion (server-only)
struct EmptyVars {}

/// Variables for the createPost mutation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // used by #[server] macro expansion (server-only)
struct CreatePostVars {
    action: String,
    input: crate::models::post::CreatePostInput,
}

/// Variables for the searchTags query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // used by #[server] macro expansion (server-only)
struct SearchTagsVars {
    tag_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
}

/// Check post eligibility and get an upload token.
///
/// This must be called before uploading files for a new post.
/// The eligibility token is used to authorize the file upload.
///
/// # Errors
///
/// - `31512`: Rate limit exceeded (max 5 tokens/hour)
/// - `51301`: Insufficient token balance
/// - `60501`: Not authenticated
#[server(CheckPostEligibility, "/api")]
pub async fn check_post_eligibility()
-> Result<crate::models::post::PostEligibilityResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{POST_ELIGIBILITY_QUERY, PostEligibilityData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let data: PostEligibilityData =
        query(POST_ELIGIBILITY_QUERY, EmptyVars {}, Some(&token)).await?;

    Ok(data.post_eligibility)
}

/// Create a new post.
///
/// # Arguments
///
/// * `input` - The post creation input containing title, description, content type, etc.
///
/// # Errors
///
/// - `30101`: Missing required fields
/// - `30210`: Invalid title length (1-63 chars)
/// - `30262`: Invalid tags (format/count)
/// - `31511`: Temporary file expired
/// - `51301`: Insufficient token balance
/// - `60501`: Not authenticated
#[server(CreatePost, "/api")]
pub async fn create_post(
    input: crate::models::post::CreatePostInput,
) -> Result<crate::models::post::CreatePostResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{CREATE_POST_MUTATION, CreatePostData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    // Validate input before sending
    input.validate().map_err(|e| ServerFnError::new(e))?;

    let vars = CreatePostVars {
        action: "POST".to_string(),
        input,
    };

    let data: CreatePostData = mutate(CREATE_POST_MUTATION, vars, Some(&token)).await?;

    Ok(data.create_post)
}

/// Search tags by name.
///
/// Used for tag autocomplete in the post creation form.
///
/// # Arguments
///
/// * `tag_name` - The search query (2-53 chars, alphanumeric/underscores)
/// * `offset` - Pagination offset
/// * `limit` - Maximum number of results to return
#[server(SearchTags, "/api")]
pub async fn search_tags(
    tag_name: String,
    offset: Option<i32>,
    limit: Option<i32>,
) -> Result<crate::models::post::TagSearchResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{SEARCH_TAGS_QUERY, SearchTagsData, query};

    let token = get_access_token_from_cookies().await.ok();

    // Validate tag name format
    if tag_name.len() < 2 {
        return Err(ServerFnError::new("Tag name must be at least 2 characters"));
    }
    if tag_name.len() > 53 {
        return Err(ServerFnError::new("Tag name must be at most 53 characters"));
    }

    let vars = SearchTagsVars {
        tag_name,
        offset,
        limit,
    };

    let data: SearchTagsData = query(SEARCH_TAGS_QUERY, vars, token.as_deref()).await?;

    Ok(data.search_tags)
}

/// Upload files for a new post.
///
/// This function handles the multipart file upload to the `/upload-post` endpoint.
/// It requires an eligibility token from `check_post_eligibility`.
///
/// # Arguments
///
/// * `eligibility_token` - Token from `check_post_eligibility`
/// * `files` - List of files to upload (name, mime_type, base64_data)
///
/// # Returns
///
/// The comma-separated list of uploaded filenames on success.
#[server(UploadPostFiles, "/api")]
pub async fn upload_post_files(
    eligibility_token: String,
    files: Vec<(String, String, String)>, // (name, mime_type, base64_data)
) -> Result<crate::models::post::UploadPostResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use reqwest::multipart::{Form, Part};
    use std::env;

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let upload_endpoint = env::var("UPLOAD_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4000/upload-post".to_string());

    // Build multipart form
    let mut form = Form::new().text("eligibilityToken", eligibility_token);

    for (idx, (name, mime_type, base64_data)) in files.into_iter().enumerate() {
        // Decode base64 data
        let data = STANDARD.decode(&base64_data).map_err(|e| {
            ServerFnError::new(format!("Invalid base64 data for file {}: {}", idx, e))
        })?;

        let part = Part::bytes(data)
            .file_name(name)
            .mime_str(&mime_type)
            .map_err(|e| ServerFnError::new(format!("Invalid MIME type: {}", e)))?;

        form = form.part("file", part);
    }

    // Send request
    let client = reqwest::Client::new();
    let response = client
        .post(&upload_endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Upload failed: {}", e)))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Upload failed: {}", body)));
    }

    let upload_response: crate::models::post::UploadPostResponse = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse upload response: {}", e)))?;

    Ok(upload_response)
}
