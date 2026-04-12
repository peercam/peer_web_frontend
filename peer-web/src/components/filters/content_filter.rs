//! Content type filter component.

use leptos::prelude::*;

use crate::models::post::PostFilterType;
use crate::state::filters::use_filter_state;

/// Content type filter section (checkboxes).
#[component]
pub fn ContentFilter() -> impl IntoView {
    let filters = use_filter_state();
    let is_expanded = RwSignal::new(true);

    let content_types = [
        (PostFilterType::Image, "Photo", "/svg/photo.svg"),
        (PostFilterType::Video, "Video", "/svg/videos.svg"),
        (PostFilterType::Text, "Text", "/svg/text.svg"),
        (PostFilterType::Audio, "Music", "/svg/music.svg"),
    ];

    let toggle_section = move |_| {
        is_expanded.update(|v| *v = !*v);
    };

    view! {
        <section class="filter-section">
            <button
                type="button"
                class="filter-toggle filter-section-header"
                aria-expanded=move || is_expanded.get()
                aria-controls="content-options"
                on:click=toggle_section
            >
                <img src="/svg/content-icon.svg" class="section-icon" alt=""/>
                <div class="filter-section-container">
                    <span class="section-title">"Content"</span>
                    <img
                        src="/svg/content-arrow.svg"
                        class="section-arrow"
                        class:rotated=move || is_expanded.get()
                        alt=""
                    />
                </div>
            </button>

            <div
                id="content-options"
                class="filter-options"
                class:open=move || is_expanded.get()
            >
                <div class="filterGroup">
                    {content_types.iter().map(|(filter_type, label, icon)| {
                        let ft = *filter_type;
                        let is_checked = Memo::new(move |_| {
                            filters.content_types.get().contains(&ft)
                        });

                        let on_change = move |_| {
                            filters.toggle_content_type(ft);
                        };

                        view! {
                            <label class="filterButton">
                                <input
                                    type="checkbox"
                                    checked=is_checked
                                    on:change=on_change
                                />
                                <img src=*icon alt=""/>
                                <span>{*label}</span>
                            </label>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
