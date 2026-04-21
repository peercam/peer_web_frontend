//! Deactivate/Delete account panel component.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::api::settings::delete_account;
use crate::components::toast::{ToastType, use_toast};
use crate::state::auth::use_auth;

/// Deactivate account panel with password confirmation.
#[component]
pub fn DeactivateAccountPanel(
    /// Called when the user wants to go back.
    on_back: impl Fn() + 'static,
) -> impl IntoView {
    let password = RwSignal::new(String::new());
    let show_confirm = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let toast = use_toast();
    let auth = use_auth();

    let on_confirm = move |_: leptos::ev::MouseEvent| {
        is_deleting.set(true);
        let pw = password.get();
        let toast = toast.clone();

        spawn_local(async move {
            match delete_account(pw).await {
                Ok(_) => {
                    auth.logout_action.dispatch(());
                    use_navigate()("/login", Default::default());
                }
                Err(e) => {
                    toast.show(
                        &format!("Failed to delete account: {}", e),
                        ToastType::Error,
                    );
                    is_deleting.set(false);
                    show_confirm.set(false);
                }
            }
        });
    };

    view! {
        <div class="deactivate-panel">
            <button
                class="button btn-transparent back-btn"
                on:click=move |_| on_back()
            >
                <i class="peer-icon peer-icon-arrow-left"></i>
                " Back"
            </button>

            <h3>"Deactivate Profile"</h3>
            <p class="warning-text">
                "This action is permanent. Your account will be deactivated and all your data will be removed. This cannot be undone."
            </p>

            <div class="input-field">
                <label>"Confirm your password"</label>
                <input
                    type="password"
                    prop:value=move || password.get()
                    on:input=move |ev| password.set(event_target_value(&ev))
                    placeholder="Enter your password"
                    class="input-textarea"
                />
            </div>

            <button
                class="full-width-btn btn-danger"
                on:click=move |_| show_confirm.set(true)
                disabled=move || password.get().is_empty() || is_deleting.get()
            >
                "Deactivate my account"
            </button>

            <Show when=move || show_confirm.get()>
                <div class="modal-overlay">
                    <div class="logOut-pop">
                        <p class="xl_font_size bold">
                            "Are you sure you want to delete your account?"
                        </p>
                        <p>"This action cannot be undone."</p>
                        <div class="button-row">
                            <button
                                class="btn-white"
                                on:click=move |_| show_confirm.set(false)
                                disabled=move || is_deleting.get()
                            >
                                "Cancel"
                            </button>
                            <button
                                class="btn-red-transparent"
                                on:click=on_confirm
                                disabled=move || is_deleting.get()
                            >
                                {move || if is_deleting.get() { "Deleting..." } else { "Delete Account" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
