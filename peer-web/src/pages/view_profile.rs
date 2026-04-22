//! View Profile page.
//!
//! Displays another user's profile with follow/block/report actions.

use std::rc::Rc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::api::profile::{get_profile, list_user_posts};
use crate::components::auth_guard::AuthGuard;
use crate::components::filters::{ContentFilter, SortFilter};
use crate::components::layout::SiteShell;
use crate::components::posts::PostCard;
use crate::components::profile::{ProfileHeader, ProfileHeaderSkeleton};
use crate::hooks::use_infinite_scroll;
use crate::models::post::FeedItem;
use crate::state::filters::{provide_filter_context, use_filter_state};

/// View another user's profile page.
///
/// Route: `/profile/:slug` or `/u/:slug`
#[component]
pub fn ViewProfilePage() -> impl IntoView {
    provide_filter_context();

    let params = use_params_map();

    // Extract user slug from route params
    let user_slug = move || params.get().get("slug").map(|s| s.to_string());

    view! {
        <AuthGuard>
            <SiteShell id="view-profile" modifier="profile-layout">
                <ViewProfileHeader/>
                <ViewProfileFilterSidebar/>
                <ViewProfileMainContent user_slug/>
                <RightSidebar/>
            </SiteShell>
        </AuthGuard>
    }
}

/// Main content area for viewing another user's profile.
#[component]
fn ViewProfileMainContent(
    user_slug: impl Fn() -> Option<String> + Send + Sync + 'static,
) -> impl IntoView {
    // Fetch the profile based on the slug
    let profile_resource = Resource::new(user_slug, |slug| async move {
        match slug {
            Some(slug) => get_profile(Some(slug), None).await,
            None => Err(ServerFnError::new("No user specified")),
        }
    });

    view! {
        <main class="site_main profile-main">
            <Suspense fallback=move || view! { <ProfileHeaderSkeleton/> }>
                {move || {
                    profile_resource.get().map(|result| {
                        match result {
                            Ok(profile) => {
                                let page_title = format!("{} - Peer Network", profile.username);
                                let user_id = profile.id.clone();

                                view! {
                                    <Title text=page_title/>
                                    <ProfileHeader
                                        profile=profile
                                        is_own_profile=false
                                    />
                                    <UserPostList user_id=user_id/>
                                }.into_any()
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                if error_msg.contains("not found") || error_msg.contains("Profile not found") {
                                    view! {
                                        <Title text="Profile Not Found - Peer Network"/>
                                        <ProfileNotFound/>
                                    }.into_any()
                                } else {
                                    view! {
                                        <Title text="Error - Peer Network"/>
                                        <ProfileError message=error_msg/>
                                    }.into_any()
                                }
                            }
                        }
                    })
                }}
            </Suspense>
        </main>
    }
}

/// Number of posts to load per batch.
const POSTS_PER_PAGE: i32 = 20;

/// User's post list with infinite scroll.
#[component]
fn UserPostList(user_id: String) -> impl IntoView {
    let filters = use_filter_state();

    let feed = RwSignal::new(Vec::<FeedItem>::new());
    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);

    // Reset feed when filters change
    Effect::new(move |prev_version: Option<u32>| {
        let current_version = filters.version.get();
        if prev_version.is_some() && prev_version != Some(current_version) {
            feed.set(Vec::new());
            offset.set(0);
            has_more.set(true);
        }
        current_version
    });

    let uid = user_id.clone();
    let load_posts = Rc::new(move || {
        if is_loading.get() || !has_more.get() {
            return;
        }
        is_loading.set(true);

        let user_id = uid.clone();
        let current_offset = offset.get();
        let filter_by = filters.get_filter_by();
        let sort_by = filters.sort_by.get();

        spawn_local(async move {
            match list_user_posts(
                user_id,
                filter_by,
                None,
                sort_by,
                current_offset,
                POSTS_PER_PAGE,
            )
            .await
            {
                Ok(response) => {
                    let new_posts = response.affected_rows;
                    let has_new = !new_posts.is_empty();
                    if has_new {
                        feed.update(|f| {
                            for post in new_posts {
                                f.push(FeedItem::Post(post));
                            }
                        });
                        offset.update(|o| *o += POSTS_PER_PAGE);
                    }
                    has_more.set(has_new && response.counter > current_offset + POSTS_PER_PAGE);
                }
                Err(e) => {
                    leptos::logging::error!("Failed to load posts: {:?}", e);
                }
            }
            is_loading.set(false);
        });
    });

    let scroll = use_infinite_scroll(is_loading, has_more, {
        let load_posts = load_posts.clone();
        move || load_posts()
    });

    // Initial load
    {
        let load_posts = load_posts.clone();
        Effect::new(move |_| {
            if feed.get().is_empty() && !is_loading.get() {
                load_posts();
            }
        });
    }

    view! {
        <div class="profile-posts">
            <h3 class="posts-heading">"Posts"</h3>
            <div class="posts-grid">
                <For
                    each=move || feed.get()
                    key=|item| item.post().id.clone()
                    children=move |item| {
                        view! { <PostCard item=item/> }
                    }
                />
            </div>

            <div class="post-loader" node_ref=scroll.loader_ref>
                <Show when=move || is_loading.get()>
                    <div class="loading-indicator">
                        <img src="/svg/logo_farbe.svg" alt="Loading..." class="loading-spinner"/>
                    </div>
                </Show>
            </div>

            <Show when=move || feed.get().is_empty() && !is_loading.get() && !has_more.get()>
                <EmptyPosts/>
            </Show>
        </div>
    }
}

/// Filter sidebar for view profile (content type + sort only).
#[component]
fn ViewProfileFilterSidebar() -> impl IntoView {
    let filters = use_filter_state();

    view! {
        <aside
            class="site_sidebar left-sidebar left-sidebar-profile"
            class:collapsed=move || filters.is_collapsed.get()
        >
            <div class="inner-scroll for-filters">
                <div class="inner-scroll-filters">
                    <ContentFilter/>
                    <SortFilter/>
                </div>
            </div>
        </aside>
    }
}

/// Page header for view profile.
#[component]
fn ViewProfileHeader() -> impl IntoView {
    view! {
        <header class="site_header">
            <div class="header-content">
                <a href="/dashboard" class="back-button" aria-label="Back to dashboard">
                    <i class="peer-icon peer-icon-arrow-left"></i>
                </a>
                <h1>"Profile"</h1>
            </div>
        </header>
    }
}

/// Right sidebar with menu and widgets.
#[component]
fn RightSidebar() -> impl IntoView {
    view! {
        <aside class="site_sidebar right-sidebar">
            <nav class="sidebar-menu">
                <a href="/dashboard" class="menu-item">
                    <i class="peer-icon peer-icon-home"></i>
                    <span>"Dashboard"</span>
                </a>
                <a href="/profile" class="menu-item">
                    <i class="peer-icon peer-icon-user"></i>
                    <span>"My Profile"</span>
                </a>
                <a href="/newpost" class="menu-item">
                    <i class="peer-icon peer-icon-plus"></i>
                    <span>"New Post"</span>
                </a>
            </nav>
        </aside>
    }
}

/// Profile not found state.
#[component]
fn ProfileNotFound() -> impl IntoView {
    view! {
        <div class="profile-not-found">
            <i class="peer-icon peer-icon-user-x"></i>
            <h2>"Profile Not Found"</h2>
            <p>"The user you're looking for doesn't exist or has been removed."</p>
            <a href="/dashboard" class="button btn-blue">"Go to Dashboard"</a>
        </div>
    }
}

/// Error state component.
#[component]
fn ProfileError(message: String) -> impl IntoView {
    view! {
        <div class="profile-error">
            <i class="peer-icon peer-icon-alert-circle"></i>
            <h2>"Something went wrong"</h2>
            <p>{message}</p>
            <a href="/dashboard" class="button btn-blue">"Go to Dashboard"</a>
        </div>
    }
}

/// Empty posts state for other users.
#[component]
fn EmptyPosts() -> impl IntoView {
    view! {
        <div class="empty-posts">
            <i class="peer-icon peer-icon-image"></i>
            <p>"This user hasn't posted anything yet."</p>
        </div>
    }
}

/// Post list skeleton loader.
#[component]
fn PostListSkeleton() -> impl IntoView {
    view! {
        <div class="post-list skeleton">
            <div class="skeleton-post"></div>
            <div class="skeleton-post"></div>
            <div class="skeleton-post"></div>
        </div>
    }
}
