//! Shop-related types for the Peer Shop feature.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Product data from Firebase Firestore.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShopProduct {
    pub post_id: String,
    /// Map of size label → stock count (e.g. {"S": 5, "M": 3, "L": 0}).
    #[serde(default)]
    pub sizes: Option<HashMap<String, u32>>,
    /// Stock for single-size (one-size) items.
    #[serde(default)]
    pub one_size_stock: Option<u32>,
}

impl ShopProduct {
    /// Whether this product has selectable sizes.
    pub fn has_sizes(&self) -> bool {
        self.sizes
            .as_ref()
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }

    /// Whether this is a one-size product.
    pub fn is_one_size(&self) -> bool {
        self.one_size_stock.is_some()
    }

    /// Whether the product is completely out of stock.
    pub fn is_out_of_stock(&self) -> bool {
        if let Some(stock) = self.one_size_stock {
            return stock == 0;
        }
        if let Some(sizes) = &self.sizes {
            return sizes.values().all(|&stock| stock == 0);
        }
        true
    }
}

/// Input for the delivery form in the checkout flow.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeliveryFormData {
    pub name: String,
    pub email: String,
    pub address_line_1: String,
    pub address_line_2: String,
    pub city: String,
    pub zipcode: String,
}

/// Result of validating the delivery form.
#[derive(Debug, Clone, Default)]
pub struct DeliveryFormErrors {
    pub name: Option<String>,
    pub email: Option<String>,
    pub address_line_1: Option<String>,
    pub city: Option<String>,
    pub zipcode: Option<String>,
    pub size: Option<String>,
}

impl DeliveryFormErrors {
    pub fn has_errors(&self) -> bool {
        self.name.is_some()
            || self.email.is_some()
            || self.address_line_1.is_some()
            || self.city.is_some()
            || self.zipcode.is_some()
            || self.size.is_some()
    }
}

/// Validate the delivery form data.
pub fn validate_delivery_form(
    data: &DeliveryFormData,
    size_required: bool,
    size_selected: &Option<String>,
) -> DeliveryFormErrors {
    let mut errors = DeliveryFormErrors::default();

    if data.name.trim().len() < 2 {
        errors.name = Some("Name is required".to_string());
    }

    // Simple email check: must contain @ with text on both sides
    let email = data.email.trim();
    let is_valid_email = {
        let parts: Vec<&str> = email.splitn(2, '@').collect();
        parts.len() == 2
            && !parts[0].is_empty()
            && parts[1].contains('.')
            && !parts[1].starts_with('.')
            && !parts[1].ends_with('.')
            && !email.contains(' ')
    };
    if !is_valid_email {
        errors.email = Some("Enter a valid email".to_string());
    }

    if data.address_line_1.trim().len() < 5 {
        errors.address_line_1 = Some("Address is required".to_string());
    }

    if data.city.trim().len() < 2 {
        errors.city = Some("City is required".to_string());
    }

    // ZIP must be exactly 5 digits
    let zip = data.zipcode.trim();
    let is_valid_zip = zip.len() == 5 && zip.chars().all(|c| c.is_ascii_digit());
    if !is_valid_zip {
        errors.zipcode = Some("Enter valid ZIP code".to_string());
    }

    if size_required && size_selected.is_none() {
        errors.size = Some("Please select a size".to_string());
    }

    errors
}

/// Response from the `performShopOrder` mutation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformShopOrderResponse {
    pub status: String,
    #[serde(default, rename = "RequestId")]
    pub request_id: Option<String>,
    #[serde(default, rename = "ResponseCode")]
    pub response_code: Option<String>,
    #[serde(default, rename = "ResponseMessage")]
    pub response_message: Option<String>,
}

impl PerformShopOrderResponse {
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }
}
