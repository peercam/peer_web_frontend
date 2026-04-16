use async_graphql::{Context, Object};

use crate::require_auth;
use crate::state::{
    today_date_string, SharedState, FREE_COMMENTS, FREE_DISLIKES, FREE_LIKES, FREE_POSTS,
    VIEW_GEM_RETURN, LIKE_GEM_RETURN, DISLIKE_GEM_RETURN, COMMENT_GEM_RETURN,
};
use crate::types::registration::DefaultResponse;
use crate::types::tokenomics::*;

#[derive(Default)]
pub struct TokenomicsQuery;

#[Object]
impl TokenomicsQuery {
    /// Get current action prices.
    async fn get_action_prices(&self, ctx: &Context<'_>) -> GetActionPricesResponse {
        if require_auth(ctx).is_err() {
            return GetActionPricesResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                affected_rows: ActionPriceResult {
                    post_price: 0.0,
                    like_price: 0.0,
                    dislike_price: 0.0,
                    comment_price: 0.0,
                },
            };
        }

        GetActionPricesResponse {
            meta: DefaultResponse::success("11304", "Prices fetched"),
            affected_rows: ActionPriceResult {
                post_price: 20.0,
                like_price: 3.0,
                dislike_price: 3.0,
                comment_price: 1.0,
            },
        }
    }

    /// Get comprehensive tokenomics data.
    async fn get_tokenomics(&self, ctx: &Context<'_>) -> TokenomicsResponse {
        if require_auth(ctx).is_err() {
            return TokenomicsResponse {
                meta: DefaultResponse::error("60501", "Not authenticated"),
                action_token_prices: ActionPriceResult {
                    post_price: 0.0,
                    like_price: 0.0,
                    dislike_price: 0.0,
                    comment_price: 0.0,
                },
                action_gems_returns: ActionGemsReturns {
                    view_gems_return: 0.0,
                    like_gems_return: 0.0,
                    dislike_gems_return: 0.0,
                    comment_gems_return: 0.0,
                },
                minting_data: MintingData {
                    tokens_minted_yesterday: 0.0,
                },
            };
        }

        TokenomicsResponse {
            meta: DefaultResponse::success("11212", "Tokenomics data retrieved"),
            action_token_prices: ActionPriceResult {
                post_price: 20.0,
                like_price: 3.0,
                dislike_price: 3.0,
                comment_price: 1.0,
            },
            action_gems_returns: ActionGemsReturns {
                view_gems_return: VIEW_GEM_RETURN,
                like_gems_return: LIKE_GEM_RETURN,
                dislike_gems_return: DISLIKE_GEM_RETURN,
                comment_gems_return: COMMENT_GEM_RETURN,
            },
            minting_data: MintingData {
                tokens_minted_yesterday: 0.0,
            },
        }
    }

    /// Get daily free action usage and availability.
    async fn get_daily_free_status(&self, ctx: &Context<'_>) -> GetDailyResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return GetDailyResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let today = today_date_string();

        let actions = vec![
            ("post", FREE_POSTS),
            ("like", FREE_LIKES),
            ("comment", FREE_COMMENTS),
            ("dislike", FREE_DISLIKES),
        ];

        let rows: Vec<DailyFreeResponse> = actions
            .into_iter()
            .map(|(name, limit)| {
                let used = state
                    .daily_actions_used
                    .get(&(user_id, today.clone(), name.to_string()))
                    .copied()
                    .unwrap_or(0);
                let available = limit.saturating_sub(used) as i32;
                DailyFreeResponse {
                    name: name.to_string(),
                    used: used as i32,
                    available,
                }
            })
            .collect();

        GetDailyResponse {
            meta: DefaultResponse::success("11303", "Daily free status loaded"),
            affected_rows: Some(rows),
        }
    }
}
