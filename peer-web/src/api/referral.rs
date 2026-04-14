//! Referral board API server functions.
//!
//! Provides server functions for fetching referral info and referral lists.

use leptos::prelude::*;
use serde::Serialize;

use crate::models::referral::{ReferralInfoResponse, ReferralListResponse};

/// Variables for the referralList query.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct ReferralListVars {
    offset: i32,
    limit: i32,
}

/// Fetch the current user's referral info (UUID and shareable link).
///
/// # Returns
///
/// The user's referral information including their unique referral UUID
/// and a shareable link.
#[server(GetReferralInfo, "/api")]
pub async fn get_referral_info() -> Result<ReferralInfoResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, GetReferralInfoData, GET_REFERRAL_INFO_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let data: GetReferralInfoData = query(GET_REFERRAL_INFO_QUERY, (), Some(&token)).await?;

    Ok(data.get_referral_info)
}

/// Fetch the referral list (who I invited and who invited me).
///
/// # Arguments
///
/// * `offset` - Number of entries to skip (for pagination)
/// * `limit` - Maximum number of entries to return
///
/// # Returns
///
/// The referral list containing:
/// - `invitedBy`: The user who invited the current user (if any)
/// - `iInvited`: Users the current user has invited
#[server(GetReferralList, "/api")]
pub async fn get_referral_list(
    offset: i32,
    limit: i32,
) -> Result<ReferralListResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, GetReferralListData, GET_REFERRAL_LIST_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ReferralListVars { offset, limit };
    let data: GetReferralListData = query(GET_REFERRAL_LIST_QUERY, vars, Some(&token)).await?;

    Ok(data.referral_list)
}
