//! Ad list component with infinite scroll.
//!
//! Manages the paginated advertisement listing with active-first sorting
//! and IntersectionObserver-based infinite scroll.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::ads::get_ad_history;
use crate::components::my_ads::ad_card::AdCard;
use crate::components::my_ads::empty_state::EmptyState;
use crate::components::my_ads::stats_header::{StatsHeader, StatsHeaderSkeleton};
use crate::hooks::use_infinite_scroll;
use crate::models::advertisement::{AdHistoryStats, Advertisement};

/// Number of ads to load per batch.
const ADS_PER_PAGE: i32 = 20;

/// Ad list with infinite scroll and stats header.
#[component]
pub fn AdList() -> impl IntoView {
    let ads = RwSignal::new(Vec::<Advertisement>::new());
    let stats = RwSignal::new(Option::<AdHistoryStats>::None);
    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let initial_loaded = RwSignal::new(false);

    // Load ads function
    let load_ads = move || {
        if is_loading.get() || !has_more.get() {
            return;
        }

        is_loading.set(true);
        let current_offset = offset.get();

        spawn_local(async move {
            match get_ad_history(None, current_offset, ADS_PER_PAGE).await {
                Ok(response) => {
                    if let Some(result) = response.affected_rows {
                        // Set stats on first load
                        if current_offset == 0 {
                            stats.set(Some(result.stats));
                        }

                        let new_ads = result.advertisements;
                        let has_new = !new_ads.is_empty();

                        if has_new {
                            ads.update(|a| a.extend(new_ads));
                            offset.update(|o| *o += ADS_PER_PAGE);
                        }

                        // Check if there are more ads
                        let total = stats.get().map(|s| s.amount_ads).unwrap_or(0);
                        has_more.set(has_new && (current_offset + ADS_PER_PAGE) < total);
                    } else {
                        // No affected_rows — no ads at all
                        if current_offset == 0 {
                            stats.set(Some(AdHistoryStats::default()));
                        }
                        has_more.set(false);
                    }
                    initial_loaded.set(true);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to load ad history: {:?}", e);
                    initial_loaded.set(true);
                }
            }

            is_loading.set(false);
        });
    };

    // Set up infinite scroll via shared hook
    let scroll = use_infinite_scroll(is_loading, has_more, load_ads);

    // Initial load
    Effect::new(move |_| {
        if ads.get().is_empty() && !is_loading.get() && !initial_loaded.get() {
            load_ads();
        }
    });

    // Sort ads: active first, then ended (preserving order within groups)
    let get_sorted_ads = move || {
        let mut all = ads.get();
        all.sort_by(|a, b| b.is_active().cmp(&a.is_active()));
        all
    };

    let get_indexed_ads = move || {
        get_sorted_ads().into_iter().enumerate().collect::<Vec<_>>()
    };

    view! {
        // Stats header
        {move || {
            match stats.get() {
                Some(s) => view! { <StatsHeader stats=s/> }.into_any(),
                None if !initial_loaded.get() => view! { <StatsHeaderSkeleton/> }.into_any(),
                None => view! { <StatsHeaderSkeleton/> }.into_any(),
            }
        }}

        // Ad listing
        <div class="my-ads-main">
            <div class="top">
                <h2 class="xxl-font-size">"All advertisements"</h2>
                <span>{move || {
                    stats.get().map(|s| format!("Total: {}", s.amount_ads)).unwrap_or_default()
                }}</span>
            </div>

            // Show empty state or ad list
            {move || {
                let ad_list = get_sorted_ads();
                if ad_list.is_empty() && initial_loaded.get() {
                    view! { <EmptyState/> }.into_any()
                } else {
                    view! {
                        <div class="my-ads-lists">
                            <For
                                each=move || get_indexed_ads()
                                key=|(_, ad)| ad.id.clone()
                                children=move |(i, ad)| {
                                    view! { <AdCard ad=ad index=i/> }
                                }
                            />
                        </div>
                    }.into_any()
                }
            }}
        </div>

        // Infinite scroll sentinel
        <div node_ref=scroll.loader_ref class="ad-loader">
            <Show when=move || is_loading.get()>
                <div class="loading-indicator">
                    <img src="/svg/logo_farbe.svg" alt="Loading..." class="loading-spinner"/>
                </div>
            </Show>
        </div>
    }
}
