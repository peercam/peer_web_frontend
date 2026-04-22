//! Stats header component for the My Ads page.
//!
//! Displays aggregated campaign statistics: earnings, spendings, and interactions.

use leptos::prelude::*;

use crate::models::advertisement::AdHistoryStats;

/// Format a number for display (1.2M, 1.2K, or raw).
fn format_number(num: f64) -> String {
    if num >= 1_000_000.0 {
        format!("{:.1}M", num / 1_000_000.0)
    } else if num >= 1_000.0 {
        format!("{:.1}K", num / 1_000.0)
    } else if num.fract() == 0.0 {
        format!("{}", num as i64)
    } else {
        format!("{:.1}", num)
    }
}

/// Format an integer with the same rules.
fn format_int(num: i32) -> String {
    format_number(num as f64)
}

/// Stats header skeleton (loading state).
#[component]
pub fn StatsHeaderSkeleton() -> impl IntoView {
    view! {
        <div class="my-ads-header skeleton">
            <div class="earnings-box header-box">
                <p class="skeleton-text">"Earnings"</p>
                <div class="ads-gems-count">
                    <span class="skeleton-text bold xl-font-size">"---"</span>
                </div>
            </div>
            <div class="spendings-box header-box">
                <p class="skeleton-text">"Spendings"</p>
                <div class="ads-tokens-count">
                    <span class="skeleton-text bold xl-font-size">"---"</span>
                </div>
            </div>
            <div class="interactions-box header-box">
                <div class="interactions-data">
                    {(0..5).map(|_| view! {
                        <div class="interaction-item">
                            <i class="peer-icon skeleton-icon"></i>
                            <span class="skeleton-text">"--"</span>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

/// Aggregated statistics header.
#[component]
pub fn StatsHeader(
    /// The aggregated stats to display.
    stats: AdHistoryStats,
) -> impl IntoView {
    view! {
        <div class="my-ads-header" id="myAds_header">
            // Earnings
            <div class="earnings-box header-box">
                <p>"Earnings"</p>
                <div class="ads-gems-count">
                    <img src="/svg/peer-icon-gems.svg" alt="gems"/>
                    <span class="bold xl-font-size">{format_number(stats.gems_earned)}</span>
                </div>
            </div>

            // Spendings
            <div class="spendings-box header-box">
                <p>"Spendings"</p>
                <div class="ads-tokens-count">
                    <img src="/svg/logo_sw.svg" alt="tokens"/>
                    <span class="bold xl-font-size">{format_number(stats.token_spent)}</span>
                </div>
            </div>

            // Interactions
            <div class="interactions-box header-box">
                <div class="interactions-data">
                    <div class="interaction-item likes">
                        <i class="peer-icon peer-icon-like"></i>
                        <span>{format_int(stats.amount_likes)}</span>
                        <p>"Likes"</p>
                    </div>
                    <div class="vr"></div>
                    <div class="interaction-item dislikes">
                        <i class="peer-icon peer-icon-dislike"></i>
                        <span>{format_int(stats.amount_dislikes)}</span>
                        <p>"Dislikes"</p>
                    </div>
                    <div class="vr"></div>
                    <div class="interaction-item comments">
                        <i class="peer-icon peer-icon-comment-alt"></i>
                        <span>{format_int(stats.amount_comments)}</span>
                        <p>"Comments"</p>
                    </div>
                    <div class="vr"></div>
                    <div class="interaction-item views">
                        <i class="peer-icon peer-icon-eye-open"></i>
                        <span>{format_int(stats.amount_views)}</span>
                        <p>"Views"</p>
                    </div>
                    <div class="vr"></div>
                    <div class="interaction-item reports">
                        <i class="peer-icon peer-icon-warning"></i>
                        <span>{format_int(stats.amount_reports)}</span>
                        <p>"Reports"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
