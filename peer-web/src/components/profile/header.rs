//! Profile header component.
//!
//! The main profile header displaying avatar, user info, bio, and actions.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;

use crate::api::profile::fetch_biography;
use crate::models::profile::{Profile, RelationsTab};

use super::actions::{OwnProfileActions, ViewProfileActions};
use super::relations_modal::RelationsModal;
use super::stats::ProfileStats;
use super::visibility::{HiddenOverlay, IllegalProfileBadge, ReportedBadge};

/// Profile header with avatar, info, and actions.
#[component]
pub fn ProfileHeader(
    /// The profile data to display.
    profile: Profile,
    /// Whether this is the current user's own profile.
    #[prop(default = false)]
    is_own_profile: bool,
    /// Callback when follow state changes (view profile only).
    #[prop(optional)]
    on_follow_change: Option<Callback<bool>>,
) -> impl IntoView {
    let is_hidden = profile.is_hidden();
    let is_illegal = profile.is_illegal();
    let has_reports = profile.has_active_reports;

    // Signal for controlling hidden content visibility
    let show_hidden = RwSignal::new(false);

    // Signal for controlling relations modal
    let show_relations_modal = RwSignal::new(false);
    let relations_tab = RwSignal::new(RelationsTab::Followers);

    let profile_for_stats = profile.clone();
    let profile_for_actions = profile.clone();
    let profile_for_actions_none = profile.clone();
    let profile_id = profile.id.clone();

    let handle_stat_click = Callback::new(move |tab: RelationsTab| {
        relations_tab.set(tab);
        show_relations_modal.set(true);
    });

    let handle_view_anyway = Callback::new(move |_: ()| {
        show_hidden.set(true);
    });

    let handle_modal_close = Callback::new(move |_: ()| {
        show_relations_modal.set(false);
    });

    view! {
        <div
            class="profile-header"
            class:illegal=is_illegal
            class:hidden=move || is_hidden && !show_hidden.get()
        >
            // Hidden content overlay
            {move || {
                if is_hidden && !show_hidden.get() {
                    Some(view! { <HiddenOverlay on_view=handle_view_anyway/> })
                } else {
                    None
                }
            }}

            <div class="profile-header-content">
                <ProfileAvatar
                    src=profile.img.clone()
                    username=profile.username.clone()
                    is_illegal=is_illegal
                />

                <ProfileInfo
                    profile=profile.clone()
                    is_illegal=is_illegal
                    has_reports=has_reports
                />

                <div class="profile-actions-wrapper">
                    {if is_own_profile {
                        view! { <OwnProfileActions/> }.into_any()
                    } else {
                        match on_follow_change {
                            Some(callback) => view! {
                                <ViewProfileActions
                                    profile=profile_for_actions
                                    on_follow_change=callback
                                />
                            }.into_any(),
                            None => view! {
                                <ViewProfileActions
                                    profile=profile_for_actions_none
                                />
                            }.into_any(),
                        }
                    }}
                </div>
            </div>

            <ProfileStats
                profile=profile_for_stats
                is_own_profile=is_own_profile
                on_stat_click=handle_stat_click
            />

            // Relations modal
            {move || {
                show_relations_modal.get().then(|| {
                    view! {
                        <RelationsModal
                            user_id=profile_id.clone()
                            is_own_profile=is_own_profile
                            default_tab=relations_tab.get()
                            on_close=handle_modal_close
                        />
                    }
                })
            }}
        </div>
    }
}

/// Profile avatar with visibility handling.
#[component]
fn ProfileAvatar(
    src: Option<String>,
    username: String,
    is_illegal: bool,
) -> impl IntoView {
    let default_avatar = "/svg/noname.svg".to_string();

    view! {
        <div class="profile-picture">
            <div class="crop-container">
                <span class="online-status"></span>
                {if is_illegal {
                    view! {
                        <div class="illegal-profile-frame">
                            <IllegalProfileBadge/>
                        </div>
                    }.into_any()
                } else {
                    let avatar_src = src.unwrap_or(default_avatar);
                    view! {
                        <img
                            class="profile-img"
                            src=avatar_src
                            alt=format!("{}'s profile picture", username)
                            on:error=|ev| {
                                if let Some(target) = ev.target() {
                                    if let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                                        img.set_src("/svg/noname.svg");
                                    }
                                }
                            }
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

/// Profile info section (username, slug, bio).
#[component]
fn ProfileInfo(
    profile: Profile,
    is_illegal: bool,
    has_reports: bool,
) -> impl IntoView {
    let bio_signal = RwSignal::new(None::<String>);
    let bio_loading = RwSignal::new(false);

    // Fetch biography from media server if available
    if let Some(bio_path) = profile.biography.clone() {
        bio_loading.set(true);
        spawn_local(async move {
            if let Ok(text) = fetch_biography(bio_path).await {
                bio_signal.set(Some(text));
            }
            bio_loading.set(false);
        });
    }

    let username = if is_illegal {
        "removed".to_string()
    } else {
        profile.username.clone()
    };

    let slug = if is_illegal {
        "removed".to_string()
    } else {
        profile.display_slug()
    };

    view! {
        <div class="profile-info">
            <h2 class="profile-title">
                <span class="username">{username}</span>
                <span class="slug">{slug}</span>
                {has_reports.then(|| view! { <ReportedBadge/> })}
            </h2>

            <div class="profile-description">
                {move || {
                    if bio_loading.get() {
                        view! { <span class="bio-loading">"Loading..."</span> }.into_any()
                    } else if let Some(bio) = bio_signal.get() {
                        view! { <p class="bio-text">{bio}</p> }.into_any()
                    } else {
                        view! { <p class="bio-empty">"No biography available"</p> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Skeleton loader for profile header.
#[component]
pub fn ProfileHeaderSkeleton() -> impl IntoView {
    view! {
        <div class="profile-header skeleton">
            <div class="profile-header-content">
                <div class="profile-picture skeleton-avatar"></div>
                <div class="profile-info">
                    <div class="skeleton-line skeleton-title"></div>
                    <div class="skeleton-line skeleton-slug"></div>
                    <div class="skeleton-line skeleton-bio"></div>
                    <div class="skeleton-line skeleton-bio short"></div>
                </div>
                <div class="profile-actions-wrapper">
                    <div class="skeleton-button"></div>
                    <div class="skeleton-button"></div>
                </div>
            </div>
            <div class="profile-stats skeleton">
                <div class="skeleton-stat"></div>
                <div class="skeleton-stat"></div>
                <div class="skeleton-stat"></div>
            </div>
        </div>
    }
}
