//! Workspace-wide constants shared across pages and the mock backend.
//!
//! Keep additions here narrow: only values that have a single, fixed meaning
//! across the whole product (e.g. the Peer Shop account UUID baked into the
//! production data set) belong in this module.

/// UUID of the account that operates the Peer Shop.
///
/// Mirrors `PEER_SHOP_ID` in `js/global.js` and is used as a UI gate to expose
/// shop-side affordances (e.g. the delivery panel inside the wallet's expanded
/// transaction row) only to the shop operator's session.
pub const PEER_SHOP_ID: &str = "292bebb1-0951-47e8-ac8a-759138a2e4a9";

/// Returns `true` when the given user id is the Peer Shop account.
#[inline]
pub fn is_shop_account(user_id: &str) -> bool {
    user_id == PEER_SHOP_ID
}
