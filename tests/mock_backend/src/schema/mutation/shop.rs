use async_graphql::{Context, Object};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::require_auth;
use crate::state::{
    SharedState, ShopDeliveryRecord, ShopOrderRecord, TransactionRecord, SYSTEM_SHOP_ACCOUNT,
};
use crate::types::registration::DefaultResponse;
use crate::types::shop::*;
use crate::types::wallet::TransactionCategory;

#[derive(Default)]
pub struct ShopMutation;

#[Object]
impl ShopMutation {
    /// Purchase a shop item using tokens.
    async fn perform_shop_order(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "tokenAmount")] token_amount: Decimal,
        #[graphql(name = "shopItemId")] shop_item_id: String,
        #[graphql(name = "orderDetails")] order_details: ShopOrderDetailsInput,
    ) -> DefaultResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("60501", "Not authenticated"),
        };

        // Validate delivery details
        if order_details.name.len() < 2 || order_details.name.len() > 100 {
            return DefaultResponse::error("30101", "Name must be 2-100 characters");
        }
        if !order_details.email.contains('@') {
            return DefaultResponse::error("30101", "Invalid email format");
        }
        if order_details.addressline1.len() < 6 || order_details.addressline1.len() > 100 {
            return DefaultResponse::error("30101", "Address line 1 must be 6-100 characters");
        }
        if order_details.city.len() < 2 || order_details.city.len() > 100 {
            return DefaultResponse::error("30101", "City must be 2-100 characters");
        }
        if order_details.zipcode.len() != 5 {
            return DefaultResponse::error("30101", "Zipcode must be exactly 5 characters");
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Check balance
        let balance = state
            .wallets
            .get(&user_id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        if balance < token_amount {
            return DefaultResponse::error("51301", "Insufficient balance");
        }

        // Deduct tokens
        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= token_amount;
        *state
            .wallets
            .entry(SYSTEM_SHOP_ACCOUNT)
            .or_insert(Decimal::ZERO) += token_amount;

        // Record transaction
        let now = Utc::now().to_rfc3339();
        let tx_id = Uuid::new_v4();
        let operation_id = Uuid::new_v4();

        state.transactions.push(TransactionRecord {
            id: tx_id,
            operation_id,
            category: Some(TransactionCategory::ShopPurchase),
            transaction_type: "DEBIT".into(),
            sender_id: user_id,
            recipient_id: SYSTEM_SHOP_ACCOUNT,
            token_amount: -token_amount,
            net_token_amount: -token_amount,
            message: Some(format!("Shop purchase: {}", shop_item_id)),
            fees: None,
            created_at: now.clone(),
        });

        // Create shop order
        let order = ShopOrderRecord {
            id: Uuid::new_v4(),
            transaction_id: tx_id,
            shop_item_id,
            buyer_id: user_id,
            token_amount,
            item_specs: order_details
                .shop_item_specs
                .as_ref()
                .and_then(|s| s.size.clone()),
            delivery: ShopDeliveryRecord {
                name: order_details.name,
                email: order_details.email,
                addressline1: order_details.addressline1,
                addressline2: order_details.addressline2,
                city: order_details.city,
                zipcode: order_details.zipcode,
                country: "GERMANY".into(),
            },
            created_at: now,
        };

        state.shop_orders.push(order);

        DefaultResponse::success("12201", "Order placed successfully")
    }
}
