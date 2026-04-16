use async_graphql::{Context, Object};
use rust_decimal::Decimal;

use crate::require_auth;
use crate::state::{
    COMMENT_GEM_RETURN, DISLIKE_GEM_RETURN, GemRecord, LIKE_GEM_RETURN, SharedState,
    TransactionRecord, VIEW_GEM_RETURN, today_date_string,
};
use crate::types::registration::DefaultResponse;
use crate::types::wallet::*;

#[derive(Default)]
pub struct WalletQuery;

#[allow(clippy::too_many_arguments)]
#[Object]
impl WalletQuery {
    /// Get the current user's token balance.
    async fn balance(&self, ctx: &Context<'_>) -> CurrentLiquidity {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return CurrentLiquidity {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    currentliquidity: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let balance = state
            .wallets
            .get(&user_id)
            .copied()
            .unwrap_or(Decimal::ZERO);

        CurrentLiquidity {
            meta: DefaultResponse::success("11204", "Balance retrieved"),
            currentliquidity: Some(balance),
        }
    }

    /// Get transaction history with pagination.
    async fn transaction_history(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "type")] filter_type: Option<TokenMovementFilterType>,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: Option<SortFilterType>,
    ) -> TransactionHistoryResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return TransactionHistoryResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 100) as usize;

        let mut user_txs: Vec<&TransactionRecord> = state
            .transactions
            .iter()
            .filter(|t| t.sender_id == user_id || t.recipient_id == user_id)
            .filter(|t| match filter_type {
                Some(TokenMovementFilterType::Transaction) => {
                    t.category == Some(TransactionCategory::P2pTransfer)
                }
                Some(TokenMovementFilterType::Mint) => {
                    t.category == Some(TransactionCategory::TokenMint)
                }
                Some(TokenMovementFilterType::Payment) => matches!(
                    t.category,
                    Some(TransactionCategory::Like)
                        | Some(TransactionCategory::Dislike)
                        | Some(TransactionCategory::Comment)
                        | Some(TransactionCategory::PostCreate)
                ),
                _ => true,
            })
            .filter(|t| {
                let in_start = start_date
                    .as_ref()
                    .map(|sd| t.created_at.as_str() >= sd.as_str())
                    .unwrap_or(true);
                let in_end = end_date
                    .as_ref()
                    .map(|ed| t.created_at.as_str() <= ed.as_str())
                    .unwrap_or(true);
                in_start && in_end
            })
            .collect();

        match sort {
            Some(SortFilterType::Oldest) => {
                user_txs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            _ => {
                user_txs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }

        if user_txs.is_empty() {
            return TransactionHistoryResponse {
                meta: DefaultResponse::success("21209", "No transactions found"),
                affected_rows: None,
            };
        }

        let end = (off + lim).min(user_txs.len());
        let page = if off < user_txs.len() {
            &user_txs[off..end]
        } else {
            &[]
        };

        let items: Vec<TransactionHistoryItem> = page
            .iter()
            .map(|t| state.transaction_record_to_graphql(t))
            .collect();

        TransactionHistoryResponse {
            meta: DefaultResponse::success("11215", "Transactions retrieved"),
            affected_rows: Some(items),
        }
    }

    /// Get the legacy transaction history with direction filter.
    async fn get_transaction_history(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "type")] filter_type: Option<TokenMovementFilterType>,
        direction: Option<DirectionFilterType>,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<i32>,
        offset: Option<i32>,
        sort: Option<SortFilterType>,
    ) -> TransactionHistoryResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return TransactionHistoryResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 100) as usize;

        let mut user_txs: Vec<&TransactionRecord> = state
            .transactions
            .iter()
            .filter(|t| t.sender_id == user_id || t.recipient_id == user_id)
            .filter(|t| match direction {
                Some(DirectionFilterType::Income) => t.recipient_id == user_id,
                Some(DirectionFilterType::Deduction) => t.sender_id == user_id,
                None => true,
            })
            .filter(|t| match filter_type {
                Some(TokenMovementFilterType::Transaction) => {
                    t.category == Some(TransactionCategory::P2pTransfer)
                }
                Some(TokenMovementFilterType::Mint) => {
                    t.category == Some(TransactionCategory::TokenMint)
                }
                Some(TokenMovementFilterType::Payment) => matches!(
                    t.category,
                    Some(TransactionCategory::Like)
                        | Some(TransactionCategory::Dislike)
                        | Some(TransactionCategory::Comment)
                        | Some(TransactionCategory::PostCreate)
                ),
                _ => true,
            })
            .filter(|t| {
                let in_start = start_date
                    .as_ref()
                    .map(|sd| t.created_at.as_str() >= sd.as_str())
                    .unwrap_or(true);
                let in_end = end_date
                    .as_ref()
                    .map(|ed| t.created_at.as_str() <= ed.as_str())
                    .unwrap_or(true);
                in_start && in_end
            })
            .collect();

        match sort {
            Some(SortFilterType::Oldest) => {
                user_txs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            _ => {
                user_txs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }

        if user_txs.is_empty() {
            return TransactionHistoryResponse {
                meta: DefaultResponse::success("21209", "No transactions found"),
                affected_rows: None,
            };
        }

        let end = (off + lim).min(user_txs.len());
        let page = if off < user_txs.len() {
            &user_txs[off..end]
        } else {
            &[]
        };

        let items: Vec<TransactionHistoryItem> = page
            .iter()
            .map(|t| state.transaction_record_to_graphql(t))
            .collect();

        TransactionHistoryResponse {
            meta: DefaultResponse::success("11215", "Transactions retrieved"),
            affected_rows: Some(items),
        }
    }

    /// List win logs for a given day.
    async fn list_win_logs(
        &self,
        ctx: &Context<'_>,
        _day: DayFilterType,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> UserLogWins {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return UserLogWins {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        // Win logs = earning events (gems received on user's posts)
        let wins: Vec<LogWins> = state
            .gems
            .iter()
            .filter(|g| g.user_id == user_id)
            .map(|g| LogWins {
                from: Some(g.from_user_id.to_string()),
                token: Some(format!("{:.2}", g.gems)),
                userid: Some(g.user_id.to_string()),
                postid: Some(g.post_id.to_string()),
                action: Some(g.action.clone()),
                numbers: Some(Decimal::from_f64_retain(g.gems).unwrap_or_default()),
                createdat: Some(g.created_at.clone()),
            })
            .collect();

        if wins.is_empty() {
            return UserLogWins {
                meta: DefaultResponse::success("21202", "No log records for date"),
                counter: 0,
                affected_rows: None,
            };
        }

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 100) as usize;
        let end = (off + lim).min(wins.len());
        let page = if off < wins.len() {
            wins[off..end].to_vec()
        } else {
            vec![]
        };

        UserLogWins {
            meta: DefaultResponse::success("11203", "Logs found"),
            counter: wins.len() as i32,
            affected_rows: Some(page),
        }
    }

    /// List payment logs for a given day.
    async fn list_payment_logs(
        &self,
        ctx: &Context<'_>,
        _day: DayFilterType,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> UserLogWins {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return UserLogWins {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        // Payment logs = spending events
        let payments: Vec<LogWins> = state
            .transactions
            .iter()
            .filter(|t| t.sender_id == user_id && t.transaction_type == "DEBIT")
            .map(|t| LogWins {
                from: Some(t.sender_id.to_string()),
                token: Some(t.token_amount.to_string()),
                userid: Some(t.sender_id.to_string()),
                postid: None,
                action: t.category.as_ref().map(|c| format!("{:?}", c)),
                numbers: Some(t.token_amount),
                createdat: Some(t.created_at.clone()),
            })
            .collect();

        if payments.is_empty() {
            return UserLogWins {
                meta: DefaultResponse::success("21202", "No log records for date"),
                counter: 0,
                affected_rows: None,
            };
        }

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 100) as usize;
        let end = (off + lim).min(payments.len());
        let page = if off < payments.len() {
            payments[off..end].to_vec()
        } else {
            vec![]
        };

        UserLogWins {
            meta: DefaultResponse::success("11203", "Logs found"),
            counter: payments.len() as i32,
            affected_rows: Some(page),
        }
    }

    /// Get today's interactions summary on the user's posts.
    async fn list_todays_interactions(&self, ctx: &Context<'_>) -> ListTodaysInteractionsResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ListTodaysInteractionsResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;
        let today = today_date_string();

        let todays_gems: Vec<&GemRecord> = state
            .gems
            .iter()
            .filter(|g| g.user_id == user_id && g.created_at.starts_with(&today))
            .collect();

        if todays_gems.is_empty() {
            return ListTodaysInteractionsResponse {
                meta: DefaultResponse::success("21204", "No interactions today"),
                affected_rows: None,
            };
        }

        let views = todays_gems.iter().filter(|g| g.action == "view").count() as f64;
        let likes = todays_gems.iter().filter(|g| g.action == "like").count() as f64;
        let dislikes = todays_gems.iter().filter(|g| g.action == "dislike").count() as f64;
        let comments = todays_gems.iter().filter(|g| g.action == "comment").count() as f64;

        let views_score = views * VIEW_GEM_RETURN;
        let likes_score = likes * LIKE_GEM_RETURN;
        let dislikes_score = dislikes * DISLIKE_GEM_RETURN;
        let comments_score = comments * COMMENT_GEM_RETURN;

        let total_interactions = views + likes + dislikes + comments;
        let total_score = views_score + likes_score + dislikes_score + comments_score;

        ListTodaysInteractionsResponse {
            meta: DefaultResponse::success("11205", "Interactions retrieved"),
            affected_rows: Some(TodaysInteractionsData {
                total_interactions: Decimal::from_f64_retain(total_interactions)
                    .unwrap_or_default(),
                total_score: Decimal::from_f64_retain(total_score).unwrap_or_default(),
                total_details: TodaysInteractionsDetailsData {
                    views: Decimal::from_f64_retain(views).unwrap_or_default(),
                    likes: Decimal::from_f64_retain(likes).unwrap_or_default(),
                    dislikes: Decimal::from_f64_retain(dislikes).unwrap_or_default(),
                    comments: Decimal::from_f64_retain(comments).unwrap_or_default(),
                    views_score: Decimal::from_f64_retain(views_score).unwrap_or_default(),
                    likes_score: Decimal::from_f64_retain(likes_score).unwrap_or_default(),
                    dislikes_score: Decimal::from_f64_retain(dislikes_score).unwrap_or_default(),
                    comments_score: Decimal::from_f64_retain(comments_score).unwrap_or_default(),
                },
            }),
        }
    }
}
