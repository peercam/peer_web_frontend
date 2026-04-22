use async_graphql::{ID, SimpleObject};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// GraphQL Objects
// ============================================================================

/// Daily gem status data aggregated across all users.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemStatusData {
    pub d0: Decimal,
    pub d1: Decimal,
    pub d2: Decimal,
    pub d3: Decimal,
    pub d4: Decimal,
    pub d5: Decimal,
    pub d6: Decimal,
    pub d7: Decimal,
    pub w0: Decimal,
    pub m0: Decimal,
    pub y0: Decimal,
}

/// Response for `gemster` and `dailygemstatus` queries.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemsterResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<DailyGemStatusData>,
}

/// Per-user gem data for a specific day.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemsResultsUserData {
    pub userid: Option<ID>,
    pub pkey: Option<ID>,
    pub gems: Option<Decimal>,
}

/// Aggregated gems results for a day.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemsResultsData {
    pub data: Option<Vec<DailyGemsResultsUserData>>,
    #[graphql(name = "totalGems")]
    pub total_gems: Option<Decimal>,
}

/// Response for `dailygemsresults` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyGemsResultsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<DailyGemsResultsData>,
}

/// Win status from a gem-to-token distribution.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct WinStatus {
    #[graphql(name = "totalGems")]
    pub total_gems: Decimal,
    pub gemsintoken: Decimal,
    pub bestatigung: Decimal,
}

/// Detailed gem source for a single distribution entry.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersUserStatusDetails {
    pub gemid: Option<ID>,
    pub userid: Option<ID>,
    pub postid: Option<ID>,
    pub fromid: Option<ID>,
    pub gems: Option<Decimal>,
    pub numbers: Option<Decimal>,
    pub whereby: Option<Decimal>,
    pub createdat: Option<String>,
}

/// Per-user distribution result.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersUserStatus {
    pub userid: Option<ID>,
    pub gems: Option<Decimal>,
    pub tokens: Option<Decimal>,
    pub percentage: Option<Decimal>,
    pub details: Option<Vec<GemstersUserStatusDetails>>,
}

/// Container for distribution results.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersData {
    #[graphql(name = "winStatus")]
    pub win_status: Option<WinStatus>,
    #[graphql(name = "userStatus")]
    pub user_status: Option<Vec<GemstersUserStatus>>,
}

/// Response for `distributeTokensForGems` / `gemsters` mutations.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GemstersResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<GemstersData>,
}

/// Mint account details.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct MintAccount {
    pub accountid: ID,
    #[graphql(name = "initialBalance")]
    pub initial_balance: Decimal,
    #[graphql(name = "currentBalance")]
    pub current_balance: Decimal,
    pub updatedat: String,
}

/// Response for `getMintAccount` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct MintAccountResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "mintAccount")]
    pub mint_account: Option<MintAccount>,
}
