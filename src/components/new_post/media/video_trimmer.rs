//! Video trimmer component with timeline scrubbing.
//!
//! Provides an interactive timeline with draggable start/end handles
//! and a click-to-seek playhead. Actual video cutting is deferred to
//! the parent: the component emits the chosen `(start, end)` times
//! which can be passed to the backend or client-side transcoder.

use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

/// Which timeline handle (if any) is currently being dragged.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DragMode {
    None,
    Start,
    End,
}

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
    let (progress, _set_progress) = signal(0);
    let (current_time, set_current_time) = signal(0.0_f64);
    let drag_mode = RwSignal::new(DragMode::None);

    let video_ref = NodeRef::<leptos::html::Video>::new();
    let timeline_ref = NodeRef::<leptos::html::Div>::new();

    // Keep current_time in sync with the <video>.
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            let Some(video) = video_ref.get() else {
                return;
            };
            let el: web_sys::HtmlMediaElement = (*video).clone();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                set_current_time.set(el.current_time());
            }) as Box<dyn FnMut()>);
            if let Some(target) = video_ref.get() {
                let _ = (*target).add_event_listener_with_callback(
                    "timeupdate",
                    closure.as_ref().unchecked_ref(),
                );
            }
            closure.forget();
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = set_current_time;
    }

    // Convert a client-X pixel coordinate to a 0..1 percent along timeline.
    let timeline_percent = move |client_x: f64| -> Option<f64> {
        #[cfg(feature = "hydrate")]
        {
            let Some(tl) = timeline_ref.get() else {
                return None;
            };
            let rect = (*tl).get_bounding_client_rect();
            let left = rect.left();
            let width = rect.width();
            if width <= 0.0 {
                return None;
            }
            let p = ((client_x - left) / width).clamp(0.0, 1.0);
            Some(p)
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = client_x;
            None
        }
    };

    let seek_to = move |p: f64| {
        set_current_time.set(p * duration);
        #[cfg(feature = "hydrate")]
        {
            if let Some(video) = video_ref.get() {
                (*video).set_current_time(p * duration);
            }
        }
    };

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

    // Timeline click → seek playhead.
    let handle_timeline_click = move |ev: leptos::ev::MouseEvent| {
        if drag_mode.get() != DragMode::None {
            return;
        }
        if let Some(p) = timeline_percent(ev.client_x() as f64) {
            seek_to(p);
        }
    };

    // Mousedown on start handle.
    let handle_start_drag = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        ev.prevent_default();
        drag_mode.set(DragMode::Start);
    };

    // Mousedown on end handle.
    let handle_end_drag = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        ev.prevent_default();
        drag_mode.set(DragMode::End);
    };

    // While dragging, update the relevant handle as the mouse moves.
    let handle_timeline_mousemove = move |ev: leptos::ev::MouseEvent| {
        let mode = drag_mode.get();
        if mode == DragMode::None {
            return;
        }
        let Some(p) = timeline_percent(ev.client_x() as f64) else {
            return;
        };
        let min_gap = if duration > 0.0 {
            (MIN_DURATION / duration).min(1.0)
        } else {
            0.0
        };
        match mode {
            DragMode::Start => {
                let max_p = (end_percent.get() - min_gap).max(0.0);
                set_start_percent.set(p.min(max_p).max(0.0));
            }
            DragMode::End => {
                let min_p = (start_percent.get() + min_gap).min(1.0);
                set_end_percent.set(p.max(min_p).min(1.0));
            }
            DragMode::None => {}
        }
    };

    let handle_timeline_mouseup = move |_: leptos::ev::MouseEvent| {
        drag_mode.set(DragMode::None);
    };

    let perform_trim = move |_| {
        set_is_processing.set(true);
        // NOTE: Actual transcoding (e.g. via ffmpeg-wasm) would happen here
        // and update `progress`. For now we forward the selected times so
        // the server can do the cut.
        on_trim_complete.run((start_time(), end_time()));
        set_is_processing.set(false);
    };

    let handle_cancel = move |_| on_cancel.run(());

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
                <div
                    node_ref=timeline_ref
                    class="timeline"
                    on:click=handle_timeline_click
                    on:mousemove=handle_timeline_mousemove
                    on:mouseup=handle_timeline_mouseup
                    on:mouseleave=handle_timeline_mouseup
                >
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
                        style=move || {
                            let pct = if duration > 0.0 {
                                (current_time.get() / duration) * 100.0
                            } else {
                                0.0
                            };
                            format!("left: {}%", pct)
                        }
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
