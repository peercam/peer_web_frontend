use async_graphql::{Context, ID, Object};
use uuid::Uuid;

use crate::CurrentUser;
use crate::state::SharedState;
use crate::types::comment::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct CommentQuery;

#[Object]
impl CommentQuery {
    /// Fetch top-level comments for a post.
    async fn list_comments(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<String>,
        #[graphql(name = "commentOffset")] comment_offset: Option<i32>,
        #[graphql(name = "commentLimit")] comment_limit: Option<i32>,
    ) -> CommentListResponse {
        let viewer_id = ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0);
        let state = ctx.data_unchecked::<SharedState>().read().await;

        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return CommentListResponse {
                    meta: DefaultResponse::error("30209", "Invalid post UUID"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        if !state.posts.iter().any(|p| p.id == post_uuid) {
            return CommentListResponse {
                meta: DefaultResponse::error("30209", "Post not found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let offset = comment_offset.unwrap_or(0).max(0) as usize;
        let limit = comment_limit.unwrap_or(10).clamp(1, 20) as usize;

        let top_level: Vec<_> = state
            .comments
            .iter()
            .filter(|c| {
                c.post_id == post_uuid
                    && c.parent_id.is_none()
                    && c.visibility_status == "VISIBLE"
                    && state.get_visibility(&c.id) != "ILLEGAL"
                    && !state.deleted_users.contains(&c.author_id)
            })
            .collect();

        let total = top_level.len();

        if total == 0 {
            return CommentListResponse {
                meta: DefaultResponse::success("21601", "No comments found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let end = (offset + limit).min(total);
        let page = if offset < total {
            &top_level[offset..end]
        } else {
            &[]
        };

        let comments: Vec<Comment> = page
            .iter()
            .map(|r| state.comment_record_to_graphql(r, viewer_id))
            .collect();

        CommentListResponse {
            meta: DefaultResponse::success("11601", "Comments retrieved"),
            counter: total as i32,
            affected_rows: Some(comments),
        }
    }

    /// Fetch replies to a top-level comment.
    async fn list_child_comments(
        &self,
        ctx: &Context<'_>,
        parent: ID,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> CommentListResponse {
        let viewer_id = ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0);
        let state = ctx.data_unchecked::<SharedState>().read().await;

        let parent_uuid = match parent.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return CommentListResponse {
                    meta: DefaultResponse::error("30209", "Invalid parent UUID"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(10).clamp(1, 20) as usize;

        let children: Vec<_> = state
            .comments
            .iter()
            .filter(|c| {
                c.parent_id == Some(parent_uuid)
                    && c.visibility_status == "VISIBLE"
                    && !state.deleted_users.contains(&c.author_id)
                    && state.get_visibility(&c.id) != "ILLEGAL"
            })
            .collect();

        let total = children.len();

        if total == 0 {
            return CommentListResponse {
                meta: DefaultResponse::success("21606", "No child comments found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let end = (off + lim).min(total);
        let page = if off < total {
            &children[off..end]
        } else {
            &[]
        };

        let comments: Vec<Comment> = page
            .iter()
            .map(|r| state.comment_record_to_graphql(r, viewer_id))
            .collect();

        CommentListResponse {
            meta: DefaultResponse::success("11607", "Child comments retrieved"),
            counter: total as i32,
            affected_rows: Some(comments),
        }
    }
}
