//! View post page.
//!
//! Displays a single post with full content, media, comments, and actions.
//! Supports both guest mode (unauthenticated) and authenticated viewing.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::api::comments::{get_post, guest_get_post};
use crate::api::posts::post_action;
use crate::components::view_post::{Comments, PostActions, PostContent, PostHeader, PostMedia};
use crate::hooks::use_mobile_redirect;
use crate::models::post::{Post, PostActionType};
use crate::state::auth::use_auth;

/// Get the base URL for sharing.
fn get_base_url() -> String {
    #[cfg(feature = "hydrate")]
    {
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "https://peer-network.com".to_string())
    }
    #[cfg(not(feature = "hydrate"))]
    {
        std::env::var("BASE_URL").unwrap_or_else(|_| "https://peer-network.com".to_string())
    }
}

/// View single post page.
///
/// Supports both guest mode (no auth) and authenticated mode.
#[component]
pub fn ViewPostPage() -> impl IntoView {
    let params = use_params_map();
    let auth = use_auth();

    let post_id = move || {
        params
            .get()
            .get("id")
            .map(|s| s.to_string())
            .unwrap_or_default()
    };
    let post_id_signal = Signal::derive(post_id);
    let is_guest = Signal::derive(move || !auth.is_authenticated.get());

    // Mobile deep-link redirect for guest users
    use_mobile_redirect(post_id_signal, is_guest.get_untracked());

    // Fetch post based on auth state
    let post_resource = Resource::new(
        move || (post_id(), auth.is_authenticated.get()),
        |(id, is_auth)| async move {
            if id.is_empty() {
                return Err(ServerFnError::new("No post ID provided"));
            }
            if is_auth {
                get_post(id).await
            } else {
                guest_get_post(id).await
            }
        },
    );

    // Track post view for authenticated users (fire-and-forget)
    Effect::new(move |prev_tracked: Option<bool>| {
        // Only track once per mount
        if prev_tracked.is_some() {
            return true;
        }

        if !auth.is_authenticated.get() {
            return true;
        }

        if let Some(Ok(_)) = post_resource.get() {
            let id = post_id();
            if !id.is_empty() {
                spawn_local(async move {
                    let _ = post_action(id, PostActionType::View).await;
                });
            }
        }
        true
    });

    view! {
        <Suspense fallback=move || view! { <PostSkeleton/> }>
            {move || {
                post_resource.get().map(|result| {
                    match result {
                        Ok(post) => {
                            let is_guest = !auth.is_authenticated.get();
                            view! {
                                <PostSeoMeta post=post.clone()/>
                                <ViewPostContent post=post is_guest=is_guest/>
                            }
                            .into_any()
                        }
                        Err(_) => view! {
                            <Title text="Post Not Found - Peer Network"/>
                            <PostNotFound/>
                        }
                        .into_any(),
                    }
                })
            }}
        </Suspense>
    }
}

/// SEO meta tags for the post.
#[component]
fn PostSeoMeta(post: Post) -> impl IntoView {
    let title = format!("{} - Peer Network", post.title);
    let description = post
        .mediadescription
        .clone()
        .unwrap_or_else(|| post.title.clone());
    let image = post
        .cover
        .clone()
        .or_else(|| post.media.clone())
        .unwrap_or_else(|| "/svg/og-default.svg".to_string());
    let url = format!("{}/post/{}", get_base_url(), post.id);

    view! {
        <Title text=title.clone()/>
        <Meta name="description" content=description.clone()/>

        // Open Graph
        <Meta property="og:title" content=title.clone()/>
        <Meta property="og:description" content=description.clone()/>
        <Meta property="og:image" content=image.clone()/>
        <Meta property="og:url" content=url.clone()/>
        <Meta property="og:type" content="article"/>

        // Twitter Card
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=title/>
        <Meta name="twitter:description" content=description/>
        <Meta name="twitter:image" content=image/>
    }
}

/// Main view post content layout.
#[component]
fn ViewPostContent(post: Post, is_guest: bool) -> impl IntoView {
    let post_id = post.id.clone();
    let post_user = post.user.clone();
    let post_for_media = post.clone();
    let post_for_actions = post.clone();
    let post_for_content = post.clone();

    view! {
        <div id="viewpost" class="viewpost">
            <div id="viewpost-container" class="inner-container">
                <div class="viewpost-left">
                    <PostMedia post=post_for_media/>
                    <PostActions post=post_for_actions is_guest=is_guest/>
                </div>
                <div class="viewpost-right">
                    <PostHeader user=post_user is_guest=is_guest/>
                    <PostContent post=post_for_content/>
                    <Comments post_id=post_id is_guest=is_guest/>
                </div>
                <CloseButton/>
            </div>
        </div>

        // Guest mode CTA banner
        <Show when=move || is_guest>
            <GuestModeBanner/>
        </Show>
    }
}

/// Close button component.
#[component]
fn CloseButton() -> impl IntoView {
    let navigate_back = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                let history = window.history().ok();
                if let Some(h) = history {
                    let _ = h.back();
                }
            }
        }
    };

    view! {
        <button class="viewpost-close" on:click=navigate_back aria-label="Close post">
            <i class="peer-icon peer-icon-cancel"/>
        </button>
    }
}

/// Post not found error state.
#[component]
fn PostNotFound() -> impl IntoView {
    view! {
        <div class="post-not-found">
            <div class="not-found-content">
                <i class="peer-icon peer-icon-warning-alt"/>
                <h2 class="xxl_font_size">"Post Not Found"</h2>
                <p class="md_font_size txt-color-gray">
                    "The post you are looking for does not exist or has been removed."
                </p>
                <a href="/dashboard" class="button btn-blue">"Go to Dashboard"</a>
            </div>
        </div>
    }
}

/// Guest mode banner showing sign-up CTA.
#[component]
fn GuestModeBanner() -> impl IntoView {
    view! {
        <div class="view-only-mode">
            <div class="view-only-mode-heading">
                <i class="peer-icon peer-icon-eye-open"/>
                <span class="xl_font_size bold">"View-only mode"</span>
            </div>
            <p class="md_font_size txt-color-gray">
                "Sign up to interact and access everything!"
            </p>
            <a href="/register" class="button btn-blue">"Sign Up"</a>
        </div>
    }
}

/// Loading skeleton for post.
#[component]
fn PostSkeleton() -> impl IntoView {
    view! {
        <div id="viewpost" class="viewpost">
            <div id="viewpost-container" class="inner-container">
                <div class="viewpost-left">
                    <div class="skeleton skeleton-media"/>
                    <div class="skeleton skeleton-actions"/>
                </div>
                <div class="viewpost-right">
                    <div class="skeleton skeleton-header"/>
                    <div class="skeleton skeleton-content"/>
                    <div class="skeleton skeleton-comments"/>
                </div>
            </div>
        </div>
    }
}
