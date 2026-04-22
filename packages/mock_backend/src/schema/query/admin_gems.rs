use async_graphql::{Context, Object, Result};
use rust_decimal::Decimal;

use crate::guards::require_admin;
use crate::state::SharedState;
use crate::types::admin_gems::*;
use crate::types::registration::DefaultResponse;
use crate::types::wallet::DayFilterType;

#[derive(Default)]
pub struct AdminGemQuery;

#[Object]
impl AdminGemQuery {
    /// Get uncollected gems statistics by time period.
    #[graphql(guard = "require_admin()")]
    async fn gemster(&self, ctx: &Context<'_>) -> Result<GemsterResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let data = state.aggregate_gems_by_period();
        Ok(GemsterResponse {
            meta: DefaultResponse::success("11207", "Gems data loaded"),
            affected_rows: Some(data),
        })
    }

    /// Daily gem status overview.
    #[graphql(guard = "require_admin()")]
    async fn dailygemstatus(&self, ctx: &Context<'_>) -> Result<GemsterResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let data = state.aggregate_gems_by_period();
        Ok(GemsterResponse {
            meta: DefaultResponse::success("11207", "Gems data loaded"),
            affected_rows: Some(data),
        })
    }

    /// Per-user gems breakdown for a specific day.
    #[graphql(guard = "require_admin()")]
    async fn dailygemsresults(
        &self,
        ctx: &Context<'_>,
        day: DayFilterType,
    ) -> Result<DailyGemsResultsResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let (user_data, total) = state.get_gems_for_day(&day);

        if user_data.is_empty() {
            return Ok(DailyGemsResultsResponse {
                meta: DefaultResponse::error("21206", "No gems for given day"),
                affected_rows: None,
            });
        }

        Ok(DailyGemsResultsResponse {
            meta: DefaultResponse::success("11207", "Gems data loaded"),
            affected_rows: Some(DailyGemsResultsData {
                data: Some(user_data),
                total_gems: Some(total),
            }),
        })
    }

    /// Get mint account balance and details.
    #[graphql(guard = "require_admin()")]
    async fn get_mint_account(&self, ctx: &Context<'_>) -> Result<MintAccountResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let current_balance = state
            .wallets
            .get(&crate::state::SYSTEM_MINT_ACCOUNT)
            .copied()
            .unwrap_or(Decimal::ZERO);

        Ok(MintAccountResponse {
            meta: DefaultResponse::success("0", "Account retrieved"),
            mint_account: Some(MintAccount {
                accountid: crate::state::SYSTEM_MINT_ACCOUNT.to_string().into(),
                initial_balance: crate::seed::MINT_INITIAL_BALANCE,
                current_balance,
                updatedat: crate::state::today_date_string(),
            }),
        })
    }
}
