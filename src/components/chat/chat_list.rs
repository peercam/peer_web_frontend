//! Chat list sidebar component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::chat::{ChatItem, ContactsOverlay};
use crate::models::chat::ChatType;
use crate::state::chat::{load_friends, use_chat};

/// Chat list sidebar with tabs and chat items.
#[component]
pub fn ChatList() -> impl IntoView {
    view! {
        <div class="chat-list">
            <TopBar/>
            <ChatPanel/>
        </div>
    }
}

/// Top bar with tabs and add button.
#[component]
fn TopBar() -> impl IntoView {
    let ctx = use_chat();

    let open_overlay = move |_| {
        ctx.is_create_overlay_open.set(true);
        // Load friends when opening overlay
        spawn_local(async move {
            load_friends(ctx).await;
        });
    };

    view! {
        <div class="top-bar">
            <div class="chat-switch">
                <ChatTabs/>
                <button
                    class="add-btn btn-blue"
                    on:click=open_overlay
                    aria-label="Start new chat"
                >
                    "+"
                </button>
            </div>
        </div>
    }
}

/// Tab buttons for switching between private and group chats.
#[component]
fn ChatTabs() -> impl IntoView {
    let ctx = use_chat();

    let is_private = move || ctx.filter_type.get() == ChatType::Private;
    let is_group = move || ctx.filter_type.get() == ChatType::Group;

    view! {
        <button
            id="privateBtn"
            class=move || if is_private() { "tab active" } else { "tab" }
            on:click=move |_| ctx.filter_type.set(ChatType::Private)
        >
            "Private"
        </button>
        <button
            id="groupBtn"
            class=move || if is_group() { "tab active" } else { "tab" }
            on:click=move |_| ctx.filter_type.set(ChatType::Group)
        >
            "Groups"
        </button>
    }
}

/// Main chat panel with items or overlay.
#[component]
fn ChatPanel() -> impl IntoView {
    let ctx = use_chat();

    view! {
        <div class="chat-pannel">
            <div class="chat-pannel-widget">
                <Show
                    when=move || ctx.is_create_overlay_open.get()
                    fallback=|| view! { <ChatItems/> }
                >
                    <ContactsOverlay/>
                </Show>
            </div>
        </div>
    }
}

/// List of chat items.
#[component]
fn ChatItems() -> impl IntoView {
    let ctx = use_chat();

    let filtered_chats = move || ctx.filtered_chats();

    let is_loading = move || ctx.is_loading_chats.get();
    let has_chats = move || !filtered_chats().is_empty();
    let search_is_active = move || !ctx.search_query.get().trim().is_empty();

    view! {
        <Show when=is_loading fallback=move || {
            view! {
                <Show when=has_chats fallback=move || {
                    view! {
                        <Show when=search_is_active fallback=|| view! { <EmptyState/> }>
                            <FilteredEmptyState/>
                        </Show>
                    }
                }>
                    <For
                        each=filtered_chats
                        key=|chat| chat.id.clone()
                        children=move |chat| {
                            view! { <ChatItem chat=chat/> }
                        }
                    />
                </Show>
            }
        }>
            <ChatListSkeleton/>
        </Show>
    }
}

/// Empty state shown when a search filter yields no results.
#[component]
fn FilteredEmptyState() -> impl IntoView {
    let ctx = use_chat();
    let query = move || ctx.search_query.get();
    view! {
        <div class="no_post_found active">
            <p>{move || format!("No chats match \"{}\"", query())}</p>
        </div>
    }
}

/// Empty state when no chats found.
#[component]
fn EmptyState() -> impl IntoView {
    let ctx = use_chat();
    let message = move || {
        if ctx.filter_type.get() == ChatType::Private {
            "No private chats yet. Start a conversation!"
        } else {
            "No group chats yet. Create one!"
        }
    };

    view! {
        <div class="no_post_found active">
            <p>{message}</p>
        </div>
    }
}

/// Loading skeleton for chat list.
#[component]
fn ChatListSkeleton() -> impl IntoView {
    view! {
        <div class="chat-list-skeleton">
            {(0..5).map(|_| view! {
                <div class="skeleton-item">
                    <div class="skeleton-avatar"/>
                    <div class="skeleton-content">
                        <div class="skeleton-line short"/>
                        <div class="skeleton-line"/>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}
