use async_graphql::{Context, ID, Object};

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
            .map(|r| state.chat_record_to_graphql(r, Some(user_id)))
            .collect();

        ListChatsResponse {
            meta: DefaultResponse::success("11801", "Chats retrieved"),
            affected_rows: Some(chats),
        }
    }

    /// List messages for a single chat, optionally filtered by an exclusive
    /// `since` timestamp (RFC3339). Used by the client polling transport.
    async fn list_chat_messages(
        &self,
        ctx: &Context<'_>,
        chatid: ID,
        since: Option<String>,
    ) -> ListChatMessagesResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return ListChatMessagesResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        let chat_uuid = match chatid.to_string().parse::<uuid::Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return ListChatMessagesResponse {
                    meta: DefaultResponse::error("30303", "Invalid chat UUID"),
                    affected_rows: None,
                };
            }
        };

        let state = ctx.data_unchecked::<SharedState>().read().await;

        let chat = match state.chats.iter().find(|c| c.id == chat_uuid) {
            Some(c) => c,
            None => {
                return ListChatMessagesResponse {
                    meta: DefaultResponse::error("30304", "Chat not found"),
                    affected_rows: None,
                };
            }
        };

        if !chat.participant_ids.contains(&user_id) {
            return ListChatMessagesResponse {
                meta: DefaultResponse::error("30305", "Not a participant in this chat"),
                affected_rows: None,
            };
        }

        let since_str = since.unwrap_or_default();

        let mut messages: Vec<ChatMessage> = state
            .chat_messages
            .iter()
            .filter(|m| m.chat_id == chat_uuid && m.created_at.as_str() > since_str.as_str())
            .map(|m| ChatMessage {
                id: m.id.to_string(),
                senderid: m.sender_id.to_string(),
                chatid: m.chat_id.to_string(),
                content: m.content.clone(),
                createdat: m.created_at.clone(),
            })
            .collect();

        messages.sort_by(|a, b| a.createdat.cmp(&b.createdat));

        ListChatMessagesResponse {
            meta: DefaultResponse::success("11805", "Messages retrieved"),
            affected_rows: Some(messages),
        }
    }
}
