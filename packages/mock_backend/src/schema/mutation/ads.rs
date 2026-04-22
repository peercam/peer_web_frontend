use async_graphql::{Context, ID, Object};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::require_auth;
use crate::state::{
    AD_BASIC_DAILY_PRICE, AD_PINNED_PRICE, AdvertisementRecord, SYSTEM_PEER_ACCOUNT, SharedState,
    TransactionRecord, is_ad_active, today_date_string,
};
use crate::types::ad::*;
use crate::types::registration::DefaultResponse;
use crate::types::wallet::TransactionCategory;

#[derive(Default)]
pub struct AdMutation;

#[Object]
impl AdMutation {
    /// Create a basic (time-based) advertisement for a post.
    async fn advertise_post_basic(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        startday: String,
        #[graphql(name = "durationInDays")] duration_in_days: AdDuration,
        #[graphql(name = "advertisePlan")] _advertise_plan: AdvertisementBasicPlan,
    ) -> ListAdvertisementData {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ListAdvertisementData {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return ListAdvertisementData {
                    meta: DefaultResponse::error("30209", "Invalid post UUID"),
                    affected_rows: None,
                };
            }
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Validate post exists and belongs to user
        match state.posts.iter().find(|p| p.id == post_uuid) {
            Some(p) if p.author_id == user_id => {}
            _ => {
                return ListAdvertisementData {
                    meta: DefaultResponse::error("31510", "Post not found or not owned by you"),
                    affected_rows: None,
                };
            }
        }

        // Check no active ad on this post
        let today = today_date_string();
        if state
            .advertisements
            .iter()
            .any(|a| a.post_id == post_uuid && is_ad_active(a, &today))
        {
            return ListAdvertisementData {
                meta: DefaultResponse::error("32006", "Post already has active advertisement"),
                affected_rows: None,
            };
        }

        // Calculate cost
        let days = duration_in_days.to_days();
        let cost = AD_BASIC_DAILY_PRICE * Decimal::from(days);

        // Check balance
        let balance = state
            .wallets
            .get(&user_id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        if balance < cost {
            return ListAdvertisementData {
                meta: DefaultResponse::error("51301", "Insufficient balance"),
                affected_rows: None,
            };
        }

        // Deduct tokens
        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= cost;
        *state
            .wallets
            .entry(SYSTEM_PEER_ACCOUNT)
            .or_insert(Decimal::ZERO) += cost;

        // Calculate end date
        let start = NaiveDate::parse_from_str(&startday, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive());
        let end = start + chrono::Duration::days(days as i64);

        let ad_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        let record = AdvertisementRecord {
            id: ad_id,
            post_id: post_uuid,
            advertiser_id: user_id,
            ad_type: AdvertisementType::Basic,
            start_date: startday.clone(),
            end_date: end.format("%Y-%m-%d").to_string(),
            token_cost: cost,
            created_at: now.clone(),
        };

        state.advertisements.push(record);

        // Record transaction
        let operation_id = Uuid::new_v4();
        state.transactions.push(TransactionRecord {
            id: Uuid::new_v4(),
            operation_id,
            category: Some(TransactionCategory::AdBasic),
            transaction_type: "DEBIT".into(),
            sender_id: user_id,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: -cost,
            net_token_amount: -cost,
            message: Some(format!("Basic advertisement for {} days", days)),
            fees: None,
            created_at: now.clone(),
        });

        let cost_f64 = cost.to_string().parse::<f64>().unwrap_or(0.0);

        ListAdvertisementData {
            meta: DefaultResponse::success("12001", "Advertisement created"),
            affected_rows: Some(vec![AdvertisementRow {
                id: ad_id.to_string().into(),
                created_at: now,
                ad_type: AdvertisementType::Basic,
                timeframe_start: startday,
                timeframe_end: end.format("%Y-%m-%d").to_string(),
                total_token_cost: cost_f64,
                total_euro_cost: 0.0,
            }]),
        }
    }

    /// Create a pinned (featured) advertisement for a post.
    async fn advertise_post_pinned(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        #[graphql(name = "advertisePlan")] _advertise_plan: AdvertisementPinnedPlan,
    ) -> ListAdvertisementData {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ListAdvertisementData {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let post_uuid = match postid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return ListAdvertisementData {
                    meta: DefaultResponse::error("30209", "Invalid post UUID"),
                    affected_rows: None,
                };
            }
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Validate post exists and belongs to user
        match state.posts.iter().find(|p| p.id == post_uuid) {
            Some(p) if p.author_id == user_id => {}
            _ => {
                return ListAdvertisementData {
                    meta: DefaultResponse::error("31510", "Post not found or not owned by you"),
                    affected_rows: None,
                };
            }
        }

        let today = today_date_string();
        if state
            .advertisements
            .iter()
            .any(|a| a.post_id == post_uuid && is_ad_active(a, &today))
        {
            return ListAdvertisementData {
                meta: DefaultResponse::error("32006", "Post already has active advertisement"),
                affected_rows: None,
            };
        }

        let cost = AD_PINNED_PRICE;
        let balance = state
            .wallets
            .get(&user_id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        if balance < cost {
            return ListAdvertisementData {
                meta: DefaultResponse::error("51301", "Insufficient balance"),
                affected_rows: None,
            };
        }

        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= cost;
        *state
            .wallets
            .entry(SYSTEM_PEER_ACCOUNT)
            .or_insert(Decimal::ZERO) += cost;

        // Pinned ads default to 7 days
        let start = Utc::now().date_naive();
        let end = start + chrono::Duration::days(7);

        let ad_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        state.advertisements.push(AdvertisementRecord {
            id: ad_id,
            post_id: post_uuid,
            advertiser_id: user_id,
            ad_type: AdvertisementType::Pinned,
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            token_cost: cost,
            created_at: now.clone(),
        });

        let operation_id = Uuid::new_v4();
        state.transactions.push(TransactionRecord {
            id: Uuid::new_v4(),
            operation_id,
            category: Some(TransactionCategory::AdPinned),
            transaction_type: "DEBIT".into(),
            sender_id: user_id,
            recipient_id: SYSTEM_PEER_ACCOUNT,
            token_amount: -cost,
            net_token_amount: -cost,
            message: Some("Pinned advertisement".into()),
            fees: None,
            created_at: now.clone(),
        });

        let cost_f64 = cost.to_string().parse::<f64>().unwrap_or(0.0);

        ListAdvertisementData {
            meta: DefaultResponse::success("12001", "Advertisement created"),
            affected_rows: Some(vec![AdvertisementRow {
                id: ad_id.to_string().into(),
                created_at: now,
                ad_type: AdvertisementType::Pinned,
                timeframe_start: start.format("%Y-%m-%d").to_string(),
                timeframe_end: end.format("%Y-%m-%d").to_string(),
                total_token_cost: cost_f64,
                total_euro_cost: 0.0,
            }]),
        }
    }
}
