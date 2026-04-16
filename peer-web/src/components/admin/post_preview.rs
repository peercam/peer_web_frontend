//! Post preview in the expanded ticket detail.

use leptos::prelude::*;

use crate::models::moderation::ModerationPost;

/// Renders post detail: author, title, text, hashtags, media, and "See full post" link.
#[component]
pub fn PostPreview(post: ModerationPost) -> impl IntoView {
    let user = post.user.clone();
    let user_img = user.as_ref().and_then(|u| u.img.clone());
    let username = user
        .as_ref()
        .and_then(|u| u.username.clone())
        .unwrap_or_default();
    let slug = user.as_ref().and_then(|u| u.slug.clone()).unwrap_or_default();
    let slug_display = format!("@{}", slug);
    let post_id = post.id.clone().unwrap_or_default();
    let title = post.title.clone().unwrap_or_default();
    let description = post.mediadescription.clone().unwrap_or_default();
    let tags = post.tags.clone().unwrap_or_default();
    let created = post.createdat.clone().unwrap_or_default();
    let content_type = post.contenttype.clone().unwrap_or_default();
    let media_url = post.media.clone();
    let cover_url = post.cover.clone();
    let post_link = format!("/post/{}", post_id);

    let type_class = format!(
        "post_detail post_type_{}",
        content_type.to_lowercase()
    );

    let has_description = !description.is_empty();
    let has_tags = !tags.is_empty();
    let tag_list: Vec<String> = if has_tags {
        tags.split(',').map(|t| t.trim().to_string()).collect()
    } else {
        vec![]
    };

    let media_view = match content_type.as_str() {
        "IMAGE" => {
            let src = cover_url.or(media_url).unwrap_or_default();
            view! {
                <div class="post_media">
                    <img src=src alt="Post media"/>
                </div>
            }.into_any()
        }
        "VIDEO" => {
            let src = media_url.unwrap_or_default();
            view! {
                <div class="post_media">
                    <video controls>
                        <source src=src type="video/mp4"/>
                    </video>
                </div>
            }.into_any()
        }
        "AUDIO" => {
            let src = media_url.unwrap_or_default();
            view! {
                <div class="post_media post_type_audio">
                    {cover_url.map(|c| view! {
                        <div class="audio_cover"><img src=c alt="Audio cover"/></div>
                    })}
                    <audio controls src=src></audio>
                </div>
            }.into_any()
        }
        _ => view! { <div></div> }.into_any(),
    };

    view! {
        <div class="content_type_post">
            // Author + full post link
            <div class="profile_post">
                <div class="profile">
                    <div class="profile_image">
                        {user_img.map(|img| view! { <img src=img alt="Author"/> })}
                    </div>
                    <div class="profile_detail">
                        <span class="bold">{username}</span>
                        <span>{slug_display}</span>
                    </div>
                </div>
                <div class="fullpost_link">
                    <a href=post_link class="button btn-blue">"See full post"</a>
                </div>
            </div>

            // Post content
            <div class=type_class>
                {media_view}

                <div class="post_info">
                    <div class="post_title">
                        <h2 class="xl_font_size">{title}</h2>
                        <span class="timeagao">{created}</span>
                    </div>
                    {has_description.then(|| view! {
                        <p class="post_description">{description.clone()}</p>
                    })}
                    {has_tags.then(|| {
                        view! {
                            <div class="hashtags">
                                {tag_list.into_iter().map(|tag| view! {
                                    <span>{"#"}{tag}</span>
                                }).collect_view()}
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}
