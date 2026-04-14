use async_graphql::{Context, Object};

use crate::require_auth;
use crate::state::SharedState;
use crate::types::chat::*;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct ChatQuery;

#[Object]
impl ChatQuery {
    /// List chats for the authenticated user.
    async fn list_chats(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> ListChatsResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ListChatsResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let off = offset.unwrap_or(0).max(0) as usize;
        let lim = limit.unwrap_or(20).clamp(1, 50) as usize;

        let mut user_chats: Vec<_> = state
            .chats
            .iter()
            .filter(|c| c.participant_ids.contains(&user_id))
            .collect();

        user_chats.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        if user_chats.is_empty() {
            return ListChatsResponse {
                meta: DefaultResponse::success("21801", "No chats found"),
                affected_rows: None,
            };
        }

        let end = (off + lim).min(user_chats.len());
        let page = if off < user_chats.len() {
            &user_chats[off..end]
        } else {
            &[]
        };

        let chats: Vec<Chat> = page
            .iter()
            .map(|r| state.chat_record_to_graphql(r))
            .collect();

        ListChatsResponse {
            meta: DefaultResponse::success("11801", "Chats retrieved"),
            affected_rows: Some(chats),
        }
    }
}
