//! User preview in the expanded ticket detail.

use leptos::prelude::*;

use crate::models::moderation::BasicUserInfo;

/// Renders user detail: profile image, bio, stats, "View profile" link.
#[component]
pub fn UserPreview(user: BasicUserInfo) -> impl IntoView {
    let username = user.username.clone().unwrap_or_default();
    let slug = user.slug.clone().unwrap_or_default();
    let slug_display = format!("@{}", slug);
    let biography = user.biography.clone().unwrap_or_default();
    let has_bio = !biography.is_empty();
    let img = user.img.clone();
    let profile_link = format!("/profile/{}", slug);

    view! {
        <div class="content_type_profile">
            <div class="profile">
                <div class="profile_image">
                    {img.map(|url| view! { <img src=url alt="Profile"/> })}
                </div>
                <div class="profile_detail">
                    <span class="bold xl_font_size">{username}</span>
                    <span>{slug_display}</span>
                    {has_bio.then(|| view! {
                        <p class="biography">{biography.clone()}</p>
                    })}
                </div>
            </div>
            <div class="profile_link">
                <a href=profile_link class="button btn-blue">"View profile"</a>
            </div>
        </div>
    }
}
