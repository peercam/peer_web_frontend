//! Feed filter component.

use leptos::prelude::*;

use crate::models::post::PostFilterType;
use crate::state::filters::use_filter_state;

/// Feed filter section (radio buttons: All/Followers/Following).
#[component]
pub fn FeedFilter() -> impl IntoView {
    let filters = use_filter_state();
    let is_expanded = RwSignal::new(true);

    let toggle_section = move |_| {
        is_expanded.update(|v| *v = !*v);
    };

    let feed_options: [(Option<PostFilterType>, &str, &str); 3] = [
        (None, "All", "all"),
        (Some(PostFilterType::Follower), "Followers", "follower"),
        (Some(PostFilterType::Followed), "Following", "followed"),
    ];

    view! {
        <section class="filter-section">
            <button
                type="button"
                class="filter-toggle filter-section-header"
                aria-expanded=move || is_expanded.get()
                aria-controls="feed-options"
                on:click=toggle_section
            >
                <img src="/svg/feed-icon.svg" class="section-icon" alt=""/>
                <div class="filter-section-container">
                    <span class="section-title">"Feed"</span>
                    <img
                        src="/svg/content-arrow.svg"
                        class="section-arrow"
                        class:rotated=move || is_expanded.get()
                        alt=""
                    />
                </div>
            </button>

            <div
                id="feed-options"
                class="filter-options"
                class:open=move || is_expanded.get()
            >
                <div class="filterGroup filterGroup-radio">
                    {feed_options.iter().map(|(filter, label, id)| {
                        let f = *filter;
                        let is_checked = Memo::new(move |_| {
                            filters.feed_filter.get() == f
                        });

                        let on_change = move |_| {
                            filters.set_feed_filter(f);
                        };

                        view! {
                            <label class="filterButton">
                                <input
                                    type="radio"
                                    name="feed-filter"
                                    value=*id
                                    checked=is_checked
                                    on:change=on_change
                                />
                                <span>{*label}</span>
                            </label>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
