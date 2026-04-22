//! Change email panel component.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::api::settings::update_email;
use crate::components::toast::{ToastType, use_toast};
use crate::components::validation::is_valid_email;

/// Panel for changing the user's email address.
///
/// Requires the new email and current password for confirmation.
#[component]
pub fn ChangeEmailPanel(
    /// Callback to navigate back to the main profile panel.
    on_back: impl Fn() + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    let email_valid = move || is_valid_email(&email.get());

    let can_submit = move || email_valid() && !password.get().is_empty() && !is_submitting.get();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !can_submit() {
            return;
        }

        is_submitting.set(true);
        response_msg.set(None);
        let new_email = email.get().trim().to_string();
        let pw = password.get().clone();
        let toast = toast;

        spawn_local(async move {
            match update_email(new_email, pw).await {
                Ok(()) => {
                    toast.show("Email updated successfully!", ToastType::Success);
                    email.set(String::new());
                    password.set(String::new());
                    response_msg.set(Some(("Email changed successfully.".to_string(), true)));
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("30103") {
                        response_msg.set(Some(("Invalid email format.".to_string(), false)));
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
            <div class="edit-profile change-email">
                <h2 class="section_heading">"Change e-mail"</h2>
                <form class="form-container" on:submit=on_submit>
                    <div class="profile-fields">
                        <div class="input-field">
                            <input
                                type="email"
                                class="input-text"
                                placeholder="Enter new e-mail address"
                                required
                                prop:value=move || email.get()
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                        </div>

                        // Email validation hint
                        <Show when=move || !email.get().is_empty() && !email_valid()>
                            <div class="validationMessage notvalid">"Please enter a valid email address"</div>
                        </Show>

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
