//! Voice recorder component built on the browser `MediaRecorder` API.
//!
//! The component requests microphone access, records audio in WebM/Opus
//! (the widest-supported format), exposes a preview player, and on
//! confirmation returns the recorded bytes along with their MIME type
//! to the parent via the `on_recording_complete` callback.

use leptos::prelude::*;

/// Recording state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecorderState {
    #[default]
    Initial,
    Recording,
    Preview,
}

/// Voice recorder with real-time timer.
#[component]
pub fn VoiceRecorder(
    /// Callback when recording is complete
    on_recording_complete: Callback<(Vec<u8>, String)>, // (data, mime_type)
) -> impl IntoView {
    let (state, set_state) = signal(RecorderState::Initial);
    let (elapsed_time, set_elapsed_time) = signal(0u32);
    let (audio_url, set_audio_url) = signal(Option::<String>::None);

    // Recorded bytes + mime type, kept out of signals to avoid clones.
    #[cfg(feature = "hydrate")]
    let recorded: StoredValue<Option<(Vec<u8>, String)>> = StoredValue::new(None);
    // Live MediaRecorder instance while recording.
    #[cfg(feature = "hydrate")]
    let recorder_store: StoredValue<Option<web_sys::MediaRecorder>> = StoredValue::new(None);
    // Accumulated chunks during recording.
    #[cfg(feature = "hydrate")]
    let chunks_store: StoredValue<Vec<web_sys::Blob>> = StoredValue::new(Vec::new());
    // Timer handle so we can clear it.
    #[cfg(feature = "hydrate")]
    let timer_handle: StoredValue<Option<i32>> = StoredValue::new(None);

    let format_time = |seconds: u32| {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", mins, secs)
    };

    let start_recording = move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen_futures::JsFuture;

            set_elapsed_time.set(0);

            leptos::task::spawn_local(async move {
                let Some(window) = web_sys::window() else {
                    return;
                };
                let navigator = window.navigator();
                let media_devices = match navigator.media_devices() {
                    Ok(md) => md,
                    Err(_) => return,
                };

                let constraints = web_sys::MediaStreamConstraints::new();
                constraints.set_audio(&wasm_bindgen::JsValue::TRUE);

                let promise = match media_devices.get_user_media_with_constraints(&constraints) {
                    Ok(p) => p,
                    Err(_) => return,
                };
                let stream_val = match JsFuture::from(promise).await {
                    Ok(v) => v,
                    Err(_) => return,
                };
                let stream: web_sys::MediaStream = match stream_val.dyn_into() {
                    Ok(s) => s,
                    Err(_) => return,
                };

                let recorder = match web_sys::MediaRecorder::new_with_media_stream(&stream) {
                    Ok(r) => r,
                    Err(_) => return,
                };

                // ondataavailable → push chunk
                let on_data =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |ev: web_sys::BlobEvent| {
                        if let Some(blob) = ev.data() {
                            chunks_store.update_value(|v| v.push(blob));
                        }
                    })
                        as Box<dyn FnMut(web_sys::BlobEvent)>);
                recorder.set_ondataavailable(Some(on_data.as_ref().unchecked_ref()));
                on_data.forget();

                // onstop → assemble blob and read bytes
                let on_stop = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    let chunks = chunks_store.with_value(|v| v.clone());
                    let array = js_sys::Array::new();
                    for chunk in &chunks {
                        array.push(chunk);
                    }

                    let mime = chunks
                        .first()
                        .map(|b| b.type_())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "audio/webm".to_string());

                    let bag = web_sys::BlobPropertyBag::new();
                    bag.set_type(&mime);
                    let Ok(blob) = web_sys::Blob::new_with_blob_sequence_and_options(&array, &bag)
                    else {
                        return;
                    };

                    // Preview URL for the <audio> element.
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        set_audio_url.set(Some(url));
                    }

                    // Read bytes asynchronously for later submission.
                    let mime_clone = mime.clone();
                    leptos::task::spawn_local(async move {
                        let buf =
                            match wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await {
                                Ok(b) => b,
                                Err(_) => return,
                            };
                        let u8arr = js_sys::Uint8Array::new(&buf);
                        let bytes = u8arr.to_vec();
                        recorded.set_value(Some((bytes, mime_clone)));
                    });

                    set_state.set(RecorderState::Preview);
                })
                    as Box<dyn FnMut()>);
                recorder.set_onstop(Some(on_stop.as_ref().unchecked_ref()));
                on_stop.forget();

                chunks_store.set_value(Vec::new());
                if recorder.start().is_err() {
                    return;
                }
                recorder_store.set_value(Some(recorder));
                set_state.set(RecorderState::Recording);

                // 1-second tick timer.
                let tick = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    set_elapsed_time.update(|t| *t += 1);
                })
                    as Box<dyn FnMut()>);
                if let Ok(handle) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                    tick.as_ref().unchecked_ref(),
                    1000,
                ) {
                    timer_handle.set_value(Some(handle));
                }
                tick.forget();
            });
        }
        #[cfg(not(feature = "hydrate"))]
        {
            set_state.set(RecorderState::Recording);
            set_elapsed_time.set(0);
        }
    };

    let stop_recording = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(handle) = timer_handle.with_value(|h| *h) {
                if let Some(w) = web_sys::window() {
                    w.clear_interval_with_handle(handle);
                }
                timer_handle.set_value(None);
            }
            if let Some(rec) = recorder_store.with_value(|r| r.clone()) {
                let _ = rec.stop();
            }
            // state transitions to Preview in onstop handler.
        }
        #[cfg(not(feature = "hydrate"))]
        {
            set_state.set(RecorderState::Preview);
        }
    };

    let toggle_playback = move |_| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("voice-recorder-preview") {
                    if let Ok(audio) = el.dyn_into::<web_sys::HtmlMediaElement>() {
                        if audio.paused() {
                            let _ = audio.play();
                        } else {
                            audio.pause().ok();
                        }
                    }
                }
            }
        }
    };

    let reset_recording = move |_| {
        set_state.set(RecorderState::Initial);
        set_elapsed_time.set(0);
        set_audio_url.set(None);
        #[cfg(feature = "hydrate")]
        {
            recorded.set_value(None);
            recorder_store.set_value(None);
            chunks_store.set_value(Vec::new());
        }
    };

    let use_recording = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some((bytes, mime)) = recorded.with_value(|r| r.clone()) {
                on_recording_complete.run((bytes, mime));
                return;
            }
        }
        // Fallback (SSR / no recording captured).
        on_recording_complete.run((Vec::new(), "audio/webm".to_string()));
    };

    view! {
        <div class="voice-recorder">
            // Waveform / preview visualisation
            <div class="waveform-container">
                <svg
                    class="waveform"
                    viewBox="0 0 200 100"
                    preserveAspectRatio="none"
                >
                    <path
                        d="M0,50 L200,50"
                        stroke="var(--color-primary)"
                        stroke-width="2"
                        fill="none"
                    />
                </svg>
                <Show when=move || audio_url.get().is_some()>
                    {move || audio_url.get().map(|url| view! {
                        <audio
                            id="voice-recorder-preview"
                            src=url
                            controls=true
                            class="audio-preview"
                        />
                    })}
                </Show>
            </div>

            // Controls
            <div class="recorder-controls">
                {move || match state.get() {
                    RecorderState::Initial => view! {
                        <button
                            type="button"
                            class="mic-btn"
                            on:click=start_recording
                            title="Start recording"
                        >
                            <i class="peer-icon peer-icon-mic"/>
                        </button>
                    }.into_any(),
                    RecorderState::Recording => view! {
                        <button
                            type="button"
                            class="mic-btn recording"
                            on:click=stop_recording
                            title="Stop recording"
                        >
                            <i class="peer-icon peer-icon-stop"/>
                        </button>
                    }.into_any(),
                    RecorderState::Preview => view! {
                        <button
                            type="button"
                            class="mic-btn"
                            on:click=toggle_playback
                            title="Play/Pause"
                        >
                            <i class="peer-icon peer-icon-play"/>
                        </button>
                    }.into_any(),
                }}

                // Timer
                <span class="recording-timer">
                    {move || format_time(elapsed_time.get())}
                </span>
            </div>

            // Preview controls
            <Show when=move || state.get() == RecorderState::Preview>
                <div class="preview-controls">
                    <button
                        type="button"
                        class="btn btn-secondary"
                        on:click=reset_recording
                    >
                        "Record again"
                    </button>
                    <button
                        type="button"
                        class="btn btn-primary"
                        on:click=use_recording
                    >
                        "Use recording"
                    </button>
                </div>
            </Show>
        </div>
    }
}
