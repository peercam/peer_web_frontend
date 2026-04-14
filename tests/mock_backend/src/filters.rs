use uuid::Uuid;

use crate::state::{ContentVisibilityState, MockState, User};

/// System role bitmask values (role values 1, 2, 4 are system accounts).
const SYSTEM_ROLE_MASK: u32 = 0b111; // 1 | 2 | 4 = 7

/// Apply all content filtering specs to a user list.
/// Returns users that pass all filters.
pub fn filter_users<'a>(
    users: impl Iterator<Item = &'a User>,
    current_user_uid: Option<&Uuid>,
    state: &MockState,
) -> Vec<&'a User> {
    users
        .filter(|u| {
            // IllegalContentFilterSpec: exclude illegal content
            if u.visibility_status == ContentVisibilityState::Illegal {
                return false;
            }
            // SystemUserSpec: exclude system accounts (role bitmask 1, 2, 4)
            if u.role & SYSTEM_ROLE_MASK != 0 {
                return false;
            }
            // DeletedUserSpec: exclude soft-deleted users
            if u.status == 6 {
                return false;
            }
            // Block-based filtering (requires current user)
            if let Some(me) = current_user_uid {
                // UserIsBlockedByMeSpec: exclude users I blocked
                if state.is_blocked_by(me, &u.uid) {
                    return false;
                }
                // CurrentUserIsBlockedUserSpec: exclude users who blocked me
                if state.is_blocked_by(&u.uid, me) {
                    return false;
                }
            }
            true
        })
        .collect()
}

/// Apply offset/limit pagination to a slice.
pub fn paginate<T>(items: &[T], offset: usize, limit: usize) -> &[T] {
    let start = offset.min(items.len());
    let end = (start + limit).min(items.len());
    &items[start..end]
}
