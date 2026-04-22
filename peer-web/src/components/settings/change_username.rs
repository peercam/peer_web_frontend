//! Change username panel component.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::api::settings::update_username;
use crate::components::toast::{ToastType, use_toast};

/// Panel for changing the user's username.
///
/// Requires the new username and current password for confirmation.
#[component]
pub fn ChangeUsernamePanel(
    /// Callback to navigate back to the main profile panel.
    on_back: impl Fn() + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    let can_submit = move || {
        !username.get().trim().is_empty()
            && !password.get().trim().is_empty()
            && !is_submitting.get()
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !can_submit() {
            return;
        }

        is_submitting.set(true);
        response_msg.set(None);
        let new_username = username.get().trim().to_string();
        let pw = password.get().clone();
        let toast = toast;

        spawn_local(async move {
            match update_username(new_username, pw).await {
                Ok(()) => {
                    toast.show("Username updated!", ToastType::Success);
                    username.set(String::new());
                    password.set(String::new());
                    response_msg.set(Some((
                        "Username changed successfully. Page will reload.".to_string(),
                        true,
                    )));

                    // Reload after short delay to reflect changes
                    #[cfg(feature = "hydrate")]
                    {
                        use std::time::Duration;
                        set_timeout(
                            move || {
                                let _ = web_sys::window().map(|w| w.location().reload());
                            },
                            Duration::from_millis(1500),
                        );
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("30202") {
                        response_msg.set(Some(("Invalid username format. Use 3-23 characters, letters, numbers, underscores, or hyphens.".to_string(), false)));
                    } else if msg.contains("31001") {
                        response_msg.set(Some(("Incorrect password.".to_string(), false)));
                    } else {
                        response_msg.set(Some((msg, false)));
                    }
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="profile-widget active">
            <div class="edit-profile change-username">
                <h2 class="section_heading">"Change username"</h2>
                <form class="form-container" on:submit=on_submit>
                    <div class="profile-fields">
                        <div class="input-field">
                            <input
                                type="text"
                                class="input-text"
                                placeholder="Enter new username"
                                minlength="3"
                                maxlength="23"
                                required
                                prop:value=move || username.get()
                                on:input=move |ev| username.set(event_target_value(&ev))
                            />
                            <small class="input-hint">
                                "3-23 characters. Letters, numbers, underscores, hyphens only."
                            </small>
                        </div>
                        <div class="input-field">
                            <input
                                type="password"
                                class="input-text"
                                placeholder="Enter password to confirm"
                                required
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    {move || {
                        response_msg.get().map(|(msg, success)| {
                            view! {
                                <div
                                    class="response_msg"
                                    class:success=success
                                    class:error=!success
                                >
                                    {msg}
                                </div>
                            }
                        })
                    }}

                    <button
                        type="submit"
                        class="save-btn full-width-btn"
                        class:btn-blue=can_submit
                        prop:disabled=move || !can_submit()
                    >
                        {move || {
                            if is_submitting.get() {
                                "Submitting..."
                            } else {
                                "Submit"
                            }
                        }}
                    </button>
                </form>
                <button class="button btn-transparent back-btn" on:click=move |_| on_back()>
                    "Back"
                </button>
            </div>
        </div>
    }
}
