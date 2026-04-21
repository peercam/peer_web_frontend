//! Chat messages display component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::models::chat::{ChatMessage, MessageStatus, format_message_time};
use crate::state::chat::{retry_message, use_chat};

/// Container for displaying chat messages.
#[component]
pub fn ChatMessages() -> impl IntoView {
    let ctx = use_chat();

    let messages = move || {
        let mut msgs = ctx.messages.get();
        msgs.sort_by_key(|m| m.createdat.clone());
        msgs
    };

    let current_user_id = move || ctx.current_user_id.get().unwrap_or_default();

    view! {
        <div class="chat-messages">
            <For
                each=messages
                key=|m| m.id.clone()
                children=move |message| {
                    let is_own = message.senderid == current_user_id();
                    view! { <Message message=message is_own=is_own/> }
                }
            />
            <ScrollAnchor/>
        </div>
    }
}

/// A single message bubble.
#[component]
fn Message(message: ChatMessage, is_own: bool) -> impl IntoView {
    let ctx = use_chat();
    let content = message.decoded_content();
    let time = format_message_time(&message.createdat);
    let status = message.status.clone();
    let msg_id = message.id.clone();

    let class = {
        let status = status.clone();
        let base = if is_own { "message right" } else { "message" };
        match status {
            MessageStatus::Sending => format!("{} sending", base),
            MessageStatus::Failed => format!("{} failed", base),
            MessageStatus::Sent => base.to_string(),
        }
    };

    let is_failed = matches!(status, MessageStatus::Failed);
    let retry_id = msg_id.clone();
    let on_retry = move |_| {
        let id = retry_id.clone();
        spawn_local(async move {
            let _ = retry_message(ctx, id).await;
        });
    };

    view! {
        <div class=class>
            <Show when=move || !is_own>
                <div class="profile_avatar">
                    <img
                        class="avatar"
                        src="/svg/noname.svg"
                        alt="Avatar"
                    />
                </div>
            </Show>
            <div class="message_content">
                <div class="bubble">
                    <span class="message-text">{content}</span>
                    <span class="time">{time}</span>
                </div>
                <Show when=move || is_failed>
                    <div class="failed-meta">
                        <span class="failed-label">"Failed to send"</span>
                        <button class="retry-btn" on:click=on_retry.clone()>"Retry"</button>
                    </div>
                </Show>
            </div>
        </div>
    }
}

/// Invisible anchor for auto-scrolling to bottom.
#[component]
fn ScrollAnchor() -> impl IntoView {
    let anchor_ref = NodeRef::<leptos::html::Div>::new();

    // Scroll to bottom when messages change
    Effect::new(move |_| {
        if let Some(el) = anchor_ref.get() {
            el.scroll_into_view();
        }
    });

    view! {
        <div node_ref=anchor_ref class="scroll-anchor"/>
    }
}
