//! Referral user card component.
//!
//! Individual user card displaying avatar, username, and slug with click navigation.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::models::referral::ReferralListUser;

/// Individual user card displaying avatar, username, and slug.
///
/// Clicking the card navigates to the user's profile.
#[component]
pub fn ReferralUserCard(user: ReferralListUser) -> impl IntoView {
    let navigate = use_navigate();
    let slug = user.slug.clone();
    let slug_for_click = slug.clone();

    let on_click = move |_| {
        let slug = slug_for_click.clone();
        navigate(&format!("/u/{}", slug), Default::default());
    };

    let slug_for_keydown = slug.clone();

    let avatar_url = user.avatar_url().to_string();

    view! {
        <div
            class="user_card button"
            role="button"
            tabindex="0"
            on:click=on_click
            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Enter" || ev.key() == " " {
                    ev.prevent_default();
                    let navigate = use_navigate();
                    navigate(&format!("/u/{}", slug_for_keydown), Default::default());
                }
            }
        >
            <div class="ref_user_info">
                <img
                    src=avatar_url.clone()
                    alt=format!("{}'s avatar", user.username)
                    class="user_avatar"
                    on:error=move |_ev| {
                        #[cfg(feature = "hydrate")]
                        {
                            let target = event_target::<web_sys::HtmlImageElement>(&_ev);
                            target.set_src("/svg/noname.svg");
                        }
                    }
                />
                <div class="user_info">
                    <span class="user_name">{user.username.clone()}</span>
                    <span class="user_slug">{user.display_slug()}</span>
                </div>
            </div>
        </div>
    }
}
