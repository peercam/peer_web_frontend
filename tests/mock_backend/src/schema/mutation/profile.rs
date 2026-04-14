use async_graphql::{Context, ID, Object};
use uuid::Uuid;

use crate::CurrentUser;
use crate::state::{ContentFilterState, SharedState, UserReport};
use crate::types::registration::DefaultResponse;
use crate::types::user::*;

#[derive(Default)]
pub struct ProfileMutation;

/// Helper: require auth for update mutations.
fn require_auth_update(ctx: &Context<'_>) -> Result<Uuid, UpdateResponseGql> {
    ctx.data_opt::<CurrentUser>()
        .and_then(|cu| cu.0)
        .ok_or_else(|| UpdateResponseGql::error("60501"))
}

#[Object]
impl ProfileMutation {
    // ========================================================================
    // toggleUserFollowStatus
    // ========================================================================

    async fn toggle_user_follow_status(
        &self,
        ctx: &Context<'_>,
        userid: ID,
    ) -> FollowStatusResponseGql {
        let me = match crate::require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => {
                return FollowStatusResponseGql {
                    meta: resp,
                    isfollowing: false,
                };
            }
        };

        let target = match Uuid::parse_str(userid.as_str()) {
            Ok(uid) => uid,
            Err(_) => {
                return FollowStatusResponseGql {
                    meta: DefaultResponse::error("30201", "Invalid UUID"),
                    isfollowing: false,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let key = (me, target);
        if state_write.follows.contains(&key) {
            state_write.follows.remove(&key);
            FollowStatusResponseGql {
                meta: DefaultResponse::success("11103", "Unfollowed user"),
                isfollowing: false,
            }
        } else {
            state_write.follows.insert(key);
            FollowStatusResponseGql {
                meta: DefaultResponse::success("11104", "Now following user"),
                isfollowing: true,
            }
        }
    }

    // ========================================================================
    // toggleBlockUserStatus
    // ========================================================================

    async fn toggle_block_user_status(&self, ctx: &Context<'_>, userid: ID) -> DefaultResponse {
        let me = match crate::require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let target = match Uuid::parse_str(userid.as_str()) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("30201", "Invalid UUID"),
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let key = (me, target);
        if state_write.blocks.contains(&key) {
            state_write.blocks.remove(&key);
            DefaultResponse::success("11106", "User unblocked")
        } else {
            state_write.blocks.insert(key);
            state_write.follows.remove(&(me, target));
            state_write.follows.remove(&(target, me));
            DefaultResponse::success("11105", "User blocked")
        }
    }

    // ========================================================================
    // reportUser
    // ========================================================================

    async fn report_user(&self, ctx: &Context<'_>, userid: ID) -> DefaultResponse {
        let me = match crate::require_auth(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let target = match Uuid::parse_str(userid.as_str()) {
            Ok(uid) => uid,
            Err(_) => return DefaultResponse::error("31007", "User not found"),
        };

        if me == target {
            return DefaultResponse::error("31009", "Cannot report yourself");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        if !state_write.users.contains_key(&target) {
            return DefaultResponse::error("31007", "User not found");
        }

        if state_write.user_has_reported(&me, &target) {
            return DefaultResponse::error("31008", "Already reported");
        }

        state_write.reports.push(UserReport {
            reporter: me,
            reported: target,
            created_at: chrono::Utc::now().to_rfc3339(),
        });

        DefaultResponse::success("11012", "User reported")
    }

    // ========================================================================
    // updateProfileImage
    // ========================================================================

    async fn update_profile_image(&self, ctx: &Context<'_>, img: String) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        if img.is_empty() {
            return UpdateResponseGql::error("30101");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        if let Some(user) = state_write.users.get_mut(&me) {
            user.img = Some(img);
        }

        UpdateResponseGql::success("11004")
    }

    // ========================================================================
    // updateBio
    // ========================================================================

    async fn update_bio(&self, ctx: &Context<'_>, biography: String) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        if biography.is_empty() {
            return UpdateResponseGql::error("30101");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        if let Some(user) = state_write.users.get_mut(&me) {
            user.biography = Some(biography);
        }

        UpdateResponseGql::success("11003")
    }

    // ========================================================================
    // updateUsername
    // ========================================================================

    async fn update_username(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
    ) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let username_re = regex::Regex::new(r"^[a-zA-Z0-9_-]{3,23}$").unwrap();
        if !username_re.is_match(&username) {
            return UpdateResponseGql::error("30202");
        }

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let stored = match state_write.user_passwords.get(&me) {
            Some(p) => p.clone(),
            None => return UpdateResponseGql::error("31001"),
        };
        if stored != password {
            return UpdateResponseGql::error("31001");
        }

        if let Some(user) = state_write.users.get_mut(&me) {
            user.username = username.clone();
            user.slug = username.to_lowercase();
        }

        UpdateResponseGql::success("11007")
    }

    // ========================================================================
    // updateEmail
    // ========================================================================

    async fn update_email(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> UpdateResponseGql {
        let me = match require_auth_update(ctx) {
            Ok(uid) => uid,
            Err(resp) => return resp,
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let stored = match state_write.user_passwords.get(&me) {
            Some(p) => p.clone(),
            None => return UpdateResponseGql::error("31001"),
        };
        if stored != password {
            return UpdateResponseGql::error("31001");
        }

        if let Some(user) = state_write.users.get_mut(&me) {
            let old_email = user.email.clone();
            user.email = email.clone();
            state_write.registered_emails.remove(&old_email);
            state_write.registered_emails.insert(email);
        }

        UpdateResponseGql::success("11006")
    }

    // ========================================================================
    // updateUserPreferences
    // ========================================================================

    async fn update_user_preferences(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "userPreferences")] user_preferences: Option<UserPreferencesInput>,
    ) -> UserPreferencesResponseGql {
        let me = match ctx.data_opt::<CurrentUser>().and_then(|cu| cu.0) {
            Some(uid) => uid,
            None => {
                return UserPreferencesResponseGql {
                    status: "error".to_string(),
                    response_code: Some("60501".to_string()),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let prefs = state_write
            .preferences
            .entry(me)
            .or_insert_with(Default::default);

        if let Some(input) = user_preferences {
            if let Some(level) = input.content_filtering_severity_level {
                prefs.content_filtering_severity_level = match level {
                    ContentFilterType::Mygrandmalikes => ContentFilterState::Mygrandmalikes,
                    ContentFilterType::Mygrandmahates => ContentFilterState::Mygrandmahates,
                };
            }
            if let Some(onboardings) = input.shown_onboardings {
                for ob in onboardings {
                    let name = match ob {
                        OnboardingType::IntroOnboarding => "INTROONBOARDING".to_string(),
                    };
                    if !prefs.onboardings_were_shown.contains(&name) {
                        prefs.onboardings_were_shown.push(name);
                    }
                }
            }
        }

        let severity = match prefs.content_filtering_severity_level {
            ContentFilterState::Mygrandmalikes => "MYGRANDMALIKES",
            ContentFilterState::Mygrandmahates => "MYGRANDMAHATES",
        };

        UserPreferencesResponseGql {
            status: "success".to_string(),
            response_code: Some("11014".to_string()),
            affected_rows: Some(UserPreferencesPayloadGql {
                content_filtering_severity_level: Some(severity.to_string()),
            }),
        }
    }
}
