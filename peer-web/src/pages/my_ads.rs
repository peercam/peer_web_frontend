//! My Ads page.
//!
//! Displays the user's advertisement history with aggregated statistics,
//! per-ad performance details, and infinite scroll listing.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::auth_guard::AuthGuard;
use crate::components::my_ads::AdList;
use crate::components::widgets::{MainMenu, ProfileWidget, VersionWidget};

/// My Ads page - advertisement history and statistics.
///
/// Requires authentication to access.
#[component]
pub fn MyAdsPage() -> impl IntoView {
    view! {
        <Title text="My Ads - Peer Network"/>
        <AuthGuard>
            <div id="my-ads-page" class="site_layout my-ads-layout">
                <MyAdsHeader/>

                <aside class="left-sidebar left-sidebar-my-ads">
                    <div class="inner-scroll">
                        <div class="general-back-btn">
                            <a href="/profile" class="widget widget-back-button">
                                <i class="peer-icon peer-icon-arrow-left"></i>
                                <span>"Back to Profile"</span>
                            </a>
                        </div>
                    </div>
                </aside>

                <main id="main" class="site-main site-main-my-ads loaded">
                    <AdList/>
                </main>

                <aside class="right-sidebar right-sidebar-my-ads">
                    <div class="inner-scroll">
                        <ProfileWidget />
                        <MainMenu />
                        <NewPostButton />
                        <VersionWidget />
                    </div>
                </aside>

                <MobileFooter />
            </div>
        </AuthGuard>
    }
}

/// My Ads page header with icon and title.
#[component]
fn MyAdsHeader() -> impl IntoView {
    view! {
        <header class="site-header header-my-ads">
            <h1 id="h1" class="my-ads-h1">
                <i class="peer-icon peer-icon-ad" />
                " My Ads"
            </h1>
        </header>
    }
}

/// New post button for sidebar.
#[component]
fn NewPostButton() -> impl IntoView {
    view! {
        <div class="new-post-widget">
            <a href="/newpost" class="new-post-btn">
                <i class="peer-icon peer-icon-plus" />
                <span>"New Post"</span>
            </a>
        </div>
    }
}

/// Mobile navigation footer.
#[component]
fn MobileFooter() -> impl IntoView {
    view! {
        <footer class="mobile-footer">
            <nav class="mobile-nav">
                <a href="/dashboard" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-home"></i>
                </a>
                <a href="/chat" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-chat"></i>
                </a>
                <a href="/newpost" class="mobile-nav-item mobile-nav-create">
                    <i class="peer-icon peer-icon-plus"></i>
                </a>
                <a href="/wallet" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-wallet-filled"></i>
                </a>
                <a href="/profile" class="mobile-nav-item">
                    <i class="peer-icon peer-icon-profile"></i>
                </a>
            </nav>
        </footer>
    }
}
