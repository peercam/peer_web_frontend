//! Title/tag search bar component.

use leptos::prelude::*;

use crate::state::filters::use_filter_state;

/// Search bar for filtering posts by title.
#[component]
pub fn SearchBar() -> impl IntoView {
    #[allow(unused)]
    let filters = use_filter_state();
    let input_value = RwSignal::new(String::new());

    // Debounce timeout handle
    #[cfg(feature = "hydrate")]
    let timeout_handle = StoredValue::new(None::<i32>);

    // Debounced search effect
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;

        let value = input_value.get();

        // Clear previous timeout
        if let Some(handle) = timeout_handle.get_value()
            && let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(handle);
            }

        // Set new debounced timeout (300ms)
        let closure = Closure::once(Box::new(move || {
            filters.set_title_query(value);
        }) as Box<dyn FnOnce()>);

        if let Some(window) = web_sys::window()
            && let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                300,
            ) {
                timeout_handle.set_value(Some(handle));
            }

        closure.forget();
    });

    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        input_value.set(value);
    };

    view! {
        <div class="search-container">
            <input
                type="text"
                id="searchTitle"
                class="search-input"
                placeholder="Search posts..."
                prop:value=move || input_value.get()
                on:input=on_input
            />
            <img src="/svg/lupe.svg" class="lupe" alt="Search"/>
        </div>
    }
}
