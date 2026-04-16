use async_graphql::{Enum, InputObject, SimpleObject, ID};
use serde::{Deserialize, Serialize};

use super::post::Post;
use super::registration::DefaultResponse;
use super::user::ProfileUserGql;

// ============================================================================
// Enums
// ============================================================================

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementType {
    Pinned,
    Basic,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdDuration {
    OneDay,
    TwoDays,
    ThreeDays,
    FourDays,
    FiveDays,
    SixDays,
    SevenDays,
}

impl AdDuration {
    pub fn to_days(&self) -> u32 {
        match self {
            Self::OneDay => 1,
            Self::TwoDays => 2,
            Self::ThreeDays => 3,
            Self::FourDays => 4,
            Self::FiveDays => 5,
            Self::SixDays => 6,
            Self::SevenDays => 7,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementBasicPlan {
    Basic,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementPinnedPlan {
    Pinned,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum AdvertisementSort {
    Newest,
    Oldest,
    BiggestCost,
    SmallestCost,
}

// ============================================================================
// Input Objects
// ============================================================================

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementHistoryFilter {
    pub from: Option<String>,
    pub to: Option<String>,
    #[graphql(name = "type")]
    pub ad_type: Option<AdvertisementType>,
    #[graphql(name = "advertisementId")]
    pub advertisement_id: Option<ID>,
    #[graphql(name = "postId")]
    pub post_id: Option<ID>,
    #[graphql(name = "userId")]
    pub user_id: Option<ID>,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvCreator {
    pub advertisementid: ID,
    pub advertisementtype: String,
    pub startdate: String,
    pub enddate: String,
    pub createdat: Option<String>,
    pub user: Option<ProfileUserGql>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementPost {
    pub post: Post,
    pub advertisement: AdvCreator,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListAdvertisementPostsResponse {
    pub meta: DefaultResponse,
    pub counter: i32,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AdvertisementPost>>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementRow {
    pub id: ID,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "type")]
    pub ad_type: AdvertisementType,
    #[graphql(name = "timeframeStart")]
    pub timeframe_start: String,
    #[graphql(name = "timeframeEnd")]
    pub timeframe_end: String,
    #[graphql(name = "totalTokenCost")]
    pub total_token_cost: f64,
    #[graphql(name = "totalEuroCost")]
    pub total_euro_cost: f64,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListAdvertisementData {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<AdvertisementRow>>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct TotalAdvertisementHistoryStats {
    #[graphql(name = "tokenSpent")]
    pub token_spent: f64,
    #[graphql(name = "euroSpent")]
    pub euro_spent: f64,
    #[graphql(name = "amountAds")]
    pub amount_ads: i32,
    #[graphql(name = "gemsEarned")]
    pub gems_earned: f64,
    #[graphql(name = "amountLikes")]
    pub amount_likes: i32,
    #[graphql(name = "amountViews")]
    pub amount_views: i32,
    #[graphql(name = "amountComments")]
    pub amount_comments: i32,
    #[graphql(name = "amountDislikes")]
    pub amount_dislikes: i32,
    #[graphql(name = "amountReports")]
    pub amount_reports: i32,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Advertisement {
    pub id: ID,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "type")]
    pub ad_type: AdvertisementType,
    #[graphql(name = "timeframeStart")]
    pub timeframe_start: String,
    #[graphql(name = "timeframeEnd")]
    pub timeframe_end: String,
    #[graphql(name = "totalTokenCost")]
    pub total_token_cost: f64,
    #[graphql(name = "totalEuroCost")]
    pub total_euro_cost: f64,
    #[graphql(name = "gemsEarned")]
    pub gems_earned: f64,
    #[graphql(name = "amountLikes")]
    pub amount_likes: i32,
    #[graphql(name = "amountViews")]
    pub amount_views: i32,
    #[graphql(name = "amountComments")]
    pub amount_comments: i32,
    #[graphql(name = "amountDislikes")]
    pub amount_dislikes: i32,
    #[graphql(name = "amountReports")]
    pub amount_reports: i32,
    pub user: ProfileUserGql,
    pub post: Post,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct AdvertisementHistoryResult {
    pub stats: TotalAdvertisementHistoryStats,
    pub advertisements: Option<Vec<Advertisement>>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListedAdvertisementData {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<AdvertisementHistoryResult>,
}
