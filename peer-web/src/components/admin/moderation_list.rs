//! Moderation ticket list with infinite scroll.

use leptos::prelude::*;

use crate::api::moderation::get_moderation_items;
use crate::components::admin::filter_bar::FilterBar;
use crate::components::admin::stats_header::StatsHeader;
use crate::components::admin::ticket_item::TicketItem;
use crate::models::moderation::{ModerationAction, ModerationItem};

const PAGE_SIZE: i32 = 20;

/// The main moderation list with filters, stats, and ticket items.
#[component]
pub fn ModerationList() -> impl IntoView {
    let active_filter = RwSignal::new(None::<String>);
    let review_only = RwSignal::new(false);
    let items = RwSignal::new(Vec::<ModerationItem>::new());
    let offset = RwSignal::new(0i32);
    let has_more = RwSignal::new(true);
    let is_loading = RwSignal::new(false);
    let expanded_ticket = RwSignal::new(None::<String>);
    let stats_version = RwSignal::new(0u32);

    let load_items = move |reset: bool| {
        if reset {
            items.set(Vec::new());
            offset.set(0);
            has_more.set(true);
        }
        if is_loading.get_untracked() || !has_more.get_untracked() {
            return;
        }
        is_loading.set(true);
        let content_type = active_filter.get_untracked();
        let status = if review_only.get_untracked() {
            Some("waiting_for_review".to_string())
        } else {
            None
        };
        let current_offset = offset.get_untracked();

        leptos::task::spawn_local(async move {
            match get_moderation_items(content_type, status, current_offset, PAGE_SIZE).await {
                Ok(response) => {
                    let new_items = response.affected_rows;
                    if new_items.len() < PAGE_SIZE as usize {
                        has_more.set(false);
                    }
                    offset.update(|o| *o += new_items.len() as i32);
                    items.update(|list| list.extend(new_items));
                }
                Err(_) => {
                    has_more.set(false);
                }
            }
            is_loading.set(false);
        });
    };

    // Reset and reload when filters change (also triggers initial load)
    Effect::new(move |_| {
        let _filter = active_filter.get();
        let _review = review_only.get();
        load_items(true);
    });

    let scroll = crate::hooks::use_infinite_scroll(is_loading, has_more, move || load_items(false));

    let on_moderation_action = move |(ticket_id, action): (String, ModerationAction)| {
        // Optimistically update the item status
        items.update(|list| {
            if let Some(item) = list
                .iter_mut()
                .find(|i| i.moderation_ticket_id == ticket_id)
            {
                item.status = action.to_string();
            }
        });
        // Bump stats version to refresh
        stats_version.update(|v| *v += 1);
    };

    view! {
        <StatsHeader stats_version=stats_version.read_only()/>
        <FilterBar active_filter=active_filter review_only=review_only/>

        <div class="content_list">
            // Table header
            <div class="content_item head">
                <div class="content_item_inner">
                    <div class="content">"Content"</div>
                    <div class="moderation_id">"Moderation ID"</div>
                    <div class="moderation_date">"Moderation date"</div>
                    <div class="reports">"Reports"</div>
                    <div class="status">"Status"</div>
                </div>
            </div>

            // Item list
            <For
                each=move || items.get()
                key=|item| item.moderation_ticket_id.clone()
                children=move |item| {
                    view! {
                        <TicketItem
                            item=item
                            expanded_ticket=expanded_ticket
                            on_moderation_action=Callback::new(on_moderation_action)
                        />
                    }
                }
            />

            // Loading state
            <Show when=move || is_loading.get()>
                <TicketListSkeleton/>
            </Show>

            // Empty state
            <Show when=move || !is_loading.get() && items.get().is_empty()>
                <div class="content_item empty-state">
                    <p>"No items found"</p>
                </div>
            </Show>

            // Infinite scroll sentinel
            <div node_ref=scroll.loader_ref class="scroll-sentinel"></div>
        </div>
    }
}

/// Skeleton rows for the ticket list.
#[component]
fn TicketListSkeleton() -> impl IntoView {
    view! {
        {(0..5).map(|_| view! {
            <div class="content_item skeleton">
                <div class="content_item_inner">
                    <div class="content"><div class="skeleton-text">"Loading..."</div></div>
                    <div class="moderation_id"><div class="skeleton-text">"..."</div></div>
                    <div class="moderation_date"><div class="skeleton-text">"..."</div></div>
                    <div class="reports"><div class="skeleton-text">"..."</div></div>
                    <div class="status"><div class="skeleton-text">"..."</div></div>
                </div>
            </div>
        }).collect_view()}
    }
}
