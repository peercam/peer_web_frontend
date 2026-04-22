//! Profile widget component.

use leptos::prelude::*;

use crate::api::profile::get_profile;
use crate::models::user::UserInfo;
use crate::state::auth::use_auth;

/// Profile widget showing current user info.
#[component]
pub fn ProfileWidget() -> impl IntoView {
    let auth = use_auth();

    let user_info = Resource::new(
        move || auth.is_authenticated.get(),
        |is_auth| async move {
            if is_auth {
                match get_profile(None, None).await {
                    Ok(profile) => {
                        let img = Some(profile.avatar_url().to_string());
                        Some(UserInfo {
                            id: profile.id,
                            username: profile.username,
                            slug: profile.slug.to_string(),
                            img,
                            biography: profile.biography,
                            amount_followers: profile.amountfollower,
                            amount_following: profile.amountfollowed,
                            amount_peers: profile.amountfriends,
                            user_preferences: None,
                        })
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        },
    );

    view! {
        <div class="widget widget-margin-bottom">
            <a href="/profile" class="widget-inner widget-type-box widget-profile">
                <Suspense fallback=|| view! { <ProfileWidgetSkeleton/> }>
                    {move || {
                        user_info.get().map(|info| {
                            match info {
                                Some(user) => view! { <ProfileWidgetContent user=user/> }.into_any(),
                                None => view! { <ProfileWidgetPlaceholder/> }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </a>
        </div>
    }
}

/// Profile widget content when user is loaded.
#[component]
fn ProfileWidgetContent(user: UserInfo) -> impl IntoView {
    let img_src = user.img.unwrap_or_else(|| "/svg/noname.svg".to_string());

    view! {
        <>
            <div class="profile-header">
                <div class="cropContainer">
                    <span class="online_status"/>
                    <img
                        src=img_src
                        alt="Profile Picture"
                        class="profilbild profile-picture"
                    />
                </div>
                <div class="pro-name">
                    <div class="username">{user.username}</div>
                    <p class="slug">{"#"}{user.slug}</p>
                </div>
            </div>

            <div class="stats">
                <Stat label="Followers" value=user.amount_followers/>
                <Stat label="Peers" value=user.amount_peers/>
                <Stat label="Following" value=user.amount_following/>
            </div>
        </>
    }
}

/// Placeholder when user info is not available.
#[component]
fn ProfileWidgetPlaceholder() -> impl IntoView {
    view! {
        <div class="profile-header">
            <div class="cropContainer">
                <img
                    src="/svg/noname.svg"
                    alt="Profile Picture"
                    class="profilbild profile-picture"
                />
            </div>
            <div class="pro-name">
                <div class="username">"Guest"</div>
                <p class="slug">"#---"</p>
            </div>
        </div>

        <div class="stats">
            <Stat label="Followers" value=0/>
            <Stat label="Peers" value=0/>
            <Stat label="Following" value=0/>
        </div>
    }
}

/// Loading skeleton for profile widget.
#[component]
fn ProfileWidgetSkeleton() -> impl IntoView {
    view! {
        <div class="profile-header skeleton">
            <div class="cropContainer">
                <div class="skeleton-circle"/>
            </div>
            <div class="pro-name">
                <div class="skeleton-text"/>
                <div class="skeleton-text small"/>
            </div>
        </div>
        <div class="stats skeleton">
            <div class="skeleton-stat"/>
            <div class="skeleton-stat"/>
            <div class="skeleton-stat"/>
        </div>
    }
}

/// Individual stat display.
#[component]
fn Stat(label: &'static str, value: i32) -> impl IntoView {
    view! {
        <div class="stat">
            <span class="stat-value">{value}</span>
            <span class="stat-label">{label}</span>
        </div>
    }
}
