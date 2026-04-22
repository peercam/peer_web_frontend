//! Dashboard page.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::auth_guard::AuthGuard;
use crate::components::dashboard::{DashboardHeader, LeftSidebar, MainContent, RightSidebar};
use crate::components::layout::SiteShell;
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
            <SiteShell id="dashboard">
                <DashboardHeader/>
                <LeftSidebar/>
                <MainContent/>
                <RightSidebar/>
            </SiteShell>
        </AuthGuard>
    }
}
