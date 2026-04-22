//! Tag input component with autocomplete.

use leptos::prelude::*;

use crate::api::posts::search_tags;
use crate::models::post::is_valid_tag;
use crate::pages::new_post::NewPostContext;

use super::tag_list::TagList;

/// Tag input with autocomplete suggestions.
#[component]
pub fn TagInput() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    let (input_value, set_input_value) = signal(String::new());
    let (suggestions, set_suggestions) = signal(Vec::<String>::new());
    let (show_suggestions, set_show_suggestions) = signal(false);
    let (is_loading, set_is_loading) = signal(false);

    // Debounced search action
    let search_action = Action::new(|query: &String| {
        let q = query.clone();
        async move {
            if q.len() >= 2 {
                search_tags(q, None, Some(10)).await.ok()
            } else {
                None
            }
        }
    });

    // Update suggestions when search completes
    Effect::new(move |_| {
        if let Some(Some(response)) = search_action.value().get() {
            set_suggestions.set(response.tag_names());
            set_is_loading.set(false);
        }
    });

    let handle_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        set_input_value.set(value.clone());

        if value.len() >= 2 {
            set_is_loading.set(true);
            search_action.dispatch(value);
        } else {
            set_suggestions.set(vec![]);
        }
    };

    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            let value = input_value.get().trim().to_lowercase();
            if is_valid_tag(&value) {
                ctx.add_tag(value);
                set_input_value.set(String::new());
                set_suggestions.set(vec![]);
            }
        } else if ev.key() == "Escape" {
            set_show_suggestions.set(false);
        }
    };

    let select_suggestion = move |tag: String| {
        ctx.add_tag(tag);
        set_input_value.set(String::new());
        set_suggestions.set(vec![]);
        set_show_suggestions.set(false);
    };

    let tag_count = move || ctx.tags.get().len();
    let can_add_more = move || tag_count() < 10;

    view! {
        <div class="tags-section">
            <label class="form-label">"Tags"</label>

            // Selected tags
            <TagList/>

            // Input with autocomplete
            <Show when=can_add_more>
                <div class="tag-input-container">
                    <input
                        type="text"
                        class="tag-input"
                        placeholder="Add tags..."
                        prop:value=input_value
                        on:input=handle_input
                        on:keydown=handle_keydown
                        on:focus=move |_| set_show_suggestions.set(true)
                        on:blur=move |_| {
                            // Delay to allow click on suggestion
                            #[cfg(feature = "hydrate")]
                            {
                                use leptos::wasm_bindgen::closure::Closure;
                                use leptos::wasm_bindgen::JsCast;

                                let closure = Closure::once(move || {
                                    set_show_suggestions.set(false);
                                });

                                if let Some(window) = leptos::web_sys::window() {
                                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                        closure.as_ref().unchecked_ref(),
                                        200,
                                    );
                                }
                                closure.forget();
                            }
                            #[cfg(not(feature = "hydrate"))]
                            {
                                set_show_suggestions.set(false);
                            }
                        }
                    />

                    // Loading indicator
                    <Show when=is_loading>
                        <span class="loading-indicator">
                            <i class="peer-icon peer-icon-loader"/>
                        </span>
                    </Show>

                    // Suggestions dropdown
                    <Show when=move || show_suggestions.get() && !suggestions.get().is_empty()>
                        <ul class="tag-suggestions">
                            {move || {
                                suggestions.get().iter().map(|tag| {
                                    let tag_clone = tag.clone();
                                    let tag_display = tag.clone();
                                    view! {
                                        <li class="suggestion-item">
                                            <button
                                                type="button"
                                                class="suggestion-btn"
                                                on:mousedown=move |_| select_suggestion(tag_clone.clone())
                                            >
                                                <span class="tag-prefix">"#"</span>
                                                {tag_display}
                                            </button>
                                        </li>
                                    }
                                }).collect_view()
                            }}
                        </ul>
                    </Show>
                </div>
            </Show>

            // Tag count
            <span class="tag-count">
                {move || format!("{}/10 tags", tag_count())}
            </span>
        </div>
    }
}
