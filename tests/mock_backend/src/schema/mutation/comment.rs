use async_graphql::{Context, ID, Object};
use chrono::Utc;
use uuid::Uuid;

use crate::require_auth;
use crate::state::{CommentRecord, SharedState};
use crate::types::comment::*;
use crate::types::registration::DefaultResponse;

/// Daily free comment limit.
const DAILY_FREE_COMMENTS: u32 = 4;

#[derive(Default)]
pub struct CommentMutation;

#[Object]
impl CommentMutation {
    /// Create a new comment or reply on a post.
    async fn create_comment(
        &self,
        ctx: &Context<'_>,
        #[allow(unused_variables)] action: CommentType,
        postid: ID,
        content: String,
        parentid: Option<ID>,
    ) -> CreateCommentResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return CreateCommentResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        if content.is_empty() || content.len() > 200 {
            return CreateCommentResponse {
                meta: DefaultResponse::error("30265", "Content must be 1-200 characters"),
                counter: 0,
                affected_rows: None,
            };
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return CreateCommentResponse {
                    meta: DefaultResponse::error("30209", "Invalid post UUID"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        if !state.posts.iter().any(|p| p.id == post_uuid) {
            return CreateCommentResponse {
                meta: DefaultResponse::error("31602", "Post not found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let parent_uuid = match &parentid {
            Some(pid) => {
                let uuid = match pid.to_string().parse::<Uuid>() {
                    Ok(u) => u,
                    Err(_) => {
                        return CreateCommentResponse {
                            meta: DefaultResponse::error("31603", "Invalid parent comment UUID"),
                            counter: 0,
                            affected_rows: None,
                        };
                    }
                };
                let parent = match state.comments.iter().find(|c| c.id == uuid) {
                    Some(c) => c,
                    None => {
                        return CreateCommentResponse {
                            meta: DefaultResponse::error("31603", "Parent comment not found"),
                            counter: 0,
                            affected_rows: None,
                        };
                    }
                };
                if parent.parent_id.is_some() {
                    return CreateCommentResponse {
                        meta: DefaultResponse::error("41604", "Parent must be a top-level comment"),
                        counter: 0,
                        affected_rows: None,
                    };
                }
                if parent.post_id != post_uuid {
                    return CreateCommentResponse {
                        meta: DefaultResponse::error(
                            "31603",
                            "Parent comment belongs to different post",
                        ),
                        counter: 0,
                        affected_rows: None,
                    };
                }
                Some(uuid)
            }
            None => None,
        };

        // Daily free action tracking
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let key = (user_id, today);
        let count = state.daily_comment_count.get(&key).copied().unwrap_or(0);
        let is_free = count < DAILY_FREE_COMMENTS;

        // TODO: Phase 5 — deduct tokens if !is_free and balance insufficient → return 51301

        let comment_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        let record = CommentRecord {
            id: comment_id,
            author_id: user_id,
            post_id: post_uuid,
            parent_id: parent_uuid,
            content,
            created_at: now,
            visibility_status: "VISIBLE".into(),
        };

        state.comments.push(record);
        *state.daily_comment_count.entry(key).or_insert(0) += 1;

        let comment =
            state.comment_record_to_graphql(state.comments.last().unwrap(), Some(user_id));

        let code = if is_free { "11608" } else { "11605" };
        let message = if is_free {
            "Comment created (free daily action)"
        } else {
            "Comment created"
        };

        CreateCommentResponse {
            meta: DefaultResponse::success(code, message),
            counter: 1,
            affected_rows: Some(vec![comment]),
        }
    }

    /// Like a comment.
    async fn like_comment(&self, ctx: &Context<'_>, commentid: ID) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let comment_uuid = match commentid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return DefaultResponse::error("30201", "Invalid comment UUID"),
        };

        let comment = match state.comments.iter().find(|c| c.id == comment_uuid) {
            Some(c) => c,
            None => return DefaultResponse::error("31601", "Comment not found"),
        };

        if comment.author_id == user_id {
            return DefaultResponse::error("31606", "Cannot like own comment");
        }

        if state.comment_likes.contains(&(user_id, comment_uuid)) {
            return DefaultResponse::error("31604", "Already liked");
        }

        state.comment_likes.insert((user_id, comment_uuid));

        DefaultResponse::success("11603", "Comment liked")
    }

    /// Unlike a comment.
    async fn unlike_comment(&self, ctx: &Context<'_>, commentid: ID) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let comment_uuid = match commentid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return DefaultResponse::error("30201", "Invalid comment UUID"),
        };

        if !state.comments.iter().any(|c| c.id == comment_uuid) {
            return DefaultResponse::error("31601", "Comment not found");
        }

        state.comment_likes.remove(&(user_id, comment_uuid));

        DefaultResponse::success("11603", "Comment unliked")
    }

    /// Report a comment for moderation.
    async fn report_comment(&self, ctx: &Context<'_>, commentid: ID) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let comment_uuid = match commentid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => return DefaultResponse::error("30201", "Invalid comment UUID"),
        };

        let comment = match state.comments.iter().find(|c| c.id == comment_uuid) {
            Some(c) => c,
            None => return DefaultResponse::error("31601", "Comment not found"),
        };

        if comment.author_id == user_id {
            return DefaultResponse::error("31607", "Cannot report own comment");
        }

        if state.comment_reports.contains(&(user_id, comment_uuid)) {
            return DefaultResponse::error("31605", "Already reported");
        }

        state.comment_reports.insert((user_id, comment_uuid));

        DefaultResponse::success("11604", "Comment reported")
    }
}
