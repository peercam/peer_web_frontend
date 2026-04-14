use async_graphql::{Context, ID, Object};
use uuid::Uuid;

use crate::schema::mutation::auth::get_current_user;
use crate::state::SharedState;
use crate::types::post::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct PostQuery;

/// Sort filtered post records in place.
fn sort_posts(
    posts: &mut [&crate::state::PostRecord],
    sort_by: PostSortType,
    state: &crate::state::MockState,
) {
    match sort_by {
        PostSortType::Newest
        | PostSortType::ForMe
        | PostSortType::Relevant
        | PostSortType::Follower
        | PostSortType::Followed
        | PostSortType::Friends => {
            posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        PostSortType::Oldest => {
            posts.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        }
        PostSortType::Likes => {
            posts.sort_by(|a, b| {
                let a_likes = state.post_likes.iter().filter(|(_, p)| *p == a.id).count();
                let b_likes = state.post_likes.iter().filter(|(_, p)| *p == b.id).count();
                b_likes.cmp(&a_likes)
            });
        }
        PostSortType::Dislikes => {
            posts.sort_by(|a, b| {
                let a_dl = state
                    .post_dislikes
                    .iter()
                    .filter(|(_, p)| *p == a.id)
                    .count();
                let b_dl = state
                    .post_dislikes
                    .iter()
                    .filter(|(_, p)| *p == b.id)
                    .count();
                b_dl.cmp(&a_dl)
            });
        }
        PostSortType::Views => {
            posts.sort_by(|a, b| {
                let a_v = state.post_views.iter().filter(|(_, p)| *p == a.id).count();
                let b_v = state.post_views.iter().filter(|(_, p)| *p == b.id).count();
                b_v.cmp(&a_v)
            });
        }
        PostSortType::Comments => {
            posts.sort_by(|a, b| {
                let a_c = state
                    .comments
                    .iter()
                    .filter(|c| c.post_id == a.id && c.visibility_status == "VISIBLE")
                    .count();
                let b_c = state
                    .comments
                    .iter()
                    .filter(|c| c.post_id == b.id && c.visibility_status == "VISIBLE")
                    .count();
                b_c.cmp(&a_c)
            });
        }
        PostSortType::Trending => {
            posts.sort_by(|a, b| {
                let score = |id: Uuid| -> usize {
                    let likes = state.post_likes.iter().filter(|(_, p)| *p == id).count();
                    let views = state.post_views.iter().filter(|(_, p)| *p == id).count();
                    let comments = state
                        .comments
                        .iter()
                        .filter(|c| c.post_id == id && c.visibility_status == "VISIBLE")
                        .count();
                    likes * 2 + views + comments
                };
                score(b.id).cmp(&score(a.id))
            });
        }
    }
}

/// Apply offset/limit pagination. Returns (paginated_slice, total_count).
fn paginate<T>(items: &[T], offset: i32, limit: i32) -> (&[T], i32) {
    let total = items.len() as i32;
    let offset = offset.max(0) as usize;
    let limit = limit.clamp(1, 20) as usize;

    if offset >= items.len() {
        return (&[], total);
    }

    let end = (offset + limit).min(items.len());
    (&items[offset..end], total)
}

#[Object]
impl PostQuery {
    /// Paginated, filtered, sorted list of posts.
    #[allow(clippy::too_many_arguments)]
    async fn list_posts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "filterBy")] filter_by: Option<Vec<PostFilterType>>,
        #[graphql(name = "contentFilterBy")] content_filter_by: Option<ContentFilterType>,
        #[graphql(name = "IgnorList")] ignor_list: Option<IgnoreOption>,
        #[graphql(name = "sortBy")] sort_by: Option<PostSortType>,
        userid: Option<ID>,
        postid: Option<ID>,
        title: Option<String>,
        tag: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
        #[graphql(name = "commentOffset")] _comment_offset: Option<i32>,
        #[graphql(name = "commentLimit")] _comment_limit: Option<i32>,
    ) -> PostListResponse {
        let viewer_id = match get_current_user(ctx) {
            Some(uid) => uid,
            None => {
                return PostListResponse {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let userid_parsed = userid.and_then(|id| Uuid::parse_str(id.as_str()).ok());
        let postid_parsed = postid
            .as_ref()
            .and_then(|id| Uuid::parse_str(id.as_str()).ok());

        if postid.is_some() && postid_parsed.is_none() {
            return PostListResponse {
                meta: DefaultResponse::error("30209", "Invalid post UUID"),
                counter: 0,
                affected_rows: None,
            };
        }

        let filters = filter_by.unwrap_or_default();
        let sort = sort_by.unwrap_or(PostSortType::Newest);
        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(10);

        let mut filtered = state_read.filter_posts(
            Some(viewer_id),
            &filters,
            content_filter_by,
            ignor_list,
            userid_parsed,
            postid_parsed,
            title.as_deref(),
            tag.as_deref(),
        );

        sort_posts(&mut filtered, sort, &state_read);

        let graphql_posts: Vec<Post> = filtered
            .iter()
            .map(|r| state_read.post_record_to_graphql(r, Some(viewer_id)))
            .collect();

        let (page, total) = paginate(&graphql_posts, off, lim);

        if page.is_empty() {
            PostListResponse {
                meta: DefaultResponse::success("21518", "No posts found"),
                counter: total,
                affected_rows: Some(vec![]),
            }
        } else {
            PostListResponse {
                meta: DefaultResponse::success("11501", "Posts retrieved successfully"),
                counter: total,
                affected_rows: Some(page.to_vec()),
            }
        }
    }

    /// Get a single post for guest viewing (no auth required).
    async fn guest_list_post(&self, ctx: &Context<'_>, postid: ID) -> PostListResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let post_uuid = match Uuid::parse_str(postid.as_str()) {
            Ok(id) => id,
            Err(_) => {
                return PostListResponse {
                    meta: DefaultResponse::error("30209", "Invalid post UUID"),
                    counter: 0,
                    affected_rows: None,
                };
            }
        };

        match state_read
            .posts
            .iter()
            .find(|p| p.id == post_uuid && p.visibility_status == "VISIBLE")
        {
            Some(record) => {
                let post = state_read.post_record_to_graphql(record, None);
                PostListResponse {
                    meta: DefaultResponse::success("11501", "Post retrieved successfully"),
                    counter: 1,
                    affected_rows: Some(vec![post]),
                }
            }
            None => PostListResponse {
                meta: DefaultResponse::error("31510", "Post not found"),
                counter: 0,
                affected_rows: None,
            },
        }
    }

    /// Check post eligibility and obtain upload token.
    async fn post_eligibility(&self, ctx: &Context<'_>) -> PostEligibilityResponse {
        let viewer_id = match get_current_user(ctx) {
            Some(uid) => uid,
            None => {
                return PostEligibilityResponse {
                    meta: DefaultResponse::error("60501", "Authentication required"),
                    eligibility_token: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        let token = format!(
            "mock-eligibility-{}-{}",
            viewer_id,
            chrono::Utc::now().timestamp_millis()
        );

        state_write
            .eligibility_tokens
            .insert(token.clone(), viewer_id);
        state_write
            .eligibility_token_status
            .insert(token.clone(), "ISSUED".into());

        PostEligibilityResponse {
            meta: DefaultResponse::success("10901", "Eligibility token issued"),
            eligibility_token: Some(token),
        }
    }

    /// Search tags by name (case-insensitive substring match).
    async fn search_tags(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "tagName")] tag_name: String,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> TagSearchResponse {
        if get_current_user(ctx).is_none() {
            return TagSearchResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: None,
            };
        }

        if tag_name.is_empty() {
            return TagSearchResponse {
                meta: DefaultResponse::error("30101", "Missing tagName"),
                counter: 0,
                affected_rows: None,
            };
        }

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let query_lower = tag_name.to_lowercase();
        let matching: Vec<Tag> = state_read
            .tags
            .iter()
            .filter(|t| t.to_lowercase().contains(&query_lower))
            .map(|t| Tag { name: t.clone() })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(20);
        let (page, total) = paginate(&matching, off, lim);

        if page.is_empty() {
            TagSearchResponse {
                meta: DefaultResponse::success("21701", "No tags found"),
                counter: total,
                affected_rows: Some(vec![]),
            }
        } else {
            TagSearchResponse {
                meta: DefaultResponse::success("11701", "Tags retrieved successfully"),
                counter: total,
                affected_rows: Some(page.to_vec()),
            }
        }
    }

    /// List all tags with pagination.
    async fn list_tags(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> TagSearchResponse {
        if get_current_user(ctx).is_none() {
            return TagSearchResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                counter: 0,
                affected_rows: None,
            };
        }

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let all_tags: Vec<Tag> = state_read
            .tags
            .iter()
            .map(|t| Tag { name: t.clone() })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(20);
        let (page, total) = paginate(&all_tags, off, lim);

        if page.is_empty() {
            TagSearchResponse {
                meta: DefaultResponse::success("21701", "No tags found"),
                counter: total,
                affected_rows: Some(vec![]),
            }
        } else {
            TagSearchResponse {
                meta: DefaultResponse::success("11701", "Tags loaded"),
                counter: total,
                affected_rows: Some(page.to_vec()),
            }
        }
    }

    /// List advertisement posts with optional filters.
    async fn list_advertisement_posts(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
        #[graphql(name = "contentFilterBy")] _content_filter_by: Option<ContentFilterType>,
        title: Option<String>,
        tag: Option<String>,
    ) -> AdListResponse {
        let viewer_id = get_current_user(ctx);

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let ad_posts: Vec<AdvertisementPost> = state_read
            .advertisements
            .iter()
            .filter_map(|ad| {
                let record = state_read.posts.iter().find(|p| p.id == ad.post_id)?;

                if let Some(ref t) = title
                    && !t.is_empty()
                    && !record.title.to_lowercase().contains(&t.to_lowercase())
                {
                    return None;
                }
                if let Some(ref tg) = tag
                    && !tg.is_empty()
                {
                    let tg_lower = tg.to_lowercase();
                    if !record.tags.iter().any(|pt| pt.to_lowercase() == tg_lower) {
                        return None;
                    }
                }

                let post = state_read.post_record_to_graphql(record, viewer_id);
                Some(AdvertisementPost {
                    post,
                    advertisement: AdvertisementInfo {
                        advertisementid: ad.id.clone(),
                        advertisementtype: ad.advertisement_type.clone(),
                        startdate: ad.start_date.clone(),
                        enddate: ad.end_date.clone(),
                    },
                })
            })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(10);
        let (page, total) = paginate(&ad_posts, off, lim);

        AdListResponse {
            meta: DefaultResponse::success("11501", "Advertisement posts retrieved"),
            counter: total,
            affected_rows: Some(page.to_vec()),
        }
    }

    /// List users who performed a specific interaction on a post.
    async fn post_interactions(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "getOnly")] get_only: GetOnly,
        #[graphql(name = "postOrCommentId")] post_or_comment_id: ID,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> PostInteractionResponse {
        if get_current_user(ctx).is_none() {
            return PostInteractionResponse {
                meta: DefaultResponse::error("60501", "Authentication required"),
                affected_rows: None,
            };
        }

        let post_uuid = match Uuid::parse_str(post_or_comment_id.as_str()) {
            Ok(id) => id,
            Err(_) => {
                return PostInteractionResponse {
                    meta: DefaultResponse::error("30201", "Invalid UUID format"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        let user_ids: Vec<Uuid> = match get_only {
            GetOnly::Like => state_read
                .post_likes
                .iter()
                .filter(|(_, pid)| *pid == post_uuid)
                .map(|(uid, _)| *uid)
                .collect(),
            GetOnly::Dislike => state_read
                .post_dislikes
                .iter()
                .filter(|(_, pid)| *pid == post_uuid)
                .map(|(uid, _)| *uid)
                .collect(),
            GetOnly::View => state_read
                .post_views
                .iter()
                .filter(|(_, pid)| *pid == post_uuid)
                .map(|(uid, _)| *uid)
                .collect(),
            GetOnly::Commentlike => vec![],
        };

        let users: Vec<PostUser> = user_ids
            .iter()
            .filter_map(|uid| state_read.users.get(uid))
            .map(|u| PostUser {
                id: u.uid.to_string().into(),
                username: u.username.clone(),
                slug: u.slug.clone(),
                img: u.img.clone(),
                isfollowed: false,
                isfollowing: false,
                isfriend: false,
            })
            .collect();

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(20);
        let (page, _) = paginate(&users, off, lim);

        PostInteractionResponse {
            meta: DefaultResponse::success("11205", "Interactions retrieved successfully"),
            affected_rows: Some(page.to_vec()),
        }
    }
}
