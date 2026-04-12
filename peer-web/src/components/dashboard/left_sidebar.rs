//! Left sidebar component with filters.

use leptos::prelude::*;

use crate::components::filters::{ContentFilter, FeedFilter, SortFilter};
use crate::state::filters::use_filter_state;

/// Left sidebar with filter controls.
#[component]
pub fn LeftSidebar() -> impl IntoView {
    let filters = use_filter_state();

    view! {
        <aside
            class="left-sidebar left-sidebar-dashboard"
            class:collapsed=move || filters.is_collapsed.get()
        >
            <div class="inner-scroll for-filters">
                <div class="inner-scroll-filters">
                    <ContentFilter/>
                    <FeedFilter/>
                    <SortFilter/>
                </div>
                <CollapseButton/>
            </div>
        </aside>
    }
}

/// Button to collapse/expand the filter sidebar.
#[component]
fn CollapseButton() -> impl IntoView {
    let filters = use_filter_state();

    let on_click = move |_| {
        filters.toggle_collapsed();
    };

    view! {
        <button
            type="button"
            class="collapse-btn"
            on:click=on_click
            aria-label="Toggle filters"
        >
            <img
                src="/svg/content-arrow.svg"
                alt=""
                class="collapse-icon"
            />
        </button>
    }
}
