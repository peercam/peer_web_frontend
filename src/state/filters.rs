//! Dashboard filter state management.
//!
//! Provides reactive filter state that persists to localStorage and
//! can be accessed by any dashboard component via context.

use leptos::prelude::*;

use crate::models::post::{PostFilterType, PostSortType};

/// Key constants for localStorage persistence.
#[allow(dead_code)]
mod storage_keys {
    pub const CONTENT_TYPES: &str = "selectedContentTypes";
    pub const FEED_FILTER: &str = "selected-feed";
    pub const SORT_BY: &str = "selected-sort";
    pub const IS_COLLAPSED: &str = "isFiltersCollapsed";
}

/// Dashboard filter state.
///
/// Contains all filter options for the post feed and persists
/// selections to localStorage for cross-session continuity.
#[derive(Clone, Copy)]
pub struct FilterState {
    /// Content types to show (empty = all).
    pub content_types: RwSignal<Vec<PostFilterType>>,
    /// Feed filter (None = all, Some(Followed) or Some(Follower)).
    pub feed_filter: RwSignal<Option<PostFilterType>>,
    /// Sort order.
    pub sort_by: RwSignal<PostSortType>,
    /// Title search query.
    pub title_query: RwSignal<String>,
    /// Tag filter.
    pub tag_query: RwSignal<String>,
    /// Sidebar collapsed state.
    pub is_collapsed: RwSignal<bool>,
    /// Signal that triggers when filters change (for feed reload).
    pub version: RwSignal<u32>,
}

impl FilterState {
    /// Create a new filter state with default values.
    pub fn new() -> Self {
        Self {
            content_types: RwSignal::new(Vec::new()),
            feed_filter: RwSignal::new(None),
            sort_by: RwSignal::new(PostSortType::Newest),
            title_query: RwSignal::new(String::new()),
            tag_query: RwSignal::new(String::new()),
            is_collapsed: RwSignal::new(false),
            version: RwSignal::new(0),
        }
    }

    /// Load filter state from localStorage (client-side only).
    #[cfg(feature = "hydrate")]
    pub fn load_from_storage(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage().ok().flatten() {
            Some(s) => s,
            None => return,
        };

        // Load content types
        if let Ok(Some(json)) = storage.get_item(storage_keys::CONTENT_TYPES)
            && let Ok(types) = serde_json::from_str::<Vec<String>>(&json) {
                let parsed: Vec<PostFilterType> = types
                    .iter()
                    .filter_map(|s| match s.as_str() {
                        "IMAGE" => Some(PostFilterType::Image),
                        "VIDEO" => Some(PostFilterType::Video),
                        "TEXT" => Some(PostFilterType::Text),
                        "AUDIO" => Some(PostFilterType::Audio),
                        _ => None,
                    })
                    .collect();
                self.content_types.set(parsed);
            }

        // Load feed filter
        if let Ok(Some(feed)) = storage.get_item(storage_keys::FEED_FILTER) {
            let filter = match feed.as_str() {
                "FOLLOWED" => Some(PostFilterType::Followed),
                "FOLLOWER" => Some(PostFilterType::Follower),
                _ => None,
            };
            self.feed_filter.set(filter);
        }

        // Load sort order
        if let Ok(Some(sort)) = storage.get_item(storage_keys::SORT_BY) {
            let sort_by = match sort.as_str() {
                "NEWEST" => PostSortType::Newest,
                "TRENDING" => PostSortType::Trending,
                "LIKES" => PostSortType::Likes,
                "DISLIKES" => PostSortType::Dislikes,
                "VIEWS" => PostSortType::Views,
                "COMMENTS" => PostSortType::Comments,
                _ => PostSortType::Newest,
            };
            self.sort_by.set(sort_by);
        }

        // Load collapsed state
        if let Ok(Some(collapsed)) = storage.get_item(storage_keys::IS_COLLAPSED) {
            self.is_collapsed.set(collapsed == "true");
        }
    }

    /// SSR stub - does nothing on server.
    #[cfg(not(feature = "hydrate"))]
    pub fn load_from_storage(&self) {}

    /// Save content types to localStorage.
    #[cfg(feature = "hydrate")]
    pub fn save_content_types(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage().ok().flatten() {
            Some(s) => s,
            None => return,
        };

        let types: Vec<String> = self
            .content_types
            .get()
            .iter()
            .map(|t| t.to_string())
            .collect();
        if let Ok(json) = serde_json::to_string(&types) {
            let _ = storage.set_item(storage_keys::CONTENT_TYPES, &json);
        }
    }

    #[cfg(not(feature = "hydrate"))]
    pub fn save_content_types(&self) {}

    /// Save feed filter to localStorage.
    #[cfg(feature = "hydrate")]
    pub fn save_feed_filter(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage().ok().flatten() {
            Some(s) => s,
            None => return,
        };

        let value = match self.feed_filter.get() {
            Some(PostFilterType::Followed) => "FOLLOWED",
            Some(PostFilterType::Follower) => "FOLLOWER",
            _ => "ALL",
        };
        let _ = storage.set_item(storage_keys::FEED_FILTER, value);
    }

    #[cfg(not(feature = "hydrate"))]
    pub fn save_feed_filter(&self) {}

    /// Save sort order to localStorage.
    #[cfg(feature = "hydrate")]
    pub fn save_sort_by(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage().ok().flatten() {
            Some(s) => s,
            None => return,
        };

        let _ = storage.set_item(storage_keys::SORT_BY, &self.sort_by.get().to_string());
    }

    #[cfg(not(feature = "hydrate"))]
    pub fn save_sort_by(&self) {}

    /// Save collapsed state to localStorage.
    #[cfg(feature = "hydrate")]
    pub fn save_collapsed(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage().ok().flatten() {
            Some(s) => s,
            None => return,
        };

        let value = if self.is_collapsed.get() {
            "true"
        } else {
            "false"
        };
        let _ = storage.set_item(storage_keys::IS_COLLAPSED, value);
    }

    #[cfg(not(feature = "hydrate"))]
    pub fn save_collapsed(&self) {}

    /// Toggle a content type filter and persist.
    pub fn toggle_content_type(&self, filter_type: PostFilterType) {
        self.content_types.update(|types| {
            if let Some(pos) = types.iter().position(|&t| t == filter_type) {
                types.remove(pos);
            } else {
                types.push(filter_type);
            }
        });
        self.save_content_types();
        self.bump_version();
    }

    /// Set the feed filter and persist.
    pub fn set_feed_filter(&self, filter: Option<PostFilterType>) {
        self.feed_filter.set(filter);
        self.save_feed_filter();
        self.bump_version();
    }

    /// Set the sort order and persist.
    pub fn set_sort_by(&self, sort: PostSortType) {
        self.sort_by.set(sort);
        self.save_sort_by();
        self.bump_version();
    }

    /// Toggle sidebar collapsed state and persist.
    pub fn toggle_collapsed(&self) {
        self.is_collapsed.update(|v| *v = !*v);
        self.save_collapsed();
    }

    /// Set the title search query (debounced externally).
    pub fn set_title_query(&self, query: String) {
        self.title_query.set(query);
        self.bump_version();
    }

    /// Set the tag query.
    pub fn set_tag_query(&self, query: String) {
        self.tag_query.set(query);
        self.bump_version();
    }

    /// Bump the version to trigger feed reload.
    fn bump_version(&self) {
        self.version.update(|v| *v = v.wrapping_add(1));
    }

    /// Get the combined filter_by list for the API.
    pub fn get_filter_by(&self) -> Vec<PostFilterType> {
        let mut filters = self.content_types.get();
        if let Some(feed) = self.feed_filter.get() {
            filters.push(feed);
        }
        filters
    }
}

impl Default for FilterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide filter state context to the component tree.
pub fn provide_filter_context() {
    let state = FilterState::new();

    // Load from localStorage on mount (client-side only)
    Effect::new(move |_| {
        state.load_from_storage();
    });

    provide_context(state);
}

/// Get the filter state from context.
pub fn use_filter_state() -> FilterState {
    expect_context::<FilterState>()
}
