//! Profile settings tab component.
//!
//! Displays and allows editing of profile picture, biography, and username.
//! Contains sub-panels for changing username, password, and email.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::profile::fetch_biography;
use crate::api::settings::{update_bio, update_profile_image};
use crate::components::settings::{
    ChangeEmailPanel, ChangePasswordPanel, ChangeUsernamePanel, ImageUploadModal,
};
use crate::components::toast::{ToastContext, ToastType, use_toast};
use crate::models::profile::Profile;

/// Active sub-panel within profile settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProfileSubPanel {
    Main,
    ChangeUsername,
    ChangePassword,
    ChangeEmail,
}

/// Profile Settings tab content.
#[component]
pub fn ProfileSettings(
    /// Resource containing the user's profile data.
    profile_resource: Resource<Result<Profile, ServerFnError>>,
) -> impl IntoView {
    let active_panel = RwSignal::new(ProfileSubPanel::Main);

    view! {
        <div id="profile-settings" class="setting-content">
            <Suspense fallback=move || view! { <ProfileSettingsSkeleton/> }>
                {move || profile_resource.get().map(|result| match result {
                    Ok(profile) => view! {
                        <Show when=move || active_panel.get() == ProfileSubPanel::Main>
                            <MainProfilePanel
                                profile=profile.clone()
                                on_change_username=move || active_panel.set(ProfileSubPanel::ChangeUsername)
                                on_change_password=move || active_panel.set(ProfileSubPanel::ChangePassword)
                                on_change_email=move || active_panel.set(ProfileSubPanel::ChangeEmail)
                            />
                        </Show>
                        <Show when=move || active_panel.get() == ProfileSubPanel::ChangeUsername>
                            <ChangeUsernamePanel on_back=move || active_panel.set(ProfileSubPanel::Main)/>
                        </Show>
                        <Show when=move || active_panel.get() == ProfileSubPanel::ChangePassword>
                            <ChangePasswordPanel on_back=move || active_panel.set(ProfileSubPanel::Main)/>
                        </Show>
                        <Show when=move || active_panel.get() == ProfileSubPanel::ChangeEmail>
                            <ChangeEmailPanel on_back=move || active_panel.set(ProfileSubPanel::Main)/>
                        </Show>
                    }.into_any(),
                    Err(_) => view! {
                        <p class="error">"Failed to load profile. Please try again."</p>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

/// Skeleton loader for profile settings.
#[component]
fn ProfileSettingsSkeleton() -> impl IntoView {
    view! {
        <div class="profile-widget">
            <div class="edit-profile">
                <div class="profile_picture">
                    <div class="skeleton-circle" style="width: 150px; height: 150px;"></div>
                </div>
                <div class="profile-fields">
                    <div class="skeleton-block" style="height: 120px;"></div>
                    <div class="skeleton-block" style="height: 40px; width: 200px;"></div>
                </div>
            </div>
        </div>
    }
}

/// Handle combined save results from parallel bio + image updates.
fn handle_save_results(
    bio_result: Result<(), ServerFnError>,
    img_result: Result<(), ServerFnError>,
    toast: ToastContext,
    response_msg: RwSignal<Option<(String, bool)>>,
) {
    match (&bio_result, &img_result) {
        (Ok(()), Ok(())) => {
            toast.show("Profile updated successfully!", ToastType::Success);
            response_msg.set(Some(("Profile saved.".to_string(), true)));
        }
        _ => {
            let mut errors = Vec::new();
            if let Err(e) = bio_result {
                errors.push(format!("Bio: {}", e));
            }
            if let Err(e) = img_result {
                errors.push(format!("Image: {}", e));
            }
            toast.show(errors.join(", "), ToastType::Error);
            response_msg.set(Some(("Save failed.".to_string(), false)));
        }
    }
}

/// Main profile panel: avatar, bio, username display, save button.
#[component]
fn MainProfilePanel(
    profile: Profile,
    on_change_username: impl Fn() + 'static,
    on_change_password: impl Fn() + 'static,
    on_change_email: impl Fn() + 'static,
) -> impl IntoView {
    let toast = use_toast();
    let biography = RwSignal::new(String::new());
    let image_data = RwSignal::new(Option::<String>::None);
    let avatar_src = RwSignal::new(profile.avatar_url().to_string());
    let is_saving = RwSignal::new(false);
    let show_image_modal = RwSignal::new(false);
    let response_msg = RwSignal::new(Option::<(String, bool)>::None);

    // Load biography text from remote URL
    let bio_url = profile.biography.clone();
    if let Some(url) = bio_url {
        spawn_local(async move {
            if let Ok(text) = fetch_biography(url).await {
                biography.set(text);
            }
        });
    }

    let on_save = {
        move |_| {
            if is_saving.get() {
                return;
            }
            is_saving.set(true);
            response_msg.set(None);

            let bio_text = biography.get();
            let img = image_data.get();
            let toast = toast;

            spawn_local(async move {
                let img_result = match img {
                    Some(img_data) => update_profile_image(img_data).await,
                    None => Ok(()),
                };
                let bio_result = update_bio(bio_text).await;
                handle_save_results(bio_result, img_result, toast, response_msg);
                is_saving.set(false);
            });
        }
    };

    view! {
        <div class="profile-widget active">
            <div class="edit-profile">
                // Avatar section
                <div class="profile_picture">
                    <div class="cropContainer">
                        <img
                            class="profile-picture my-profile-picture"
                            src=move || avatar_src.get()
                            alt="Profile Picture"
                        />
                    </div>
                    <a
                        href="#"
                        class="button change-picture"
                        on:click=move |e| {
                            e.prevent_default();
                            show_image_modal.set(true);
                        }
                    >
                        "Change Picture"
                    </a>
                </div>

                // Biography
                <div class="profile-fields">
                    <div class="input-field transparent">
                        <label>"Description"</label>
                        <textarea
                            cols="40"
                            rows="5"
                            maxlength="5000"
                            class="input-textarea"
                            placeholder="Write a description to your profile..."
                            prop:value=move || biography.get()
                            on:input=move |ev| biography.set(event_target_value(&ev))
                        />
                    </div>

                    // Response message
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

                    // Username display
                    <div class="input-field username-row transparent">
                        <label>"Username"</label>
                        <span>
                            "@"
                            <span>{profile.username.clone()}</span>
                        </span>
                        <a
                            href="#"
                            on:click=move |e| {
                                e.prevent_default();
                                on_change_username();
                            }
                        >
                            "Change"
                        </a>
                    </div>
                </div>

                <button
                    class="full-width-btn btn-white"
                    prop:disabled=move || is_saving.get()
                    on:click=on_save
                >
                    {move || if is_saving.get() { "Saving..." } else { "Save Changes" }}
                </button>
            </div>

            // Credential change buttons
            <div class="profile-setting-buttons">
                <div class="change-password-email">
                    <a
                        href="#"
                        class="button change-pass-btn btn-transparent"
                        on:click=move |e| {
                            e.prevent_default();
                            on_change_password();
                        }
                    >
                        "Change password"
                    </a>
                    <a
                        href="#"
                        class="button change-email-btn btn-transparent"
                        on:click=move |e| {
                            e.prevent_default();
                            on_change_email();
                        }
                    >
                        "Change e-mail"
                    </a>
                </div>
            </div>

            // Image upload modal
            <Show when=move || show_image_modal.get()>
                <ImageUploadModal
                    on_apply=move |data_url: String| {
                        avatar_src.set(data_url.clone());
                        image_data.set(Some(data_url));
                        show_image_modal.set(false);
                    }
                    on_cancel=move || show_image_modal.set(false)
                />
            </Show>
        </div>
    }
}
