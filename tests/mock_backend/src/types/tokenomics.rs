use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// GraphQL Objects
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ActionPriceResult {
    #[graphql(name = "postPrice")]
    pub post_price: f64,
    #[graphql(name = "likePrice")]
    pub like_price: f64,
    #[graphql(name = "dislikePrice")]
    pub dislike_price: f64,
    #[graphql(name = "commentPrice")]
    pub comment_price: f64,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetActionPricesResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: ActionPriceResult,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ActionGemsReturns {
    #[graphql(name = "viewGemsReturn")]
    pub view_gems_return: f64,
    #[graphql(name = "likeGemsReturn")]
    pub like_gems_return: f64,
    #[graphql(name = "dislikeGemsReturn")]
    pub dislike_gems_return: f64,
    #[graphql(name = "commentGemsReturn")]
    pub comment_gems_return: f64,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct MintingData {
    #[graphql(name = "tokensMintedYesterday")]
    pub tokens_minted_yesterday: f64,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TokenomicsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "actionTokenPrices")]
    pub action_token_prices: ActionPriceResult,
    #[graphql(name = "actionGemsReturns")]
    pub action_gems_returns: ActionGemsReturns,
    #[graphql(name = "mintingData")]
    pub minting_data: MintingData,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct DailyFreeResponse {
    pub name: String,
    pub used: i32,
    pub available: i32,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct GetDailyResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<DailyFreeResponse>>,
}
