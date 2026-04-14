//! Advertisements API server functions.
//!
//! Provides server functions for fetching advertisement history
//! and creating pinned advertisements.

use leptos::prelude::*;
use serde::Serialize;

use crate::models::advertisement::{
    AdHistoryResponse, AdvertisePostResponse, AdvertisementSort,
};

/// Variables for the advertisementHistory query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdHistoryFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdHistoryVars {
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<AdHistoryFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<AdvertisementSort>,
    offset: i32,
    limit: i32,
}

/// Variables for the advertisePostPinned mutation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdvertisePostPinnedVars {
    postid: String,
    advertise_plan: String,
}

/// Fetch the user's advertisement history with aggregated stats.
///
/// # Arguments
///
/// * `sort` - Sort order (defaults to NEWEST)
/// * `offset` - Pagination offset
/// * `limit` - Maximum number of ads to return (max 20)
#[server(GetAdHistory, "/api")]
pub async fn get_ad_history(
    sort: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<AdHistoryResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, AdvertisementHistoryData, ADVERTISEMENT_HISTORY_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let sort_enum = match sort.as_deref() {
        Some("OLDEST") => Some(AdvertisementSort::Oldest),
        Some("BIGGEST_COST") => Some(AdvertisementSort::BiggestCost),
        Some("SMALLEST_COST") => Some(AdvertisementSort::SmallestCost),
        _ => Some(AdvertisementSort::Newest),
    };

    let vars = AdHistoryVars {
        filter: None,
        sort: sort_enum,
        offset,
        limit,
    };

    let data: AdvertisementHistoryData =
        query(ADVERTISEMENT_HISTORY_QUERY, vars, Some(&token)).await?;

    Ok(data.advertisement_history)
}

/// Create a pinned advertisement for a post.
///
/// Costs 200 tokens.
///
/// # Arguments
///
/// * `post_id` - UUID of the post to promote
#[server(AdvertisePostPinned, "/api")]
pub async fn advertise_post_pinned(
    post_id: String,
) -> Result<AdvertisePostResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, AdvertisePostPinnedData, ADVERTISE_POST_PINNED_MUTATION};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = AdvertisePostPinnedVars {
        postid: post_id,
        advertise_plan: "PINNED".to_string(),
    };

    let data: AdvertisePostPinnedData =
        mutate(ADVERTISE_POST_PINNED_MUTATION, vars, Some(&token)).await?;

    Ok(data.advertise_post_pinned)
}
