//! Content settings tab component.
//!
//! Displays toggle for reported content visibility.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::posts::get_user_info;
use crate::api::settings::update_content_preferences;
use crate::components::toast::{use_toast, ToastType};

/// Content Settings tab — reported content toggle.
#[component]
pub fn ContentSettings() -> impl IntoView {
    let toast = use_toast();
    let is_lenient = RwSignal::new(false);
    let is_loading = RwSignal::new(true);
    let show_confirm = RwSignal::new(false);
    let pending_state = RwSignal::new(false);
    let show_success = RwSignal::new(false);
    let success_message = RwSignal::new(String::new());

    // Load current preference
    let user_info_resource = Resource::new(
        || (),
        |_| async move { get_user_info("self".to_string()).await },
    );

    Effect::new(move |_| {
        if let Some(Ok(info)) = user_info_resource.get() {
            if let Some(prefs) = info.user_preferences {
                if let Some(level) = prefs.content_filtering_severity_level {
                    is_lenient.set(level == "MYGRANDMAHATES");
                }
            }
            is_loading.set(false);
        }
    });

    let on_toggle = move |_| {
        pending_state.set(!is_lenient.get());
        show_confirm.set(true);
    };

    let on_confirm = {
        let toast = toast.clone();
        move || {
            show_confirm.set(false);
            let new_level = if pending_state.get() {
                "MYGRANDMAHATES"
            } else {
                "MYGRANDMALIKES"
            };

            let toast = toast.clone();
            spawn_local(async move {
                match update_content_preferences(new_level.to_string()).await {
                    Ok(()) => {
                        is_lenient.set(pending_state.get());
                        if pending_state.get() {
                            success_message.set(
                                "Content restored. Reported posts are now visible in your feed."
                                    .to_string(),
                            );
                        } else {
                            success_message.set(
                                "Hidden successfully. Reported posts have been removed from your feed."
                                    .to_string(),
                            );
                        }
                        show_success.set(true);
                    }
                    Err(e) => {
                        toast.show(&format!("Failed to update: {}", e), ToastType::Error);
                    }
                }
            });
        }
    };

    view! {
        <div id="content-settings" class="setting-content">
            <div class="content-settings-inner">
                <h3 class="settings-section-title">"Content Filtering"</h3>
                <p class="md_font_size txt-color-gray">
                    "Control whether reported content is visible in your feed."
                </p>

                <div class="reported-content-btn">
                    <span class="md_font_size">"Show reported content"</span>
                    <label class="switch">
                        <input
                            type="checkbox"
                            prop:checked=move || is_lenient.get()
                            prop:disabled=move || is_loading.get()
                            on:click=on_toggle
                        />
                        <span>
                            <span class="label-on md_font_size">"on"</span>
                            <span class="label-off md_font_size">"off"</span>
                        </span>
                    </label>
                </div>

                <Show when=move || is_loading.get()>
                    <p class="md_font_size txt-color-gray">"Loading preferences..."</p>
                </Show>
            </div>

            // Confirmation dialog
            <Show when=move || show_confirm.get()>
                <ContentConfirmDialog
                    is_enabling=pending_state.get()
                    on_confirm=on_confirm.clone()
                    on_cancel=move || show_confirm.set(false)
                />
            </Show>

            // Success modal
            <Show when=move || show_success.get()>
                <ContentSuccessModal
                    message=success_message.get()
                    on_close=move || show_success.set(false)
                />
            </Show>
        </div>
    }
}

/// Confirmation dialog for content filtering toggle.
#[component]
fn ContentConfirmDialog(
    is_enabling: bool,
    on_confirm: impl Fn() + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    let (title, description) = if is_enabling {
        (
            "Show reported content?",
            "You're about to see all reported posts in your feed. This content may be inappropriate or offensive.",
        )
    } else {
        (
            "Hide reported content?",
            "You're about to hide all reported posts from your feed.",
        )
    };

    view! {
        <div class="modal-overlay">
            <div class="logOut-pop">
                <i class="peer-icon peer-icon-warning"></i>
                <h3 class="xxl_font_size bold">{title}</h3>
                <p class="xl_font_size">{description}</p>
                <div class="button-row">
                    <button class="btn-white" on:click=move |_| on_cancel()>
                        "Cancel"
                    </button>
                    <button class="btn-blue" on:click=move |_| on_confirm()>
                        "Confirm"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Success modal after content preference update.
#[component]
fn ContentSuccessModal(message: String, on_close: impl Fn() + 'static) -> impl IntoView {
    view! {
        <div class="modal-overlay">
            <div class="logOut-pop">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="80"
                    height="80"
                    viewBox="0 0 186 186"
                    fill="none"
                >
                    <path
                        d="M93 186C80.135 186 68.045 183.559 56.73 178.676C45.415 173.794 35.573 167.055 27.204 158.461C18.834 149.867 12.322 139.756 7.666 128.127C3.012 116.499 0.46 104.044 0.011 90.763C0.011 77.483 2.563 64.715 7.666 52.458C12.769 40.202 19.73 29.463 28.548 20.242C37.366 11.021 47.657 3.767 59.42 -1.519C71.183 -6.806 83.863 -9.449 97.46 -9.449C103.459 -9.449 109.457 -8.823 115.456 -7.571C121.455 -6.319 127.005 -4.536 132.107 -2.222L118.062 11.334C114.754 10.395 111.221 9.612 107.465 8.986C103.708 8.36 99.952 8.047 96.196 8.047C75.064 8.047 56.955 15.301 41.868 29.809C26.781 44.317 19.238 61.793 19.238 82.236C19.238 102.68 26.781 120.155 41.868 134.663C56.955 149.171 75.064 156.425 96.196 156.425C117.327 156.425 135.436 149.171 150.523 134.663C165.61 120.155 173.153 102.68 173.153 82.236V76.323L186 63.319V82.236C186 95.516 183.448 108.127 178.345 120.071C173.242 132.014 166.281 142.597 157.463 151.818C148.645 161.039 138.354 168.293 126.591 173.58C114.828 178.867 102.148 181.51 88.551 181.51L93 186ZM81.012 127.5L47.882 93.886L60.728 80.882L81.012 101.326L165.835 15.614L178.682 28.618L81.012 127.5Z"
                        fill="#AAFF67"
                    />
                </svg>
                <p class="xl_font_size">{message}</p>
                <div class="button-row">
                    <button class="btn-blue" on:click=move |_| on_close()>
                        "OK"
                    </button>
                </div>
            </div>
        </div>
    }
}
