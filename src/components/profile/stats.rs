//! Profile statistics component.
//!
//! Displays a user's post count, follower count, following count, and peers count.

use leptos::prelude::*;

use crate::models::profile::{Profile, RelationsTab};

/// Profile statistics (posts, followers, following, peers).
///
/// Each stat count is clickable to open the relations modal.
#[component]
pub fn ProfileStats(
    /// The profile data.
    profile: Profile,
    /// Whether this is the user's own profile (shows Peers count).
    #[prop(default = false)]
    is_own_profile: bool,
    /// Callback when a stat is clicked to open the relations modal.
    #[prop(optional)]
    on_stat_click: Option<Callback<RelationsTab>>,
) -> impl IntoView {
    let handle_click = move |tab: RelationsTab| {
        if let Some(callback) = on_stat_click {
            callback.run(tab);
        }
    };

    view! {
        <div class="profile-stats">
            <span class="stat post-count">
                <em>{profile.amountposts}</em>
                " Publications"
            </span>

            <span
                class="stat followers-count clickable"
                on:click=move |_| handle_click(RelationsTab::Followers)
                role="button"
                tabindex="0"
            >
                <em>{profile.amountfollower}</em>
                " Followers"
            </span>

            <span
                class="stat following-count clickable"
                on:click=move |_| handle_click(RelationsTab::Following)
                role="button"
                tabindex="0"
            >
                <em>{profile.amountfollowed}</em>
                " Following"
            </span>

            {is_own_profile.then(|| {
                view! {
                    <span
                        class="stat peers-count clickable"
                        on:click=move |_| handle_click(RelationsTab::Peers)
                        role="button"
                        tabindex="0"
                    >
                        <em>{profile.amountfriends}</em>
                        " Peers"
                    </span>
                }
            })}
        </div>
    }
}

/// Compact stats display for mobile or smaller spaces.
#[component]
pub fn ProfileStatsCompact(
    /// The profile data.
    profile: Profile,
    /// Whether this is the user's own profile.
    #[prop(default = false)]
    is_own_profile: bool,
    /// Callback when a stat is clicked.
    #[prop(optional)]
    on_stat_click: Option<Callback<RelationsTab>>,
) -> impl IntoView {
    let handle_click = move |tab: RelationsTab| {
        if let Some(callback) = on_stat_click {
            callback.run(tab);
        }
    };

    view! {
        <div class="profile-stats compact">
            <span class="stat" title="Posts">
                <i class="peer-icon peer-icon-grid"></i>
                <em>{profile.amountposts}</em>
            </span>

            <span
                class="stat clickable"
                title="Followers"
                on:click=move |_| handle_click(RelationsTab::Followers)
            >
                <i class="peer-icon peer-icon-users"></i>
                <em>{profile.amountfollower}</em>
            </span>

            <span
                class="stat clickable"
                title="Following"
                on:click=move |_| handle_click(RelationsTab::Following)
            >
                <i class="peer-icon peer-icon-user-plus"></i>
                <em>{profile.amountfollowed}</em>
            </span>

            {is_own_profile.then(|| {
                view! {
                    <span
                        class="stat clickable"
                        title="Peers"
                        on:click=move |_| handle_click(RelationsTab::Peers)
                    >
                        <i class="peer-icon peer-icon-link"></i>
                        <em>{profile.amountfriends}</em>
                    </span>
                }
            })}
        </div>
    }
}
