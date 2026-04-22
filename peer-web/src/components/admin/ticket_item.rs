//! Single moderation ticket row with expandable detail.

use leptos::prelude::*;

use crate::components::admin::action_panel::ActionPanel;
use crate::components::admin::content_preview::ContentPreview;
use crate::models::moderation::{ModerationAction, ModerationItem};

/// A single ticket row that expands to show detail.
#[component]
pub fn TicketItem(
    item: ModerationItem,
    expanded_ticket: RwSignal<Option<String>>,
    on_moderation_action: Callback<(String, ModerationAction)>,
) -> impl IntoView {
    let ticket_id = item.moderation_ticket_id.clone();
    let ticket_id_display = format!("#{}", &ticket_id[..8.min(ticket_id.len())]);
    let ticket_id_for_click = ticket_id.clone();

    let is_expanded = {
        let ticket_id_for_expand = ticket_id.clone();
        move || expanded_ticket.get().as_deref() == Some(&ticket_id_for_expand)
    };
    let is_expanded_class = {
        let ticket_id_for_expand2 = ticket_id.clone();
        move || expanded_ticket.get().as_deref() == Some(&ticket_id_for_expand2)
    };

    let toggle_expand = move |_| {
        expanded_ticket.update(|current| {
            if current.as_deref() == Some(&ticket_id_for_click) {
                *current = None;
            } else {
                *current = Some(ticket_id_for_click.clone());
            }
        });
    };

    // Determine content thumbnail & type info
    let (thumbnail, content_label, type_icon) = extract_content_info(&item);

    let status_class = match item.status.as_str() {
        "waiting_for_review" => "review",
        "hidden" | "illegal" => "hidden",
        "restored" => "restored",
        _ => "",
    };

    let status_label = match item.status.as_str() {
        "waiting_for_review" => "Waiting for review",
        "hidden" => "Hidden",
        "restored" => "Restored",
        "illegal" => "Illegal",
        other => other,
    }
    .to_string();

    let reports_count = item.reportscount;
    let reports_class = if reports_count >= 5 {
        "high-reports"
    } else {
        ""
    };

    let date_display = format_moderation_date(&item.createdat);

    let item_for_preview = item.clone();
    let item_for_action = item.clone();

    view! {
        <div class="content_item" class:expanded=is_expanded_class>
            // Summary row
            <div class="content_item_inner" on:click=toggle_expand style="cursor: pointer;">
                <div class="content">
                    <div class="content_image">
                        {thumbnail.map(|url| view! {
                            <img src=url alt="Content thumbnail"/>
                        })}
                        <span class=format!("peer-icon {}", type_icon)></span>
                    </div>
                    <div class="content_detail">
                        <span class="content-label">{content_label}</span>
                    </div>
                </div>
                <div class="moderation_id">
                    <span>{ticket_id_display}</span>
                </div>
                <div class="moderation_date">
                    <span>{date_display}</span>
                </div>
                <div class=format!("reports {}", reports_class)>
                    <span>{reports_count}</span>
                    <span class="peer-icon peer-icon-flag"></span>
                </div>
                <div class="status">
                    <span class=status_class>{status_label}</span>
                </div>
            </div>

            // Expandable detail
            <Show when=is_expanded>
                <div class="content_box">
                    <div class="content_box_left">
                        <ContentPreview item=item_for_preview.clone()/>
                    </div>
                    <div class="content_box_right">
                        <ActionPanel
                            item=item_for_action.clone()
                            on_moderation_action=on_moderation_action
                        />
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Extract thumbnail URL, label, and icon from the moderation item.
fn extract_content_info(item: &ModerationItem) -> (Option<String>, String, &'static str) {
    match item.targettype.as_str() {
        "post" => {
            if let Some(post) = &item.targetcontent.post {
                let thumb = post.cover.clone().or_else(|| post.media.clone());
                let user_name = post
                    .user
                    .as_ref()
                    .and_then(|u| u.username.clone())
                    .unwrap_or_default();
                let title = post.title.clone().unwrap_or_default();
                let label = if title.is_empty() {
                    user_name
                } else {
                    format!("{} — {}", user_name, title)
                };
                let icon = match post.contenttype.as_deref() {
                    Some("IMAGE") => "peer-icon-camera",
                    Some("VIDEO") => "peer-icon-play",
                    Some("AUDIO") => "peer-icon-audio",
                    _ => "peer-icon-text",
                };
                (thumb, label, icon)
            } else {
                (None, "Post".to_string(), "peer-icon-text")
            }
        }
        "comment" => {
            if let Some(comment) = &item.targetcontent.comment {
                let user_name = comment
                    .user
                    .as_ref()
                    .and_then(|u| u.username.clone())
                    .unwrap_or_default();
                let content = comment
                    .content
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(50)
                    .collect::<String>();
                let label = format!("{} — {}", user_name, content);
                let thumb = comment.user.as_ref().and_then(|u| u.img.clone());
                (thumb, label, "peer-icon-comment")
            } else {
                (None, "Comment".to_string(), "peer-icon-comment")
            }
        }
        "user" => {
            if let Some(user) = &item.targetcontent.user {
                let label = user.username.clone().unwrap_or_else(|| "User".to_string());
                let thumb = user.img.clone();
                (thumb, label, "peer-icon-profile")
            } else {
                (None, "User".to_string(), "peer-icon-profile")
            }
        }
        _ => (None, "Unknown".to_string(), "peer-icon-warning"),
    }
}

/// Format a date string for display in the moderation table.
///
/// Parses ISO-8601 / RFC-3339 timestamps and formats as "DD Mon YYYY, HH:MM".
/// Falls back to first 10 characters (YYYY-MM-DD) if parsing fails.
fn format_moderation_date(date_str: &str) -> String {
    static MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    // Try to parse "YYYY-MM-DDTHH:MM:SS" (with or without timezone/fractional seconds)
    let parts: Vec<&str> = date_str.splitn(2, 'T').collect();
    if parts.len() == 2 {
        let date_part = parts[0];
        let time_part = parts[1];
        let date_fields: Vec<&str> = date_part.split('-').collect();
        let time_fields: Vec<&str> = time_part.split(':').collect();

        if date_fields.len() == 3 && time_fields.len() >= 2
            && let (Ok(year), Ok(month), Ok(day)) = (
                date_fields[0].parse::<u32>(),
                date_fields[1].parse::<usize>(),
                date_fields[2].parse::<u32>(),
            )
                && (1..=12).contains(&month) {
                    let hour_min = format!("{}:{}", time_fields[0], time_fields[1]);
                    return format!("{} {} {}, {}", day, MONTHS[month - 1], year, hour_min);
                }
    }

    // Fallback: show YYYY-MM-DD
    if date_str.len() >= 10 {
        date_str[..10].to_string()
    } else {
        date_str.to_string()
    }
}
