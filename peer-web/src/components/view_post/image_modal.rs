//! Fullscreen image modal component.

use leptos::prelude::*;

/// Fullscreen image modal viewer.
#[component]
pub fn ImageModal<F>(
    images: Vec<String>,
    current_index: RwSignal<usize>,
    on_close: F,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    let images_clone = images.clone();
    let images_for_nav = images.clone();
    let images_len = images.len();
    let on_close_clone = on_close.clone();

    let prev = move |_| {
        current_index.update(|i| {
            *i = if *i == 0 {
                images_len.saturating_sub(1)
            } else {
                *i - 1
            };
        });
    };

    let next = move |_| {
        current_index.update(|i| {
            *i = (*i + 1) % images_len.max(1);
        });
    };

    // Close on escape key
    #[cfg(feature = "hydrate")]
    {
        use leptos::ev::keydown;
        let on_close_esc = on_close.clone();
        let handle_keydown = window_event_listener(keydown, move |ev| {
            if ev.key() == "Escape" {
                on_close_esc();
            }
        });
        on_cleanup(move || drop(handle_keydown));
    }

    view! {
        <div class="image-modal-overlay" on:click=move |_| on_close_clone()>
            <div class="image-modal-content" on:click=|ev| ev.stop_propagation()>
                <button class="modal-close" on:click=move |_| on_close()>
                    <i class="peer-icon peer-icon-cancel"/>
                </button>

                <img
                    src=move || images_clone.get(current_index.get()).cloned().unwrap_or_default()
                    class="modal-image"
                />

                // Navigation arrows
                <Show when=move || { images_for_nav.len() > 1 }>
                    <button class="modal-nav prev" on:click=prev>
                        <i class="peer-icon peer-icon-arrow-left"/>
                    </button>
                    <button class="modal-nav next" on:click=next>
                        <i class="peer-icon peer-icon-arrow-right"/>
                    </button>
                </Show>

                // Image counter
                <div class="modal-counter">
                    {move || format!("{} / {}", current_index.get() + 1, images_len)}
                </div>
            </div>
        </div>
    }
}
