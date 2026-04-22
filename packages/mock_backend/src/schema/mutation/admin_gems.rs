use async_graphql::{Context, Object, Result};
use rust_decimal::Decimal;

use crate::guards::require_admin;
use crate::state::SharedState;
use crate::types::admin_gems::*;
use crate::types::registration::DefaultResponse;
use crate::types::wallet::DayFilterType;

/// Daily token mint amount (total distributed per day).
const DAILY_NUMBER_TOKEN: f64 = 5000.0;

#[derive(Default)]
pub struct AdminGemMutation;

#[Object]
impl AdminGemMutation {
    /// Convert all pending post interactions into gems.
    #[graphql(guard = "require_admin()")]
    async fn globalwins(&self, ctx: &Context<'_>) -> Result<DefaultResponse> {
        let mut state = ctx.data::<SharedState>()?.write().await;
        let converted = state.convert_interactions_to_gems();

        if converted == 0 {
            return Ok(DefaultResponse::error(
                "21205",
                "No interactions to convert",
            ));
        }

        Ok(DefaultResponse::success(
            "11206",
            "Interactions converted to gems",
        ))
    }

    /// Mint and distribute tokens from gems for a specific date.
    #[graphql(guard = "require_admin()")]
    async fn distribute_tokens_for_gems(
        &self,
        ctx: &Context<'_>,
        date: String,
    ) -> Result<GemstersResponse> {
        let parsed = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d");
        if parsed.is_err() {
            return Ok(GemstersResponse {
                meta: DefaultResponse::error("30105", "Invalid date format"),
                counter: 0,
                affected_rows: None,
            });
        }

        let today = chrono::Utc::now().date_naive();
        if parsed.unwrap() > today {
            return Ok(GemstersResponse {
                meta: DefaultResponse::error("30105", "Cannot mint for future dates"),
                counter: 0,
                affected_rows: None,
            });
        }

        let mut state = ctx.data::<SharedState>()?.write().await;

        if state.minted_dates.contains(&date) {
            return Ok(GemstersResponse {
                meta: DefaultResponse::error("31204", "Already minted for date"),
                counter: 0,
                affected_rows: None,
            });
        }

        let result = state.distribute_gems_to_tokens(&date, DAILY_NUMBER_TOKEN);

        match result {
            Some((data, counter)) => {
                state.minted_dates.insert(date);
                Ok(GemstersResponse {
                    meta: DefaultResponse::success("11208", "Tokens distributed"),
                    counter,
                    affected_rows: Some(data),
                })
            }
            None => Ok(GemstersResponse {
                meta: DefaultResponse::error("21206", "No gems for date"),
                counter: 0,
                affected_rows: None,
            }),
        }
    }

    /// Mint and distribute tokens from gems for a relative day.
    #[graphql(guard = "require_admin()")]
    async fn gemsters(&self, ctx: &Context<'_>, day: DayFilterType) -> Result<GemstersResponse> {
        let date = crate::state::day_filter_to_date(&day);
        self.distribute_tokens_for_gems(ctx, date).await
    }

    /// One-time alpha token distribution.
    #[graphql(guard = "require_admin()")]
    async fn alpha_mint(&self, ctx: &Context<'_>) -> Result<DefaultResponse> {
        let mut state = ctx.data::<SharedState>()?.write().await;

        if state.alpha_minted {
            return Ok(DefaultResponse::error(
                "31204",
                "Alpha mint already completed",
            ));
        }

        let user_ids: Vec<_> = state
            .users
            .keys()
            .filter(|uid| !state.is_system_account(**uid))
            .copied()
            .collect();

        let amount = Decimal::from(100);
        for uid in &user_ids {
            *state.wallets.entry(*uid).or_insert(Decimal::ZERO) += amount;
            *state
                .wallets
                .entry(crate::state::SYSTEM_MINT_ACCOUNT)
                .or_insert(Decimal::ZERO) -= amount;
        }

        state.alpha_minted = true;
        Ok(DefaultResponse::success("200", "Alpha mint complete"))
    }
}
