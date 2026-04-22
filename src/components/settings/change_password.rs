//! Change password panel component.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys;

use crate::api::settings::update_password;
use crate::components::password_strength::PasswordStrengthMeter;
use crate::components::toast::{ToastType, use_toast};
use crate::components::validation::validate_password;

/// Panel for changing the user's password.
///
/// Includes password strength indicator and confirmation matching.
#[component]
pub fn ChangePasswordPanel(
    /// Callback to navigate back to the main profile panel.
    on_back: impl Fn() + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let old_password = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    // Password validation state
    let password_validation = Memo::new(move |_| validate_password(&new_password.get()));

    let password_visible = Memo::new(move |_| !new_password.get().is_empty());

    let passwords_match = move || {
        let np = new_password.get();
        let cp = confirm_password.get();
        !np.is_empty() && np == cp
    };

    let password_valid = move || password_validation.get().requirements.is_sufficient();

    let can_submit = move || {
        !old_password.get().is_empty()
            && password_valid()
            && passwords_match()
            && !is_submitting.get()
    };

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !can_submit() {
            return;
        }

        is_submitting.set(true);
        response_msg.set(None);
        let new_pw = new_password.get().clone();
        let old_pw = old_password.get().clone();
        let toast = toast;

        spawn_local(async move {
            match update_password(new_pw, old_pw).await {
                Ok(()) => {
                    toast.show("Password updated successfully!", ToastType::Success);
                    old_password.set(String::new());
                    new_password.set(String::new());
                    confirm_password.set(String::new());
                    response_msg.set(Some(("Password changed successfully.".to_string(), true)));
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("31001") {
                        response_msg
                            .set(Some(("Current password is incorrect.".to_string(), false)));
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
            <div class="edit-profile change-password">
                <h2 class="section_heading">"Change password"</h2>
                <form class="form-container" on:submit=on_submit>
                    <div class="profile-fields">
                        // Old password
                        <div class="input-field">
                            <img class="password-icon" src="/svg/lock1.svg" alt=""/>
                            <input
                                type="password"
                                class="input-text"
                                placeholder="Enter current password"
                                required
                                prop:value=move || old_password.get()
                                on:input=move |ev| old_password.set(event_target_value(&ev))
                            />
                        </div>

                        // New password with strength indicator
                        <div class="password-component">
                            <div class="input-field">
                                <img class="password-icon" src="/svg/lock1.svg" alt=""/>
                                <input
                                    type="password"
                                    class="input-text"
                                    placeholder="Enter new password"
                                    required
                                    prop:value=move || new_password.get()
                                    on:input=move |ev| new_password.set(event_target_value(&ev))
                                />
                            </div>
                            <PasswordStrengthMeter
                                validation=password_validation
                                visible=password_visible
                            />
                        </div>

                        // Confirm password
                        <div class="input-field">
                            <img class="password-icon" src="/svg/lock1.svg" alt=""/>
                            <input
                                type="password"
                                class="input-text"
                                placeholder="Confirm new password"
                                required
                                prop:value=move || confirm_password.get()
                                on:input=move |ev| confirm_password.set(event_target_value(&ev))
                            />
                        </div>

                        // Match validation message
                        <Show when=move || {
                            !confirm_password.get().is_empty() && !passwords_match()
                        }>
                            <div class="validationMessage notvalid">"Passwords do not match"</div>
                        </Show>
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
