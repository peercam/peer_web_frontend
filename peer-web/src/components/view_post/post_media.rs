//! Post media component.
//!
//! Displays post media based on content type:
//! - Image gallery with slider
//! - Video player
//! - Audio player
//! - Text-only display

use leptos::prelude::*;

use crate::models::post::{ContentType, Post};

/// Parse media URLs from a comma-separated string or JSON array.
fn parse_media_urls(media: &Option<String>) -> Vec<String> {
    match media {
        Some(s) if !s.is_empty() => {
            // Try parsing as JSON array first
            if s.starts_with('[') {
                serde_json::from_str::<Vec<String>>(s).unwrap_or_else(|_| vec![s.clone()])
            } else if s.contains(',') {
                s.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                vec![s.clone()]
            }
        }
        _ => vec![],
    }
}

/// Post media display (image gallery, video, audio, or text).
#[component]
pub fn PostMedia(post: Post) -> impl IntoView {
    let content_type = post.contenttype;

    view! {
        {move || match content_type {
            ContentType::Image => view! { <ImageGallery post=post.clone()/> }.into_any(),
            ContentType::Video => view! { <VideoPlayer post=post.clone()/> }.into_any(),
            ContentType::Audio => view! { <AudioPlayer post=post.clone()/> }.into_any(),
            ContentType::Text => view! { <TextPost post=post.clone()/> }.into_any(),
        }}
    }
}

/// Image gallery with slider navigation.
#[component]
fn ImageGallery(post: Post) -> impl IntoView {
    let images = parse_media_urls(&post.media);
    let images_clone = images.clone();
    let images_for_nav = images.clone();
    let images_for_dots_when = images.clone();
    let images_for_dots_inner = images.clone();
    let images_for_thumbs_when = images.clone();
    let images_for_thumbs_inner = images.clone();
    let images_len = images.len();

    let current_index = RwSignal::new(0usize);
    let show_modal = RwSignal::new(false);

    let prev = move |_| {
        current_index.update(|i| {
            *i = if *i == 0 { images_len.saturating_sub(1) } else { *i - 1 };
        });
    };

    let next = move |_| {
        current_index.update(|i| {
            *i = (*i + 1) % images_len.max(1);
        });
    };

    view! {
        <div class="post_gallery">
            // Main image display
            <div class="gallery-main">
                <img
                    src={move || images_clone.get(current_index.get()).cloned().unwrap_or_default()}
                    alt=post.title.clone()
                    on:click=move |_| show_modal.set(true)
                    class="gallery-main-img"
                />

                // Navigation arrows (if multiple images)
                <Show when=move || { images_for_nav.len() > 1 }>
                    <button
                        class="gallery-nav prev"
                        on:click=prev
                        aria-label="Previous image"
                    >
                        <i class="peer-icon peer-icon-arrow-left"/>
                    </button>
                    <button
                        class="gallery-nav next"
                        on:click=next
                        aria-label="Next image"
                    >
                        <i class="peer-icon peer-icon-arrow-right"/>
                    </button>
                </Show>

                // Dots indicator
                <Show when=move || { images_for_dots_when.len() > 1 }>
                    <div class="gallery-dots">
                        {images_for_dots_inner.iter().enumerate().map(|(i, _)| {
                            view! {
                                <span
                                    class="gallery-dot"
                                    class:active=move || current_index.get() == i
                                    on:click=move |_| current_index.set(i)
                                />
                            }
                        }).collect_view()}
                    </div>
                </Show>
            </div>

            // Thumbnails (if multiple images)
            <Show when=move || { images_for_thumbs_when.len() > 1 }>
                <div class="gallery-thumbnails">
                    {images_for_thumbs_inner.iter().enumerate().map(|(i, url)| {
                        let url = url.clone();
                        view! {
                            <img
                                src=url
                                class="thumbnail"
                                class:active=move || current_index.get() == i
                                on:click=move |_| current_index.set(i)
                            />
                        }
                    }).collect_view()}
                </div>
            </Show>
        </div>

        // Fullscreen modal
        <Show when=move || show_modal.get()>
            <super::ImageModal
                images=images.clone()
                current_index=current_index
                on_close=move || show_modal.set(false)
            />
        </Show>
    }
}

/// Video player component.
#[component]
fn VideoPlayer(post: Post) -> impl IntoView {
    let video_url = post.media.clone().unwrap_or_default();
    let cover_url = post.cover.clone().unwrap_or_default();

    view! {
        <div class="post_gallery video-container">
            <video
                controls
                poster=cover_url
                preload="metadata"
            >
                <source src=video_url type="video/mp4"/>
                "Your browser does not support the video tag."
            </video>
        </div>
    }
}

/// Audio player component.
#[component]
fn AudioPlayer(post: Post) -> impl IntoView {
    let audio_url = post.media.clone().unwrap_or_default();
    let cover_url = post
        .cover
        .clone()
        .unwrap_or_else(|| "/svg/audio-placeholder.svg".to_string());

    view! {
        <div class="post_gallery audio-container">
            <img src=cover_url alt="Audio cover" class="audio-cover"/>
            <audio controls>
                <source src=audio_url type="audio/mpeg"/>
                "Your browser does not support the audio element."
            </audio>
        </div>
    }
}

/// Text-only post display.
#[component]
fn TextPost(post: Post) -> impl IntoView {
    let description = post.mediadescription.clone().unwrap_or_default();

    view! {
        <div class="post_gallery text-container">
            <div class="text-content">
                <p>{description}</p>
            </div>
        </div>
    }
}
