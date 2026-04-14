//! Referral tabs component.
//!
//! Tab navigation for switching between "Invited Friends" and "My Inviter" views.

use leptos::prelude::*;

use crate::components::referral_board::referral_user_grid::ReferralUserGrid;
use crate::models::referral::ReferralListResponse;
use crate::pages::referral_board::ReferralTab;

/// Tab navigation with content switching for the referral board.
#[component]
pub fn ReferralTabs(
    /// Signal controlling the active tab.
    active_tab: RwSignal<ReferralTab>,
    /// The referral list data.
    referral_list: ReferralListResponse,
) -> impl IntoView {
    let invited = referral_list.i_invited();
    let inviter = referral_list.invited_by();

    view! {
        <div class="referralBoard_body">
            <div class="referral_tabs" role="tablist" aria-label="Referral sections">
                <button
                    type="button"
                    role="tab"
                    aria-selected=move || active_tab.get() == ReferralTab::Invited
                    aria-controls="invited-panel"
                    class=move || {
                        if active_tab.get() == ReferralTab::Invited {
                            "button referral_tab active"
                        } else {
                            "button referral_tab"
                        }
                    }
                    on:click=move |_| active_tab.set(ReferralTab::Invited)
                >
                    <span class="tab_label">"Invited Friends"</span>
                </button>
                <button
                    type="button"
                    role="tab"
                    aria-selected=move || active_tab.get() == ReferralTab::Inviter
                    aria-controls="inviter-panel"
                    class=move || {
                        if active_tab.get() == ReferralTab::Inviter {
                            "button referral_tab active"
                        } else {
                            "button referral_tab"
                        }
                    }
                    on:click=move |_| active_tab.set(ReferralTab::Inviter)
                >
                    <span class="tab_label">"My Inviter"</span>
                </button>
            </div>

            <div class="referral_top">
                <h3>"Account"</h3>
            </div>

            <div class="referral_content">
                <Show
                    when=move || active_tab.get() == ReferralTab::Invited
                    fallback=move || {
                        let inviter_clone = inviter.clone();
                        view! {
                            <ReferralUserGrid
                                users=inviter_clone
                                empty_message="No inviter found \u{2014} you joined directly without a referral"
                            />
                        }
                    }
                >
                    {
                        let invited_clone = invited.clone();
                        view! {
                            <ReferralUserGrid
                                users=invited_clone
                                empty_message="You haven't referred anyone yet \u{2014} share your referral link to invite friends"
                            />
                        }
                    }
                </Show>
            </div>
        </div>
    }
}

/// Loading skeleton for the referral tabs/list section.
#[component]
pub fn ReferralListSkeleton() -> impl IntoView {
    view! {
        <div class="referralBoard_body referral-list-skeleton">
            <div class="referral_tabs">
                <div class="skeleton-tab"></div>
                <div class="skeleton-tab"></div>
            </div>
            <div class="referral_top">
                <div class="skeleton-line skeleton-heading"></div>
            </div>
            <div class="skeleton-grid">
                <div class="skeleton-card"></div>
                <div class="skeleton-card"></div>
                <div class="skeleton-card"></div>
            </div>
        </div>
    }
}
