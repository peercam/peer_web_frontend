//! Action panel — reporters list, moderation actions, confirmation dialogs.

use leptos::prelude::*;

use crate::api::moderation::perform_moderation;
use crate::components::toast::{ToastContext, ToastType};
use crate::models::common::response_codes;
use crate::models::moderation::{ModerationAction, ModerationItem};

/// Right-side detail panel: reporters, action buttons, moderation result.
#[component]
pub fn ActionPanel(
    item: ModerationItem,
    on_moderation_action: Callback<(String, ModerationAction)>,
) -> impl IntoView {
    let ticket_id = item.moderation_ticket_id.clone();
    let status = RwSignal::new(item.status.clone());
    let pending_action = RwSignal::new(None::<ModerationAction>);
    let is_submitting = RwSignal::new(false);
    let reporters = item.reporters.clone();
    let moderated_by = item.moderated_by.clone();

    let ticket_id_for_action = ticket_id.clone();

    let confirm_action = move || {
        let Some(action) = pending_action.get_untracked() else {
            return;
        };
        let tid = ticket_id_for_action.clone();
        is_submitting.set(true);

        leptos::task::spawn_local(async move {
            match perform_moderation(tid.clone(), action).await {
                Ok(response) => match response.response_code.as_str() {
                    response_codes::MODERATION_ACTION_SUCCESS => {
                        status.set(action.to_string());
                        on_moderation_action.run((tid, action));
                        if let Some(toast) = use_context::<ToastContext>() {
                            toast.show("Moderation action performed", ToastType::Success);
                        }
                    }
                    response_codes::MODERATION_ALREADY_TERMINAL => {
                        if let Some(toast) = use_context::<ToastContext>() {
                            toast.show("This ticket has already been resolved", ToastType::Info);
                        }
                    }
                    response_codes::MODERATION_TICKET_NOT_FOUND => {
                        if let Some(toast) = use_context::<ToastContext>() {
                            toast.show("Ticket not found", ToastType::Error);
                        }
                    }
                    _ => {
                        let msg = &response.response_message;
                        if let Some(toast) = use_context::<ToastContext>() {
                            toast.show(format!("Error: {}", msg), ToastType::Error);
                        }
                    }
                },
                Err(e) => {
                    if let Some(toast) = use_context::<ToastContext>() {
                        toast.show(format!("Failed: {}", e), ToastType::Error);
                    }
                }
            }
            pending_action.set(None);
            is_submitting.set(false);
        });
    };

    view! {
        // Status bar
        <div class="conten_status">
            <span>"Status"</span>
            <span class=move || {
                match status.get().as_str() {
                    "waiting_for_review" => "review",
                    "hidden" | "illegal" => "hidden",
                    "restored" => "restored",
                    _ => "",
                }.to_string()
            }>
                {move || match status.get().as_str() {
                    "waiting_for_review" => "Waiting for review",
                    "hidden" => "Hidden",
                    "restored" => "Restored",
                    "illegal" => "Illegal",
                    other => other,
                }.to_string()}
            </span>
        </div>

        // Reporters list
        <div class="reported_by">
            <div class="head">
                <span class="bold">"Reported by"</span>
                <span>{reporters.len()}" reports"</span>
            </div>
            <div class="reported_by_profiles">
                {reporters.iter().map(|reporter| {
                    let img = reporter.img.clone();
                    let username = reporter.username.clone().unwrap_or_default();
                    let slug = reporter.slug.clone().unwrap_or_default();
                    let date = reporter.updatedat.clone().unwrap_or_default();
                    view! {
                        <div class="profile_item">
                            <div class="profile">
                                <div class="profile_image">
                                    {img.map(|url| view! { <img src=url alt="Reporter"/> })}
                                </div>
                                <div class="profile_detail">
                                    <span class="bold">{username}</span>
                                    <span>{format!("@{}", slug)}</span>
                                </div>
                            </div>
                            <div class="report_time">{date}</div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>

        // Action buttons or confirmation dialog
        {move || {
            let current_status = status.get();
            if current_status == "waiting_for_review" {
                match pending_action.get() {
                    None => view! {
                        <div class="action_buttons">
                            <button
                                class="button btn-blue"
                                on:click=move |_| pending_action.set(Some(ModerationAction::Restored))
                            >
                                "Restore"
                            </button>
                            <button
                                class="button btn-transparent"
                                on:click=move |_| pending_action.set(Some(ModerationAction::Hidden))
                            >
                                "Hide"
                            </button>
                            <button
                                class="button btn-transparent btn-red"
                                on:click=move |_| pending_action.set(Some(ModerationAction::Illegal))
                            >
                                "Mark as illegal"
                            </button>
                        </div>
                    }.into_any(),
                    Some(action) => {
                        let (title, description, box_class) = match action {
                            ModerationAction::Restored => (
                                "Are you sure you want to restore this content?",
                                "It will reappear in everyone's feed.",
                                "action_box action_restore",
                            ),
                            ModerationAction::Hidden => (
                                "Are you sure you want to hide this content?",
                                "It will require additional confirmation from users to be shown.",
                                "action_box action_hide",
                            ),
                            ModerationAction::Illegal => (
                                "Are you sure this content is illegal?",
                                "It will never be shown to anyone again.",
                                "action_box action_illegal",
                            ),
                        };
                        let confirm = confirm_action.clone();
                        view! {
                            <div class=box_class>
                                <div class="action_info">
                                    <p class="bold">{title}</p>
                                    <p>{description}</p>
                                </div>
                                <div class="action_buttons">
                                    <button
                                        class="button btn-blue"
                                        disabled=move || is_submitting.get()
                                        on:click=move |_| confirm()
                                    >
                                        {move || if is_submitting.get() { "Processing..." } else { "Yes" }}
                                    </button>
                                    <button
                                        class="button btn-transparent"
                                        disabled=move || is_submitting.get()
                                        on:click=move |_| pending_action.set(None)
                                    >
                                        "No"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }
                }
            } else {
                // Already moderated — show result
                view! {
                    <ModeratedByInfo status=current_status moderated_by=moderated_by.clone()/>
                }.into_any()
            }
        }}
    }
}

/// Shows who moderated the ticket and the action taken.
#[component]
fn ModeratedByInfo(
    status: String,
    moderated_by: Option<crate::models::moderation::BasicUserInfo>,
) -> impl IntoView {
    let action_class = match status.as_str() {
        "restored" => "moderated_action restored",
        "hidden" => "moderated_action hidden",
        "illegal" => "moderated_action illegal",
        _ => "moderated_action",
    };

    let action_label = match status.as_str() {
        "restored" => "Restored",
        "hidden" => "Hidden",
        "illegal" => "Marked as illegal",
        other => other,
    }
    .to_string();

    view! {
        <div class="moderated_by_box">
            <span class="bold">"Moderated by"</span>
            {moderated_by.map(|moderator| {
                let img = moderator.img.clone();
                let username = moderator.username.clone().unwrap_or_else(|| "Unknown".to_string());
                let slug = moderator.slug.clone().unwrap_or_default();
                view! {
                    <div class="moderated_info">
                        <div class="profile">
                            <div class="profile_image">
                                {img.map(|url| view! { <img src=url alt="Moderator"/> })}
                            </div>
                            <div class="profile_detail">
                                <span class="bold">{username}</span>
                                <span>{format!("@{}", slug)}</span>
                            </div>
                        </div>
                    </div>
                }
            })}
            <div class=action_class>
                <span>{action_label}</span>
            </div>
        </div>
    }
}
