//! Right sidebar for new post page.

use leptos::prelude::*;

use crate::components::widgets::{AddPostButton, MainMenu, ProfileWidget, VersionWidget};

/// Right sidebar with user widgets.
#[component]
pub fn NewPostRightSidebar() -> impl IntoView {
    view! {
        <aside class="rightside" id="createPostSidebar">
            <div class="sidebar-content">
                <ProfileWidget/>
                <MainMenu/>
                <AddPostButton/>
                <VersionWidget/>
            </div>
        </aside>
    }
}
