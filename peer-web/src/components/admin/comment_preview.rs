//! Comment preview in the expanded ticket detail.

use leptos::prelude::*;

use crate::models::moderation::ModerationComment;

/// Renders comment detail: commenter info, comment text, parent post link.
#[component]
pub fn CommentPreview(comment: ModerationComment) -> impl IntoView {
    let user = comment.user.clone();
    let user_img = user.as_ref().and_then(|u| u.img.clone());
    let username = user
        .as_ref()
        .and_then(|u| u.username.clone())
        .unwrap_or_default();
    let slug = user
        .as_ref()
        .and_then(|u| u.slug.clone())
        .unwrap_or_default();
    let content = comment.content.clone().unwrap_or_default();
    let created = comment.createdat.clone().unwrap_or_default();
    let post_id = comment.postid.clone().unwrap_or_default();
    let has_post = !post_id.is_empty();
    let post_link = format!("/post/{}", post_id);

    view! {
        <div class="comment_box">
            <h2 class="xl_font_size">
                <span class="peer-icon peer-icon-comment"></span>
                " Reported comment"
            </h2>

            <div class="comment_item">
                <div class="commenter-pic">
                    {user_img.map(|img| view! { <img src=img alt="Commenter"/> })}
                </div>
                <div class="comment_body">
                    <div class="commenter_info">
                        <span class="bold">{username}</span>
                        <span class="timeagao">{created}</span>
                    </div>
                    <p>{content}</p>
                </div>
            </div>

            // Link to parent post
            {has_post.then(|| view! {
                <div class="comment_post_detail">
                    <a href=post_link class="button btn-transparent">"View parent post"</a>
                </div>
            })}
        </div>
    }
}
