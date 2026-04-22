//! Post header component.
//!
//! Displays author information and follow button.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::profile::toggle_follow;
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

    let user_id_for_follow = StoredValue::new(user.id.clone());

    let on_follow = move |_| {
        if is_guest {
            return;
        }

        let currently_followed = is_followed.get();
        is_followed.set(!currently_followed); // Optimistic update

        let id = user_id_for_follow.get_value();
        spawn_local(async move {
            if toggle_follow(id).await.is_err() {
                is_followed.set(currently_followed); // Revert on failure
            }
        });
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
