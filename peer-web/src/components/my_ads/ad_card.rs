//! Individual ad card component.
//!
//! Displays a single advertisement with post info, status badge,
//! timeframe, and expandable detail panel.

use leptos::prelude::*;

use crate::models::advertisement::{Advertisement, AdvertisementType};
use crate::models::post::ContentType;

/// Format a date string for display (e.g., "14 Apr 2026, 10:30").
fn format_ad_date(date_str: &str) -> String {
    // Try parsing ISO 8601 date
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.format("%d %b %Y, %H:%M").to_string()
    } else {
        // Fallback: return as-is
        date_str.to_string()
    }
}

/// Format a number for display.
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

/// Single ad card with expandable detail panel.
#[component]
pub fn AdCard(
    /// The advertisement data.
    ad: Advertisement,
    /// Animation delay index for stagger effect.
    #[prop(default = 0)]
    index: usize,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let is_active = ad.is_active();
    let is_pinned = ad.ad_type == AdvertisementType::Pinned;

    let visibility_status = ad.post.visibility_status.clone().unwrap_or_default();
    let is_hidden = ad.post.is_hidden_for_users.unwrap_or(false);
    let is_illegal = visibility_status == "ILLEGAL";

    let post_title = ad.post.title.clone();
    let post_desc = ad.post.mediadescription.clone().unwrap_or_default();
    let content_type = ad.post.contenttype;

    // Thumbnail: use cover or media, fallback to placeholder
    let thumbnail_url = ad.post.cover.clone()
        .or_else(|| ad.post.media.clone());

    let start_date = format_ad_date(&ad.timeframe_start);
    let end_date = format_ad_date(&ad.timeframe_end);

    // Per-ad stats for the dropdown
    let gems_earned = format_number(ad.gems_earned);
    let total_cost = format_number(ad.total_token_cost);
    let likes = ad.amount_likes;
    let dislikes = ad.amount_dislikes;
    let comments = ad.amount_comments;
    let views = ad.amount_views;
    let reports = ad.amount_reports;

    let ad_start = ad.timeframe_start.clone();
    let ad_end = ad.timeframe_end.clone();
    let detail_start = format_ad_date(&ad_start);
    let detail_end = format_ad_date(&ad_end);

    let stagger_delay = format!("{}ms", index * 100);

    view! {
        <div
            class="my-ads-list-item"
            class:active=is_active
            class:ended=!is_active
            class:illegal-ads-post=is_illegal
            style=format!("animation-delay: {stagger_delay}")
            on:click=move |_| expanded.update(|v| *v = !*v)
        >
            // Main info row
            <div class="ad-main-info">
                <div class="ad-info">
                    // Thumbnail
                    <div class="ad-avatar">
                        {if is_illegal {
                            view! {
                                <div class="illegal-ads-post-frame xl-font-size">
                                    <div class="illegal-content">
                                        <div class="icon-illegal">
                                            <i class="peer-icon peer-icon-warning"></i>
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            match thumbnail_url.clone() {
                                Some(url) => view! {
                                    <img class="post-image" src=url alt="Post thumbnail"/>
                                }.into_any(),
                                None => view! {
                                    <div class="post-image-placeholder"></div>
                                }.into_any(),
                            }
                        }}

                        // Content type icon
                        {match content_type {
                            ContentType::Video => view! {
                                <i class="peer-icon peer-icon-play-btn"></i>
                            }.into_any(),
                            ContentType::Audio => view! {
                                <i class="peer-icon peer-icon-audio"></i>
                            }.into_any(),
                            ContentType::Image => view! {
                                <i class="peer-icon peer-icon-camera"></i>
                            }.into_any(),
                            ContentType::Text => view! {
                                <i class="peer-icon peer-icon-text"></i>
                            }.into_any(),
                        }}

                        // Pinned badge
                        {is_pinned.then(|| view! {
                            <div class="pin-badge">
                                <img src="/svg/pin-icon.svg" alt="Pinned"/>
                            </div>
                        })}
                    </div>

                    // Title and description
                    <div class="ad-details">
                        {if is_illegal {
                            view! {
                                <span class="ad-title">"Content removed"</span>
                                <span class="ad-description">"This post has been removed for policy violations"</span>
                            }.into_any()
                        } else {
                            view! {
                                <span class="ad-title">{post_title}</span>
                                <span class="ad-description">{post_desc}</span>
                            }.into_any()
                        }}

                        // Hidden badge
                        {is_hidden.then(|| view! {
                            <div class="ad-hidden-badge">
                                <i class="peer-icon peer-icon-eye-close"></i>
                                <span class="ads-hidden-texts">"This post is hidden"</span>
                            </div>
                        })}
                    </div>
                </div>

                // Timeframe and status
                <div class="time-badge-status">
                    <div class="ad-timeframe-box">
                        <div>
                            <span class="ad-timer">{start_date}</span>
                        </div>
                        <hr/>
                        <div>
                            <span class="ad-timer">{end_date}</span>
                        </div>
                    </div>

                    <div class="ad-status">
                        <div class=move || {
                            if is_active { "status-badge active" } else { "status-badge ended" }
                        }>
                            <div class="status-dot"></div>
                            <span>{if is_active { "Active" } else { "Ended" }}</span>
                        </div>
                    </div>
                </div>
            </div>

            // Expandable dropdown panel
            <div class="ad-dropdown" class:open=move || expanded.get()>
                <div class="ad-dropdown-content">
                    // Per-ad stats
                    <div class="my-ads-header">
                        <div class="interactions-box header-box" style="background: var(--background-color)">
                            <div class="interactions-data">
                                <div class="interaction-item">
                                    <img src="/svg/peer-icon-gems.svg" alt="gems"/>
                                    <span class="bold">{gems_earned.clone()}</span>
                                    <p>"Gems"</p>
                                </div>
                                <div class="vr"></div>
                                <div class="interaction-item">
                                    <i class="peer-icon peer-icon-like"></i>
                                    <span>{likes}</span>
                                    <p>"Likes"</p>
                                </div>
                                <div class="vr"></div>
                                <div class="interaction-item">
                                    <i class="peer-icon peer-icon-dislike"></i>
                                    <span>{dislikes}</span>
                                    <p>"Dislikes"</p>
                                </div>
                                <div class="vr"></div>
                                <div class="interaction-item">
                                    <i class="peer-icon peer-icon-comment-alt"></i>
                                    <span>{comments}</span>
                                    <p>"Comments"</p>
                                </div>
                                <div class="vr"></div>
                                <div class="interaction-item">
                                    <i class="peer-icon peer-icon-eye-open"></i>
                                    <span>{views}</span>
                                    <p>"Views"</p>
                                </div>
                                <div class="vr"></div>
                                <div class="interaction-item">
                                    <i class="peer-icon peer-icon-warning"></i>
                                    <span>{reports}</span>
                                    <p>"Reports"</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Campaign details
                    <div class="campaign-details">
                        <div class="detail-item">
                            <span class="detail-title">"Start date"</span>
                            <span class="detail-value">{detail_start}</span>
                        </div>
                        <div class="detail-item">
                            <span class="detail-title">"End date"</span>
                            <span class="detail-value">{detail_end}</span>
                        </div>
                        <div class="detail-item">
                            <span class="detail-title">"Total cost"</span>
                            <div class="detail-value">
                                <div class="ads-tokens-count">
                                    <img src="/svg/logo_sw.svg" alt="tokens"/>
                                    <span>{total_cost.clone()}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
