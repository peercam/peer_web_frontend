//! Main content area with search bar and post list.

use leptos::prelude::*;

use crate::components::posts::PostList;
use crate::components::search::{SearchBar, UserSearch};

/// Main content area with search and post feed.
#[component]
pub fn MainContent() -> impl IntoView {
    view! {
        <main class="main-content main-content-dashboard">
            <div class="dashboard-search-container">
                <SearchBar/>
                <UserSearch/>
            </div>
            <PostList/>
        </main>
    }
}
