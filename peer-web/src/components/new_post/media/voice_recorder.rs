//! Voice recorder component with waveform visualization.

use leptos::prelude::*;

/// Recording state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecorderState {
    #[default]
    Initial,
    Recording,
    Paused,
    Preview,
}

/// Voice recorder with real-time waveform visualization.
#[component]
pub fn VoiceRecorder(
    /// Callback when recording is complete
    on_recording_complete: Callback<(Vec<u8>, String)>, // (data, mime_type)
) -> impl IntoView {
    let (state, set_state) = signal(RecorderState::Initial);
    let (elapsed_time, set_elapsed_time) = signal(0u32);
    let (audio_url, set_audio_url) = signal(Option::<String>::None);

    let format_time = move |seconds: u32| {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", mins, secs)
    };

    let start_recording = move |_| {
        set_state.set(RecorderState::Recording);
        set_elapsed_time.set(0);
        
        #[cfg(feature = "hydrate")]
        {
            // In a full implementation, this would:
            // 1. Request microphone access via navigator.mediaDevices.getUserMedia
            // 2. Create MediaRecorder
            // 3. Start recording and waveform animation
            // 4. Update elapsed_time via setInterval
        }
    };

    let stop_recording = move |_| {
        set_state.set(RecorderState::Preview);
        
        #[cfg(feature = "hydrate")]
        {
            // In a full implementation, this would:
            // 1. Stop MediaRecorder
            // 2. Convert blob to URL for playback
            // 3. Convert to WAV if needed (Chrome/Safari use WebM)
        }
    };

    let toggle_playback = move |_| {
        // Play/pause the recorded audio
    };

    let reset_recording = move |_| {
        set_state.set(RecorderState::Initial);
        set_elapsed_time.set(0);
        set_audio_url.set(None);
    };

    let use_recording = {
        let on_recording_complete = on_recording_complete.clone();
        move |_| {
            // In a full implementation, this would:
            // 1. Get the recorded blob data
            // 2. Call on_recording_complete with the data
            on_recording_complete.run((vec![], "audio/wav".to_string()));
        }
    };

    view! {
        <div class="voice-recorder">
            // Waveform visualization
            <div class="waveform-container">
                <svg
                    class="waveform"
                    viewBox="0 0 200 100"
                    preserveAspectRatio="none"
                >
                    // Waveform paths would be animated here
                    <path
                        d="M0,50 L200,50"
                        stroke="var(--color-primary)"
                        stroke-width="2"
                        fill="none"
                    />
                </svg>
            </div>

            // Controls
            <div class="recorder-controls">
                // Mic / Stop button
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
                    RecorderState::Paused | RecorderState::Preview => view! {
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
