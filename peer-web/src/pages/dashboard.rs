//! Dashboard page.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::auth_guard::AuthGuard;
use crate::components::dashboard::{DashboardHeader, LeftSidebar, MainContent, RightSidebar};
use crate::state::filters::provide_filter_context;

/// Main dashboard page.
///
/// Displays the post feed with filters, search, and user widgets.
/// Requires authentication.
#[component]
pub fn DashboardPage() -> impl IntoView {
    // Provide filter context for all dashboard components
    provide_filter_context();

    view! {
        <Title text="Dashboard - Peer Network"/>
        <AuthGuard>
            <div id="dashboard" class="site_layout">
                <DashboardHeader/>
                <LeftSidebar/>
                <MainContent/>
                <RightSidebar/>
                <MobileFooter/>
            </div>
        </AuthGuard>
    }
}

/// Mobile navigation footer.
#[component]
fn MobileFooter() -> impl IntoView {
    view! {
        <footer class="mobile-footer">
            <nav class="mobile-nav">
                <a href="/dashboard" class="nav-item active">
                    <i class="peer-icon peer-icon-home"/>
                    <span>"Home"</span>
                </a>
                <a href="/search" class="nav-item">
                    <i class="peer-icon peer-icon-search"/>
                    <span>"Search"</span>
                </a>
                <a href="/newpost" class="nav-item add-post">
                    <i class="peer-icon peer-icon-plus"/>
                </a>
                <a href="/notifications" class="nav-item">
                    <i class="peer-icon peer-icon-bell"/>
                    <span>"Alerts"</span>
                </a>
                <a href="/profile" class="nav-item">
                    <i class="peer-icon peer-icon-user"/>
                    <span>"Profile"</span>
                </a>
            </nav>
        </footer>
    }
}
