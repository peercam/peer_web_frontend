//! Transaction-related types for the wallet feature.
//!
//! These types model the transaction history, balance, transfer operations,
//! and shop order details from the Peer GraphQL API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::DefaultResponse;

/// Transaction category from backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionCategory {
    P2pTransfer,
    AdPinned,
    PostCreate,
    Like,
    Dislike,
    Comment,
    TokenMint,
    ShopPurchase,
    InviterFeeEarn,
    Fee,
    #[serde(other)]
    Unknown,
}

impl TransactionCategory {
    /// Get display title for this category.
    pub fn display_title(&self, is_incoming: bool) -> &'static str {
        match self {
            Self::P2pTransfer => {
                if is_incoming {
                    "Received from"
                } else {
                    "Transfer to"
                }
            }
            Self::TokenMint => "Daily Mint",
            Self::Like => "Extra Like",
            Self::Dislike => "Dislike",
            Self::Comment => "Extra comment",
            Self::PostCreate => "Extra post",
            Self::AdPinned => "Pinned post promo",
            Self::ShopPurchase => "Peer Shop",
            Self::InviterFeeEarn => "Inviter fee",
            Self::Fee => "Fee",
            Self::Unknown => "Transaction",
        }
    }

    /// Get icon class for this category.
    pub fn icon_class(&self) -> &'static str {
        match self {
            Self::P2pTransfer => "", // Uses user avatar
            Self::TokenMint => "peer-icon-daily-mint",
            Self::Like => "peer-icon-like-fill red-text",
            Self::Dislike => "peer-icon-dislike-fill red-text",
            Self::Comment => "peer-icon-comment-fill",
            Self::PostCreate => "peer-icon-camera-fill",
            Self::AdPinned => "peer-icon-pinpost",
            Self::ShopPurchase => "peer-icon-shop",
            Self::InviterFeeEarn => "peer-icon-invite",
            Self::Fee => "peer-icon-wallet",
            Self::Unknown => "peer-icon-wallet",
        }
    }
}

/// Basic user info embedded in transactions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionUser {
    pub userid: String,
    #[serde(default)]
    pub img: Option<String>,
    pub username: String,
    pub slug: String,
    #[serde(default)]
    pub visibility_status: Option<String>,
    #[serde(default)]
    pub has_active_reports: Option<bool>,
    #[serde(default)]
    pub is_hidden_for_users: Option<bool>,
}

impl TransactionUser {
    /// Get the avatar URL for this user.
    pub fn avatar_url(&self) -> String {
        self.img
            .as_ref()
            .map(|img| {
                if img.starts_with("media/") {
                    format!("/media/{}", img.trim_start_matches("media/"))
                } else {
                    format!("/media/{}", img)
                }
            })
            .unwrap_or_else(|| "/svg/noname.svg".to_string())
    }

    /// Check if user profile should be hidden/blurred.
    pub fn is_hidden(&self) -> bool {
        self.is_hidden_for_users.unwrap_or(false)
            || self.visibility_status.as_deref() == Some("HIDDEN")
    }
}

/// Fee breakdown for a transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionFees {
    pub total: Decimal,
    pub burn: Decimal,
    pub peer: Decimal,
    #[serde(default)]
    pub inviter: Option<Decimal>,
}

/// Transaction history item.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub transaction_id: String,
    pub operationid: String,
    #[serde(default)]
    pub transaction_category: Option<TransactionCategory>,
    pub transactiontype: String,
    pub tokenamount: String,
    pub net_token_amount: String,
    #[serde(default)]
    pub message: Option<String>,
    pub createdat: String,
    pub sender: TransactionUser,
    pub recipient: TransactionUser,
    #[serde(default)]
    pub fees: Option<TransactionFees>,
}

impl Transaction {
    /// Check if this is an incoming transaction (positive amount).
    pub fn is_incoming(&self) -> bool {
        self.tokenamount
            .parse::<f64>()
            .map(|amt| amt >= 0.0)
            .unwrap_or(false)
    }

    /// Get the amount as a decimal.
    pub fn amount(&self) -> Decimal {
        self.tokenamount.parse().unwrap_or_default()
    }

    /// Get the net amount (after fees) as a decimal.
    pub fn net_amount(&self) -> Decimal {
        self.net_token_amount.parse().unwrap_or_default()
    }

    /// Get the display amount string (with + or - prefix).
    pub fn display_amount(&self) -> String {
        if self.is_incoming() {
            let net = self.net_amount();
            format!("+{}", format_decimal(net))
        } else {
            format_decimal(self.amount())
        }
    }

    /// Get category with fallback.
    pub fn category(&self) -> TransactionCategory {
        self.transaction_category
            .unwrap_or(TransactionCategory::Unknown)
    }

    /// Get the counterparty user (sender for incoming, recipient for outgoing).
    pub fn counterparty(&self) -> &TransactionUser {
        if self.is_incoming() {
            &self.sender
        } else {
            &self.recipient
        }
    }

    /// Check if this transaction has a message.
    pub fn has_message(&self) -> bool {
        self.message
            .as_ref()
            .map(|m| !m.is_empty())
            .unwrap_or(false)
    }

    /// Get truncated message for preview (max 50 chars).
    pub fn short_message(&self) -> Option<String> {
        self.message.as_ref().filter(|m| !m.is_empty()).map(|msg| {
            if msg.len() > 50 {
                format!("{}...", &msg[..50])
            } else {
                msg.clone()
            }
        })
    }
}

/// Transaction history response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHistoryResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub affected_rows: Option<Vec<Transaction>>,
}

impl TransactionHistoryResponse {
    /// Check if the response was successful.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }

    /// Get the transactions, or empty vec on failure.
    pub fn transactions(&self) -> Vec<Transaction> {
        self.affected_rows.clone().unwrap_or_default()
    }
}

/// Balance response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub currentliquidity: Option<Decimal>,
}

impl BalanceResponse {
    /// Get the balance, defaulting to zero if not present.
    pub fn balance(&self) -> Decimal {
        self.currentliquidity.unwrap_or_default()
    }
}

/// Transfer response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponse {
    pub meta: DefaultResponse,
    #[serde(default)]
    pub affected_rows: Option<TransferResult>,
}

/// Transfer result details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    #[serde(default)]
    pub token_send_formatted: Option<String>,
    #[serde(default)]
    pub tokens_substracted_from_wallet_formatted: Option<String>,
    #[serde(default)]
    pub createdat: Option<String>,
}

impl TransferResponse {
    /// Check if the transfer was successful.
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Shop order details for shop purchase transactions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopOrderDetails {
    pub shop_order_id: String,
    pub shop_item_id: String,
    #[serde(default)]
    pub shop_item_specs: Option<ShopItemSpecs>,
    #[serde(default)]
    pub delivery_details: Option<DeliveryDetails>,
}

/// Shop item specifications.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShopItemSpecs {
    #[serde(default)]
    pub size: Option<String>,
}

/// Delivery details for shop orders.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeliveryDetails {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub addressline1: Option<String>,
    #[serde(default)]
    pub addressline2: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub zipcode: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
}

/// Shop order details response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopOrderDetailsResponse {
    #[serde(default)]
    pub affected_rows: Option<Vec<ShopOrderDetails>>,
}

// ============================================================================
// Utility functions
// ============================================================================

/// Format a decimal for display (remove trailing zeros).
pub fn format_decimal(d: Decimal) -> String {
    let s = d.to_string();
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Format a balance with thousand separators (en-US style).
///
/// Mirrors the legacy `formatAmount()` helper from `js/wallet.js`, which calls
/// `Number(x).toLocaleString('en-US', { maximumFractionDigits: 4 })`. The
/// fractional part is rounded to at most 4 digits (banker's rounding via
/// `Decimal::round_dp`) and trailing zeros are stripped, then commas are
/// inserted every three digits in the integer portion.
pub fn format_balance(balance: Decimal) -> String {
    let rounded = balance.round_dp(4);
    let raw = rounded.to_string();
    // Split on the decimal point and strip trailing zeros only from the
    // fractional side (so "1000" stays "1000" but "1234.5000" becomes
    // "1234.5"). Use the raw `Decimal::to_string` rather than the trim-happy
    // `format_decimal` helper, which would also clip the integer's zeros.
    let (int_part, frac_part) = match raw.split_once('.') {
        Some((i, f)) => {
            let trimmed_frac = f.trim_end_matches('0');
            let frac = if trimmed_frac.is_empty() {
                None
            } else {
                Some(trimmed_frac)
            };
            (i, frac)
        }
        None => (raw.as_str(), None),
    };

    let (sign, digits) = if let Some(rest) = int_part.strip_prefix('-') {
        ("-", rest)
    } else {
        ("", int_part)
    };
    let digits = if digits.is_empty() { "0" } else { digits };

    let mut grouped_rev = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped_rev.push(',');
        }
        grouped_rev.push(ch);
    }
    let grouped: String = grouped_rev.chars().rev().collect();

    match frac_part {
        Some(f) => format!("{sign}{grouped}.{f}"),
        None => format!("{sign}{grouped}"),
    }
}

/// Calculate total amount with fees (4% fee rate).
pub fn calculate_total_with_fees(amount: Decimal) -> Decimal {
    let fee_rate = Decimal::new(4, 2); // 0.04 = 4%
    amount + (amount * fee_rate)
}

/// Calculate the fee breakdown for an amount.
pub fn calculate_fees(amount: Decimal) -> TransactionFees {
    let burn = amount * Decimal::new(1, 2); // 1%
    let peer = amount * Decimal::new(2, 2); // 2%
    let inviter = amount * Decimal::new(1, 2); // 1%
    let total = burn + peer + inviter;

    TransactionFees {
        total,
        burn,
        peer,
        inviter: Some(inviter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_balance_groups_thousands() {
        let cases = [
            ("0", "0"),
            ("999", "999"),
            ("1000", "1,000"),
            ("12345", "12,345"),
            ("1234567", "1,234,567"),
            ("1234.5", "1,234.5"),
            ("1234.5000", "1,234.5"),
            ("0.0001", "0.0001"),
            ("-1000", "-1,000"),
            ("0.000049", "0"),
            ("-1234.56789", "-1,234.5679"),
        ];
        for (input, expected) in cases {
            let d: Decimal = input.parse().unwrap();
            assert_eq!(format_balance(d), expected, "input={input}");
        }
    }
}
