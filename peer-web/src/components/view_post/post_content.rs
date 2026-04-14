//! Post content component.
//!
//! Displays post title, description, tags, and timestamp.

use leptos::prelude::*;

use crate::models::post::Post;

/// Format a timestamp into a relative time string.
///
/// Parses an ISO 8601 timestamp and returns a human-readable relative time
/// such as "just now", "5 min ago", "2 h ago", "3 d ago", or the date for older posts.
pub fn format_time_ago(timestamp: &str) -> String {
    use chrono::{NaiveDateTime, Utc};

    // Parse various ISO 8601 formats
    let dt = timestamp
        .replace('T', " ")
        .replace('Z', "");

    // Try parsing with optional fractional seconds
    let parsed = NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S"));

    let Ok(parsed_dt) = parsed else {
        // Fallback to just showing the date portion
        return timestamp.get(..10).unwrap_or(timestamp).to_string();
    };

    let now = Utc::now().naive_utc();
    let diff = now.signed_duration_since(parsed_dt);

    let seconds = diff.num_seconds();
    if seconds < 0 {
        // Future timestamp - just show date
        return timestamp.get(..10).unwrap_or(timestamp).to_string();
    }

    if seconds < 60 {
        return "just now".to_string();
    }

    let minutes = diff.num_minutes();
    if minutes < 60 {
        return format!("{} min ago", minutes);
    }

    let hours = diff.num_hours();
    if hours < 24 {
        return format!("{} h ago", hours);
    }

    let days = diff.num_days();
    if days < 7 {
        return format!("{} d ago", days);
    }

    let weeks = days / 7;
    if weeks < 5 {
        return format!("{} w ago", weeks);
    }

    // For older posts, show the date
    timestamp.get(..10).unwrap_or(timestamp).to_string()
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
