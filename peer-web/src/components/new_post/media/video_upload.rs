//! Video upload component.

use leptos::prelude::*;

use super::video_cover::VideoCover;
use super::video_trimmer::VideoTrimmer;
use crate::pages::new_post::NewPostContext;

/// Video upload area with multi-video support and trimming.
#[component]
pub fn VideoUpload() -> impl IntoView {
    let ctx = NewPostContext::use_context();
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // State for trimming
    let (trim_video_src, set_trim_video_src) = signal(Option::<String>::None);
    let (trim_video_duration, set_trim_video_duration) = signal(0.0_f64);

    let has_videos = move || !ctx.media_files.get().is_empty();
    let can_add_more = move || ctx.media_files.get().len() < 2;
    let video_count = move || ctx.media_files.get().len();

    let handle_click = move |_| {
        if let Some(input) = input_ref.get() {
            input.click();
        }
    };

    let handle_change = move |ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;

            use crate::models::post::MediaFile;

            let target = ev.target().unwrap();
            let input: web_sys::HtmlInputElement = target.unchecked_into();
            if let Some(files) = input.files() {
                let max_to_add = 2 - ctx.media_files.get().len();
                let files_to_process = (files.length() as usize).min(max_to_add);

                for i in 0..files_to_process {
                    if let Some(file) = files.get(i as u32) {
                        let name = file.name();
                        let mime_type = file.type_();

                        if !mime_type.starts_with("video/") {
                            continue;
                        }

                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&file) {
                            let file_clone = file.clone();
                            let url_clone = url.clone();

                            leptos::task::spawn_local(async move {
                                if let Ok(data) =
                                    super::drop_zone::read_file_as_bytes(&file_clone).await
                                {
                                    let media_file = MediaFile::new(name, mime_type, data)
                                        .with_preview(url_clone);
                                    ctx.add_media(media_file);
                                }
                            });
                        }
                    }
                }
            }
        }
        let _ = ev;
    };

    let start_trim = move |index: usize| {
        let files = ctx.media_files.get();
        if let Some(file) = files.get(index)
            && let Some(ref url) = file.preview_url {
                set_trim_video_src.set(Some(url.clone()));
                set_trim_video_duration.set(30.0);
            }
    };

    let on_trim_complete = move |(_start, _end): (f64, f64)| {
        set_trim_video_src.set(None);
    };

    let on_trim_cancel = move |_: ()| {
        set_trim_video_src.set(None);
    };

    let remove_video = move |index: usize| {
        ctx.remove_media(index);
    };

    view! {
        <div class="video-upload">
            // Video previews
            <Show when=has_videos>
                <div class="video-previews">
                    {move || {
                        ctx.media_files.get().iter().enumerate().map(|(idx, file)| {
                            let url = file.preview_url.clone().unwrap_or_default();
                            let idx_clone = idx;
                            view! {
                                <div class="video-preview-item">
                                    <video src=url class="video-thumbnail" controls=false/>
                                    <div class="video-actions">
                                        <button
                                            type="button"
                                            class="action-btn trim"
                                            on:click=move |_| start_trim(idx_clone)
                                            title="Trim video"
                                        >
                                            <i class="peer-icon peer-icon-scissors"/>
                                        </button>
                                        <button
                                            type="button"
                                            class="action-btn remove"
                                            on:click=move |_| remove_video(idx_clone)
                                            title="Remove video"
                                        >
                                            <i class="peer-icon peer-icon-x"/>
                                        </button>
                                    </div>
                                    <span class="video-label">{format!("Video {}", idx + 1)}</span>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </Show>

            // Upload drop zone
            <Show when=can_add_more>
                <div class="drop-zone" on:click=handle_click>
                    <input
                        node_ref=input_ref
                        type="file"
                        accept="video/*"
                        multiple=true
                        class="file-input"
                        on:change=handle_change
                    />
                    <div class="drop-zone-content">
                        <i class="peer-icon peer-icon-video"/>
                        <p>"Drag & drop videos or click to upload (max 2)"</p>
                    </div>
                </div>
            </Show>

            // Video count indicator
            <Show when=has_videos>
                <div class="video-count">
                    {move || format!("{}/2 videos", video_count())}
                </div>
            </Show>

            // Cover image section
            <VideoCover/>

            // Trim modal
            <Show when=move || trim_video_src.get().is_some()>
                {move || {
                    trim_video_src.get().map(|src| {
                        view! {
                            <dialog class="trim-modal" open=true>
                                <VideoTrimmer
                                    video_src=src
                                    duration=trim_video_duration.get()
                                    on_trim_complete=Callback::new(on_trim_complete)
                                    on_cancel=Callback::new(on_trim_cancel)
                                />
                            </dialog>
                        }
                    })
                }}
            </Show>
        </div>
    }
}
