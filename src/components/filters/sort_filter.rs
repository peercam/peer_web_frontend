//! Sort filter component.

use leptos::prelude::*;

use crate::models::post::PostSortType;
use crate::state::filters::use_filter_state;

/// Sort filter section (radio buttons).
#[component]
pub fn SortFilter() -> impl IntoView {
    let filters = use_filter_state();
    let is_expanded = RwSignal::new(true);

    let toggle_section = move |_| {
        is_expanded.update(|v| *v = !*v);
    };

    let sort_options: [(PostSortType, &str); 6] = [
        (PostSortType::Newest, "Newest"),
        (PostSortType::Trending, "Trending"),
        (PostSortType::Likes, "Most Liked"),
        (PostSortType::Views, "Most Viewed"),
        (PostSortType::Comments, "Most Comments"),
        (PostSortType::Dislikes, "Most Disliked"),
    ];

    view! {
        <section class="filter-section">
            <button
                type="button"
                class="filter-toggle filter-section-header"
                aria-expanded=move || is_expanded.get()
                aria-controls="sort-options"
                on:click=toggle_section
            >
                <img src="/svg/sort-icon.svg" class="section-icon" alt=""/>
                <div class="filter-section-container">
                    <span class="section-title">"Sort"</span>
                    <img
                        src="/svg/content-arrow.svg"
                        class="section-arrow"
                        class:rotated=move || is_expanded.get()
                        alt=""
                    />
                </div>
            </button>

            <div
                id="sort-options"
                class="filter-options"
                class:open=move || is_expanded.get()
            >
                <div class="filterGroup filterGroup-radio">
                    {sort_options.iter().map(|(sort_type, label)| {
                        let st = *sort_type;
                        let is_checked = Memo::new(move |_| {
                            filters.sort_by.get() == st
                        });

                        let on_change = move |_| {
                            filters.set_sort_by(st);
                        };

                        view! {
                            <label class="filterButton">
                                <input
                                    type="radio"
                                    name="sort-filter"
                                    value=st.to_string()
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
