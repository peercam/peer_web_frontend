//! Image slider for multi-image posts.

use leptos::prelude::*;

use crate::pages::new_post::NewPostContext;

/// Image slider for navigating multiple uploaded images.
#[component]
pub fn ImageSlider() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    let has_images = move || !ctx.media_files.get().is_empty();
    let image_count = move || ctx.media_files.get().len();
    let current_index = move || ctx.current_media_index.get();

    let can_go_prev = move || current_index() > 0;
    let can_go_next = move || current_index() < image_count().saturating_sub(1);

    let go_prev = move |_| {
        if can_go_prev() {
            ctx.set_current_media_index.update(|i| *i -= 1);
        }
    };

    let go_next = move |_| {
        if can_go_next() {
            ctx.set_current_media_index.update(|i| *i += 1);
        }
    };

    let remove_current = move |_| {
        ctx.remove_media(current_index());
    };

    view! {
        <Show when=has_images>
            <div class="image-slider">
                // Navigation arrows
                <button
                    type="button"
                    class="slider-btn prev"
                    disabled=move || !can_go_prev()
                    on:click=go_prev
                >
                    <i class="peer-icon peer-icon-chevron-left"/>
                </button>

                // Current image
                <div class="slider-viewport">
                    {move || {
                        let files = ctx.media_files.get();
                        let idx = current_index();
                        files.get(idx).map(|file| {
                            let preview_url = file.preview_url.clone()
                                .unwrap_or_else(|| "#".to_string());
                            view! {
                                <div class="slider-image">
                                    <img src=preview_url alt="Preview"/>
                                    <button
                                        type="button"
                                        class="remove-btn"
                                        on:click=remove_current
                                    >
                                        <i class="peer-icon peer-icon-x"/>
                                    </button>
                                </div>
                            }
                        })
                    }}
                </div>

                <button
                    type="button"
                    class="slider-btn next"
                    disabled=move || !can_go_next()
                    on:click=go_next
                >
                    <i class="peer-icon peer-icon-chevron-right"/>
                </button>

                // Dots indicator
                <div class="slider-dots">
                    {move || {
                        let count = image_count();
                        let current = current_index();
                        (0..count)
                            .map(|i| {
                                let is_active = i == current;
                                view! {
                                    <button
                                        type="button"
                                        class="dot"
                                        class:active=is_active
                                        on:click=move |_| ctx.set_current_media_index.set(i)
                                    />
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        </Show>
    }
}
