//! Comments component.
//!
//! Displays comments list with infinite scroll, nested replies,
//! comment creation, and like functionality.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::comments::{
    create_comment, like_comment, list_child_comments, list_comments, unlike_comment,
};
use crate::components::view_post::post_content::format_time_ago;
use crate::models::comment::Comment;

const COMMENTS_PER_PAGE: i32 = 10;

/// Comments section with infinite scroll.
#[component]
pub fn Comments(post_id: String, is_guest: bool) -> impl IntoView {
    let comments = RwSignal::new(Vec::<Comment>::new());
    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let total_count = RwSignal::new(0i32);

    let post_id_for_input = post_id.clone();
    let post_id_for_load = StoredValue::new(post_id.clone());

    // Load comments action
    let load_more = Action::new(move |_: &()| {
        let id = post_id_for_load.get_value();
        let current_offset = offset.get();
        async move { list_comments(id, current_offset, COMMENTS_PER_PAGE).await }
    });

    // Handle load results
    Effect::new(move |_| {
        if let Some(Ok(response)) = load_more.value().get() {
            let new_comments = response.affected_rows;
            let has_new = !new_comments.is_empty();
            let current_offset = offset.get();

            total_count.set(response.counter);

            if has_new {
                comments.update(|c| c.extend(new_comments));
                offset.update(|o| *o += COMMENTS_PER_PAGE);
            }

            has_more.set(has_new && response.counter > current_offset + COMMENTS_PER_PAGE);
            is_loading.set(false);
        }
    });

    // Initial load
    Effect::new(move |prev: Option<bool>| {
        if prev.is_none() {
            is_loading.set(true);
            load_more.dispatch(());
        }
        true
    });

    let on_load_more = move |_| {
        if !is_loading.get() && has_more.get() {
            is_loading.set(true);
            load_more.dispatch(());
        }
    };

    // Comment added callback - updates the comment list
    let on_comment_added = move |new_comment: Comment| {
        comments.update(|c| c.insert(0, new_comment));
        total_count.update(|t| *t += 1);
    };

    let show_input = !is_guest;
    let show_loading = move || is_loading.get();
    let show_load_more = move || !is_loading.get() && has_more.get();
    let show_empty = move || !is_loading.get() && comments.get().is_empty();

    view! {
        <div class="comments-container">
            <div class="comments-header">
                <h3 class="cmt_head xxl_font_size bold">"Comments"</h3>
                <span class="comment_total md_font_size">
                    <i class="peer-icon peer-icon-comment-dot"/>
                    <span class="comment_count">{move || total_count.get()}</span>
                </span>
            </div>

            <div class="comment_list">
                <div id="comments" class="inner">
                    <For
                        each=move || comments.get()
                        key=|c| c.commentid.clone()
                        children=move |comment| {
                            view! {
                                <CommentItem comment=comment is_guest=is_guest/>
                            }
                        }
                    />

                    {move || show_loading().then(|| view! {
                        <div class="comment-loading">
                            <i class="peer-icon peer-icon-spinner spin"/>
                            " Loading..."
                        </div>
                    })}

                    {move || show_load_more().then(|| view! {
                        <button class="load-more-comments btn-white" on:click=on_load_more>
                            "Load more comments"
                        </button>
                    })}

                    {move || show_empty().then(|| view! {
                        <div class="no-comments txt-color-gray md_font_size">
                            "No comments yet. Be the first to comment!"
                        </div>
                    })}
                </div>
            </div>

            // Comment input (authenticated only)
            {show_input.then(|| view! {
                <CommentInput post_id=post_id_for_input on_comment_added=on_comment_added/>
            })}
        </div>
    }
}

/// Single comment item.
#[component]
fn CommentItem(comment: Comment, is_guest: bool) -> impl IntoView {
    let is_liked = RwSignal::new(comment.isliked);
    let like_count = RwSignal::new(comment.amountlikes);
    let show_replies = RwSignal::new(false);
    let show_reply_form = RwSignal::new(false);

    let comment_id = comment.commentid.clone();
    let comment_id_for_like = comment.commentid.clone();
    let comment_id_for_replies = comment.commentid.clone();
    let post_id_for_reply = comment.postid.clone();
    let parent_id_for_reply = comment.commentid.clone();
    let has_replies = comment.amountreplies > 0;
    let reply_count = comment.amountreplies;
    let img_src = comment
        .user
        .img
        .clone()
        .unwrap_or_else(|| "/svg/noname.svg".to_string());
    let profile_url = format!("/profile/{}", comment.user.slug);
    let profile_url_clone = profile_url.clone();
    let time_ago = format_time_ago(&comment.createdat);

    let on_like = move |_| {
        if is_guest {
            return;
        }

        let id = comment_id_for_like.clone();
        let currently_liked = is_liked.get();

        // Optimistic update
        if currently_liked {
            is_liked.set(false);
            like_count.update(|c| *c = (*c - 1).max(0));
        } else {
            is_liked.set(true);
            like_count.update(|c| *c += 1);
        }

        spawn_local(async move {
            if currently_liked {
                let _ = unlike_comment(id).await;
            } else {
                let _ = like_comment(id).await;
            }
        });
    };

    let show_reply_btn = !is_guest;

    view! {
        <div class="comment_item" id=comment_id>
            <div class="commenter-pic">
                <a href=profile_url.clone()>
                    <img class="profile-picture" src=img_src alt="Commenter avatar"/>
                </a>
            </div>
            <div class="comment_body">
                <div class="commenter_info">
                    <a href=profile_url_clone>
                        <span class="cmt_userName md_font_size bold">{comment.user.username}</span>
                    </a>
                    <span class="timeago txt-color-gray sm_font_size">{time_ago}</span>
                </div>
                <div class="comment_text md_font_size">{comment.content}</div>

                <div class="comment_reply_container">
                    {show_reply_btn.then(|| view! {
                        <span
                            class="reply_btn"
                            on:click=move |_| show_reply_form.update(|s| *s = !*s)
                        >
                            <a class="md_font_size bold">"Reply"</a>
                        </span>
                    })}

                    {has_replies.then(|| view! {
                        <span
                            class="show_reply txt-color-gray md_font_size"
                            on:click=move |_| show_replies.update(|s| *s = !*s)
                        >
                            {move || {
                                if show_replies.get() {
                                    "Hide replies".to_string()
                                } else {
                                    format!("Show {} replies...", reply_count)
                                }
                            }}
                        </span>
                    })}
                </div>

                // Reply form
                {move || show_reply_form.get().then(|| view! {
                    <ReplyForm
                        post_id=post_id_for_reply.clone()
                        parent_id=parent_id_for_reply.clone()
                        on_reply_added=move || show_reply_form.set(false)
                    />
                })}

                // Nested replies
                {move || show_replies.get().then(|| view! {
                    <ChildComments
                        parent_id=comment_id_for_replies.clone()
                        is_guest=is_guest
                    />
                })}
            </div>

            // Like button
            <div
                class="comment_like"
                class:liked=move || is_liked.get()
                class:disabled=is_guest
                on:click=on_like
            >
                <i class="peer-icon peer-icon-like"/>
                <span>{move || like_count.get()}</span>
            </div>
        </div>
    }
}

/// Child comments (replies) component.
#[component]
fn ChildComments(parent_id: String, is_guest: bool) -> impl IntoView {
    let replies = RwSignal::new(Vec::<Comment>::new());
    let is_loading = RwSignal::new(true);

    // Load replies
    let load_action = Action::new({
        let id = parent_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { list_child_comments(id, 0, COMMENTS_PER_PAGE).await }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(response)) = load_action.value().get() {
            replies.set(response.affected_rows);
            is_loading.set(false);
        }
    });

    // Trigger load
    Effect::new(move |prev: Option<bool>| {
        if prev.is_none() {
            load_action.dispatch(());
        }
        true
    });

    view! {
        <div class="child-comments">
            {move || is_loading.get().then(|| view! {
                <div class="loading-replies txt-color-gray sm_font_size">
                    <i class="peer-icon peer-icon-spinner spin"/>
                    " Loading replies..."
                </div>
            })}

            <For
                each=move || replies.get()
                key=|c| c.commentid.clone()
                children=move |reply| {
                    let img_src = reply
                        .user
                        .img
                        .clone()
                        .unwrap_or_else(|| "/svg/noname.svg".to_string());
                    let profile_url = format!("/profile/{}", reply.user.slug);
                    let profile_url_clone = profile_url.clone();
                    let time_ago = format_time_ago(&reply.createdat);
                    let is_liked = RwSignal::new(reply.isliked);
                    let like_count = RwSignal::new(reply.amountlikes);
                    let reply_id = reply.commentid.clone();

                    let on_like = move |_| {
                        if is_guest {
                            return;
                        }
                        let id = reply_id.clone();
                        let currently_liked = is_liked.get();

                        if currently_liked {
                            is_liked.set(false);
                            like_count.update(|c| *c = (*c - 1).max(0));
                        } else {
                            is_liked.set(true);
                            like_count.update(|c| *c += 1);
                        }

                        spawn_local(async move {
                            if currently_liked {
                                let _ = unlike_comment(id).await;
                            } else {
                                let _ = like_comment(id).await;
                            }
                        });
                    };

                    view! {
                        <div class="reply_item">
                            <div class="commenter-pic">
                                <a href=profile_url.clone()>
                                    <img class="profile-picture" src=img_src alt="Reply author"/>
                                </a>
                            </div>
                            <div class="comment_body">
                                <div class="commenter_info">
                                    <a href=profile_url_clone>
                                        <span class="cmt_userName md_font_size bold">
                                            {reply.user.username}
                                        </span>
                                    </a>
                                    <span class="timeago txt-color-gray sm_font_size">{time_ago}</span>
                                </div>
                                <div class="comment_text md_font_size">{reply.content}</div>
                            </div>
                            <div
                                class="comment_like"
                                class:liked=move || is_liked.get()
                                class:disabled=is_guest
                                on:click=on_like
                            >
                                <i class="peer-icon peer-icon-like"/>
                                <span>{move || like_count.get()}</span>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// Comment input form.
#[component]
fn CommentInput<F>(post_id: String, on_comment_added: F) -> impl IntoView
where
    F: Fn(Comment) + Clone + 'static,
{
    let content = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);

    // Create action for submitting comments
    let submit_action = Action::new({
        let id = post_id.clone();
        move |text: &String| {
            let id = id.clone();
            let text = text.clone();
            async move { create_comment(id, text, None).await }
        }
    });

    // Handle submit results
    let callback = on_comment_added.clone();
    Effect::new(move |_| {
        if let Some(result) = submit_action.value().get() {
            is_submitting.set(false);
            if let Ok(new_comment) = result {
                content.set(String::new());
                callback(new_comment);
            }
        }
    });

    let on_submit = move |_| {
        let text = content.get().trim().to_string();
        if text.is_empty() || is_submitting.get() {
            return;
        }
        is_submitting.set(true);
        submit_action.dispatch(text);
    };

    let is_not_submitting = move || !is_submitting.get();

    view! {
        <div id="post_comment" class="post_comment">
            <textarea
                placeholder="Share your thoughts..."
                prop:value=move || content.get()
                on:input=move |ev| content.set(event_target_value(&ev))
                disabled=move || is_submitting.get()
            />
            <button
                on:click=on_submit
                disabled=move || is_submitting.get() || content.get().trim().is_empty()
            >
                {move || if is_not_submitting() {
                    view! { <i class="peer-icon peer-icon-arrow-right"/> }.into_any()
                } else {
                    view! { <i class="peer-icon peer-icon-spinner spin"/> }.into_any()
                }}
            </button>
        </div>
    }
}

/// Reply form (similar to comment input but for replies).
#[component]
fn ReplyForm<F>(post_id: String, parent_id: String, on_reply_added: F) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let content = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);

    // Create action for submitting replies
    let submit_action = Action::new({
        let id = post_id.clone();
        let parent = parent_id.clone();
        move |text: &String| {
            let id = id.clone();
            let parent = parent.clone();
            let text = text.clone();
            async move { create_comment(id, text, Some(parent)).await }
        }
    });

    // Handle submit results
    let callback = on_reply_added.clone();
    Effect::new(move |_| {
        if let Some(result) = submit_action.value().get() {
            is_submitting.set(false);
            if result.is_ok() {
                content.set(String::new());
                callback();
            }
        }
    });

    let on_submit = move |_| {
        let text = content.get().trim().to_string();
        if text.is_empty() || is_submitting.get() {
            return;
        }
        is_submitting.set(true);
        submit_action.dispatch(text);
    };

    let is_not_submitting = move || !is_submitting.get();

    view! {
        <div class="reply_form">
            <textarea
                placeholder="Write a reply..."
                prop:value=move || content.get()
                on:input=move |ev| content.set(event_target_value(&ev))
                disabled=move || is_submitting.get()
                rows="2"
            />
            <button
                class="btn-blue sm_font_size"
                on:click=on_submit
                disabled=move || is_submitting.get() || content.get().trim().is_empty()
            >
                {move || if is_not_submitting() {
                    view! { "Reply" }.into_any()
                } else {
                    view! { <i class="peer-icon peer-icon-spinner spin"/> }.into_any()
                }}
            </button>
        </div>
    }
}
