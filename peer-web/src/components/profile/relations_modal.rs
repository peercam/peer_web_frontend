//! Relations modal component.
//!
//! Modal for displaying followers, following, and peers lists.

use leptos::prelude::*;

use crate::api::profile::{list_follow_relations, list_friends};
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
        if let Some(target) = ev.target() {
            if let Some(current_target) = ev.current_target() {
                if target == current_target {
                    on_close_for_overlay.run(());
                }
            }
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

/// Tab content with infinite scroll loading.
#[component]
fn RelationsTabContent(
    user_id: String,
    tab: RwSignal<RelationsTab>,
    #[allow(unused)]
    is_own_profile: bool,
) -> impl IntoView {
    // Clone user_id for each resource
    let user_id_for_follow = user_id.clone();
    let user_id_for_friends = user_id.clone();

    // Separate resources for followers/following and friends
    let follow_relations = Resource::new(
        move || (user_id_for_follow.clone(), tab.get()),
        |(user_id, current_tab)| async move {
            if current_tab == RelationsTab::Peers {
                return Ok((Vec::new(), Vec::new()));
            }
            
            let res = list_follow_relations(Some(user_id), 0, 50).await?;
            let relations = res.affected_rows.unwrap_or_default();
            Ok::<_, ServerFnError>((relations.followers, relations.following))
        },
    );

    let friends_resource = Resource::new(
        move || (user_id_for_friends.clone(), tab.get()),
        |(user_id, current_tab)| async move {
            if current_tab != RelationsTab::Peers {
                return Ok(Vec::new());
            }
            
            let res = list_friends(Some(user_id), 0, 50).await?;
            Ok::<_, ServerFnError>(res.affected_rows)
        },
    );

    view! {
        <div class="relations-tab-content" role="tabpanel">
            <Suspense fallback=move || view! { <LoadingSpinner/> }>
                {move || {
                    let current_tab = tab.get();
                    match current_tab {
                        RelationsTab::Followers => {
                            follow_relations.get().map(|result| {
                                match result {
                                    Ok((followers, _)) => view! {
                                        <UserList users=followers/>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <ErrorMessage message=e.to_string()/>
                                    }.into_any(),
                                }
                            })
                        },
                        RelationsTab::Following => {
                            follow_relations.get().map(|result| {
                                match result {
                                    Ok((_, following)) => view! {
                                        <UserList users=following/>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <ErrorMessage message=e.to_string()/>
                                    }.into_any(),
                                }
                            })
                        },
                        RelationsTab::Peers => {
                            friends_resource.get().map(|result| {
                                match result {
                                    Ok(friends) => view! {
                                        <FriendList users=friends/>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <ErrorMessage message=e.to_string()/>
                                    }.into_any(),
                                }
                            })
                        },
                    }
                }}
            </Suspense>
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
        }.into_any()
    } else {
        view! {
            <div class="user-list">
                <For
                    each=move || users.clone()
                    key=|user| user.userid.clone()
                    children=|user| view! { <UserListItem user/> }
                />
            </div>
        }.into_any()
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
        }.into_any()
    } else {
        view! {
            <div class="user-list friend-list">
                <For
                    each=move || users.clone()
                    key=|user| user.userid.clone()
                    children=|user| view! { <FriendListItem user/> }
                />
            </div>
        }.into_any()
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

/// Error message component.
#[component]
fn ErrorMessage(message: String) -> impl IntoView {
    view! {
        <div class="error-message">
            <i class="peer-icon peer-icon-alert-circle"></i>
            <p>{message}</p>
        </div>
    }
}
