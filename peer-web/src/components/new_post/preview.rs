//! Post preview component.

use leptos::prelude::*;

use crate::models::post::CreateContentType;
use crate::pages::new_post::NewPostContext;

/// Preview the post before submission.
#[component]
pub fn PostPreview() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    let (preview_mode, set_preview_mode) = signal(PreviewMode::Full);

    view! {
        <div class="post-preview">
            // Mode toggle
            <div class="preview-mode-toggle">
                <button
                    type="button"
                    class="mode-btn"
                    class:active=move || preview_mode.get() == PreviewMode::Full
                    on:click=move |_| set_preview_mode.set(PreviewMode::Full)
                >
                    "Full View"
                </button>
                <button
                    type="button"
                    class="mode-btn"
                    class:active=move || preview_mode.get() == PreviewMode::Card
                    on:click=move |_| set_preview_mode.set(PreviewMode::Card)
                >
                    "Card View"
                </button>
            </div>

            // Preview content
            {move || match preview_mode.get() {
                PreviewMode::Full => view! { <FullPreview/> }.into_any(),
                PreviewMode::Card => view! { <CardPreview/> }.into_any(),
            }}

            // Back button
            <div class="preview-actions">
                <button
                    type="button"
                    class="btn btn-secondary"
                    on:click=move |_| ctx.set_show_preview.set(false)
                >
                    <i class="peer-icon peer-icon-arrow-left"/>
                    " Back to Edit"
                </button>
            </div>
        </div>
    }
}

/// Preview modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PreviewMode {
    #[default]
    Full,
    Card,
}

/// Full view preview (modal-style).
#[component]
fn FullPreview() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    view! {
        <div class="full-preview">
            <div class="preview-layout">
                // Media section
                <div class="preview-media">
                    {move || {
                        let files = ctx.media_files.get();
                        let content_type = ctx.content_type.get();
                        
                        match content_type {
                            CreateContentType::Text => view! {
                                <div class="text-placeholder">
                                    <i class="peer-icon peer-icon-text"/>
                                </div>
                            }.into_any(),
                            CreateContentType::Image => {
                                if let Some(file) = files.first() {
                                    let url = file.preview_url.clone().unwrap_or_default();
                                    view! {
                                        <img src=url alt="Preview" class="preview-image"/>
                                    }.into_any()
                                } else {
                                    view! { <div class="no-media">"No image"</div> }.into_any()
                                }
                            },
                            CreateContentType::Audio => {
                                if let Some(file) = files.first() {
                                    let url = file.preview_url.clone().unwrap_or_default();
                                    let cover_url = ctx.cover_file.get()
                                        .and_then(|f| f.preview_url.clone());
                                    view! {
                                        <div class="audio-preview-container">
                                            {cover_url.map(|url| view! {
                                                <img src=url alt="Cover" class="audio-cover"/>
                                            })}
                                            <audio src=url controls class="preview-audio"/>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div class="no-media">"No audio"</div> }.into_any()
                                }
                            },
                            CreateContentType::Video => {
                                if let Some(file) = files.first() {
                                    let url = file.preview_url.clone().unwrap_or_default();
                                    view! {
                                        <video src=url controls class="preview-video"/>
                                    }.into_any()
                                } else {
                                    view! { <div class="no-media">"No video"</div> }.into_any()
                                }
                            },
                        }
                    }}
                </div>

                // Info section
                <div class="preview-info">
                    // User header (placeholder)
                    <div class="preview-user">
                        <img
                            src="/img/default-avatar.png"
                            alt="Avatar"
                            class="user-avatar"
                        />
                        <span class="username">"You"</span>
                    </div>

                    // Title
                    <h2 class="preview-title">{move || ctx.title.get()}</h2>

                    // Description
                    <Show when=move || !ctx.description.get().is_empty()>
                        <p class="preview-description">{move || ctx.description.get()}</p>
                    </Show>

                    // Tags
                    <Show when=move || !ctx.tags.get().is_empty()>
                        <div class="preview-tags">
                            {move || {
                                ctx.tags.get().iter().map(|tag| {
                                    view! {
                                        <span class="preview-tag">{"#"}{tag.clone()}</span>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </Show>

                    // Stats placeholder
                    <div class="preview-stats">
                        <span class="stat"><i class="peer-icon peer-icon-heart"/> "0"</span>
                        <span class="stat"><i class="peer-icon peer-icon-eye"/> "0"</span>
                        <span class="stat"><i class="peer-icon peer-icon-message"/> "0"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Collapsed card preview.
#[component]
fn CardPreview() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    view! {
        <div class="card-preview">
            <div class="preview-card">
                // Media thumbnail
                <div class="card-media">
                    {move || {
                        let files = ctx.media_files.get();
                        let content_type = ctx.content_type.get();
                        
                        match content_type {
                            CreateContentType::Text => view! {
                                <div class="text-thumb">
                                    <i class="peer-icon peer-icon-text"/>
                                </div>
                            }.into_any(),
                            CreateContentType::Image => {
                                if let Some(file) = files.first() {
                                    let url = file.preview_url.clone().unwrap_or_default();
                                    view! {
                                        <img src=url alt="Preview" class="card-image"/>
                                    }.into_any()
                                } else {
                                    view! { <div class="no-media-thumb"/> }.into_any()
                                }
                            },
                            CreateContentType::Audio => {
                                let cover_url = ctx.cover_file.get()
                                    .and_then(|f| f.preview_url.clone())
                                    .unwrap_or_else(|| "/img/audio-default.png".to_string());
                                view! {
                                    <div class="audio-thumb">
                                        <img src=cover_url alt="Audio" class="card-image"/>
                                        <i class="peer-icon peer-icon-audio overlay-icon"/>
                                    </div>
                                }.into_any()
                            },
                            CreateContentType::Video => {
                                if let Some(file) = files.first() {
                                    let url = file.preview_url.clone().unwrap_or_default();
                                    view! {
                                        <div class="video-thumb">
                                            <video src=url class="card-video"/>
                                            <i class="peer-icon peer-icon-play overlay-icon"/>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div class="no-media-thumb"/> }.into_any()
                                }
                            },
                        }
                    }}
                </div>

                // Card info
                <div class="card-info">
                    <h3 class="card-title">{move || ctx.title.get()}</h3>
                    <Show when=move || !ctx.description.get().is_empty()>
                        <p class="card-description">
                            {move || {
                                let desc = ctx.description.get();
                                if desc.len() > 100 {
                                    format!("{}...", &desc[..100])
                                } else {
                                    desc
                                }
                            }}
                        </p>
                    </Show>
                    <Show when=move || !ctx.tags.get().is_empty()>
                        <div class="card-tags">
                            {move || {
                                ctx.tags.get().iter().take(3).map(|tag| {
                                    view! {
                                        <span class="card-tag">{"#"}{tag.clone()}</span>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
