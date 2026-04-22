//! Submit button and logic for post creation.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::api::posts::check_post_eligibility;
#[cfg(feature = "hydrate")]
use crate::api::posts::upload_post_files;
use crate::api::posts::create_post;
use crate::components::toast::{ToastType, use_toast};
use crate::pages::new_post::NewPostContext;

/// Submit button with loading state and submission logic.
#[component]
pub fn SubmitButton() -> impl IntoView {
    let ctx = NewPostContext::use_context();
    let toast = use_toast();
    let navigate = use_navigate();

    let submit_action = Action::new(move |_: &()| {
        let ctx = ctx;
        let toast = toast;
        let navigate = navigate.clone();

        async move {
            // Validate form
            if let Err(err) = ctx.validate() {
                ctx.set_error.set(Some(err.clone()));
                toast.show(&err, ToastType::Error);
                return Err(err);
            }

            ctx.set_is_submitting.set(true);
            ctx.set_error.set(None);

            // Step 1: Get eligibility token
            let eligibility = match check_post_eligibility().await {
                Ok(resp) => resp,
                Err(e) => {
                    let err = format!("Failed to check eligibility: {}", e);
                    ctx.set_error.set(Some(err.clone()));
                    ctx.set_is_submitting.set(false);
                    toast.show(&err, ToastType::Error);
                    return Err(err);
                }
            };

            let token = match eligibility.token() {
                Some(t) => t.to_string(),
                None => {
                    let err = format!(
                        "Not eligible to post: {}",
                        eligibility.meta.response_message
                    );
                    ctx.set_error.set(Some(err.clone()));
                    ctx.set_is_submitting.set(false);
                    toast.show(&err, ToastType::Error);
                    return Err(err);
                }
            };
            #[cfg(not(feature = "hydrate"))]
            let _ = token;

            // Step 2: Upload files (if any)
            let media_files = ctx.media_files.get();
            let uploaded_files = if !media_files.is_empty() {
                #[cfg(feature = "hydrate")]
                {
                    use base64::{Engine, engine::general_purpose::STANDARD};

                    let files: Vec<(String, String, String)> = media_files
                        .iter()
                        .map(|f| {
                            (
                                f.name.clone(),
                                f.mime_type.clone(),
                                STANDARD.encode(&f.data),
                            )
                        })
                        .collect();

                    match upload_post_files(token.clone(), files).await {
                        Ok(resp) => {
                            if resp.is_success() {
                                resp.uploaded_files().map(|s| s.to_string())
                            } else {
                                let err = format!("Upload failed: {}", resp.response_code);
                                ctx.set_error.set(Some(err.clone()));
                                ctx.set_is_submitting.set(false);
                                toast.show(&err, ToastType::Error);
                                return Err(err);
                            }
                        }
                        Err(e) => {
                            let err = format!("Upload failed: {}", e);
                            ctx.set_error.set(Some(err.clone()));
                            ctx.set_is_submitting.set(false);
                            toast.show(&err, ToastType::Error);
                            return Err(err);
                        }
                    }
                }
                #[cfg(not(feature = "hydrate"))]
                None
            } else {
                None
            };

            // Step 3: Create post
            let mut input = ctx.build_input();
            input.uploaded_files = uploaded_files;

            // Handle cover file
            #[cfg(feature = "hydrate")]
            {
                use base64::{Engine, engine::general_purpose::STANDARD};

                if let Some(cover) = ctx.cover_file.get() {
                    let data_url = format!(
                        "data:{};base64,{}",
                        cover.mime_type,
                        STANDARD.encode(&cover.data)
                    );
                    input.cover = Some(vec![data_url]);
                }
            }

            match create_post(input).await {
                Ok(resp) => {
                    if resp.is_success() {
                        ctx.set_is_submitting.set(false);
                        toast.show("Post created successfully!", ToastType::Success);

                        // Navigate to profile or post
                        if let Some(post_id) = resp.post_id() {
                            navigate(&format!("/post/{}", post_id), Default::default());
                        } else {
                            navigate("/profile", Default::default());
                        }

                        Ok(())
                    } else {
                        let err = format!("Failed to create post: {}", resp.meta.response_message);
                        ctx.set_error.set(Some(err.clone()));
                        ctx.set_is_submitting.set(false);
                        toast.show(&err, ToastType::Error);
                        Err(err)
                    }
                }
                Err(e) => {
                    let err = format!("Failed to create post: {}", e);
                    ctx.set_error.set(Some(err.clone()));
                    ctx.set_is_submitting.set(false);
                    toast.show(&err, ToastType::Error);
                    Err(err)
                }
            }
        }
    });

    let is_pending = submit_action.pending();

    let handle_submit = move |_| {
        submit_action.dispatch(());
    };

    view! {
        <button
            type="button"
            class="btn btn-primary submit-btn"
            on:click=handle_submit
            disabled=move || ctx.is_submitting.get() || is_pending.get()
        >
            {move || {
                if ctx.is_submitting.get() || is_pending.get() {
                    view! {
                        <i class="peer-icon peer-icon-loader spinning"/>
                        " Posting..."
                    }.into_any()
                } else {
                    view! {
                        <i class="peer-icon peer-icon-send"/>
                        " Post"
                    }.into_any()
                }
            }}
        </button>
    }
}
