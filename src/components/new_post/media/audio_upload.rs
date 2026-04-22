//! Audio upload component.

use leptos::prelude::*;

use super::voice_recorder::VoiceRecorder;
use crate::models::post::MediaFile;
use crate::pages::new_post::NewPostContext;

/// Audio upload area with file upload and voice recording options.
#[component]
pub fn AudioUpload() -> impl IntoView {
    let ctx = NewPostContext::use_context();
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let cover_input_ref = NodeRef::<leptos::html::Input>::new();

    let has_audio = move || !ctx.media_files.get().is_empty();
    let has_cover = move || ctx.cover_file.get().is_some();

    let handle_audio_click = move |_| {
        if let Some(input) = input_ref.get() {
            input.click();
        }
    };

    let handle_audio_change = move |ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let target = ev.target().unwrap();
            let input: web_sys::HtmlInputElement = target.unchecked_into();
            if let Some(files) = input.files()
                && let Some(file) = files.get(0)
            {
                let name = file.name();
                let mime_type = file.type_();

                if !mime_type.starts_with("audio/") {
                    return;
                }

                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                    let file_clone = file.clone();
                    leptos::task::spawn_local(async move {
                        if let Ok(data) = super::drop_zone::read_file_as_bytes(&file_clone).await {
                            let media_file =
                                MediaFile::new(name, mime_type, data).with_preview(url);
                            ctx.set_media_files.set(vec![media_file]);
                        }
                    });
                }
            }
        }
        let _ = ev;
    };

    let handle_cover_click = move |_| {
        if let Some(input) = cover_input_ref.get() {
            input.click();
        }
    };

    let handle_cover_change = move |ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let target = ev.target().unwrap();
            let input: web_sys::HtmlInputElement = target.unchecked_into();
            if let Some(files) = input.files()
                && let Some(file) = files.get(0)
            {
                let name = file.name();
                let mime_type = file.type_();

                if !mime_type.starts_with("image/") {
                    return;
                }

                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                    let file_clone = file.clone();
                    leptos::task::spawn_local(async move {
                        if let Ok(data) = super::drop_zone::read_file_as_bytes(&file_clone).await {
                            let media_file =
                                MediaFile::new(name, mime_type, data).with_preview(url);
                            ctx.set_cover_file.set(Some(media_file));
                        }
                    });
                }
            }
        }
        let _ = ev;
    };

    let handle_recording = move |(data, mime_type): (Vec<u8>, String)| {
        let media_file = MediaFile::new("recording.wav".to_string(), mime_type, data);
        ctx.set_media_files.set(vec![media_file]);
    };

    let remove_audio = move |_| {
        ctx.set_media_files.set(vec![]);
    };

    let remove_cover = move |_| {
        ctx.set_cover_file.set(None);
    };

    view! {
        <div class="audio-upload">
            <div class="audio-upload-grid">
                // Left side: File upload
                <div class="audio-file-section">
                    <h3 class="section-title">"Upload Audio File"</h3>
                    <Show
                        when=move || !has_audio()
                        fallback=move || {
                            view! {
                                <AudioPreview on_remove=Callback::new(remove_audio)/>
                            }
                        }
                    >
                        <div class="drop-zone" on:click=handle_audio_click>
                            <input
                                node_ref=input_ref
                                type="file"
                                accept=".mp3,.wav,.flac,.aac,.m4a,audio/*"
                                class="file-input"
                                on:change=handle_audio_change
                            />
                            <div class="drop-zone-content">
                                <i class="peer-icon peer-icon-music"/>
                                <p>"Drag & drop audio file or click to upload"</p>
                            </div>
                        </div>
                    </Show>
                </div>

                // Right side: Voice recording
                <div class="voice-record-section">
                    <h3 class="section-title">"Or Record Voice"</h3>
                    <Show when=move || !has_audio()>
                        <VoiceRecorder
                            on_recording_complete=Callback::new(handle_recording)
                        />
                    </Show>
                </div>
            </div>

            // Cover image section
            <div class="cover-section">
                <h3 class="section-title">"Cover Image"</h3>
                <Show
                    when=move || !has_cover()
                    fallback=move || {
                        view! {
                            <CoverPreview on_remove=Callback::new(remove_cover)/>
                        }
                    }
                >
                    <div class="drop-zone" on:click=handle_cover_click>
                        <input
                            node_ref=cover_input_ref
                            type="file"
                            accept="image/*"
                            class="file-input"
                            on:change=handle_cover_change
                        />
                        <div class="drop-zone-content">
                            <i class="peer-icon peer-icon-image"/>
                            <p>"Add cover image (optional)"</p>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

/// Audio file preview.
#[component]
fn AudioPreview(on_remove: Callback<()>) -> impl IntoView {
    let ctx = NewPostContext::use_context();

    view! {
        <div class="audio-preview">
            {move || {
                ctx.media_files.get().first().map(|file| {
                    let url = file.preview_url.clone().unwrap_or_default();
                    view! {
                        <audio controls src=url class="audio-player"/>
                        <div class="file-info">
                            <span class="file-name">{file.name.clone()}</span>
                            <button
                                type="button"
                                class="remove-btn"
                                on:click=move |_| on_remove.run(())
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

/// Cover image preview.
#[component]
fn CoverPreview(on_remove: Callback<()>) -> impl IntoView {
    let ctx = NewPostContext::use_context();

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
                                on:click=move |_| on_remove.run(())
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
