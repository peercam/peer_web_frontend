use async_graphql::{Context, Object, Result};

use crate::guards::require_moderator;
use crate::state::SharedState;
use crate::types::moderation::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct ModerationQuery;

#[Object]
impl ModerationQuery {
    /// Get moderation dashboard statistics.
    #[graphql(guard = "require_moderator()")]
    async fn moderation_stats(&self, ctx: &Context<'_>) -> Result<ModerationStatsResponse> {
        let state = ctx.data::<SharedState>()?.read().await;

        let mut awaiting = 0;
        let mut hidden = 0;
        let mut restored = 0;
        let mut illegal = 0;

        for ticket in &state.moderation_tickets {
            match ticket.status.as_str() {
                "waiting_for_review" => awaiting += 1,
                "hidden" => hidden += 1,
                "restored" => restored += 1,
                "illegal" => illegal += 1,
                _ => {}
            }
        }

        Ok(ModerationStatsResponse {
            status: "success".into(),
            response_code: Some("12101".into()),
            meta: DefaultResponse::success("12101", "Stats retrieved"),
            affected_rows: Some(ModerationStats {
                amount_awaiting_review: awaiting,
                amount_hidden: hidden,
                amount_restored: restored,
                amount_illegal: illegal,
            }),
        })
    }

    /// List moderation tickets with optional filters.
    #[graphql(guard = "require_moderator()")]
    async fn moderation_items(
        &self,
        ctx: &Context<'_>,
        status: Option<ModerationStatus>,
        content_type: Option<ModerationContentType>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<ModerationItemListResponse> {
        let state = ctx.data::<SharedState>()?.read().await;
        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(10).clamp(1, 20) as usize;

        let mut tickets: Vec<_> = state.moderation_tickets.iter().collect();

        // Filter by status
        if let Some(s) = &status {
            let s_str = s.as_str();
            tickets.retain(|t| t.status == s_str);
        }

        // Filter by content type
        if let Some(ct) = &content_type {
            let ct_str = match ct {
                ModerationContentType::Post => "post",
                ModerationContentType::Comment => "comment",
                ModerationContentType::User => "user",
            };
            tickets.retain(|t| t.target_type == ct_str);
        }

        // Paginate
        let page: Vec<_> = tickets.into_iter().skip(offset).take(limit).collect();

        // Resolve each ticket to ModerationItem
        let items = page
            .iter()
            .map(|ticket| state.resolve_moderation_item(ticket))
            .collect();

        Ok(ModerationItemListResponse {
            status: "success".into(),
            response_code: Some("12102".into()),
            meta: DefaultResponse::success("12102", "Items retrieved"),
            affected_rows: items,
        })
    }
}
