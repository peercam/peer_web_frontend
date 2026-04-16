//! Filter bar component — content type tabs and review-only checkbox.

use leptos::prelude::*;

/// Content type filter tabs and "Waiting for review" checkbox.
#[component]
pub fn FilterBar(
    active_filter: RwSignal<Option<String>>,
    review_only: RwSignal<bool>,
) -> impl IntoView {
    let filters: Vec<(Option<String>, &str)> = vec![
        (None, "All"),
        (Some("post".to_string()), "Posts"),
        (Some("comment".to_string()), "Comments"),
        (Some("user".to_string()), "Accounts"),
    ];

    view! {
        <div class="content_filter_row">
            <div class="content_filter">
                <ul>
                    {filters.into_iter().map(|(value, label)| {
                        let value_clone = value.clone();
                        let is_active = move || active_filter.get() == value_clone;
                        let value_for_click = value.clone();
                        view! {
                            <li>
                                <a
                                    href="#"
                                    class:active=is_active
                                    on:click=move |e| {
                                        e.prevent_default();
                                        active_filter.set(value_for_click.clone());
                                    }
                                >
                                    {label}
                                </a>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </div>
            <div class="waiting_review_filter">
                <label>
                    "Waiting for review"
                    <input
                        type="checkbox"
                        prop:checked=move || review_only.get()
                        on:change=move |_| {
                            review_only.update(|v| *v = !*v);
                        }
                    />
                </label>
            </div>
        </div>
    }
}
