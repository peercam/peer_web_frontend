//! Image cropper component with canvas-based editing.
//!
//! Loads the source image, displays it on a fixed-size canvas with
//! draggable / zoomable positioning, and on confirmation writes the
//! visible crop window to an output canvas at the selected aspect
//! ratio, returning a PNG data URL.

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
    /// Numeric ratio value (height / width); used for canvas sizing.
    pub fn value(&self) -> f64 {
        match self {
            AspectRatio::Square => 1.0,
            AspectRatio::Portrait => 1.25, // 4:5 → height / width = 5/4
        }
    }

    /// Human-readable label used in the UI.
    pub fn label(&self) -> &'static str {
        match self {
            AspectRatio::Square => "1:1 Square",
            AspectRatio::Portrait => "4:5 Portrait",
        }
    }
}

/// Canvas preview width in CSS pixels. Height derives from aspect ratio.
const CANVAS_WIDTH: f64 = 400.0;

/// Output PNG width in pixels (height derives from aspect ratio).
#[cfg(feature = "hydrate")]
const OUTPUT_WIDTH: f64 = 1080.0;

/// Image cropper modal component.
#[component]
pub fn ImageCropper(
    /// The source image URL to crop
    image_src: String,
    /// Callback when cropping is complete, receives cropped PNG data URL
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

    // Preview canvas height (reactive on aspect ratio).
    let canvas_height = move || CANVAS_WIDTH * aspect_ratio.get().value();

    #[cfg(feature = "hydrate")]
    let image_store: StoredValue<Option<web_sys::HtmlImageElement>> = StoredValue::new(None);

    #[cfg(feature = "hydrate")]
    {
        use web_sys::HtmlImageElement;

        // Load image once.
        let img_src = image_src.clone();
        Effect::new(move |_| {
            let img = match HtmlImageElement::new() {
                Ok(i) => i,
                Err(_) => return,
            };
            // Allow drawing cross-origin blob URLs to the canvas without tainting.
            img.set_cross_origin(Some("anonymous"));

            let img_for_closure = img.clone();
            let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                image_store.set_value(Some(img_for_closure.clone()));
                // Reset view so image fits centered.
                set_position.set((0.0, 0.0));
                set_scale.set(1.0);
                image_loaded.set(true);
            }) as Box<dyn FnMut()>);

            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();

            img.set_src(&img_src);
        });

        // Redraw the preview canvas whenever inputs change.
        Effect::new(move |_| {
            let (pos_x, pos_y) = position.get();
            let s = scale.get();
            let ratio = aspect_ratio.get();
            if !image_loaded.get() {
                return;
            }
            let Some(canvas) = canvas_ref.get() else {
                return;
            };
            let Some(img) = image_store.get_value() else {
                return;
            };
            draw_preview(&canvas, &img, pos_x, pos_y, s, ratio);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = image_src;
        let _ = image_loaded;
        let _ = scale;
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

    let perform_crop = move |_| {
        #[cfg(feature = "hydrate")]
        {
            let Some(output_canvas) = output_canvas_ref.get() else {
                return;
            };
            let ratio = aspect_ratio.get();
            let out_w = OUTPUT_WIDTH;
            let out_h = OUTPUT_WIDTH * ratio.value();
            output_canvas.set_width(out_w as u32);
            output_canvas.set_height(out_h as u32);

            if let Some(img) = image_store.get_value() {
                let (pos_x, pos_y) = position.get();
                let s = scale.get();
                draw_to_output(&output_canvas, &img, pos_x, pos_y, s);
            }

            if let Ok(data_url) = output_canvas.to_data_url_with_type("image/png") {
                on_crop.run(data_url);
            }
        }
        #[cfg(not(feature = "hydrate"))]
        {
            let _ = output_canvas_ref;
            on_crop.run(String::new());
        }
    };

    let handle_cancel = move |_| on_cancel.run(());

    view! {
        <dialog class="crop-modal" open=true>
            <div class="crop-dialog-content">
                <h2 class="crop-title">"Crop Image"</h2>

                <div class="crop-container">
                    <canvas
                        node_ref=canvas_ref
                        prop:width=CANVAS_WIDTH as u32
                        prop:height=move || canvas_height() as u32
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
                        {AspectRatio::Square.label()}
                    </button>
                    <button
                        type="button"
                        class="ratio-btn"
                        class:active=move || aspect_ratio.get() == AspectRatio::Portrait
                        on:click=move |_| set_aspect_ratio.set(AspectRatio::Portrait)
                    >
                        {AspectRatio::Portrait.label()}
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

/// Draw the image onto the preview canvas with the current position/scale.
#[cfg(feature = "hydrate")]
fn draw_preview(
    canvas: &web_sys::HtmlCanvasElement,
    img: &web_sys::HtmlImageElement,
    pos_x: f64,
    pos_y: f64,
    scale: f64,
    ratio: AspectRatio,
) {
    let w = CANVAS_WIDTH;
    let h = CANVAS_WIDTH * ratio.value();
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);
    draw_image_centered(canvas, img, pos_x, pos_y, scale, w, h);
}

/// Draw the image onto the output canvas for the final crop.
#[cfg(feature = "hydrate")]
fn draw_to_output(
    canvas: &web_sys::HtmlCanvasElement,
    img: &web_sys::HtmlImageElement,
    pos_x: f64,
    pos_y: f64,
    scale: f64,
) {
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    // Scale preview-space offset up to output-space.
    let factor = w / CANVAS_WIDTH;
    draw_image_centered(canvas, img, pos_x * factor, pos_y * factor, scale, w, h);
}

/// Shared drawing routine: renders the image centered inside `(w, h)`
/// with a base "cover" fit, then applies the user's drag offset and zoom.
#[cfg(feature = "hydrate")]
fn draw_image_centered(
    canvas: &web_sys::HtmlCanvasElement,
    img: &web_sys::HtmlImageElement,
    pos_x: f64,
    pos_y: f64,
    scale: f64,
    w: f64,
    h: f64,
) {
    let ctx = match canvas.get_context("2d") {
        Ok(Some(obj)) => match obj.dyn_into::<web_sys::CanvasRenderingContext2d>() {
            Ok(c) => c,
            Err(_) => return,
        },
        _ => return,
    };

    // Clear.
    ctx.set_fill_style_str("#000");
    ctx.fill_rect(0.0, 0.0, w, h);

    let img_w = img.natural_width() as f64;
    let img_h = img.natural_height() as f64;
    if img_w <= 0.0 || img_h <= 0.0 {
        return;
    }

    // Compute "cover" base fit: fill the canvas fully.
    let base = (w / img_w).max(h / img_h);
    let draw_w = img_w * base * scale;
    let draw_h = img_h * base * scale;
    let dx = (w - draw_w) / 2.0 + pos_x;
    let dy = (h - draw_h) / 2.0 + pos_y;

    let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(img, dx, dy, draw_w, draw_h);
}
