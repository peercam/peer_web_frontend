//! Content preview dispatcher — delegates to post/comment/user preview.

use leptos::prelude::*;

use crate::components::admin::comment_preview::CommentPreview;
use crate::components::admin::post_preview::PostPreview;
use crate::components::admin::user_preview::UserPreview;
use crate::models::moderation::ModerationItem;

/// Dispatches to the appropriate content preview based on target type.
#[component]
pub fn ContentPreview(item: ModerationItem) -> impl IntoView {
    match item.targettype.as_str() {
        "post" => {
            if let Some(post) = item.targetcontent.post {
                view! { <PostPreview post=post/> }.into_any()
            } else {
                view! { <p>"Post data unavailable"</p> }.into_any()
            }
        }
        "comment" => {
            if let Some(comment) = item.targetcontent.comment {
                view! { <CommentPreview comment=comment/> }.into_any()
            } else {
                view! { <p>"Comment data unavailable"</p> }.into_any()
            }
        }
        "user" => {
            if let Some(user) = item.targetcontent.user {
                view! { <UserPreview user=user/> }.into_any()
            } else {
                view! { <p>"User data unavailable"</p> }.into_any()
            }
        }
        _ => view! { <p>"Unknown content type"</p> }.into_any(),
    }
}
