//! Right sidebar component with widgets.

use leptos::prelude::*;

use crate::components::widgets::{AddPostButton, MainMenu, ProfileWidget, VersionWidget};

/// Right sidebar with profile widget and navigation.
#[component]
pub fn RightSidebar() -> impl IntoView {
    view! {
        <aside class="right-sidebar right-sidebar-dashboard">
            <div class="inner-scroll">
                <ProfileWidget/>
                <MainMenu/>
                <AddPostButton/>
                <VersionWidget/>
            </div>
        </aside>
    }
}
