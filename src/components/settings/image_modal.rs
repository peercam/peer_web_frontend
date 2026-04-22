//! Image upload preview modal with zoom slider.

use leptos::prelude::*;
use leptos::web_sys;

/// Image preview modal with zoom slider.
///
/// Allows the user to select an image file, preview it with zoom control,
/// and apply the selection.
#[component]
pub fn ImageUploadModal(
    /// Callback when the user clicks Apply with the base64 data URL.
    on_apply: impl Fn(String) + 'static + Clone,
    /// Callback when the user cancels.
    on_cancel: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let preview_src = RwSignal::new(String::new());
    let zoom_level = RwSignal::new(1.0f64);
    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    // Clone callbacks for multiple uses
    let on_cancel_close = on_cancel.clone();
    let on_cancel_btn = on_cancel.clone();

    // Handle file selection
    let on_file_change = move |ev: web_sys::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;

            let input = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());

            if let Some(input) = input
                && let Some(files) = input.files()
                && let Some(file) = files.get(0)
            {
                let reader = match web_sys::FileReader::new() {
                    Ok(r) => r,
                    Err(_) => return,
                };

                let reader_clone = reader.clone();
                let onload =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                        if let Ok(result) = reader_clone.result()
                            && let Some(data_url) = result.as_string()
                        {
                            preview_src.set(data_url);
                        }
                    })
                        as Box<dyn FnMut(_)>);

                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();

                let _ = reader.read_as_data_url(&file);
            }
        }

        // SSR fallback - does nothing
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = ev;
        }
    };

    // Trigger file input click
    let open_file_picker = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(input) = file_input_ref.get() {
                input.click();
            }
        }
    };

    view! {
        <div class="modal" style="display: flex;">
            <div class="modal-content">
                <span class="closeBtn" on:click=move |_| on_cancel_close()>
                    <img src="/svg/close.svg" alt="Close"/>
                </span>
                <h2 class="modal-content-heading">"Preview"</h2>

                // Hidden file input
                <input
                    type="file"
                    accept="image/*"
                    style="display: none;"
                    node_ref=file_input_ref
                    on:change=on_file_change
                />

                // Select image button (shown when no preview)
                <Show when=move || preview_src.get().is_empty()>
                    <div class="select-image-prompt">
                        <button class="btn-blue" on:click=open_file_picker>
                            "Select Image"
                        </button>
                    </div>
                </Show>

                // Image preview (shown when image selected)
                <Show when=move || !preview_src.get().is_empty()>
                    <div class="image-preview-wrapper">
                        <img
                            src=move || preview_src.get()
                            alt="Preview"
                            style=move || format!("transform: scale({});", zoom_level.get())
                        />
                    </div>
                    <div class="img-zoom">
                        <label>"Zoom"</label>
                        <input
                            type="range"
                            min="1"
                            max="3"
                            step="0.1"
                            prop:value=move || zoom_level.get().to_string()
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                    zoom_level.set(val);
                                }
                            }
                        />
                    </div>
                    <button class="btn-transparent change-image-btn" on:click=open_file_picker>
                        "Choose Different Image"
                    </button>
                </Show>

                <div class="button-row modal-buttons">
                    <button class="btn-transparent" on:click=move |_| on_cancel_btn()>
                        "Cancel"
                    </button>
                    <button
                        class="btn-blue"
                        prop:disabled=move || preview_src.get().is_empty()
                        on:click=move |_| on_apply(preview_src.get())
                    >
                        "Apply"
                    </button>
                </div>
            </div>
        </div>
    }
}
