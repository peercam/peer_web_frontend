//! Relations modal component.
//!
//! Modal for displaying followers, following, and peers lists
//! with infinite scroll pagination.

use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::profile::{list_follow_relations, list_friends};
use crate::hooks::use_infinite_scroll;
use crate::models::profile::{BasicUserInfo, ProfileUser, RelationsTab};

use super::user_list::{FriendListItem, UserListItem};

/// Modal showing followers, following, and peers.
#[component]
pub fn RelationsModal(
    /// User ID to show relations for.
    user_id: String,
    /// Whether viewing own profile (shows Peers tab).
    #[prop(default = false)]
    is_own_profile: bool,
    /// Default tab to open.
    #[prop(default = RelationsTab::Followers)]
    default_tab: RelationsTab,
    /// Called when modal should close.
    on_close: Callback<()>,
) -> impl IntoView {
    let active_tab = RwSignal::new(default_tab);

    // Close modal when clicking overlay background
    let on_close_for_overlay = on_close;
    let handle_overlay_click = move |ev: leptos::ev::MouseEvent| {
        if let Some(target) = ev.target()
            && let Some(current_target) = ev.current_target()
            && target == current_target
        {
            on_close_for_overlay.run(());
        }
    };

    // Close on escape key
    let on_close_for_keydown = on_close;
    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" {
            on_close_for_keydown.run(());
        }
    };

    view! {
        <div
            class="modal-overlay relations-modal-overlay"
            on:click=handle_overlay_click
            on:keydown=handle_keydown
            tabindex="-1"
        >
            <div class="modal-content relations-modal" role="dialog" aria-modal="true">
                <div class="modal-header">
                    <h2>"Relations"</h2>
                    <button
                        class="modal-close"
                        aria-label="Close"
                        on:click=move |_| on_close.run(())
                    >
                        <i class="peer-icon peer-icon-x"></i>
                    </button>
                </div>

                <div class="tabs" role="tablist">
                    <TabButton
                        label="Followers"
                        tab=RelationsTab::Followers
                        active_tab
                    />
                    <TabButton
                        label="Following"
                        tab=RelationsTab::Following
                        active_tab
                    />
                    {is_own_profile.then(|| {
                        view! {
                            <TabButton
                                label="Peers"
                                tab=RelationsTab::Peers
                                active_tab
                            />
                        }
                    })}
                </div>

                <div class="modal-body">
                    <RelationsTabContent
                        user_id=user_id.clone()
                        tab=active_tab
                        is_own_profile=is_own_profile
                    />
                </div>
            </div>
        </div>
    }
}

/// Tab button component.
#[component]
fn TabButton(
    label: &'static str,
    tab: RelationsTab,
    active_tab: RwSignal<RelationsTab>,
) -> impl IntoView {
    view! {
        <button
            class="tab-btn"
            class:active=move || active_tab.get() == tab
            role="tab"
            aria-selected=move || active_tab.get() == tab
            on:click=move |_| active_tab.set(tab)
        >
            {label}
        </button>
    }
}

/// Number of relations per batch.
const RELATIONS_PER_PAGE: i32 = 50;

/// Tab content with infinite scroll loading.
#[component]
fn RelationsTabContent(
    user_id: String,
    tab: RwSignal<RelationsTab>,
    #[allow(unused)] is_own_profile: bool,
) -> impl IntoView {
    // Per-tab state
    let followers = RwSignal::new(Vec::<ProfileUser>::new());
    let following = RwSignal::new(Vec::<ProfileUser>::new());
    let friends = RwSignal::new(Vec::<BasicUserInfo>::new());

    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let initial_loaded = RwSignal::new(false);

    // Reset when tab changes
    Effect::new(move |prev_tab: Option<RelationsTab>| {
        let current_tab = tab.get();
        if prev_tab.is_some() && prev_tab != Some(current_tab) {
            followers.set(Vec::new());
            following.set(Vec::new());
            friends.set(Vec::new());
            offset.set(0);
            has_more.set(true);
            initial_loaded.set(false);
        }
        current_tab
    });

    let uid = user_id.clone();
    let load_more = Rc::new(move || {
        if is_loading.get() || !has_more.get() {
            return;
        }
        is_loading.set(true);
        let user_id = uid.clone();
        let current_tab = tab.get_untracked();
        let current_offset = offset.get();

        spawn_local(async move {
            match current_tab {
                RelationsTab::Followers | RelationsTab::Following => {
                    match list_follow_relations(Some(user_id), current_offset, RELATIONS_PER_PAGE)
                        .await
                    {
                        Ok(res) => {
                            let relations = res.affected_rows.unwrap_or_default();
                            let (new_followers, new_following) =
                                (relations.followers, relations.following);
                            let got_results = match current_tab {
                                RelationsTab::Followers => {
                                    let has_new = !new_followers.is_empty();
                                    followers.update(|f| f.extend(new_followers));
                                    has_new
                                }
                                RelationsTab::Following => {
                                    let has_new = !new_following.is_empty();
                                    following.update(|f| f.extend(new_following));
                                    has_new
                                }
                                _ => false,
                            };
                            if got_results {
                                offset.update(|o| *o += RELATIONS_PER_PAGE);
                            }
                            has_more.set(got_results);
                        }
                        Err(e) => {
                            leptos::logging::error!("Failed to load relations: {:?}", e);
                            has_more.set(false);
                        }
                    }
                }
                RelationsTab::Peers => {
                    match list_friends(Some(user_id), current_offset, RELATIONS_PER_PAGE).await {
                        Ok(res) => {
                            let new_friends = res.affected_rows;
                            let has_new = !new_friends.is_empty();
                            friends.update(|f| f.extend(new_friends));
                            if has_new {
                                offset.update(|o| *o += RELATIONS_PER_PAGE);
                            }
                            has_more.set(has_new);
                        }
                        Err(e) => {
                            leptos::logging::error!("Failed to load friends: {:?}", e);
                            has_more.set(false);
                        }
                    }
                }
            }
            is_loading.set(false);
            initial_loaded.set(true);
        });
    });

    let scroll = use_infinite_scroll(is_loading, has_more, {
        let load_more = load_more.clone();
        move || load_more()
    });

    // Initial load
    {
        let load_more = load_more.clone();
        Effect::new(move |_| {
            // Re-run when tab changes (tracked by tab.get())
            let _tab = tab.get();
            if !is_loading.get() && !initial_loaded.get() {
                load_more();
            }
        });
    }

    view! {
        <div class="relations-tab-content" role="tabpanel">
            {move || {
                let current_tab = tab.get();
                match current_tab {
                    RelationsTab::Followers => {
                        let users = followers.get();
                        view! { <UserList users=users/> }.into_any()
                    }
                    RelationsTab::Following => {
                        let users = following.get();
                        view! { <UserList users=users/> }.into_any()
                    }
                    RelationsTab::Peers => {
                        let users = friends.get();
                        view! { <FriendList users=users/> }.into_any()
                    }
                }
            }}

            <div class="relations-loader" node_ref=scroll.loader_ref>
                <Show when=move || is_loading.get()>
                    <LoadingSpinner/>
                </Show>
            </div>
        </div>
    }
}

/// List of ProfileUser items.
#[component]
fn UserList(users: Vec<ProfileUser>) -> impl IntoView {
    if users.is_empty() {
        view! {
            <div class="empty-list">
                <p>"No users found"</p>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="user-list">
                <For
                    each=move || users.clone()
                    key=|user| user.userid.clone()
                    children=|user| view! { <UserListItem user/> }
                />
            </div>
        }
        .into_any()
    }
}

/// List of BasicUserInfo items (friends/peers).
#[component]
fn FriendList(users: Vec<BasicUserInfo>) -> impl IntoView {
    if users.is_empty() {
        view! {
            <div class="empty-list">
                <p>"No peers found"</p>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="user-list friend-list">
                <For
                    each=move || users.clone()
                    key=|user| user.userid.clone()
                    children=|user| view! { <FriendListItem user/> }
                />
            </div>
        }
        .into_any()
    }
}

/// Loading spinner component.
#[component]
fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="loading-spinner">
            <div class="spinner"></div>
        </div>
    }
}
