//! Shop profile header with gradient background.

use leptos::prelude::*;

use crate::models::profile::Profile;

/// Shop-specific gradient profile header with avatar, info, and follow/info buttons.
#[component]
pub fn ShopProfileHeader(
    profile: Profile,
    #[prop(into)] on_info_click: Callback<()>,
    #[prop(into)] on_follow_click: Callback<()>,
    is_following: RwSignal<bool>,
) -> impl IntoView {
    let img_src = profile
        .img
        .clone()
        .unwrap_or_else(|| "/svg/noname.svg".to_string());
    let username = profile.username.clone();
    let slug = format!("#{}", profile.slug);
    let bio = profile.biography.clone().unwrap_or_default();

    let follow_text = move || {
        if is_following.get() {
            "Following"
        } else {
            "Follow"
        }
    };

    view! {
        <div class="shop-profile-header">
            <div class="shop-header-gradient">
                <div class="shop-header-content">
                    <div class="shop-avatar-wrapper">
                        <img
                            src=img_src
                            alt=format!("{}'s avatar", username)
                            class="shop-avatar"
                        />
                    </div>
                    <div class="shop-info">
                        <h2 class="shop-username">{username}</h2>
                        <span class="shop-slug txt-color-gray">{slug}</span>
                        <p class="shop-bio">{bio}</p>
                    </div>
                </div>
                <div class="shop-header-actions">
                    <button
                        class="btn-follow"
                        class:btn-following=move || is_following.get()
                        on:click=move |_| on_follow_click.run(())
                    >
                        {follow_text}
                    </button>
                    <button
                        class="btn-info"
                        on:click=move |_| on_info_click.run(())
                        aria-label="Shop information"
                    >
                        <i class="peer-icon peer-icon-info"></i>
                    </button>
                </div>
            </div>
        </div>
    }
}
