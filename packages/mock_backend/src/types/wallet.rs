use async_graphql::{Enum, SimpleObject};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum TransactionCategory {
    // The default `SCREAMING_SNAKE_CASE` mapping turns `P2pTransfer` into
    // `P_2P_TRANSFER` (an underscore is inserted before each digit), which
    // does not match the frontend's serde mapping (`P2P_TRANSFER`). Pin
    // the GraphQL name explicitly so the schema stays in sync.
    #[graphql(name = "P2P_TRANSFER")]
    P2pTransfer,
    AdBasic,
    AdPinned,
    PostCreate,
    Like,
    Dislike,
    Comment,
    TokenMint,
    ShopPurchase,
    InviterFeeEarn,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum TokenMovementFilterType {
    Transaction,
    Airdrop,
    Mint,
    Payment,
    Burn,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum DirectionFilterType {
    Income,
    Deduction,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum SortFilterType {
    Newest,
    Oldest,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum DayFilterType {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    W0,
    M0,
    Y0,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionUser {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: String,
    #[graphql(name = "visibilityStatus")]
    pub visibility_status: Option<String>,
    #[graphql(name = "hasActiveReports")]
    pub has_active_reports: Option<bool>,
    #[graphql(name = "isHiddenForUsers")]
    pub is_hidden_for_users: Option<bool>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionFees {
    pub total: Decimal,
    pub burn: Decimal,
    pub peer: Decimal,
    pub inviter: Option<Decimal>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionHistoryItem {
    #[graphql(name = "transactionId")]
    pub transaction_id: String,
    pub operationid: String,
    #[graphql(name = "transactionCategory")]
    pub transaction_category: Option<TransactionCategory>,
    pub transactiontype: String,
    pub tokenamount: String,
    #[graphql(name = "netTokenAmount")]
    pub net_token_amount: String,
    pub message: Option<String>,
    pub createdat: String,
    pub sender: TransactionUser,
    pub recipient: TransactionUser,
    pub fees: Option<TransactionFees>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CurrentLiquidity {
    pub meta: DefaultResponse,
    pub currentliquidity: Option<Decimal>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransactionHistoryResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<TransactionHistoryItem>>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransferToken {
    #[graphql(name = "tokenSendFormatted")]
    pub token_send_formatted: String,
    #[graphql(name = "tokensSubstractedFromWalletFormatted")]
    pub tokens_substracted_from_wallet_formatted: String,
    pub createdat: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TransferTokenResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<TransferToken>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct LogWins {
    pub from: Option<String>,
    pub token: Option<String>,
    pub userid: Option<String>,
    pub postid: Option<String>,
    pub action: Option<String>,
    pub numbers: Option<Decimal>,
    pub createdat: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct UserLogWins {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<LogWins>>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TodaysInteractionsDetailsData {
    pub views: Decimal,
    pub likes: Decimal,
    pub dislikes: Decimal,
    pub comments: Decimal,
    #[graphql(name = "viewsScore")]
    pub views_score: Decimal,
    #[graphql(name = "likesScore")]
    pub likes_score: Decimal,
    #[graphql(name = "dislikesScore")]
    pub dislikes_score: Decimal,
    #[graphql(name = "commentsScore")]
    pub comments_score: Decimal,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TodaysInteractionsData {
    #[graphql(name = "totalInteractions")]
    pub total_interactions: Decimal,
    #[graphql(name = "totalScore")]
    pub total_score: Decimal,
    #[graphql(name = "totalDetails")]
    pub total_details: TodaysInteractionsDetailsData,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListTodaysInteractionsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<TodaysInteractionsData>,
}
