//! Focus management utilities for multi-step forms.

/// Focus the first invalid input inside the given container.
///
/// Looks for `.input-field.invalid input` or `input[aria-invalid="true"]`.
/// Used when form submission fails validation.
pub fn focus_first_error(container_id: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;

        let container_id = container_id.to_string();
        // Use request_animation_frame to defer until after DOM update
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::prelude::Closure::once_into_js(move || {
                if let Some(document) = web_sys::window().and_then(|w| w.document())
                    && let Some(container) = document.get_element_by_id(&container_id) {
                        let selectors = ".input-field.invalid input, input[aria-invalid='true']";
                        if let Ok(Some(first)) = container.query_selector(selectors)
                            && let Some(html_el) = first.dyn_ref::<web_sys::HtmlElement>() {
                                let _ = html_el.focus();
                            }
                    }
            });
            let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = container_id;
    }
}

/// Focus a specific element by its ID.
pub fn focus_element(element_id: &str) {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(element) = document.get_element_by_id(element_id)
                && let Some(html_el) = element.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = html_el.focus();
                }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = element_id;
    }
}
