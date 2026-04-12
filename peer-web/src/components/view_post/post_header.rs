//! Post header component.
//!
//! Displays author information and follow button.

use leptos::prelude::*;

use crate::models::post::PostUser;

/// Post author header with avatar, username, and follow button.
#[component]
pub fn PostHeader(user: PostUser, is_guest: bool) -> impl IntoView {
    let is_followed = RwSignal::new(user.isfollowed);
    let profile_url = format!("/profile/{}", user.slug);
    let img_src = user
        .img
        .clone()
        .unwrap_or_else(|| "/svg/noname.svg".to_string());

    let on_follow = move |_| {
        if is_guest {
            return;
        }

        let currently_followed = is_followed.get();
        is_followed.set(!currently_followed);

        // TODO: Call follow/unfollow API
        // spawn_local(async move {
        //     if currently_followed {
        //         let _ = unfollow_user(user_id.clone()).await;
        //     } else {
        //         let _ = follow_user(user_id.clone()).await;
        //     }
        // });
    };

    view! {
        <div class="post-header">
            <div class="user-info">
                <a href=profile_url.clone() class="user-avatar">
                    <img
                        src=img_src
                        alt=format!("{}'s profile picture", user.username)
                        class="profile-picture"
                    />
                </a>
                <div class="user-details">
                    <a href=profile_url class="username xl_font_size bold">
                        {user.username}
                    </a>
                    <span class="user-slug txt-color-gray md_font_size">
                        {"#"}{user.slug}
                    </span>
                </div>
            </div>

            // Follow button (authenticated only)
            <Show when=move || !is_guest>
                <button
                    class="follow-btn btn-white md_font_size"
                    class:following=move || is_followed.get()
                    on:click=on_follow
                >
                    {move || if is_followed.get() { "Following" } else { "Follow" }}
                </button>
            </Show>
        </div>
    }
}
