//! Image upload component.

use leptos::prelude::*;

use super::image_cropper::ImageCropper;
use super::image_slider::ImageSlider;
use crate::pages::new_post::NewPostContext;

/// Image upload area with multi-image support and cropping.
#[component]
pub fn ImageUpload() -> impl IntoView {
    let ctx = NewPostContext::use_context();
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // State for cropping
    let (crop_image_src, set_crop_image_src) = signal(Option::<String>::None);
    #[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
    let pending_file_name = RwSignal::new(String::new());

    let has_images = move || !ctx.media_files.get().is_empty();
    let can_add_more = move || ctx.media_files.get().len() < 5;
    let image_count = move || ctx.media_files.get().len();

    let handle_click = move |_| {
        if let Some(input) = input_ref.get() {
            input.click();
        }
    };

    let handle_change = move |ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let target = ev.target().unwrap();
            let input: web_sys::HtmlInputElement = target.unchecked_into();
            if let Some(files) = input.files() {
                let max_to_add = 5 - ctx.media_files.get().len();
                let files_to_process = (files.length() as usize).min(max_to_add);

                for i in 0..files_to_process {
                    if let Some(file) = files.get(i as u32) {
                        let name = file.name();
                        let mime_type = file.type_();

                        if !mime_type.starts_with("image/") {
                            continue;
                        }

                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                            set_crop_image_src.set(Some(url));
                            pending_file_name.set(name);
                            break; // Only process first image for cropping
                        }
                    }
                }
            }
        }
        let _ = ev;
    };

    let on_crop = move |cropped_data_url: String| {
        #[cfg(feature = "hydrate")]
        {
            use base64::{Engine, engine::general_purpose::STANDARD};

            use crate::models::post::MediaFile;

            if let Some(base64_data) = cropped_data_url.strip_prefix("data:image/png;base64,") {
                if let Ok(data) = STANDARD.decode(base64_data) {
                    let file =
                        MediaFile::new(pending_file_name.get(), "image/png".to_string(), data)
                            .with_preview(cropped_data_url.clone());

                    ctx.add_media(file);
                }
            }
        }
        let _ = &cropped_data_url;
        set_crop_image_src.set(None);
    };

    let on_cancel_crop = move |_: ()| {
        set_crop_image_src.set(None);
    };

    view! {
        <div class="image-upload">
            // Show slider if images exist
            <ImageSlider/>

            // Show drop zone if can add more images
            <Show when=can_add_more>
                <div class="drop-zone" on:click=handle_click>
                    <input
                        node_ref=input_ref
                        type="file"
                        accept="image/*"
                        multiple=true
                        class="file-input"
                        on:change=handle_change
                    />
                    <div class="drop-zone-content">
                        <i class="peer-icon peer-icon-image"/>
                        <p>"Drag & drop images or click to upload"</p>
                    </div>
                </div>
            </Show>

            // Image count indicator
            <Show when=has_images>
                <div class="image-count">
                    {move || format!("{}/5 images", image_count())}
                </div>
            </Show>

            // Crop modal
            <Show when=move || crop_image_src.get().is_some()>
                {move || {
                    crop_image_src.get().map(|src| {
                        view! {
                            <ImageCropper
                                image_src=src
                                on_crop=Callback::new(on_crop)
                                on_cancel=Callback::new(on_cancel_crop)
                            />
                        }
                    })
                }}
            </Show>
        </div>
    }
}
