//! Moderation API server functions.
//!
//! Provides server functions for the admin dashboard:
//! fetching stats, listing moderation tickets, and performing actions.

use leptos::prelude::*;
use serde::Serialize;

use crate::models::common::DefaultResponse;
use crate::models::moderation::{
    ModerationAction, ModerationItemListResponse, ModerationStatsResponse,
};

/// Variables for the moderationItems query.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct ModerationItemsVars {
    offset: i32,
    limit: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

/// Variables for the performModeration mutation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct PerformModerationVars {
    moderation_ticket_id: String,
    moderation_action: String,
}

/// Fetch moderation stats (ticket counts by status).
#[server(GetModerationStats, "/api")]
pub async fn get_moderation_stats() -> Result<ModerationStatsResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{MODERATION_STATS_QUERY, ModerationStatsData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let data: ModerationStatsData =
        query(MODERATION_STATS_QUERY, serde_json::json!({}), Some(&token)).await?;

    Ok(data.moderation_stats)
}

/// Fetch moderation tickets with optional filters.
#[server(GetModerationItems, "/api")]
pub async fn get_moderation_items(
    content_type: Option<String>,
    status: Option<String>,
    offset: i32,
    limit: i32,
) -> Result<ModerationItemListResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{MODERATION_ITEMS_QUERY, ModerationItemsData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ModerationItemsVars {
        offset,
        limit,
        content_type,
        status,
    };

    let data: ModerationItemsData = query(MODERATION_ITEMS_QUERY, vars, Some(&token)).await?;

    Ok(data.moderation_items)
}

/// Perform a moderation action on a ticket.
#[server(PerformModeration, "/api")]
pub async fn perform_moderation(
    moderation_ticket_id: String,
    moderation_action: ModerationAction,
) -> Result<DefaultResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{PERFORM_MODERATION_MUTATION, PerformModerationData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = PerformModerationVars {
        moderation_ticket_id,
        moderation_action: moderation_action.to_string(),
    };

    let data: PerformModerationData =
        mutate(PERFORM_MODERATION_MUTATION, vars, Some(&token)).await?;

    Ok(data.perform_moderation)
}
