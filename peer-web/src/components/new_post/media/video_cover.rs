//! Video cover image selection.

use leptos::prelude::*;

use crate::models::post::MediaFile;
use crate::pages::new_post::NewPostContext;

/// Video cover image selection component.
#[component]
pub fn VideoCover() -> impl IntoView {
    let ctx = NewPostContext::use_context();
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let has_cover = move || ctx.cover_file.get().is_some();

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
                if let Some(file) = files.get(0) {
                    let name = file.name();
                    let mime_type = file.type_();

                    if !mime_type.starts_with("image/") {
                        return;
                    }

                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                        let file_clone = file.clone();
                        leptos::task::spawn_local(async move {
                            if let Ok(data) = super::drop_zone::read_file_as_bytes(&file_clone).await {
                                let media_file = MediaFile::new(name, mime_type, data)
                                    .with_preview(url);
                                ctx.set_cover_file.set(Some(media_file));
                            }
                        });
                    }
                }
            }
        }
        let _ = ev;
    };

    let remove_cover = move |_| {
        ctx.set_cover_file.set(None);
    };

    view! {
        <div class="video-cover">
            <h4 class="cover-title">"Video Cover"</h4>
            <Show
                when=move || !has_cover()
                fallback=move || {
                    view! {
                        <div class="cover-preview">
                            {move || {
                                ctx.cover_file.get().map(|file| {
                                    let url = file.preview_url.clone().unwrap_or_default();
                                    view! {
                                        <div class="cover-image-wrapper">
                                            <img src=url alt="Cover" class="cover-image"/>
                                            <button
                                                type="button"
                                                class="remove-btn"
                                                on:click=remove_cover
                                            >
                                                <i class="peer-icon peer-icon-x"/>
                                            </button>
                                        </div>
                                    }
                                })
                            }}
                        </div>
                    }
                }
            >
                <div class="drop-zone" on:click=handle_click>
                    <input
                        node_ref=input_ref
                        type="file"
                        accept="image/*"
                        class="file-input"
                        on:change=handle_change
                    />
                    <div class="drop-zone-content">
                        <i class="peer-icon peer-icon-image"/>
                        <p>"Add cover image for video"</p>
                    </div>
                </div>
            </Show>
        </div>
    }
}
