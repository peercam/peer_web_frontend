//! Dashboard header component.

use leptos::prelude::*;

/// Dashboard header with logo and title.
#[component]
pub fn DashboardHeader() -> impl IntoView {
    view! {
        <header class="site-header header-dashboard">
            <img class="logo" src="/svg/Home.svg" alt="Peer Network"/>
            <h1 class="dashboard_h1">"Dashboard"</h1>
        </header>
    }
}
