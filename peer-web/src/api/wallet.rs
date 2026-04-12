//! Wallet API server functions.
//!
//! Provides server functions for fetching balance, transaction history,
//! and performing token transfers.

use leptos::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::models::transaction::{
    BalanceResponse, Transaction, TransactionHistoryResponse, TransferResponse,
};
use crate::models::profile::FriendsResponse;
use crate::models::post::UserSearchResult;

/// Variables for the transactionHistory query.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct TransactionHistoryVars {
    offset: i32,
    limit: i32,
}

/// Variables for the transfer mutation.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct TransferVars {
    recipient: String,
    numberoftokens: Decimal,
    message: String,
}

/// Variables for user search.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct SearchUserVars {
    username: String,
}

/// Fetch the current user's token balance.
///
/// # Returns
///
/// The user's current token balance as a Decimal.
#[server(GetBalance, "/api")]
pub async fn get_balance() -> Result<Decimal, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, BalanceData, BALANCE_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let data: BalanceData = query(BALANCE_QUERY, (), Some(&token)).await?;

    Ok(data.balance.balance())
}

/// Fetch the full balance response (includes meta info).
#[server(GetBalanceResponse, "/api")]
pub async fn get_balance_response() -> Result<BalanceResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, BalanceData, BALANCE_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let data: BalanceData = query(BALANCE_QUERY, (), Some(&token)).await?;

    Ok(data.balance)
}

/// Fetch transaction history with pagination.
///
/// # Arguments
///
/// * `offset` - Number of transactions to skip
/// * `limit` - Maximum number of transactions to return (max 20)
///
/// # Returns
///
/// A list of transactions.
#[server(GetTransactionHistory, "/api")]
pub async fn get_transaction_history(
    offset: i32,
    limit: i32,
) -> Result<Vec<Transaction>, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, TransactionHistoryData, TRANSACTION_HISTORY_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = TransactionHistoryVars { offset, limit };
    let data: TransactionHistoryData =
        query(TRANSACTION_HISTORY_QUERY, vars, Some(&token)).await?;

    Ok(data.transaction_history.transactions())
}

/// Fetch the full transaction history response (includes meta).
#[server(GetTransactionHistoryResponse, "/api")]
pub async fn get_transaction_history_response(
    offset: i32,
    limit: i32,
) -> Result<TransactionHistoryResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, TransactionHistoryData, TRANSACTION_HISTORY_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = TransactionHistoryVars { offset, limit };
    let data: TransactionHistoryData =
        query(TRANSACTION_HISTORY_QUERY, vars, Some(&token)).await?;

    Ok(data.transaction_history)
}

/// List friends for transfer recipient selection.
///
/// Returns the user's mutual follows (friends) that can receive transfers.
#[server(ListTransferRecipients, "/api")]
pub async fn list_transfer_recipients() -> Result<FriendsResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, ListFriendsData, LIST_FRIENDS_QUERY};
    use serde::Serialize;

    #[derive(Serialize)]
    struct ListFriendsVars {
        #[serde(skip_serializing_if = "Option::is_none")]
        userid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content_filter_by: Option<String>,
        offset: i32,
        limit: i32,
    }

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ListFriendsVars {
        userid: None,
        content_filter_by: None,
        offset: 0,
        limit: 100, // Get all friends for transfer selection
    };

    let data: ListFriendsData = query(LIST_FRIENDS_QUERY, vars, Some(&token)).await?;

    Ok(data.list_friends)
}

/// Search for users by username.
///
/// # Arguments
///
/// * `username` - Partial username to search for
///
/// # Returns
///
/// List of matching users.
#[server(SearchTransferRecipient, "/api")]
pub async fn search_transfer_recipient(username: String) -> Result<Vec<UserSearchResult>, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, SearchUserData, SEARCH_USERS_QUERY};
    use serde::Serialize;

    #[derive(Serialize)]
    struct SearchVars {
        username: String,
    }

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = SearchVars { username };
    let data: SearchUserData = query(SEARCH_USERS_QUERY, vars, Some(&token)).await?;

    Ok(data.search_user.affected_rows)
}

/// Transfer tokens to another user.
///
/// # Arguments
///
/// * `recipient_id` - UUID of the recipient user
/// * `amount` - Amount of tokens to transfer (min: 0.000001, max 8 decimals)
/// * `message` - Optional message (max 500 chars, no URLs)
///
/// # Returns
///
/// Transfer response indicating success or failure.
#[server(TransferTokens, "/api")]
pub async fn transfer_tokens(
    recipient_id: String,
    amount: Decimal,
    message: Option<String>,
) -> Result<TransferResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, TransferData, TRANSFER_MUTATION};

    // Validate amount
    let min_amount = Decimal::new(1, 6); // 0.000001
    if amount < min_amount {
        return Err(ServerFnError::new("Amount must be at least 0.000001"));
    }

    // Validate message if provided
    if let Some(ref msg) = message {
        if msg.len() > 500 {
            return Err(ServerFnError::new("Message exceeds 500 characters"));
        }

        // Check for URLs (simple check)
        let url_patterns = ["://", "www.", ".com", ".net", ".org", ".io"];
        if url_patterns.iter().any(|p| msg.to_lowercase().contains(p)) {
            return Err(ServerFnError::new("URLs are not allowed in messages"));
        }
    }

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = TransferVars {
        recipient: recipient_id,
        numberoftokens: amount,
        message: message.unwrap_or_default(),
    };

    let data: TransferData = mutate(TRANSFER_MUTATION, vars, Some(&token)).await?;

    if !data.resolve_transfer_v2.is_success() {
        return Err(ServerFnError::new(
            data.resolve_transfer_v2.meta.response_message.clone(),
        ));
    }

    Ok(data.resolve_transfer_v2)
}
