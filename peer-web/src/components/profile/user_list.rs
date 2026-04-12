//! User list item components.
//!
//! Components for displaying users in lists (followers, following, peers).

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;

use crate::api::profile::toggle_follow;
use crate::models::profile::{BasicUserInfo, ContentVisibilityStatus, ProfileUser};

/// A user item in a follow list.
///
/// Displays avatar, username, slug, and a follow button.
/// Clicking the item navigates to the user's profile.
#[component]
pub fn UserListItem(
    /// The user to display.
    user: ProfileUser,
    /// Whether to show the follow button.
    #[prop(default = true)]
    show_follow_button: bool,
) -> impl IntoView {
    let user_id = user.userid.clone();
    let is_following = RwSignal::new(user.isfollowed);
    let is_loading = RwSignal::new(false);

    let is_hidden = user.visibility_status == ContentVisibilityStatus::Hidden
        || user.is_hidden_for_users;
    let is_illegal = user.visibility_status == ContentVisibilityStatus::Illegal;

    let handle_follow = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();

        let user_id = user_id.clone();
        is_loading.set(true);

        spawn_local(async move {
            if let Ok(response) = toggle_follow(user_id).await {
                is_following.set(response.isfollowing);
            }
            is_loading.set(false);
        });
    };

    let follow_button_text = move || {
        if is_loading.get() {
            "..."
        } else if is_following.get() {
            "Following"
        } else {
            "Follow"
        }
    };

    let display_username = if is_illegal {
        "removed".to_string()
    } else {
        user.username.clone()
    };

    let display_slug = if is_illegal {
        "removed".to_string()
    } else {
        user.display_slug()
    };

    let avatar_url = if is_illegal {
        "/svg/noname.svg".to_string()
    } else {
        user.avatar_url().to_string()
    };

    let profile_url = format!("/profile/{}", user.slug);

    view! {
        <div
            class="user-list-item"
            class:hidden=is_hidden
            class:illegal=is_illegal
        >
            <a href=profile_url class="user-info">
                <div class="user-avatar">
                    <img
                        class="avatar"
                        src=avatar_url
                        alt=display_username.clone()
                        on:error=|ev| {
                            if let Some(target) = ev.target() {
                                if let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                                    img.set_src("/svg/noname.svg");
                                }
                            }
                        }
                    />
                </div>
                <div class="user-details">
                    <span class="username">{display_username}</span>
                    <span class="slug">{display_slug}</span>
                </div>
            </a>

            {show_follow_button.then(|| view! {
                <button
                    class="button btn-transparent follow-btn"
                    class:following=move || is_following.get()
                    disabled=move || is_loading.get() || is_illegal
                    on:click=handle_follow
                >
                    {follow_button_text}
                </button>
            })}
        </div>
    }
}

/// A user item in the friends/peers list.
///
/// Similar to UserListItem but uses BasicUserInfo which has a biography field.
#[component]
pub fn FriendListItem(
    /// The friend/peer to display.
    user: BasicUserInfo,
) -> impl IntoView {
    let is_hidden = user.visibility_status == ContentVisibilityStatus::Hidden
        || user.is_hidden_for_users;
    let is_illegal = user.visibility_status == ContentVisibilityStatus::Illegal;

    let display_username = if is_illegal {
        "removed".to_string()
    } else {
        user.username.clone()
    };

    let display_slug = if is_illegal {
        "removed".to_string()
    } else {
        user.display_slug()
    };

    let avatar_url = if is_illegal {
        "/svg/noname.svg".to_string()
    } else {
        user.avatar_url().to_string()
    };

    let profile_url = format!("/profile/{}", user.slug);

    view! {
        <div
            class="user-list-item friend-item"
            class:hidden=is_hidden
            class:illegal=is_illegal
        >
            <a href=profile_url class="user-info">
                <div class="user-avatar">
                    <img
                        class="avatar"
                        src=avatar_url
                        alt=display_username.clone()
                        on:error=|ev| {
                            if let Some(target) = ev.target() {
                                if let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                                    img.set_src("/svg/noname.svg");
                                }
                            }
                        }
                    />
                </div>
                <div class="user-details">
                    <span class="username">{display_username}</span>
                    <span class="slug">{display_slug}</span>
                    {user.biography.map(|bio| view! {
                        <span class="bio-preview">{bio}</span>
                    })}
                </div>
            </a>
        </div>
    }
}
