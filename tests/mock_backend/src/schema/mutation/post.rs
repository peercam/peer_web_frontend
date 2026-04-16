use async_graphql::{Context, ID, Object};
use chrono::Utc;
use uuid::Uuid;

use crate::schema::mutation::auth::get_current_user;
use crate::state::{
    GemRecord, PostRecord, SharedState, DISLIKE_GEM_RETURN, LIKE_GEM_RETURN, VIEW_GEM_RETURN,
};
use crate::types::post::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct PostMutation;

#[Object]
impl PostMutation {
    /// Create a new post.
    async fn create_post(
        &self,
        ctx: &Context<'_>,
        _action: PostType,
        input: PostInput,
    ) -> CreatePostResponse {
        let viewer_id = match get_current_user(ctx) {
            Some(uid) => uid,
            None => {
                return CreatePostResponse {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    affected_rows: None,
                };
            }
        };

        // Validate title (1–63 chars)
        if input.title.is_empty() || input.title.len() > 63 {
            return CreatePostResponse {
                meta: DefaultResponse::error("30210", "Invalid title length (1-63 characters)"),
                affected_rows: None,
            };
        }

        // Validate description (≤500 chars)
        if let Some(ref desc) = input.mediadescription
            && desc.len() > 500
        {
            return CreatePostResponse {
                meta: DefaultResponse::error("30263", "Invalid description length (max 500)"),
                affected_rows: None,
            };
        }

        // Validate tags
        if let Some(ref tags) = input.tags {
            if tags.len() > 10 {
                return CreatePostResponse {
                    meta: DefaultResponse::error("30262", "Too many tags (max 10)"),
                    affected_rows: None,
                };
            }
            for tag in tags {
                if tag.len() < 2
                    || tag.len() > 53
                    || !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    return CreatePostResponse {
                        meta: DefaultResponse::error(
                            "30262",
                            &format!("Invalid tag format: {}", tag),
                        ),
                        affected_rows: None,
                    };
                }
            }
        }

        // Validate media limits per content type
        let max_media = match input.contenttype {
            ContentType::Image => 5,
            ContentType::Video => 2,
            ContentType::Audio | ContentType::Text => 1,
        };
        if let Some(ref media) = input.media
            && media.len() > max_media
        {
            return CreatePostResponse {
                meta: DefaultResponse::error("30267", "Too many media items"),
                affected_rows: None,
            };
        }
        if let Some(ref cover) = input.cover
            && cover.len() > 1
        {
            return CreatePostResponse {
                meta: DefaultResponse::error("30268", "Too many cover items (max 1)"),
                affected_rows: None,
            };
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Token deduction for post action
        if let Err(_code) = state_write.try_deduct_for_action(viewer_id, "post") {
            return CreatePostResponse {
                meta: DefaultResponse::error("51301", "Insufficient balance to create post"),
                affected_rows: None,
            };
        }

        let post_id = Uuid::new_v4();
        let contenttype_str = match input.contenttype {
            ContentType::Image => "image",
            ContentType::Audio => "audio",
            ContentType::Video => "video",
            ContentType::Text => "text",
        };

        // Normalize and register tags
        let tags: Vec<String> = input
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(|t| t.to_lowercase())
            .collect();
        for tag in &tags {
            state_write.tags.insert(tag.clone());
        }

        let now = Utc::now().to_rfc3339();
        let record = PostRecord {
            id: post_id,
            author_id: viewer_id,
            contenttype: contenttype_str.to_string(),
            title: input.title.clone(),
            media: input.media.map(|m| m.join(",")).unwrap_or_default(),
            cover: input.cover.map(|c| c.join(",")).unwrap_or_default(),
            mediadescription: input.mediadescription.unwrap_or_default(),
            created_at: now,
            tags,
            visibility_status: "VISIBLE".into(),
            uploaded_files: input.uploaded_files,
        };

        state_write.posts.insert(0, record);

        CreatePostResponse {
            meta: DefaultResponse::success("11508", "Post created successfully"),
            affected_rows: Some(CreatedPost {
                id: post_id.to_string().into(),
                contenttype: contenttype_str.to_string(),
                title: input.title,
            }),
        }
    }

    /// Perform an interaction on a post.
    async fn resolve_post_action(
        &self,
        ctx: &Context<'_>,
        action: PostActionType,
        postid: ID,
    ) -> DefaultResponse {
        let viewer_id = match get_current_user(ctx) {
            Some(uid) => uid,
            None => return DefaultResponse::error("60501", "Authentication required"),
        };

        let post_uuid = match Uuid::parse_str(postid.as_str()) {
            Ok(id) => id,
            Err(_) => return DefaultResponse::error("30209", "Invalid post UUID"),
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let post = match state_write.posts.iter().find(|p| p.id == post_uuid) {
            Some(p) => p.clone(),
            None => return DefaultResponse::error("31510", "Post not found"),
        };

        match action {
            PostActionType::Like => {
                if post.author_id == viewer_id {
                    return DefaultResponse::error("31506", "Cannot like own post");
                }
                // Token deduction for like
                if let Err(_code) = state_write.try_deduct_for_action(viewer_id, "like") {
                    return DefaultResponse::error("51301", "Insufficient balance for like");
                }
                if !state_write.post_likes.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31501", "Already liked");
                }
                state_write.post_dislikes.remove(&(viewer_id, post_uuid));
                // Gem accumulation for post author
                let now = Utc::now().to_rfc3339();
                state_write.gems.push(GemRecord {
                    user_id: post.author_id,
                    post_id: post_uuid,
                    from_user_id: viewer_id,
                    gems: LIKE_GEM_RETURN,
                    action: "like".to_string(),
                    created_at: now,
                });
                DefaultResponse::success("11503", "Post liked")
            }
            PostActionType::Unlike => {
                state_write.post_likes.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11503", "Post unliked")
            }
            PostActionType::Dislike => {
                if post.author_id == viewer_id {
                    return DefaultResponse::error("31507", "Cannot dislike own post");
                }
                // Token deduction for dislike
                if let Err(_code) = state_write.try_deduct_for_action(viewer_id, "dislike") {
                    return DefaultResponse::error("51301", "Insufficient balance for dislike");
                }
                if !state_write.post_dislikes.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31502", "Already disliked");
                }
                state_write.post_likes.remove(&(viewer_id, post_uuid));
                // Negative gem for post author
                let now = Utc::now().to_rfc3339();
                state_write.gems.push(GemRecord {
                    user_id: post.author_id,
                    post_id: post_uuid,
                    from_user_id: viewer_id,
                    gems: DISLIKE_GEM_RETURN,
                    action: "dislike".to_string(),
                    created_at: now,
                });
                DefaultResponse::success("11504", "Post disliked")
            }
            PostActionType::Undislike => {
                state_write.post_dislikes.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11504", "Post undisliked")
            }
            PostActionType::View => {
                if !state_write.post_views.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31505", "Already viewed");
                }
                // Gem accumulation for views (on other users' posts)
                if post.author_id != viewer_id {
                    let now = Utc::now().to_rfc3339();
                    state_write.gems.push(GemRecord {
                        user_id: post.author_id,
                        post_id: post_uuid,
                        from_user_id: viewer_id,
                        gems: VIEW_GEM_RETURN,
                        action: "view".to_string(),
                        created_at: now,
                    });
                }
                DefaultResponse::success("11506", "Post viewed")
            }
            PostActionType::Save => {
                if state_write.post_saves.contains(&(viewer_id, post_uuid)) {
                    state_write.post_saves.remove(&(viewer_id, post_uuid));
                    DefaultResponse::success("11511", "Post unsaved")
                } else {
                    state_write.post_saves.insert((viewer_id, post_uuid));
                    DefaultResponse::success("11512", "Post saved")
                }
            }
            PostActionType::Unsave => {
                state_write.post_saves.remove(&(viewer_id, post_uuid));
                DefaultResponse::success("11511", "Post unsaved")
            }
            PostActionType::Report => {
                if post.author_id == viewer_id {
                    return DefaultResponse::error("31508", "Cannot report own post");
                }
                if !state_write.post_reports.insert((viewer_id, post_uuid)) {
                    return DefaultResponse::error("31503", "Already reported");
                }
                DefaultResponse::success("11505", "Post reported")
            }
            PostActionType::Share => {
                state_write.post_shares.insert((viewer_id, post_uuid));
                DefaultResponse::success("11507", "Post shared")
            }
        }
    }
}
