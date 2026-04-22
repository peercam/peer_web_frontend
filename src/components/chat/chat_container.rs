//! Main chat container component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::chat::{ChatInput, ChatMessages};
use crate::state::chat::{ConnectionState, poll_active_chat, refresh_chat_list, use_chat};

/// Main chat container with header, messages, and input.
#[component]
pub fn ChatContainer() -> impl IntoView {
    let ctx = use_chat();

    let has_active_chat = move || ctx.active_chat.get().is_some();
    let is_lost = move || ctx.connection_state.get() == ConnectionState::Lost;

    let retry_now = move |_| {
        spawn_local(async move {
            refresh_chat_list(ctx).await;
            poll_active_chat(ctx).await;
        });
    };

    view! {
        <div class="chat-container">
            <Show when=is_lost>
                <div class="connection-lost-banner" role="status" aria-live="polite">
                    <span>"Connection lost. Retrying…"</span>
                    <button class="retry-btn" on:click=retry_now>"Retry now"</button>
                </div>
            </Show>
            <Show
                when=has_active_chat
                fallback=|| view! { <NoChatSelected/> }
            >
                <ActiveChat/>
            </Show>
        </div>
    }
}

/// Placeholder when no chat is selected.
#[component]
fn NoChatSelected() -> impl IntoView {
    view! {
        <div class="no-chat-selected">
            <div class="placeholder-content">
                <i class="peer-icon peer-icon-chat-large"/>
                <h3>"Select a chat"</h3>
                <p>"Choose a conversation from the list or start a new one."</p>
            </div>
        </div>
    }
}

/// Active chat view with header, messages, and input.
#[component]
fn ActiveChat() -> impl IntoView {
    let ctx = use_chat();

    view! {
        {move || {
            ctx.active_chat.get().map(|chat| {
                let current_user_id = ctx.current_user_id.get().unwrap_or_default();
                let display_name = chat.display_name(&current_user_id);
                let avatar_url = chat.avatar_url(&current_user_id);

                view! {
                    <ChatHeader name=display_name avatar=avatar_url/>
                    <ChatMessages/>
                    <ChatInput/>
                }
            })
        }}
    }
}

/// Chat header with avatar and name.
#[component]
fn ChatHeader(name: String, avatar: String) -> impl IntoView {
    view! {
        <div class="chat-header">
            <div class="header-left">
                <img src=avatar alt="Avatar" class="avatar"/>
                <span class="username">{name}</span>
            </div>
            <div class="header-right">
                <button class="icon-btn" aria-label="Search">
                    <i class="peer-icon peer-icon-search"/>
                </button>
                <button class="icon-btn" aria-label="Options">
                    <i class="peer-icon peer-icon-more"/>
                </button>
            </div>
        </div>
    }
}
