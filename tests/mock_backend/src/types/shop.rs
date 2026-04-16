use async_graphql::{Enum, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// Enums
// ============================================================================

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ShopSupportedDeliveryCountry {
    Germany,
}

// ============================================================================
// Input Objects
// ============================================================================

#[derive(InputObject, Clone, Debug)]
pub struct ShopItemSpecsInput {
    pub size: Option<String>,
}

#[derive(InputObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDetailsInput {
    pub name: String,
    pub email: String,
    pub addressline1: String,
    pub addressline2: Option<String>,
    pub city: String,
    pub zipcode: String,
    pub country: ShopSupportedDeliveryCountry,
    #[graphql(name = "shopItemSpecs")]
    pub shop_item_specs: Option<ShopItemSpecsInput>,
}

// ============================================================================
// GraphQL Objects
// ============================================================================

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct ShopItemSpecs {
    pub size: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDeliveryDetails {
    pub name: Option<String>,
    pub email: Option<String>,
    pub addressline1: Option<String>,
    pub addressline2: Option<String>,
    pub city: Option<String>,
    pub zipcode: Option<String>,
    pub country: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDetails {
    #[graphql(name = "shopOrderId")]
    pub shop_order_id: String,
    #[graphql(name = "shopItemId")]
    pub shop_item_id: String,
    #[graphql(name = "shopItemSpecs")]
    pub shop_item_specs: Option<ShopItemSpecs>,
    #[graphql(name = "deliveryDetails")]
    pub delivery_details: Option<ShopOrderDeliveryDetails>,
    pub createdat: Option<String>,
}

#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ShopOrderDetailsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<ShopOrderDetails>>,
}
