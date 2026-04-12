//! Profile action buttons.
//!
//! Components for profile action buttons (Edit, Follow, Ads, More actions).

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::profile::{report_user, toggle_block, toggle_follow};
use crate::components::toast::{use_toast, ToastType};
use crate::models::profile::Profile;

/// Actions for viewing your own profile.
///
/// Shows Edit Profile and Ads dropdown buttons.
#[component]
pub fn OwnProfileActions(
    /// Callback when boost post mode is activated.
    #[prop(optional)]
    on_boost_posts: Option<Callback<()>>,
) -> impl IntoView {
    let ads_expanded = RwSignal::new(false);

    let toggle_ads = move |_| {
        ads_expanded.update(|v| *v = !*v);
    };

    // Close dropdown when clicking outside
    let close_dropdown = move |_| {
        ads_expanded.set(false);
    };

    view! {
        <div class="profile-edit-box">
            <a href="/settings" class="button btn-white edit-profile">
                <i class="peer-icon peer-icon-edit-pencil"></i>
                <span>"Edit"</span>
            </a>

            <div
                class="ads-container-wrap"
                on:focusout=close_dropdown
            >
                <button
                    class="button ads-button"
                    aria-expanded=move || ads_expanded.get()
                    aria-haspopup="menu"
                    on:click=toggle_ads
                >
                    <span>"Ads"</span>
                    <i class="peer-icon peer-icon-chevron-down"></i>
                </button>

                <div
                    class="ads-dropdown"
                    class:open=move || ads_expanded.get()
                    role="menu"
                >
                    <button
                        class="button btn-blue boost-posts"
                        role="menuitem"
                        on:click=move |_| {
                            ads_expanded.set(false);
                            if let Some(callback) = on_boost_posts {
                                callback.run(());
                            }
                        }
                    >
                        <i class="peer-icon peer-icon-megaphone"></i>
                        <span>"Boost post"</span>
                    </button>
                    <a
                        href="/my-ads"
                        class="button btn-white my-ads"
                        role="menuitem"
                    >
                        <i class="peer-icon peer-icon-ad"></i>
                        <span>"My Ads"</span>
                    </a>
                </div>
            </div>
        </div>
    }
}

/// Actions when viewing another user's profile.
///
/// Shows Follow button and More actions dropdown (Report, Block).
#[component]
pub fn ViewProfileActions(
    /// The profile being viewed.
    profile: Profile,
    /// Callback when follow state changes.
    #[prop(optional)]
    on_follow_change: Option<Callback<bool>>,
) -> impl IntoView {
    let toast = use_toast();
    let user_id = profile.id.clone();
    let is_following = RwSignal::new(profile.i_follow_this_user);
    let they_follow_me = profile.this_user_follows_me;
    let is_reported = RwSignal::new(profile.isreported);
    let more_expanded = RwSignal::new(false);
    let is_loading = RwSignal::new(false);

    let follow_text = move || {
        if is_loading.get() {
            "..."
        } else if is_following.get() {
            "Following"
        } else if they_follow_me {
            "Follow Back"
        } else {
            "Follow"
        }
    };

    let handle_follow = {
        let user_id = user_id.clone();
        let toast = toast;
        move |_| {
            let user_id = user_id.clone();
            is_loading.set(true);

            spawn_local(async move {
                match toggle_follow(user_id).await {
                    Ok(response) => {
                        is_following.set(response.isfollowing);
                        if let Some(callback) = on_follow_change {
                            callback.run(response.isfollowing);
                        }
                    }
                    Err(e) => {
                        toast.show(format!("Failed to update follow status: {}", e), ToastType::Error);
                    }
                }
                is_loading.set(false);
            });
        }
    };

    let handle_report = {
        let user_id = user_id.clone();
        let toast = toast;
        move |_| {
            let user_id = user_id.clone();
            more_expanded.set(false);

            spawn_local(async move {
                match report_user(user_id).await {
                    Ok(_) => {
                        is_reported.set(true);
                        toast.show("User reported successfully", ToastType::Success);
                    }
                    Err(e) => {
                        toast.show(format!("Failed to report user: {}", e), ToastType::Error);
                    }
                }
            });
        }
    };

    let handle_block = {
        let user_id = user_id.clone();
        let toast = toast;
        move |_| {
            let user_id = user_id.clone();
            more_expanded.set(false);

            spawn_local(async move {
                match toggle_block(user_id).await {
                    Ok(_) => {
                        toast.show("User blocked", ToastType::Success);
                    }
                    Err(e) => {
                        toast.show(format!("Failed to block user: {}", e), ToastType::Error);
                    }
                }
            });
        }
    };

    let toggle_more = move |_| {
        more_expanded.update(|v| *v = !*v);
    };

    let close_dropdown = move |_| {
        more_expanded.set(false);
    };

    view! {
        <div class="profile-edit-box view-profile-actions">
            <button
                class="button btn-transparent follow-button"
                class:following=move || is_following.get()
                disabled=move || is_loading.get()
                on:click=handle_follow
            >
                {follow_text}
            </button>

            <div
                class="more-actions-wrap"
                on:focusout=close_dropdown
            >
                <button
                    class="button more-actions"
                    aria-expanded=move || more_expanded.get()
                    aria-haspopup="menu"
                    on:click=toggle_more
                >
                    <span>"More"</span>
                    <i class="peer-icon peer-icon-chevron-down"></i>
                </button>

                <div
                    class="more-dropdown"
                    class:open=move || more_expanded.get()
                    role="menu"
                >
                    <button
                        class="report-profile"
                        role="menuitem"
                        disabled=move || is_reported.get()
                        on:click=handle_report
                    >
                        <i class="peer-icon peer-icon-flag"></i>
                        <span>{move || if is_reported.get() { "Reported" } else { "Report profile" }}</span>
                    </button>
                    <button
                        class="block-content"
                        role="menuitem"
                        on:click=handle_block
                    >
                        <i class="peer-icon peer-icon-ban"></i>
                        <span>"Block content"</span>
                    </button>
                </div>
            </div>
        </div>
    }
}
