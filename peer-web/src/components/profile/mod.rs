//! Profile components.
//!
//! This module provides reusable components for displaying user profiles,
//! including headers, statistics, action buttons, and relation modals.

mod actions;
mod header;
mod relations_modal;
mod stats;
mod user_list;
mod visibility;

pub use actions::{OwnProfileActions, ViewProfileActions};
pub use header::{ProfileHeader, ProfileHeaderSkeleton};
pub use relations_modal::RelationsModal;
pub use stats::{ProfileStats, ProfileStatsCompact};
pub use user_list::{FriendListItem, UserListItem};
pub use visibility::{HiddenOverlay, IllegalProfileBadge, ReportedBadge};
