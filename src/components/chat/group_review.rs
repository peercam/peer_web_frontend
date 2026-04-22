//! Group review screen for creating group chats.

use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::state::chat::{create_group_chat, use_chat};

/// Group creation review screen (step 2).
#[component]
pub fn GroupReviewScreen() -> impl IntoView {
    let ctx = use_chat();
    let (error, set_error) = signal(Option::<String>::None);

    let selected_users = move || ctx.selected_users.get();
    let group_name = move || ctx.group_name.get();
    let is_sending = move || ctx.is_sending.get();

    #[cfg(target_arch = "wasm32")]
    let on_name_input = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input = target.unchecked_ref::<web_sys::HtmlInputElement>();
        ctx.group_name.set(input.value());
    };

    #[cfg(not(target_arch = "wasm32"))]
    let on_name_input = move |_ev: leptos::ev::Event| {};

    let go_back = move |_| {
        ctx.is_review_screen.set(false);
    };

    let create_group = move |_| {
        set_error.set(None);
        spawn_local(async move {
            match create_group_chat(ctx).await {
                Ok(_) => {
                    // Success - overlay will close automatically
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="group-review">
            // Group image upload
            <div class="group-image-upload">
                <div class="image-placeholder">
                    <i class="peer-icon peer-icon-camera"/>
                    <span>"Add Image"</span>
                </div>
            </div>

            // Group name input
            <div class="form-group">
                <label for="groupName">"Group Name"</label>
                <input
                    type="text"
                    id="groupName"
                    class="input title"
                    placeholder="Enter group name"
                    prop:value=group_name
                    on:input=on_name_input
                    maxlength="50"
                />
            </div>

            // Selected members preview
            <div class="selected-members">
                <h4>"Members (" {move || selected_users().len()} ")"</h4>
                <div class="member-list">
                    <For
                        each=selected_users
                        key=|u| u.userid.clone()
                        children=move |user| {
                            let avatar = user.img.clone()
                                .unwrap_or_else(|| "/svg/noname.svg".to_string());
                            view! {
                                <div class="member-chip">
                                    <img src=avatar alt=user.username.clone() class="avatar"/>
                                    <span>{user.username.clone()}</span>
                                </div>
                            }
                        }
                    />
                </div>
            </div>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="error-block">
                    <label>{move || error.get()}</label>
                </div>
            </Show>

            // Action buttons
            <div class="action-buttons">
                <button
                    class="btn btn-secondary"
                    on:click=go_back
                    disabled=is_sending
                >
                    "Back"
                </button>
                <button
                    class="btn btn-primary create-btn"
                    on:click=create_group
                    disabled=move || {
                        is_sending() ||
                        group_name().trim().is_empty() ||
                        selected_users().is_empty()
                    }
                >
                    {move || if is_sending() { "Creating..." } else { "Create Group" }}
                </button>
            </div>
        </div>
    }
}
