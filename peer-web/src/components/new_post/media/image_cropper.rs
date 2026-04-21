//! Image cropper component with canvas-based editing.

use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

/// Aspect ratio options for cropping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AspectRatio {
    /// 1:1 square crop
    #[default]
    Square,
    /// 4:5 vertical/portrait crop
    Portrait,
}

impl AspectRatio {
    /// Get the numeric ratio value.
    pub fn value(&self) -> f64 {
        match self {
            AspectRatio::Square => 1.0,
            AspectRatio::Portrait => 0.8, // 4:5
        }
    }

    /// Get display name.
    pub fn label(&self) -> &'static str {
        match self {
            AspectRatio::Square => "1:1 Square",
            AspectRatio::Portrait => "4:5 Portrait",
        }
    }
}

/// Image cropper modal component.
#[component]
pub fn ImageCropper(
    /// The source image URL to crop
    image_src: String,
    /// Callback when cropping is complete, receives cropped base64 data URL
    on_crop: Callback<String>,
    /// Callback when cropping is cancelled
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (aspect_ratio, set_aspect_ratio) = signal(AspectRatio::Square);
    let (position, set_position) = signal((0.0_f64, 0.0_f64));
    let (scale, set_scale) = signal(1.0_f64);
    let (is_dragging, set_is_dragging) = signal(false);
    let (drag_start, set_drag_start) = signal((0.0_f64, 0.0_f64));

    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let output_canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let image_loaded = RwSignal::new(false);

    // Canvas dimensions
    let canvas_size = 400.0_f64;

    #[cfg(feature = "hydrate")]
    {
        // Load image and draw initial state
        let img_src = image_src.clone();
        Effect::new(move |_| {
            use web_sys::HtmlImageElement;

            let Some(canvas) = canvas_ref.get() else {
                return;
            };

            let img = HtmlImageElement::new().unwrap();
            let img_clone = img.clone();
            let canvas_clone = canvas.clone();

            let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                image_loaded.set(true);
                // Initial draw will be triggered by the signals
            }) as Box<dyn FnMut()>);

            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            
            img.set_src(&img_src);
        });

        // Redraw on position/scale/aspect changes
        Effect::new(move |_| {
            let _ = position.get();
            let _ = scale.get();
            let _ = aspect_ratio.get();
            
            if !image_loaded.get() {
                return;
            }

            let Some(canvas) = canvas_ref.get() else {
                return;
            };

            // Draw logic would go here
            // For now, this is a placeholder structure
        });
    }

    let handle_mouse_down = move |ev: leptos::ev::MouseEvent| {
        set_is_dragging.set(true);
        set_drag_start.set((ev.client_x() as f64, ev.client_y() as f64));
    };

    let handle_mouse_move = move |ev: leptos::ev::MouseEvent| {
        if !is_dragging.get() {
            return;
        }
        
        let (start_x, start_y) = drag_start.get();
        let (pos_x, pos_y) = position.get();
        
        let dx = ev.client_x() as f64 - start_x;
        let dy = ev.client_y() as f64 - start_y;
        
        set_position.set((pos_x + dx, pos_y + dy));
        set_drag_start.set((ev.client_x() as f64, ev.client_y() as f64));
    };

    let handle_mouse_up = move |_: leptos::ev::MouseEvent| {
        set_is_dragging.set(false);
    };

    let handle_wheel = move |ev: leptos::ev::WheelEvent| {
        ev.prevent_default();
        let delta = if ev.delta_y() > 0.0 { -0.1 } else { 0.1 };
        set_scale.update(|s| {
            *s = (*s + delta).clamp(0.3, 8.0);
        });
    };

    let perform_crop = {
        let on_crop = on_crop.clone();
        move |_| {
            #[cfg(feature = "hydrate")]
            {
                if let Some(canvas) = output_canvas_ref.get() {
                    if let Ok(data_url) = canvas.to_data_url() {
                        on_crop.run(data_url);
                    }
                }
            }
            #[cfg(not(feature = "hydrate"))]
            {
                on_crop.run(String::new());
            }
        }
    };

    let handle_cancel = {
        let on_cancel = on_cancel.clone();
        move |_| {
            on_cancel.run(());
        }
    };

    view! {
        <dialog class="crop-modal" open=true>
            <div class="crop-dialog-content">
                <h2 class="crop-title">"Crop Image"</h2>
                
                <div class="crop-container">
                    <canvas
                        node_ref=canvas_ref
                        width=canvas_size as u32
                        height=canvas_size as u32
                        class="crop-canvas"
                        on:mousedown=handle_mouse_down
                        on:mousemove=handle_mouse_move
                        on:mouseup=handle_mouse_up
                        on:mouseleave=handle_mouse_up
                        on:wheel=handle_wheel
                    />
                    <canvas
                        node_ref=output_canvas_ref
                        class="output-canvas hidden"
                    />
                </div>

                <div class="crop-info">
                    <p>"Drag to position, scroll to zoom"</p>
                </div>

                <div class="aspect-ratio-toggle">
                    <button
                        type="button"
                        class="ratio-btn"
                        class:active=move || aspect_ratio.get() == AspectRatio::Square
                        on:click=move |_| set_aspect_ratio.set(AspectRatio::Square)
                    >
                        "1:1 Square"
                    </button>
                    <button
                        type="button"
                        class="ratio-btn"
                        class:active=move || aspect_ratio.get() == AspectRatio::Portrait
                        on:click=move |_| set_aspect_ratio.set(AspectRatio::Portrait)
                    >
                        "4:5 Portrait"
                    </button>
                </div>

                <div class="crop-actions">
                    <button
                        type="button"
                        class="btn btn-secondary"
                        on:click=handle_cancel
                    >
                        "Cancel"
                    </button>
                    <button
                        type="button"
                        class="btn btn-primary"
                        on:click=perform_crop
                    >
                        "Crop"
                    </button>
                </div>
            </div>
        </dialog>
    }
}
