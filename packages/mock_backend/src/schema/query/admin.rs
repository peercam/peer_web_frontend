use async_graphql::{Context, ID, Object, Result};

use crate::guards::require_admin;
use crate::state::SharedState;
use crate::types::admin::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct AdminQuery;

#[Object]
impl AdminQuery {
    /// Extended user search with admin-specific filter fields.
    #[graphql(guard = "require_admin()")]
    #[allow(clippy::too_many_arguments)]
    async fn list_users_admin_v2(
        &self,
        ctx: &Context<'_>,
        _content_filter_by: Option<String>,
        userid: Option<ID>,
        email: Option<String>,
        username: Option<String>,
        status: Option<i32>,
        verified: Option<i32>,
        ip: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<AdminUserListResponse> {
        let state = ctx.data::<SharedState>()?.read().await;

        // Validate: userid and username cannot both be present
        if userid.is_some() && username.is_some() {
            return Ok(AdminUserListResponse {
                meta: DefaultResponse::error(
                    "31012",
                    "userid and username cannot be used together",
                ),
                status: "error".into(),
                counter: 0,
                response_code: Some("31012".into()),
                affected_rows: None,
            });
        }

        // Validate UUID format if provided
        if let Some(ref uid) = userid
            && uuid::Uuid::parse_str(uid.as_ref()).is_err()
        {
            return Ok(AdminUserListResponse {
                meta: DefaultResponse::error("30201", "Invalid UUID format"),
                status: "error".into(),
                counter: 0,
                response_code: Some("30201".into()),
                affected_rows: None,
            });
        }

        // Validate IP format if provided
        if let Some(ref ip_str) = ip
            && ip_str.parse::<std::net::IpAddr>().is_err()
        {
            return Ok(AdminUserListResponse {
                meta: DefaultResponse::error("30257", "Invalid IP address"),
                status: "error".into(),
                counter: 0,
                response_code: Some("30257".into()),
                affected_rows: None,
            });
        }

        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(10).clamp(1, 20) as usize;

        let results = state.search_users_admin(
            userid.as_ref(),
            email.as_deref(),
            username.as_deref(),
            status,
            verified,
            ip.as_deref(),
            offset,
            limit,
        );

        if results.is_empty() {
            return Ok(AdminUserListResponse {
                meta: DefaultResponse::error("31007", "No users found"),
                status: "error".into(),
                counter: 0,
                response_code: Some("31007".into()),
                affected_rows: None,
            });
        }

        let counter = results.len() as i32;
        Ok(AdminUserListResponse {
            meta: DefaultResponse::success("11009", "User data prepared"),
            status: "success".into(),
            counter,
            response_code: Some("11009".into()),
            affected_rows: Some(results),
        })
    }

    /// List all follow relationships across the platform.
    #[graphql(guard = "require_admin()")]
    async fn allfriends(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<AllUserFriends> {
        let state = ctx.data::<SharedState>()?.read().await;
        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(20).clamp(1, 100) as usize;

        let all_follows: Vec<AllUserInfo> = state
            .follows
            .iter()
            .map(|(follower_id, followed_id)| {
                let follower_name = state.get_username(*follower_id);
                let followed_name = state.get_username(*followed_id);
                AllUserInfo {
                    followerid: Some(follower_id.to_string().into()),
                    followername: Some(follower_name),
                    followedid: Some(followed_id.to_string().into()),
                    followedname: Some(followed_name),
                }
            })
            .collect();

        let total = all_follows.len();
        let page: Vec<_> = all_follows.into_iter().skip(offset).take(limit).collect();

        Ok(AllUserFriends {
            meta: DefaultResponse::success("11101", "Friends retrieved"),
            status: "success".into(),
            counter: total as i32,
            response_code: Some("11101".into()),
            affected_rows: Some(page),
        })
    }

    /// Admin view of all comments on a post (with subcomments).
    #[graphql(guard = "require_admin()")]
    async fn postcomments(
        &self,
        ctx: &Context<'_>,
        postid: ID,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<PostCommentsResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(10).clamp(1, 20) as usize;

        let comments = state.get_admin_post_comments(postid.as_ref(), offset, limit);

        Ok(PostCommentsResponse {
            meta: DefaultResponse::success("11101", "Comments retrieved"),
            status: "success".into(),
            counter: comments.len() as i32,
            response_code: Some("11101".into()),
            affected_rows: Some(comments),
        })
    }

    /// Generate a leaderboard CSV (mock: returns a fake download link).
    #[graphql(guard = "require_admin()")]
    async fn generate_leaderboard(
        &self,
        _ctx: &Context<'_>,
        leaderboard_params: LeaderboardParamsInput,
    ) -> Result<LeaderboardResponse> {
        let start_ok =
            chrono::NaiveDate::parse_from_str(&leaderboard_params.start_date, "%Y-%m-%d");
        let end_ok = chrono::NaiveDate::parse_from_str(&leaderboard_params.end_date, "%Y-%m-%d");

        match (start_ok, end_ok) {
            (Ok(start), Ok(end)) => {
                if end < start {
                    return Ok(LeaderboardResponse {
                        meta: DefaultResponse::error("33002", "Invalid date range"),
                        leaderboard_result_link: None,
                    });
                }

                let n = leaderboard_params.leaderboard_users_count;
                let link = format!(
                    "runtime-data/media/other/power_power_contest_leaderboards_data/leaderboard_{}_{}_top{}.csv",
                    leaderboard_params.start_date.replace('-', ""),
                    leaderboard_params.end_date.replace('-', ""),
                    n
                );

                Ok(LeaderboardResponse {
                    meta: DefaultResponse::success("12301", "Leaderboard generated"),
                    leaderboard_result_link: Some(link),
                })
            }
            _ => Ok(LeaderboardResponse {
                meta: DefaultResponse::error("30301", "Invalid parameters"),
                leaderboard_result_link: None,
            }),
        }
    }
}
