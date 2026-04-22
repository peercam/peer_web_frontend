//! New post form component.

use leptos::prelude::*;

use crate::components::new_post::media::{AudioUpload, ImageUpload, VideoUpload};
use crate::components::new_post::submit::SubmitButton;
use crate::components::new_post::tags::TagInput;
use crate::models::post::CreateContentType;
use crate::pages::new_post::NewPostContext;

/// Main form for creating a new post.
#[component]
pub fn NewPostForm() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    view! {
        <div class="create-post-form">
            // Media upload area (varies by content type)
            <div class="media-section">
                {move || match ctx.content_type.get() {
                    CreateContentType::Text => view! { <TextPlaceholder/> }.into_any(),
                    CreateContentType::Image => view! { <ImageUpload/> }.into_any(),
                    CreateContentType::Audio => view! { <AudioUpload/> }.into_any(),
                    CreateContentType::Video => view! { <VideoUpload/> }.into_any(),
                }}
            </div>

            // Title input
            <div class="form-group">
                <label for="post-title" class="form-label">"Title"</label>
                <input
                    type="text"
                    id="post-title"
                    class="form-input"
                    placeholder="Enter a title..."
                    maxlength="63"
                    prop:value=move || ctx.title.get()
                    on:input=move |ev| {
                        ctx.set_title.set(event_target_value(&ev));
                    }
                />
                <span class="char-count">
                    {move || format!("{}/63", ctx.title.get().len())}
                </span>
            </div>

            // Description textarea
            <div class="form-group">
                <label for="post-description" class="form-label">"Description"</label>
                <textarea
                    id="post-description"
                    class="form-textarea"
                    placeholder="Add a description..."
                    maxlength="500"
                    rows="4"
                    prop:value=move || ctx.description.get()
                    on:input=move |ev| {
                        ctx.set_description.set(event_target_value(&ev));
                    }
                />
                <span class="char-count">
                    {move || format!("{}/500", ctx.description.get().len())}
                </span>
            </div>

            // Tags section
            <TagInput/>

            // Error message
            <Show when=move || ctx.error.get().is_some()>
                <div class="error-message">
                    {move || ctx.error.get().unwrap_or_default()}
                </div>
            </Show>

            // Action buttons
            <div class="form-actions">
                <button
                    type="button"
                    class="btn btn-secondary"
                    on:click=move |_| ctx.set_show_preview.set(true)
                >
                    "Preview"
                </button>
                <SubmitButton/>
            </div>

            // Token cost indicator
            <div class="token-cost">
                <i class="peer-icon peer-icon-token"/>
                <span>"20 Tokens"</span>
                <span class="free-daily">" (or use your free daily post)"</span>
            </div>
        </div>
    }
}

/// Placeholder for text posts (no media upload required).
#[component]
fn TextPlaceholder() -> impl IntoView {
    view! {
        <div class="text-post-placeholder">
            <i class="peer-icon peer-icon-text"/>
            <p>"Text posts don't require media upload"</p>
        </div>
    }
}
