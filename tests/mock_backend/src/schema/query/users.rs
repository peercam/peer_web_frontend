use async_graphql::{Context, ID, Object};
use uuid::Uuid;

use crate::CurrentUser;
use crate::filters::{filter_users, paginate};
use crate::state::{ContentFilterState, SharedState};
use crate::types::registration::DefaultResponse;
use crate::types::user::*;

#[derive(Default)]
pub struct UserQuery;

/// Build a ProfileUserGql from a User, annotated relative to the current user.
fn build_profile_user(
    user: &crate::state::User,
    me: &Uuid,
    state: &crate::state::MockState,
) -> ProfileUserGql {
    ProfileUserGql {
        userid: ID::from(user.uid.to_string()),
        username: user.username.clone(),
        slug: user.slug_num,
        img: user.img.clone(),
        visibility_status: convert_visibility(user.visibility_status),
        is_hidden_for_users: false,
        has_active_reports: state.has_active_reports(&user.uid),
        isfollowed: state.is_following(me, &user.uid),
        isfollowing: state.is_following(&user.uid, me),
    }
}

/// Build a BasicUserInfoGql from a User.
fn build_basic_user_info(
    user: &crate::state::User,
    state: &crate::state::MockState,
) -> BasicUserInfoGql {
    BasicUserInfoGql {
        userid: ID::from(user.uid.to_string()),
        img: user.img.clone(),
        username: user.username.clone(),
        slug: user.slug_num,
        biography: user.biography.clone(),
        visibility_status: convert_visibility(user.visibility_status),
        is_hidden_for_users: false,
        has_active_reports: state.has_active_reports(&user.uid),
        updatedat: Some(user.updated_at.clone()),
    }
}

fn build_user_prefs(prefs: &crate::state::UserPreferencesState) -> UserPreferencesGql {
    UserPreferencesGql {
        content_filtering_severity_level: Some(match prefs.content_filtering_severity_level {
            ContentFilterState::Mygrandmalikes => ContentFilterType::Mygrandmalikes,
            ContentFilterState::Mygrandmahates => ContentFilterType::Mygrandmahates,
        }),
        onboardings_were_shown: prefs
            .onboardings_were_shown
            .iter()
            .filter_map(|s| match s.as_str() {
                "INTROONBOARDING" => Some(OnboardingType::IntroOnboarding),
                _ => None,
            })
            .collect(),
    }
}

#[Object]
impl UserQuery {
    // ========================================================================
    // getProfile
    // ========================================================================

    async fn get_profile(
        &self,
        ctx: &Context<'_>,
        userid: Option<ID>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<ContentFilterType>,
    ) -> ProfileInfoResponse {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return ProfileInfoResponse {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let target_uid = match &userid {
            Some(id) => match Uuid::parse_str(id.as_str()) {
                Ok(uid) => uid,
                Err(_) => {
                    return ProfileInfoResponse {
                        meta: DefaultResponse::error("30201", "Invalid user UUID"),
                        affected_rows: None,
                    };
                }
            },
            None => me,
        };

        let user = match state_read.users.get(&target_uid) {
            Some(u) => u,
            None => {
                return ProfileInfoResponse {
                    meta: DefaultResponse::error("21001", "User not found"),
                    affected_rows: None,
                };
            }
        };

        let profile = ProfileGql {
            id: ID::from(user.uid.to_string()),
            username: user.username.clone(),
            status: user.status as i32,
            slug: user.slug_num,
            img: user.img.clone(),
            biography: user.biography.clone(),
            visibility_status: convert_visibility(user.visibility_status),
            is_hidden_for_users: false,
            has_active_reports: state_read.has_active_reports(&target_uid),
            i_follow_this_user: state_read.is_following(&me, &target_uid),
            this_user_follows_me: state_read.is_following(&target_uid, &me),
            isreported: state_read.user_has_reported(&me, &target_uid),
            amountposts: 0,
            amounttrending: 0,
            amountfollowed: state_read.count_following(&target_uid),
            amountfollower: state_read.count_followers(&target_uid),
            amountfriends: state_read.count_friends(&target_uid),
            amountblocked: state_read.count_blocked(&target_uid),
            amountreports: state_read.count_reports(&target_uid),
        };

        ProfileInfoResponse {
            meta: DefaultResponse::success("11008", "Profile loaded successfully"),
            affected_rows: Some(profile),
        }
    }

    // ========================================================================
    // searchUser
    // ========================================================================

    async fn search_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> SearchUserResponse {
        let current_user = ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0);
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let matching = state_read.search_users_by_username(&username);
        let filtered = filter_users(matching.into_iter(), current_user.as_ref(), &state_read);

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(10).clamp(1, 20) as usize;
        let total = filtered.len() as i32;
        let page = paginate(&filtered, off, lim);

        if page.is_empty() {
            return SearchUserResponse {
                meta: DefaultResponse::success("21001", "No users found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let results: Vec<SearchUserResult> = page
            .iter()
            .map(|u| SearchUserResult {
                id: ID::from(u.uid.to_string()),
                username: u.username.clone(),
                slug: u.slug_num,
                img: u.img.clone(),
            })
            .collect();

        SearchUserResponse {
            meta: DefaultResponse::success("11001", "Users retrieved successfully"),
            counter: total,
            affected_rows: Some(results),
        }
    }

    // ========================================================================
    // listUsersV2
    // ========================================================================

    async fn list_users_v2(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<ContentFilterType>,
        userid: Option<ID>,
        username: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> UserListResponse {
        let current_user = ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0);
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let candidates: Vec<&crate::state::User> = if let Some(ref id) = userid {
            match Uuid::parse_str(id.as_str()) {
                Ok(uid) => state_read.users.get(&uid).into_iter().collect(),
                Err(_) => vec![],
            }
        } else if let Some(ref name) = username {
            state_read.search_users_by_username(name)
        } else {
            state_read.users.values().collect()
        };

        let filtered = filter_users(candidates.into_iter(), current_user.as_ref(), &state_read);

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(10).clamp(1, 20) as usize;
        let total = filtered.len() as i32;
        let page = paginate(&filtered, off, lim);

        if page.is_empty() {
            return UserListResponse {
                meta: DefaultResponse::success("21001", "No users found"),
                counter: 0,
                affected_rows: None,
            };
        }

        let items: Vec<UserListItem> = page
            .iter()
            .map(|u| UserListItem {
                id: ID::from(u.uid.to_string()),
                username: u.username.clone(),
                status: u.status as i32,
                slug: u.slug_num,
                img: u.img.clone(),
                biography: u.biography.clone(),
                visibility_status: convert_visibility(u.visibility_status),
                is_hidden_for_users: false,
                has_active_reports: state_read.has_active_reports(&u.uid),
                createdat: Some(u.created_at.clone()),
                updatedat: Some(u.updated_at.clone()),
            })
            .collect();

        UserListResponse {
            meta: DefaultResponse::success("11001", "Users retrieved successfully"),
            counter: total,
            affected_rows: Some(items),
        }
    }

    // ========================================================================
    // getUser
    // ========================================================================

    async fn get_user(&self, ctx: &Context<'_>, id: ID) -> GetUserResponseGql {
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let uid = match Uuid::parse_str(id.as_str()) {
            Ok(uid) => uid,
            Err(_) => {
                return GetUserResponseGql {
                    meta: DefaultResponse::error("21001", "User not found"),
                    affected_rows: None,
                };
            }
        };

        let user = match state_read.users.get(&uid) {
            Some(u) => u,
            None => {
                return GetUserResponseGql {
                    meta: DefaultResponse::error("21001", "User not found"),
                    affected_rows: None,
                };
            }
        };

        let user_prefs = state_read.preferences.get(&uid).map(build_user_prefs);

        GetUserResponseGql {
            meta: DefaultResponse::success("11001", "User found"),
            affected_rows: Some(GetUserResult {
                id: ID::from(user.uid.to_string()),
                username: user.username.clone(),
                slug: user.slug_num,
                img: user.img.clone(),
                biography: user.biography.clone(),
                amount_followers: state_read.count_followers(&uid),
                amount_following: state_read.count_following(&uid),
                amount_peers: state_read.count_friends(&uid),
                user_preferences: user_prefs,
            }),
        }
    }

    // ========================================================================
    // listFollowRelations
    // ========================================================================

    async fn list_follow_relations(
        &self,
        ctx: &Context<'_>,
        userid: Option<ID>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<ContentFilterType>,
        _offset: i32,
        _limit: i32,
    ) -> FollowRelationsResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return FollowRelationsResponseGql {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let target_uid = match &userid {
            Some(id) => Uuid::parse_str(id.as_str()).unwrap_or(me),
            None => me,
        };

        // Collect followers (users who follow target)
        let followers: Vec<ProfileUserGql> = state_read
            .follows
            .iter()
            .filter(|(_, followed)| *followed == target_uid)
            .filter_map(|(follower, _)| state_read.users.get(follower))
            .map(|u| build_profile_user(u, &me, &state_read))
            .collect();

        // Collect following (users target follows)
        let following: Vec<ProfileUserGql> = state_read
            .follows
            .iter()
            .filter(|(follower, _)| *follower == target_uid)
            .filter_map(|(_, followed)| state_read.users.get(followed))
            .map(|u| build_profile_user(u, &me, &state_read))
            .collect();

        let total = followers.len() + following.len();

        FollowRelationsResponseGql {
            meta: DefaultResponse::success("11101", "Follow relations loaded"),
            counter: total as i32,
            affected_rows: Some(FollowRelationsGql {
                followers,
                following,
            }),
        }
    }

    // ========================================================================
    // listFriends
    // ========================================================================

    async fn list_friends(
        &self,
        ctx: &Context<'_>,
        userid: Option<ID>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<ContentFilterType>,
        offset: i32,
        limit: i32,
    ) -> FriendsResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return FriendsResponseGql {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    counter: 0,
                    affected_rows: vec![],
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let target_uid = match &userid {
            Some(id) => Uuid::parse_str(id.as_str()).unwrap_or(me),
            None => me,
        };

        let friends: Vec<BasicUserInfoGql> = state_read
            .follows
            .iter()
            .filter(|(follower, followed)| {
                *follower == target_uid && state_read.is_following(followed, &target_uid)
            })
            .filter_map(|(_, followed)| state_read.users.get(followed))
            .map(|u| build_basic_user_info(u, &state_read))
            .collect();

        let total = friends.len() as i32;
        let off = offset.max(0) as usize;
        let lim = limit.clamp(1, 20) as usize;
        let page: Vec<BasicUserInfoGql> = friends.into_iter().skip(off).take(lim).collect();

        if page.is_empty() {
            return FriendsResponseGql {
                meta: DefaultResponse::success("21101", "No friends found"),
                counter: 0,
                affected_rows: vec![],
            };
        }

        FriendsResponseGql {
            meta: DefaultResponse::success("11102", "Friends loaded"),
            counter: total,
            affected_rows: page,
        }
    }

    // ========================================================================
    // listBlockedUsers
    // ========================================================================

    async fn list_blocked_users(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<ContentFilterType>,
        _offset: i32,
        _limit: i32,
    ) -> BlockedUsersResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return BlockedUsersResponseGql {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let i_blocked: Vec<BlockedUserGql> = state_read
            .blocks
            .iter()
            .filter(|(blocker, _)| *blocker == me)
            .filter_map(|(_, blocked)| state_read.users.get(blocked))
            .map(|u| BlockedUserGql {
                userid: u.uid.to_string(),
                img: u.img.clone(),
                username: u.username.clone(),
                slug: u.slug_num,
                has_active_reports: state_read.has_active_reports(&u.uid),
                visibility_status: convert_visibility(u.visibility_status),
                is_hidden_for_users: false,
            })
            .collect();

        let blocked_by: Vec<BlockedUserGql> = state_read
            .blocks
            .iter()
            .filter(|(_, blocked)| *blocked == me)
            .filter_map(|(blocker, _)| state_read.users.get(blocker))
            .map(|u| BlockedUserGql {
                userid: u.uid.to_string(),
                img: u.img.clone(),
                username: u.username.clone(),
                slug: u.slug_num,
                has_active_reports: state_read.has_active_reports(&u.uid),
                visibility_status: convert_visibility(u.visibility_status),
                is_hidden_for_users: false,
            })
            .collect();

        let total = (i_blocked.len() + blocked_by.len()) as i32;

        BlockedUsersResponseGql {
            meta: DefaultResponse::success("11107", "Blocked users loaded"),
            counter: total,
            affected_rows: Some(BlockedUsersGql {
                i_blocked,
                blocked_by,
            }),
        }
    }

    // ========================================================================
    // getUserInfo
    // ========================================================================

    async fn get_user_info(&self, ctx: &Context<'_>) -> UserInfoResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return UserInfoResponseGql {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let user = match state_read.users.get(&me) {
            Some(u) => u,
            None => {
                return UserInfoResponseGql {
                    meta: DefaultResponse::error("21001", "User not found"),
                    affected_rows: None,
                };
            }
        };

        let user_prefs = state_read.preferences.get(&me).map(build_user_prefs);

        let invited_by = state_read
            .referral_invitations
            .get(&me)
            .map(|uid| ID::from(uid.to_string()));

        UserInfoResponseGql {
            meta: DefaultResponse::success("11009", "User info loaded"),
            affected_rows: Some(UserInfoGql {
                userid: ID::from(user.uid.to_string()),
                liquidity: 0.0,
                amountposts: 0,
                amountreports: state_read.count_reports(&me),
                amountblocked: state_read.count_blocked(&me),
                amountfollower: state_read.count_followers(&me),
                amountfollowed: state_read.count_following(&me),
                amountfriends: state_read.count_friends(&me),
                invited: invited_by,
                updatedat: Some(user.updated_at.clone()),
                user_preferences: user_prefs,
            }),
        }
    }

    // ========================================================================
    // getReferralInfo
    // ========================================================================

    async fn get_referral_info(&self, ctx: &Context<'_>) -> ReferralInfoResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return ReferralInfoResponseGql {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    referral_uuid: None,
                    referral_link: None,
                };
            }
        };

        let referral_uuid = me.to_string();
        let referral_link = format!("https://peer.com/invite?referralUuid={}", referral_uuid);

        ReferralInfoResponseGql {
            meta: DefaultResponse::success("11011", "Referral info loaded"),
            referral_uuid: Some(ID::from(referral_uuid)),
            referral_link: Some(referral_link),
        }
    }

    // ========================================================================
    // referralList
    // ========================================================================

    async fn referral_list(
        &self,
        ctx: &Context<'_>,
        _offset: i32,
        _limit: i32,
    ) -> ReferralListResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return ReferralListResponseGql {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    counter: 0,
                    affected_rows: ReferralUsersGql {
                        invited_by: None,
                        i_invited: vec![],
                    },
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let invited_by = state_read
            .referral_invitations
            .get(&me)
            .and_then(|inviter_uid| state_read.users.get(inviter_uid))
            .map(|u| build_profile_user(u, &me, &state_read));

        let i_invited: Vec<ProfileUserGql> = state_read
            .referral_invitations
            .iter()
            .filter(|(_, inviter)| **inviter == me)
            .filter_map(|(invitee, _)| state_read.users.get(invitee))
            .map(|u| build_profile_user(u, &me, &state_read))
            .collect();

        let total = i_invited.len() as i32 + if invited_by.is_some() { 1 } else { 0 };

        ReferralListResponseGql {
            meta: DefaultResponse::success("11011", "Referral list loaded"),
            counter: total,
            affected_rows: ReferralUsersGql {
                invited_by,
                i_invited,
            },
        }
    }
}
