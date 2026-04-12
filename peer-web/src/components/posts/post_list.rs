//! Post list with infinite scroll.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::posts::{list_ad_posts, list_posts};
use crate::components::posts::PostCard;
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

    // Loader element ref for intersection observer
    let loader_ref = NodeRef::<leptos::html::Div>::new();

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

    // Set up intersection observer for infinite scroll
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        use std::sync::{Arc, Mutex};

        let Some(el) = loader_ref.get() else {
            return;
        };

        // Create observer callback
        let callback = Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
            for entry in entries.iter() {
                let entry: web_sys::IntersectionObserverEntry = entry.unchecked_into();
                if entry.is_intersecting() && !is_loading.get() && has_more.get() {
                    load_posts();
                }
            }
        });

        let options = web_sys::IntersectionObserverInit::new();
        options.set_root_margin("0px 0px 200px 0px");
        options.set_threshold(&JsValue::from_f64(0.1));

        if let Ok(observer) = web_sys::IntersectionObserver::new_with_options(
            callback.as_ref().unchecked_ref(),
            &options,
        ) {
            observer.observe(&el);

            // Leak callback to keep it alive for the observer's lifetime.
            // The observer holds a reference to it via JS, so dropping it
            // here would invalidate the callback pointer.
            callback.forget();

            // Wrap observer in Arc<Mutex> so it's Send + Sync for on_cleanup
            let observer = Arc::new(Mutex::new(Some(observer)));
            on_cleanup(move || {
                if let Ok(mut guard) = observer.lock() {
                    if let Some(obs) = guard.take() {
                        obs.disconnect();
                    }
                }
            });
        }
    });

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

        <div id="post_loader" node_ref=loader_ref class="post-loader">
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
