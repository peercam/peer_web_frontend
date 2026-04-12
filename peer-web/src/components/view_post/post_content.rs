//! Post content component.
//!
//! Displays post title, description, tags, and timestamp.

use leptos::prelude::*;

use crate::models::post::Post;

/// Format a timestamp into a relative time string.
/// Simplified implementation that extracts date from ISO timestamp.
pub fn format_time_ago(timestamp: &str) -> String {
    // Simple implementation - just show the date portion
    // In production, could use chrono or a wasm-friendly time library
    if timestamp.len() >= 10 {
        timestamp[..10].to_string()
    } else {
        timestamp.to_string()
    }
}

/// Post content section: title, description, tags, timestamp.
#[component]
pub fn PostContent(post: Post) -> impl IntoView {
    let tags = post.tags.clone().unwrap_or_default();
    let description = post.mediadescription.clone().unwrap_or_default();
    let time_ago = format_time_ago(&post.createdat);
    let has_description = !description.is_empty();
    let has_tags = !tags.is_empty();
    let tags_for_render = tags.clone();

    view! {
        <div class="post-content">
            <div class="post-meta">
                <span class="post-timestamp txt-color-gray md_font_size">
                    <i class="peer-icon peer-icon-clock"/>
                    {time_ago}
                </span>
            </div>

            <h1 class="post-title xxl_font_size bold">
                {post.title}
            </h1>

            {has_description.then(|| view! {
                <p class="post-description md_font_size">
                    {description}
                </p>
            })}

            {has_tags.then(|| view! {
                <div class="post-tags">
                    {tags_for_render.iter().map(|tag| {
                        let tag_url = format!("/dashboard?tag={}", tag);
                        let tag_display = tag.clone();
                        view! {
                            <a href=tag_url class="tag btn-white md_font_size">
                                {"#"}{tag_display}
                            </a>
                        }
                    }).collect_view()}
                </div>
            })}
        </div>
    }
}
