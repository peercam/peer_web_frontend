//! Individual chat item component.

use leptos::prelude::*;

use crate::models::chat::{Chat, format_relative_time};
use crate::state::chat::{select_chat, use_chat};

/// A single chat item in the list.
#[component]
pub fn ChatItem(chat: Chat) -> impl IntoView {
    let ctx = use_chat();
    let chat_id = chat.id.clone();
    let chat_for_click = chat.clone();

    let current_user_id = move || ctx.current_user_id.get().unwrap_or_default();

    let display_name = {
        let chat = chat.clone();
        move || chat.display_name(&current_user_id())
    };

    let avatar_url = {
        let chat = chat.clone();
        move || chat.avatar_url(&current_user_id())
    };

    let preview = {
        let chat = chat.clone();
        move || chat.message_preview()
    };

    let time = {
        let chat = chat.clone();
        move || {
            chat.last_message()
                .map(|m| format_relative_time(&m.createdat))
                .unwrap_or_else(|| "—".to_string())
        }
    };

    let is_active = {
        let chat_id = chat_id.clone();
        move || ctx.is_active(&chat_id)
    };

    let unread_count = {
        let chat_id = chat_id.clone();
        Memo::new(move |_| {
            ctx.unread_counts
                .get()
                .get(&chat_id)
                .copied()
                .unwrap_or(0u32)
        })
    };

    let badge_label = move || {
        let n = unread_count.get();
        if n > 99 {
            "99+".to_string()
        } else {
            n.to_string()
        }
    };

    let badge_aria = move || {
        let n = unread_count.get();
        if n > 99 {
            "99 or more unread messages".to_string()
        } else {
            format!("{} unread messages", n)
        }
    };

    let has_unread = move || unread_count.get() > 0;

    let on_click = move |_| {
        select_chat(ctx, chat_for_click.clone());
    };

    view! {
        <div
            class=move || if is_active() { "chat-item active-chat" } else { "chat-item" }
            data-chatid=chat_id.clone()
            on:click=on_click
        >
            <img
                class="avatar"
                src=avatar_url
                alt=display_name.clone()
            />
            <div class="chat-details">
                <div class="chat-row">
                    <span class="name">{display_name}</span>
                    <span class="time">{time}</span>
                </div>
                <div class="message-preview-row">
                    <div class="message-preview">{preview}</div>
                    <Show when=has_unread>
                        <span class="unread-badge" aria-label=badge_aria>
                            {badge_label}
                        </span>
                    </Show>
                </div>
            </div>
        </div>
    }
}
