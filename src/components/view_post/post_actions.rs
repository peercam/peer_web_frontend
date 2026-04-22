//! Post actions component.
//!
//! Provides like, dislike, save, share, and report functionality.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::posts::post_action;
use crate::models::post::{Post, PostActionType};

/// Post action bar (like, dislike, save, share, report).
#[component]
pub fn PostActions(post: Post, is_guest: bool) -> impl IntoView {
    let is_liked = RwSignal::new(post.isliked);
    let is_disliked = RwSignal::new(post.isdisliked);
    let is_saved = RwSignal::new(post.issaved);
    let like_count = RwSignal::new(post.amountlikes);
    let dislike_count = RwSignal::new(post.amountdislikes);
    let show_share = RwSignal::new(false);
    let show_more = RwSignal::new(false);

    let post_id = post.id.clone();
    // Use StoredValue for post IDs to make closures Copy
    let post_id_like = StoredValue::new(post.id.clone());
    let post_id_dislike = StoredValue::new(post.id.clone());
    let post_id_save = StoredValue::new(post.id.clone());
    let post_id_report = StoredValue::new(post.id.clone());

    let on_like = move |_| {
        if is_guest {
            return;
        }

        let id = post_id_like.get_value();
        let currently_liked = is_liked.get();
        let currently_disliked = is_disliked.get();

        // Optimistic update
        if currently_liked {
            is_liked.set(false);
            like_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_liked.set(true);
            like_count.update(|c| *c += 1);
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

    let on_dislike = move |_| {
        if is_guest {
            return;
        }

        let id = post_id_dislike.get_value();
        let currently_liked = is_liked.get();
        let currently_disliked = is_disliked.get();

        if currently_disliked {
            is_disliked.set(false);
            dislike_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_disliked.set(true);
            dislike_count.update(|c| *c += 1);
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
        ev.prevent_default();
        if is_guest {
            return;
        }

        let id = post_id_save.get_value();
        let currently_saved = is_saved.get();
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

    let on_report = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        if is_guest {
            return;
        }

        let id = post_id_report.get_value();
        spawn_local(async move {
            let _ = post_action(id, PostActionType::Report).await;
        });
    };

    let on_share = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        show_share.set(true);
    };

    let on_toggle_more = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        show_more.update(|s| *s = !*s);
    };

    let show_report = !is_guest;
    let show_save_btn = !is_guest;

    view! {
        <div class="postview_footer">
            <div class="social md_font_size">
                <div class="post-view txt-color-gray">
                    <i class="peer-icon peer-icon-eye-open"/>
                    <span>{post.amountviews}</span>
                </div>

                <div
                    class="post-like"
                    class:active=move || is_liked.get()
                    class:disabled=is_guest
                    on:click=on_like
                >
                    <i class="peer-icon peer-icon-like"/>
                    <span>{move || like_count.get()}</span>
                </div>

                <div
                    class="post-dislike"
                    class:active=move || is_disliked.get()
                    class:disabled=is_guest
                    on:click=on_dislike
                >
                    <i class="peer-icon peer-icon-dislike"/>
                    <span>{move || dislike_count.get()}</span>
                </div>

                <div class="post-comments">
                    <i class="peer-icon peer-icon-comment-alt"/>
                    <span>{post.amountcomments}</span>
                </div>
            </div>

            <div class="more md_font_size">
                <ul>
                    <li>
                        <a href="#" class="morebtn" on:click=on_toggle_more>
                            <span class="textval">"More"</span>
                            <span class="dots"/>
                        </a>
                    </li>
                    <Show when=move || show_more.get()>
                        <ul class="sublist">
                            <Show when=move || show_report>
                                <li>
                                    <a href="#" class="reportpost" on:click=on_report>
                                        <i class="peer-icon peer-icon-flag-fill"/>
                                        <span>"Report post"</span>
                                    </a>
                                </li>
                            </Show>
                            <li class="sharelinks">
                                <a href="#" class="share" on:click=on_share>
                                    <i class="peer-icon peer-icon-share-link"/>
                                    " Share"
                                </a>
                            </li>
                            <Show when=move || show_save_btn>
                                <li>
                                    <a
                                        href="#"
                                        class="save"
                                        class:active=move || is_saved.get()
                                        on:click=on_save
                                    >
                                        <i class="peer-icon peer-icon-save-fill"/>
                                        {move || if is_saved.get() { " Saved" } else { " Save" }}
                                    </a>
                                </li>
                            </Show>
                        </ul>
                    </Show>
                </ul>
            </div>
        </div>

        // Share modal
        {move || show_share.get().then(|| view! {
            <super::ShareModal
                post_id=post_id.clone()
                on_close=move || show_share.set(false)
            />
        })}
    }
}
