//! Chat input component.

use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::state::chat::{send_message, use_chat};

/// Message input with send functionality.
#[component]
pub fn ChatInput() -> impl IntoView {
    let ctx = use_chat();
    let (message, set_message) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);

    let is_sending = move || ctx.is_sending.get();

    let handle_send = move || {
        let content = message.get().trim().to_string();
        if content.is_empty() {
            return;
        }

        // Client-side validation
        if content.len() > 500 {
            set_error.set(Some("Message must be 500 characters or fewer".to_string()));
            return;
        }

        // Clear input immediately (optimistic)
        set_message.set(String::new());
        set_error.set(None);

        spawn_local(async move {
            if let Err(e) = send_message(ctx, content).await {
                set_error.set(Some(e));
            }
        });
    };

    #[cfg(target_arch = "wasm32")]
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() {
            ev.prevent_default();
            handle_send();
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let on_keydown = move |_ev: leptos::ev::KeyboardEvent| {};

    #[cfg(target_arch = "wasm32")]
    let on_input = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let textarea = target.unchecked_ref::<web_sys::HtmlTextAreaElement>();
        set_message.set(textarea.value());

        // Clear error when typing
        if error.get().is_some() {
            set_error.set(None);
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let on_input = move |ev: leptos::ev::Event| {
        let _ = ev;

        // Clear error when typing
        if error.get().is_some() {
            set_error.set(None);
        }
    };

    let avatar_url = move || {
        ctx.current_user_img
            .get()
            .unwrap_or_else(|| "/svg/noname.svg".to_string())
    };

    let char_count = move || message.get().len();
    let is_over_limit = move || char_count() > 500;

    view! {
        <div class="chat-input-container">
            <Show when=move || error.get().is_some()>
                <div class="error-block">
                    <label>{move || error.get()}</label>
                </div>
            </Show>

            <div class="chat-input">
                <img
                    src=avatar_url
                    class="avatar"
                    alt="Your avatar"
                />
                <textarea
                    id="sendPrivateMessage"
                    placeholder="Write a message ..."
                    prop:value=message
                    on:input=on_input
                    on:keydown=on_keydown
                    disabled=is_sending
                    rows="1"
                />
                <button
                    class="send-btn"
                    on:click=move |_| handle_send()
                    disabled=move || is_sending() || message.get().trim().is_empty()
                    aria-label="Send message"
                >
                    <i class="peer-icon peer-icon-send"/>
                </button>
            </div>

            <div class="char-counter" class:over-limit=is_over_limit>
                <span>{char_count}</span>
                <span>"/"</span>
                <span>"500"</span>
            </div>
        </div>
    }
}
