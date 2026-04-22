//! Referral Board page.
//!
//! Displays the user's referral link, users they've invited,
//! and who invited them.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::api::referral::{get_referral_info, get_referral_list};
use crate::components::auth_guard::AuthGuard;
use crate::components::layout::SiteShell;
use crate::components::referral_board::{ReferralHeader, ReferralHeaderSkeleton, ReferralTabs};
use crate::components::widgets::{MainMenu, ProfileWidget, VersionWidget};

/// Active tab in the referral board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReferralTab {
    /// "Invited Friends" tab - users the current user has referred.
    #[default]
    Invited,
    /// "My Inviter" tab - the user who referred the current user.
    Inviter,
}

/// Referral Board page - referral link and referral relationships.
///
/// Requires authentication to access.
#[component]
pub fn ReferralBoardPage() -> impl IntoView {
    let active_tab = RwSignal::new(ReferralTab::Invited);

    // Fetch referral info (link) and referral list in parallel
    let referral_info = Resource::new(|| (), |_| async { get_referral_info().await.ok() });
    let referral_list = Resource::new(|| (), |_| async { get_referral_list(0, 20).await.ok() });

    view! {
        <Title text="Referral Program - Peer Network"/>
        <AuthGuard>
            <SiteShell id="referral-board" modifier="referral-board-layout">
                <ReferralBoardHeader/>

                <aside class="left-sidebar left-sidebar-referralBoard">
                    <div class="inner-scroll">
                        // Left sidebar - empty per design
                    </div>
                </aside>

                <main id="main" class="site-main site-main-referralBoard">
                    <div class="referralBoard_container">
                        <Suspense fallback=move || view! { <ReferralHeaderSkeleton/> }>
                            {move || {
                                referral_info.get().flatten().map(|info| {
                                    view! { <ReferralHeader info=info/> }
                                })
                            }}
                        </Suspense>

                        <Suspense fallback=move || view! { <ReferralListSkeleton/> }>
                            {move || {
                                referral_list.get().flatten().map(|list| {
                                    view! {
                                        <ReferralTabs
                                            active_tab=active_tab
                                            referral_list=list
                                        />
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </main>

                <aside class="right-sidebar right-sidebar-referralBoard">
                    <div class="inner-scroll">
                        <ProfileWidget/>
                        <MainMenu/>
                        <NewPostButton/>
                        <VersionWidget/>
                    </div>
                </aside>
            </SiteShell>
        </AuthGuard>
    }
}

/// Referral Board page header with icon and title.
#[component]
fn ReferralBoardHeader() -> impl IntoView {
    view! {
        <header class="site-header header-referralBoard">
            <h1 id="h1">
                <img class="header-icon" src="/svg/Home.svg" alt=""/>
                " Dashboard"
            </h1>
        </header>
    }
}

/// Loading skeleton for the referral list section.
#[component]
fn ReferralListSkeleton() -> impl IntoView {
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

/// New post button for sidebar.
#[component]
fn NewPostButton() -> impl IntoView {
    view! {
        <div class="new-post-widget">
            <a href="/newpost" class="new-post-btn">
                <i class="peer-icon peer-icon-plus"></i>
                <span>"New Post"</span>
            </a>
        </div>
    }
}
