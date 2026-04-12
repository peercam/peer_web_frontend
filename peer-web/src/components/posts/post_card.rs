//! Post card component.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::posts::post_action;
use crate::models::post::{ContentType, FeedItem, PostActionType, PostUser};

/// Individual post card in the feed.
#[component]
pub fn PostCard(item: FeedItem) -> impl IntoView {
    let post = item.post().clone();
    let is_ad = item.is_ad();

    // Local state for interactions
    let is_liked = RwSignal::new(post.isliked);
    let is_disliked = RwSignal::new(post.isdisliked);
    let is_saved = RwSignal::new(post.issaved);
    let like_count = RwSignal::new(post.amountlikes);
    let dislike_count = RwSignal::new(post.amountdislikes);

    let post_id = post.id.clone();
    let post_id_like = post.id.clone();
    let post_id_dislike = post.id.clone();
    let post_id_save = post.id.clone();
    let post_id_view = post.id.clone();

    // Mark as viewed when card becomes visible
    Effect::new(move |ran: Option<bool>| {
        if ran.is_none() {
            let id = post_id_view.clone();
            spawn_local(async move {
                let _ = post_action(id, PostActionType::View).await;
            });
        }
        true
    });

    let on_click = move |_| {
        // TODO: Open view post modal/overlay
        leptos::logging::log!("Post clicked: {}", post_id);
    };

    let on_like = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        let id = post_id_like.clone();
        let currently_liked = is_liked.get();
        let currently_disliked = is_disliked.get();

        // Optimistic update
        if currently_liked {
            is_liked.set(false);
            like_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_liked.set(true);
            like_count.update(|c| *c += 1);

            // Remove dislike if present
            if currently_disliked {
                is_disliked.set(false);
                dislike_count.update(|c| *c = (*c - 1).max(0));
            }
        }

        spawn_local(async move {
            let action = if currently_liked {
                PostActionType::Unlike
            } else {
                PostActionType::Like
            };
            let _ = post_action(id, action).await;
        });
    };

    let on_dislike = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        let id = post_id_dislike.clone();
        let currently_liked = is_liked.get();
        let currently_disliked = is_disliked.get();

        // Optimistic update
        if currently_disliked {
            is_disliked.set(false);
            dislike_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_disliked.set(true);
            dislike_count.update(|c| *c += 1);

            // Remove like if present
            if currently_liked {
                is_liked.set(false);
                like_count.update(|c| *c = (*c - 1).max(0));
            }
        }

        spawn_local(async move {
            let action = if currently_disliked {
                PostActionType::Undislike
            } else {
                PostActionType::Dislike
            };
            let _ = post_action(id, action).await;
        });
    };

    let on_save = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        let id = post_id_save.clone();
        let currently_saved = is_saved.get();

        // Optimistic update
        is_saved.set(!currently_saved);

        spawn_local(async move {
            let action = if currently_saved {
                PostActionType::Unsave
            } else {
                PostActionType::Save
            };
            let _ = post_action(id, action).await;
        });
    };

    let content_type_str = post.contenttype.to_string().to_lowercase();

    view! {
        <section
            class="card"
            class:card-shop-product=is_ad
            tabindex="0"
            data-content=content_type_str.clone()
            on:click=on_click
        >
            <div class="post">
                <div class="shadow"/>
                <div class="post-inhalt">
                    <PostCardHeader user=post.user.clone()/>
                    <PostMedia
                        content_type=post.contenttype
                        media=post.media.clone()
                        cover=post.cover.clone()
                        title=post.title.clone()
                    />
                    <PostContent
                        title=post.title.clone()
                        tags=post.tags.clone()
                    />
                    <PostActions
                        is_liked=is_liked
                        is_disliked=is_disliked
                        is_saved=is_saved
                        like_count=like_count
                        dislike_count=dislike_count
                        view_count=post.amountviews
                        comment_count=post.amountcomments
                        on_like=on_like
                        on_dislike=on_dislike
                        on_save=on_save
                    />
                </div>
            </div>
            {is_ad.then(|| view! {
                <div class="ad-badge">"Ad"</div>
            })}
        </section>
    }
}

/// Post card header with user info.
#[component]
fn PostCardHeader(user: PostUser) -> impl IntoView {
    let profile_url = format!("/profile/{}", user.slug);
    let img_src = user.img.unwrap_or_else(|| "/svg/noname.svg".to_string());

    view! {
        <div class="post-header">
            <a href=profile_url class="user-link">
                <img src=img_src alt="" class="user-avatar"/>
                <span class="username">{user.username}</span>
            </a>
        </div>
    }
}

/// Post media (image, video, audio, or text placeholder).
#[component]
fn PostMedia(
    content_type: ContentType,
    media: Option<String>,
    cover: Option<String>,
    title: String,
) -> impl IntoView {
    let display_url = cover.or(media).unwrap_or_default();

    match content_type {
        ContentType::Image | ContentType::Video => {
            view! {
                <div class="post-media">
                    <img src=display_url alt=title class="post-image"/>
                    {(content_type == ContentType::Video).then(|| view! {
                        <div class="play-overlay">
                            <img src="/svg/play.svg" alt="Play" class="play-icon"/>
                        </div>
                    })}
                </div>
            }.into_any()
        }
        ContentType::Audio => {
            view! {
                <div class="post-media post-media-audio">
                    <img src="/svg/music.svg" alt="Audio" class="audio-icon"/>
                </div>
            }.into_any()
        }
        ContentType::Text => {
            view! {
                <div class="post-media post-media-text">
                    <span class="text-preview">{title.chars().take(100).collect::<String>()}</span>
                </div>
            }.into_any()
        }
    }
}

/// Post title and tags.
#[component]
fn PostContent(title: String, tags: Option<Vec<String>>) -> impl IntoView {
    view! {
        <div class="post-content">
            <h3 class="post-title">{title}</h3>
            {tags.map(|t| view! {
                <div class="post-tags">
                    {t.into_iter().map(|tag| view! {
                        <span class="tag">{"#"}{tag}</span>
                    }).collect_view()}
                </div>
            })}
        </div>
    }
}

/// Post action buttons (like, dislike, save) and stats.
#[component]
fn PostActions(
    is_liked: RwSignal<bool>,
    is_disliked: RwSignal<bool>,
    is_saved: RwSignal<bool>,
    like_count: RwSignal<i32>,
    dislike_count: RwSignal<i32>,
    view_count: i32,
    comment_count: i32,
    on_like: impl Fn(leptos::ev::MouseEvent) + 'static,
    on_dislike: impl Fn(leptos::ev::MouseEvent) + 'static,
    on_save: impl Fn(leptos::ev::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="post-actions">
            <div class="action-buttons">
                <button
                    type="button"
                    class="action-btn like-btn"
                    class:active=move || is_liked.get()
                    on:click=on_like
                    aria-label="Like"
                >
                    <i class="peer-icon peer-icon-heart"/>
                    <span class="count">{move || like_count.get()}</span>
                </button>

                <button
                    type="button"
                    class="action-btn dislike-btn"
                    class:active=move || is_disliked.get()
                    on:click=on_dislike
                    aria-label="Dislike"
                >
                    <i class="peer-icon peer-icon-dislike"/>
                    <span class="count">{move || dislike_count.get()}</span>
                </button>

                <button
                    type="button"
                    class="action-btn save-btn"
                    class:active=move || is_saved.get()
                    on:click=on_save
                    aria-label="Save"
                >
                    <i class="peer-icon peer-icon-bookmark"/>
                </button>
            </div>

            <div class="post-stats">
                <span class="stat">
                    <i class="peer-icon peer-icon-eye"/>
                    {view_count}
                </span>
                <span class="stat">
                    <i class="peer-icon peer-icon-comment"/>
                    {comment_count}
                </span>
            </div>
        </div>
    }
}
