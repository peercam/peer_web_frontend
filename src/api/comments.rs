//! Comments API server functions.
//!
//! Provides server functions for fetching, creating, and liking comments.

use leptos::prelude::*;
use serde::Serialize;

use crate::models::comment::{Comment, CommentListResponse};
use crate::models::post::Post;

/// Variables for the listComments query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct ListCommentsVars {
    postid: String,
    comment_offset: i32,
    comment_limit: i32,
}

/// Variables for the listChildComments query.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ListChildCommentsVars {
    parent: String,
    offset: i32,
    limit: i32,
}

/// Variables for the createComment mutation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CreateCommentVars {
    postid: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parentid: Option<String>,
}

/// Variables for the likeComment/unlikeComment mutations.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct LikeCommentVars {
    commentid: String,
}

/// Variables for guest post query.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct GuestPostVars {
    postid: String,
}

/// Variables for authenticated post query.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct GetPostVars {
    postid: String,
}

/// Fetch a single post for guest viewing (no auth required).
#[server(GuestGetPost, "/api")]
pub async fn guest_get_post(post_id: String) -> Result<Post, ServerFnError> {
    use crate::api::graphql::{GUEST_POST_QUERY, GuestPostData, query};

    let vars = GuestPostVars { postid: post_id };
    let data: GuestPostData = query(GUEST_POST_QUERY, vars, None).await?;

    data.guest_list_post
        .affected_rows
        .into_iter()
        .next()
        .ok_or_else(|| ServerFnError::new("Post not found"))
}

/// Fetch a single post for authenticated viewing.
#[server(GetPost, "/api")]
pub async fn get_post(post_id: String) -> Result<Post, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{GET_POST_QUERY, GetPostData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = GetPostVars { postid: post_id };
    let data: GetPostData = query(GET_POST_QUERY, vars, Some(&token)).await?;

    data.list_posts
        .affected_rows
        .into_iter()
        .next()
        .ok_or_else(|| ServerFnError::new("Post not found"))
}

/// Fetch top-level comments for a post.
#[server(ListComments, "/api")]
pub async fn list_comments(
    post_id: String,
    offset: i32,
    limit: i32,
) -> Result<CommentListResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{LIST_COMMENTS_QUERY, ListCommentsData, query};

    let token = get_access_token_from_cookies().await.ok();

    let vars = ListCommentsVars {
        postid: post_id,
        comment_offset: offset,
        comment_limit: limit,
    };

    let data: ListCommentsData = query(LIST_COMMENTS_QUERY, vars, token.as_deref()).await?;
    Ok(data.list_comments)
}

/// Fetch child comments (replies) for a parent comment.
#[server(ListChildComments, "/api")]
pub async fn list_child_comments(
    parent_id: String,
    offset: i32,
    limit: i32,
) -> Result<CommentListResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{LIST_CHILD_COMMENTS_QUERY, ListChildCommentsData, query};

    let token = get_access_token_from_cookies().await.ok();

    let vars = ListChildCommentsVars {
        parent: parent_id,
        offset,
        limit,
    };

    let data: ListChildCommentsData =
        query(LIST_CHILD_COMMENTS_QUERY, vars, token.as_deref()).await?;
    Ok(data.list_child_comments)
}

/// Create a new comment or reply.
#[server(CreateComment, "/api")]
pub async fn create_comment(
    post_id: String,
    content: String,
    parent_id: Option<String>,
) -> Result<Comment, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{CREATE_COMMENT_MUTATION, CreateCommentData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = CreateCommentVars {
        postid: post_id,
        content,
        parentid: parent_id,
    };

    let data: CreateCommentData = mutate(CREATE_COMMENT_MUTATION, vars, Some(&token)).await?;

    data.create_comment
        .comment()
        .cloned()
        .ok_or_else(|| ServerFnError::new("Failed to create comment"))
}

/// Like a comment.
#[server(LikeComment, "/api")]
pub async fn like_comment(comment_id: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{LIKE_COMMENT_MUTATION, LikeCommentData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = LikeCommentVars {
        commentid: comment_id,
    };

    let _data: LikeCommentData = mutate(LIKE_COMMENT_MUTATION, vars, Some(&token)).await?;
    Ok(())
}

/// Unlike a comment.
#[server(UnlikeComment, "/api")]
pub async fn unlike_comment(comment_id: String) -> Result<(), ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{UNLIKE_COMMENT_MUTATION, UnlikeCommentData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = LikeCommentVars {
        commentid: comment_id,
    };

    let _data: UnlikeCommentData = mutate(UNLIKE_COMMENT_MUTATION, vars, Some(&token)).await?;
    Ok(())
}
