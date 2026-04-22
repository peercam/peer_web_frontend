//! Shop API server functions.
//!
//! Provides server functions for placing shop orders and fetching order details.

#![allow(clippy::too_many_arguments)]

use leptos::prelude::*;
use serde::Serialize;

use crate::models::shop::PerformShopOrderResponse;
use crate::models::transaction::ShopOrderDetailsResponse;

/// Variables for the performShopOrder mutation.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PerformShopOrderVars {
    token_amount: String,
    shop_item_id: String,
    name: String,
    email: String,
    addressline1: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    addressline2: Option<String>,
    city: String,
    zipcode: String,
    country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
}

/// Variables for the shopOrderDetails query.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShopOrderDetailsVars {
    transaction_id: String,
}

/// Place a shop order (purchase a product with tokens).
///
/// Performs server-side validation of all delivery fields before calling GraphQL.
#[server(PerformShopOrder, "/api")]
pub async fn perform_shop_order(
    token_amount: String,
    shop_item_id: String,
    name: String,
    email: String,
    address_line_1: String,
    address_line_2: Option<String>,
    city: String,
    zipcode: String,
    size: Option<String>,
) -> Result<PerformShopOrderResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{PERFORM_SHOP_ORDER_MUTATION, PerformShopOrderData, mutate};

    // Server-side validation (Defense in Depth)
    if name.trim().len() < 2 {
        return Err(ServerFnError::new("Name must be at least 2 characters"));
    }
    {
        let email_trimmed = email.trim();
        let parts: Vec<&str> = email_trimmed.splitn(2, '@').collect();
        let is_valid_email = parts.len() == 2
            && !parts[0].is_empty()
            && parts[1].contains('.')
            && !parts[1].starts_with('.')
            && !parts[1].ends_with('.')
            && !email_trimmed.contains(' ');
        if !is_valid_email {
            return Err(ServerFnError::new("Invalid email address"));
        }
    }
    if address_line_1.trim().len() < 5 {
        return Err(ServerFnError::new("Address must be at least 5 characters"));
    }
    if city.trim().len() < 2 {
        return Err(ServerFnError::new("City must be at least 2 characters"));
    }
    {
        let zip = zipcode.trim();
        if zip.len() != 5 || !zip.chars().all(|c| c.is_ascii_digit()) {
            return Err(ServerFnError::new("ZIP code must be exactly 5 digits"));
        }
    }

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let addr2 = address_line_2.filter(|s| !s.trim().is_empty());

    let vars = PerformShopOrderVars {
        token_amount,
        shop_item_id,
        name: name.trim().to_string(),
        email: email.trim().to_string(),
        addressline1: address_line_1.trim().to_string(),
        addressline2: addr2.map(|s| s.trim().to_string()),
        city: city.trim().to_string(),
        zipcode: zipcode.trim().to_string(),
        country: "GERMANY".to_string(),
        size,
    };

    let data: PerformShopOrderData =
        mutate(PERFORM_SHOP_ORDER_MUTATION, vars, Some(&token)).await?;

    Ok(data.perform_shop_order)
}

/// Fetch shop order details for a transaction.
#[server(GetShopOrderDetails, "/api")]
pub async fn get_shop_order_details(
    transaction_id: String,
) -> Result<ShopOrderDetailsResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{SHOP_ORDER_DETAILS_QUERY, ShopOrderDetailsData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ShopOrderDetailsVars { transaction_id };
    let data: ShopOrderDetailsData = query(SHOP_ORDER_DETAILS_QUERY, vars, Some(&token)).await?;

    Ok(data.shop_order_details)
}
