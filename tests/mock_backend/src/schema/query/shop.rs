use async_graphql::{Context, Object};

use crate::require_auth;
use crate::seed::SEED_USER_SHOP;
use crate::state::SharedState;
use crate::types::registration::DefaultResponse;
use crate::types::shop::*;

#[derive(Default)]
pub struct ShopQuery;

#[Object]
impl ShopQuery {
    /// Get shop order details by transaction ID.
    async fn shop_order_details(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "transactionId")] transaction_id: String,
    ) -> ShopOrderDetailsResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ShopOrderDetailsResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        if transaction_id.is_empty() {
            return ShopOrderDetailsResponse {
                meta: DefaultResponse::error("30101", "Missing transaction ID"),
                affected_rows: None,
            };
        }

        let state = ctx.data_unchecked::<SharedState>().read().await;

        // The Peer Shop operator may view any order's delivery details
        // (mirrors the legacy `PEER_SHOP_ID` UI gate); other users may only
        // view orders where they are the buyer.
        let viewer_is_shop = user_id == SEED_USER_SHOP;
        let order = state.shop_orders.iter().find(|o| {
            o.transaction_id.to_string() == transaction_id
                && (viewer_is_shop || o.buyer_id == user_id)
        });

        match order {
            Some(o) => ShopOrderDetailsResponse {
                meta: DefaultResponse::success("12202", "Order details retrieved"),
                affected_rows: Some(vec![ShopOrderDetails {
                    shop_order_id: o.id.to_string(),
                    shop_item_id: o.shop_item_id.clone(),
                    shop_item_specs: o.item_specs.as_ref().map(|s| ShopItemSpecs {
                        size: Some(s.clone()),
                    }),
                    delivery_details: Some(ShopOrderDeliveryDetails {
                        name: Some(o.delivery.name.clone()),
                        email: Some(o.delivery.email.clone()),
                        addressline1: Some(o.delivery.addressline1.clone()),
                        addressline2: o.delivery.addressline2.clone(),
                        city: Some(o.delivery.city.clone()),
                        zipcode: Some(o.delivery.zipcode.clone()),
                        country: Some(o.delivery.country.clone()),
                    }),
                    createdat: Some(o.created_at.clone()),
                }]),
            },
            None => ShopOrderDetailsResponse {
                meta: DefaultResponse::success("22101", "Order not found"),
                affected_rows: None,
            },
        }
    }
}
