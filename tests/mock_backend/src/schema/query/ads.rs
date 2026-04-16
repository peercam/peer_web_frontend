use async_graphql::{Context, ID, Object};

use crate::require_auth;
use crate::state::{AdvertisementRecord, SharedState, is_ad_active, today_date_string};
use crate::types::ad::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct AdQuery;

#[allow(clippy::too_many_arguments)]
#[Object]
impl AdQuery {
    /// List currently active advertisement posts.
    async fn list_advertisement_posts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "filterBy")] _filter_by: Option<Vec<String>>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<String>,
        userid: Option<ID>,
        postid: Option<ID>,
        title: Option<String>,
        tag: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
        #[graphql(name = "commentOffset")] _comment_offset: Option<i32>,
        #[graphql(name = "commentLimit")] _comment_limit: Option<i32>,
    ) -> ListAdvertisementPostsResponse {
        let viewer_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ListAdvertisementPostsResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let today = today_date_string();

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 100) as usize;

        let active_ads: Vec<&AdvertisementRecord> = state
            .advertisements
            .iter()
            .filter(|a| is_ad_active(a, &today))
            .filter(|a| {
                userid
                    .as_ref()
                    .map(|uid| uid.to_string() == a.advertiser_id.to_string())
                    .unwrap_or(true)
            })
            .filter(|a| {
                postid
                    .as_ref()
                    .map(|pid| pid.to_string() == a.post_id.to_string())
                    .unwrap_or(true)
            })
            .filter(|a| {
                if let Some(ref t) = title
                    && !t.is_empty()
                {
                    return state
                        .posts
                        .iter()
                        .find(|p| p.id == a.post_id)
                        .map(|p| p.title.to_lowercase().contains(&t.to_lowercase()))
                        .unwrap_or(false);
                }
                true
            })
            .filter(|a| {
                if let Some(ref tg) = tag
                    && !tg.is_empty()
                {
                    let tg_lower = tg.to_lowercase();
                    return state
                        .posts
                        .iter()
                        .find(|p| p.id == a.post_id)
                        .map(|p| p.tags.iter().any(|pt| pt.to_lowercase() == tg_lower))
                        .unwrap_or(false);
                }
                true
            })
            .collect();

        if active_ads.is_empty() {
            return ListAdvertisementPostsResponse {
                meta: DefaultResponse::success("22002", "No advertisements found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let total = active_ads.len();
        let end = (off + lim).min(total);
        let page = if off < total {
            &active_ads[off..end]
        } else {
            &[]
        };

        let ad_posts: Vec<AdvertisementPost> = page
            .iter()
            .filter_map(|a| state.ad_record_to_graphql(a, Some(viewer_id)))
            .collect();

        ListAdvertisementPostsResponse {
            meta: DefaultResponse::success("12002", "Advertisements fetched"),
            counter: total as i32,
            affected_rows: Some(ad_posts),
        }
    }

    /// Get the user's advertisement history with aggregated statistics.
    async fn advertisement_history(
        &self,
        ctx: &Context<'_>,
        filter: Option<AdvertisementHistoryFilter>,
        sort: Option<AdvertisementSort>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> ListedAdvertisementData {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ListedAdvertisementData {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 100) as usize;

        let mut user_ads: Vec<&AdvertisementRecord> = state
            .advertisements
            .iter()
            .filter(|a| a.advertiser_id == user_id)
            .filter(|a| {
                if let Some(ref f) = filter {
                    let type_match = f.ad_type.map(|t| a.ad_type == t).unwrap_or(true);
                    let from_match = f
                        .from
                        .as_ref()
                        .map(|d| a.start_date.as_str() >= d.as_str())
                        .unwrap_or(true);
                    let to_match =
                        f.to.as_ref()
                            .map(|d| a.end_date.as_str() <= d.as_str())
                            .unwrap_or(true);
                    type_match && from_match && to_match
                } else {
                    true
                }
            })
            .collect();

        match sort {
            Some(AdvertisementSort::Oldest) => {
                user_ads.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            Some(AdvertisementSort::BiggestCost) => {
                user_ads.sort_by(|a, b| b.token_cost.cmp(&a.token_cost));
            }
            Some(AdvertisementSort::SmallestCost) => {
                user_ads.sort_by(|a, b| a.token_cost.cmp(&b.token_cost));
            }
            _ => {
                user_ads.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }

        if user_ads.is_empty() {
            return ListedAdvertisementData {
                meta: DefaultResponse::success("22002", "No advertisements found"),
                affected_rows: None,
            };
        }

        let total_cost: f64 = user_ads
            .iter()
            .map(|a| a.token_cost.to_string().parse::<f64>().unwrap_or(0.0))
            .sum();
        let stats = TotalAdvertisementHistoryStats {
            token_spent: total_cost,
            euro_spent: 0.0,
            amount_ads: user_ads.len() as i32,
            gems_earned: 0.0,
            amount_likes: 0,
            amount_views: 0,
            amount_comments: 0,
            amount_dislikes: 0,
            amount_reports: 0,
        };

        let end = (off + lim).min(user_ads.len());
        let page = if off < user_ads.len() {
            &user_ads[off..end]
        } else {
            &[]
        };

        let ads: Vec<Advertisement> = page
            .iter()
            .filter_map(|a| state.ad_record_to_full_graphql(a))
            .collect();

        ListedAdvertisementData {
            meta: DefaultResponse::success("12002", "History fetched"),
            affected_rows: Some(AdvertisementHistoryResult {
                stats,
                advertisements: Some(ads),
            }),
        }
    }
}
