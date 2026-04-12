//! Video trimmer component with timeline scrubbing.

use leptos::prelude::*;

/// Video trimmer with timeline and handle dragging.
#[component]
pub fn VideoTrimmer(
    /// Video source URL
    video_src: String,
    /// Video duration in seconds
    duration: f64,
    /// Callback when trimming is complete
    on_trim_complete: Callback<(f64, f64)>, // (start_time, end_time)
    /// Callback when cancelled
    on_cancel: Callback<()>,
) -> impl IntoView {
    const MIN_DURATION: f64 = 3.0;

    let (start_percent, set_start_percent) = signal(0.0_f64);
    let (end_percent, set_end_percent) = signal(1.0_f64);
    let (is_processing, set_is_processing) = signal(false);
    let (progress, set_progress) = signal(0);
    let (current_time, set_current_time) = signal(0.0_f64);

    let video_ref = NodeRef::<leptos::html::Video>::new();

    // Calculate actual times from percentages
    let start_time = move || start_percent.get() * duration;
    let end_time = move || end_percent.get() * duration;
    let trim_duration = move || end_time() - start_time();

    // Format time as MM:SS
    let format_time = |seconds: f64| {
        let mins = (seconds / 60.0).floor() as u32;
        let secs = (seconds % 60.0).floor() as u32;
        format!("{:02}:{:02}", mins, secs)
    };

    // Handle seeking on timeline click
    let handle_timeline_click = move |ev: leptos::ev::MouseEvent| {
        // Calculate percentage from click position
        // Update current_time and seek video
    };

    // Handle start handle drag
    let handle_start_drag = move |ev: leptos::ev::MouseEvent| {
        // Implement drag logic for start handle
    };

    // Handle end handle drag
    let handle_end_drag = move |ev: leptos::ev::MouseEvent| {
        // Implement drag logic for end handle
    };

    let perform_trim = {
        let on_trim_complete = on_trim_complete.clone();
        move |_| {
            set_is_processing.set(true);
            
            // In a full implementation, this would:
            // 1. Load FFmpeg WASM
            // 2. Execute trim command
            // 3. Report progress
            // 4. Return trimmed video blob
            
            // For now, just return the times
            on_trim_complete.run((start_time(), end_time()));
        }
    };

    let handle_cancel = {
        let on_cancel = on_cancel.clone();
        move |_| {
            on_cancel.run(());
        }
    };

    view! {
        <div class="video-trimmer">
            // Video preview
            <div class="video-preview">
                <video
                    node_ref=video_ref
                    src=video_src
                    controls=false
                    class="trimmer-video"
                />
                <button type="button" class="play-overlay">
                    <i class="peer-icon peer-icon-play"/>
                </button>
            </div>

            // Timeline
            <div class="timeline-container">
                <div class="timeline" on:click=handle_timeline_click>
                    // Left overlay (trimmed out)
                    <div
                        class="trim-overlay left"
                        style=move || format!("width: {}%", start_percent.get() * 100.0)
                    />
                    
                    // Trim window
                    <div
                        class="trim-window"
                        style=move || format!(
                            "left: {}%; width: {}%",
                            start_percent.get() * 100.0,
                            (end_percent.get() - start_percent.get()) * 100.0
                        )
                    />
                    
                    // Right overlay (trimmed out)
                    <div
                        class="trim-overlay right"
                        style=move || format!("width: {}%", (1.0 - end_percent.get()) * 100.0)
                    />
                    
                    // Left handle
                    <div
                        class="trim-handle left"
                        style=move || format!("left: {}%", start_percent.get() * 100.0)
                        on:mousedown=handle_start_drag
                    >
                        <div class="handle-bar"/>
                    </div>
                    
                    // Right handle
                    <div
                        class="trim-handle right"
                        style=move || format!("left: {}%", end_percent.get() * 100.0)
                        on:mousedown=handle_end_drag
                    >
                        <div class="handle-bar"/>
                    </div>
                    
                    // Playhead
                    <div
                        class="playhead"
                        style=move || format!("left: {}%", (current_time.get() / duration) * 100.0)
                    />
                </div>
                
                // Time labels
                <div class="time-labels">
                    <span class="start-time">{move || format_time(start_time())}</span>
                    <span class="duration">
                        {move || format!("Duration: {}", format_time(trim_duration()))}
                    </span>
                    <span class="end-time">{move || format_time(end_time())}</span>
                </div>
            </div>

            // Minimum duration warning
            <Show when=move || trim_duration() < MIN_DURATION>
                <div class="warning-message">
                    {format!("Minimum duration is {} seconds", MIN_DURATION as u32)}
                </div>
            </Show>

            // Processing progress
            <Show when=is_processing>
                <div class="processing-overlay">
                    <progress value=progress max=100 class="progress-bar"/>
                    <span class="progress-text">{move || format!("Processing... {}%", progress.get())}</span>
                </div>
            </Show>

            // Action buttons
            <div class="trimmer-actions">
                <button
                    type="button"
                    class="btn btn-secondary"
                    on:click=handle_cancel
                    disabled=is_processing
                >
                    "Cancel"
                </button>
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=perform_trim
                    disabled=move || is_processing.get() || trim_duration() < MIN_DURATION
                >
                    {move || if is_processing.get() { "Processing..." } else { "Trim Video" }}
                </button>
            </div>
        </div>
    }
}
