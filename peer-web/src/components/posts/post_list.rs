//! Post list with infinite scroll.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::posts::{list_ad_posts, list_posts};
use crate::components::posts::PostCard;
use crate::hooks::use_infinite_scroll;
use crate::models::post::{AdvertisementPost, FeedItem};
use crate::state::filters::use_filter_state;

/// Number of posts to load per batch.
const POSTS_PER_PAGE: i32 = 20;

/// Interval for inserting ads (every N posts).
const AD_INTERVAL: usize = 5;

/// Post list with infinite scroll.
#[component]
pub fn PostList() -> impl IntoView {
    let filters = use_filter_state();

    // Feed state
    let feed = RwSignal::new(Vec::<FeedItem>::new());
    let offset = RwSignal::new(0i32);
    let is_loading = RwSignal::new(false);
    let has_more = RwSignal::new(true);
    let ad_posts = RwSignal::new(Vec::<AdvertisementPost>::new());
    let ads_loaded = RwSignal::new(false);

    // Load ads once on mount
    Effect::new(move |_| {
        if !ads_loaded.get() {
            ads_loaded.set(true);
            spawn_local(async move {
                if let Ok(response) = list_ad_posts(None, 0, 50, None, None).await {
                    ad_posts.set(response.affected_rows);
                }
            });
        }
    });

    // Reset feed when filters change
    Effect::new(move |prev_version: Option<u32>| {
        let current_version = filters.version.get();

        // Skip initial run
        if prev_version.is_some() && prev_version != Some(current_version) {
            feed.set(Vec::new());
            offset.set(0);
            has_more.set(true);
        }

        current_version
    });

    // Load posts function
    let load_posts = move || {
        if is_loading.get() || !has_more.get() {
            return;
        }

        is_loading.set(true);

        let filter_by = filters.get_filter_by();
        let sort_by = filters.sort_by.get();
        let title = filters.title_query.get();
        let tag = filters.tag_query.get();
        let current_offset = offset.get();
        let current_ads = ad_posts.get();

        spawn_local(async move {
            let title_opt = if title.is_empty() { None } else { Some(title) };
            let tag_opt = if tag.is_empty() { None } else { Some(tag) };

            match list_posts(
                filter_by,
                None,
                sort_by,
                title_opt,
                tag_opt,
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
                            let start_idx = f.len();

                            for (i, post) in new_posts.into_iter().enumerate() {
                                let feed_idx = start_idx + i;

                                // Insert ad every AD_INTERVAL posts
                                if feed_idx > 0 && feed_idx % AD_INTERVAL == 0 {
                                    let ad_idx = (feed_idx / AD_INTERVAL - 1) % current_ads.len().max(1);
                                    if let Some(ad) = current_ads.get(ad_idx) {
                                        f.push(FeedItem::Ad {
                                            post: ad.post.clone(),
                                            ad_info: ad.advertisement.clone(),
                                        });
                                    }
                                }

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
    };

    // Set up infinite scroll via shared hook
    let scroll = use_infinite_scroll(is_loading, has_more, load_posts);

    // Initial load
    Effect::new(move |_| {
        if feed.get().is_empty() && !is_loading.get() {
            load_posts();
        }
    });

    view! {
        <div id="allpost" class="list_all_post">
            <For
                each=move || feed.get()
                key=|item| item.id().to_string()
                children=move |item| {
                    view! { <PostCard item=item/> }
                }
            />
        </div>

        <div id="post_loader" node_ref=scroll.loader_ref class="post-loader">
            <Show when=move || is_loading.get()>
                <div class="loading-indicator">
                    <img src="/svg/logo_farbe.svg" alt="Loading..." class="loading-spinner"/>
                </div>
            </Show>
        </div>

        <Show when=move || feed.get().is_empty() && !is_loading.get() && !has_more.get()>
            <div class="no_post_found">
                <p>"No posts found..."</p>
            </div>
        </Show>
    }
}
