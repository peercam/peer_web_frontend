//! Contacts overlay for starting new chats.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::chat::GroupReviewScreen;
use crate::models::chat::ChatType;
use crate::models::profile::BasicUserInfo;
use crate::state::chat::{start_private_chat, use_chat};

/// Contacts overlay for creating new chats.
#[component]
pub fn ContactsOverlay() -> impl IntoView {
    let ctx = use_chat();

    let is_review = move || ctx.is_review_screen.get();

    view! {
        <div class="contacts-overlay">
            <OverlayHeader/>
            <Show
                when=is_review
                fallback=|| view! { <ContactsList/> }
            >
                <GroupReviewScreen/>
            </Show>
        </div>
    }
}

/// Overlay header with close button.
#[component]
fn OverlayHeader() -> impl IntoView {
    let ctx = use_chat();

    let title = move || {
        if ctx.is_review_screen.get() {
            "Create Group"
        } else if ctx.filter_type.get() == ChatType::Group {
            "Select Members"
        } else {
            "Select Contact"
        }
    };

    let close = move |_| {
        ctx.close_overlay();
    };

    view! {
        <div class="overlay-header">
            <button class="back-btn" on:click=close aria-label="Close">
                <i class="peer-icon peer-icon-arrow-left"/>
            </button>
            <h3>{title}</h3>
        </div>
    }
}

/// List of contacts/friends.
#[component]
fn ContactsList() -> impl IntoView {
    let ctx = use_chat();

    let friends = move || ctx.friends.get();
    let is_loading = move || ctx.is_loading_friends.get();
    let is_group = move || ctx.filter_type.get() == ChatType::Group;
    let selected_count = move || ctx.selected_users.get().len();

    let go_to_review = move |_| {
        ctx.is_review_screen.set(true);
    };

    view! {
        <div class="contacts-list">
            <Show when=is_loading fallback=move || {
                view! {
                    <Show when=move || !friends().is_empty() fallback=|| view! { <NoFriends/> }>
                        <For
                            each=friends
                            key=|f| f.userid.clone()
                            children=move |friend| {
                                view! { <ContactCard friend=friend/> }
                            }
                        />
                    </Show>
                }
            }>
                <ContactsSkeleton/>
            </Show>
        </div>

        // Show "Next" button for group mode
        <Show when=is_group>
            <div class="chat_buttons selected">
                <span class="count-selected">
                    {move || format!("{} account{} selected",
                        selected_count(),
                        if selected_count() == 1 { "" } else { "s" }
                    )}
                </span>
                <button
                    class="next-btn btn-blue"
                    on:click=go_to_review
                    disabled=move || selected_count() == 0
                >
                    "Next"
                </button>
            </div>
        </Show>
    }
}

/// A single contact card.
#[component]
fn ContactCard(friend: BasicUserInfo) -> impl IntoView {
    let ctx = use_chat();
    let friend_for_toggle = friend.clone();
    let friend_for_start = friend.clone();
    let userid_for_check = friend.userid.clone();

    let is_group = move || ctx.filter_type.get() == ChatType::Group;

    // Use Memo for derived state that needs to be reactive
    let is_selected = Memo::new(move |_| ctx.is_user_selected(&userid_for_check));

    let avatar_url = friend
        .img
        .clone()
        .unwrap_or_else(|| "/svg/noname.svg".to_string());

    let toggle_selection = {
        let friend = friend_for_toggle.clone();
        move |_| {
            ctx.toggle_user_selection(friend.clone());
        }
    };

    let start_chat = {
        let friend = friend_for_start.clone();
        move |_| {
            let friend = friend.clone();
            spawn_local(async move {
                let _ = start_private_chat(ctx, friend).await;
            });
        }
    };

    view! {
        <div class="contact-card" data-user-id=friend.userid.clone()>
            <div class="contact-info">
                <img src=avatar_url alt=friend.username.clone() class="avatar"/>
                <span class="username">{friend.username.clone()}</span>
            </div>

            <Show
                when=is_group
                fallback=move || view! {
                    <button
                        class="chat-icon-btn"
                        on:click=start_chat.clone()
                        aria-label="Start chat"
                    >
                        <i class="peer-icon peer-icon-chat"/>
                    </button>
                }
            >
                <input
                    type="checkbox"
                    prop:checked=move || is_selected.get()
                    on:change=toggle_selection.clone()
                />
            </Show>
        </div>
    }
}

/// Empty state when no friends found.
#[component]
fn NoFriends() -> impl IntoView {
    view! {
        <div class="no_post_found active">
            <p>"No friends found. Follow users to add them as friends!"</p>
        </div>
    }
}

/// Loading skeleton for contacts.
#[component]
fn ContactsSkeleton() -> impl IntoView {
    view! {
        <div class="contacts-skeleton">
            {(0..5).map(|_| view! {
                <div class="skeleton-item">
                    <div class="skeleton-avatar"/>
                    <div class="skeleton-line"/>
                </div>
            }).collect_view()}
        </div>
    }
}
