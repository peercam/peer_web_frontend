//! Stats header component — four stat boxes with live counts.

use leptos::prelude::*;

use crate::api::moderation::get_moderation_stats;

/// Displays the four moderation stat boxes.
#[component]
pub fn StatsHeader(
    /// Trigger signal to re-fetch stats (bumped after moderation actions).
    stats_version: ReadSignal<u32>,
) -> impl IntoView {
    let stats = Resource::new(
        move || stats_version.get(),
        |_| async move { get_moderation_stats().await.ok() },
    );

    view! {
        <Suspense fallback=move || view! { <StatsHeaderSkeleton/> }>
            {move || {
                stats.get().map(|maybe_stats| {
                    match maybe_stats.and_then(|s| s.affected_rows) {
                        Some(data) => view! {
                            <div class="main_stats">
                                <StatBox
                                    label="Awaiting Review"
                                    count=data.amount_awaiting_review
                                    icon="peer-icon-warning"
                                    color_class="stat-yellow"
                                />
                                <StatBox
                                    label="Hidden"
                                    count=data.amount_hidden
                                    icon="peer-icon-eye-close"
                                    color_class="stat-red"
                                />
                                <StatBox
                                    label="Restored"
                                    count=data.amount_restored
                                    icon="peer-icon-checkmark"
                                    color_class="stat-green"
                                />
                                <StatBox
                                    label="Illegal"
                                    count=data.amount_illegal
                                    icon="peer-icon-close"
                                    color_class="stat-red"
                                />
                            </div>
                        }.into_any(),
                        None => view! {
                            <div class="main_stats">
                                <p class="error-text">"Failed to load stats"</p>
                            </div>
                        }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}

/// A single stat box.
#[component]
fn StatBox(
    label: &'static str,
    count: i32,
    icon: &'static str,
    color_class: &'static str,
) -> impl IntoView {
    view! {
        <div class=format!("stat_box {}", color_class)>
            <div class="stat_head">
                <span class="xl_font_size">{label}</span>
                <span class=format!("peer-icon {}", icon)></span>
            </div>
            <span class="xxl_font_size stat_count">{count}</span>
        </div>
    }
}

/// Skeleton loading state for the stats header.
#[component]
fn StatsHeaderSkeleton() -> impl IntoView {
    view! {
        <div class="main_stats skeleton">
            <div class="stat_box"><div class="skeleton-text">"Loading..."</div></div>
            <div class="stat_box"><div class="skeleton-text">"Loading..."</div></div>
            <div class="stat_box"><div class="skeleton-text">"Loading..."</div></div>
            <div class="stat_box"><div class="skeleton-text">"Loading..."</div></div>
        </div>
    }
}
