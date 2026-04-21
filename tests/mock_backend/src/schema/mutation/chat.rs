use async_graphql::{Context, ID, InputObject, Object};
use chrono::Utc;
use uuid::Uuid;

use crate::require_auth;
use crate::state::{ChatMessageRecord, ChatRecord, SharedState};
use crate::types::chat::*;
use crate::types::registration::DefaultResponse;

/// Input for createChat mutation.
#[derive(InputObject, Clone, Debug)]
pub struct CreateChatInput {
    pub name: String,
    pub recipients: Vec<String>,
    pub image: Option<String>,
}

#[derive(Default)]
pub struct ChatMutation;

#[Object]
impl ChatMutation {
    /// Create a new chat (private 1:1 or group).
    async fn create_chat(&self, ctx: &Context<'_>, input: CreateChatInput) -> CreateChatResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return CreateChatResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        if input.recipients.is_empty() {
            return CreateChatResponse {
                meta: DefaultResponse::error("30301", "At least one recipient is required"),
                affected_rows: None,
            };
        }

        let mut recipient_ids: Vec<Uuid> = Vec::new();
        for r in &input.recipients {
            match r.parse::<Uuid>() {
                Ok(u) => recipient_ids.push(u),
                Err(_) => {
                    return CreateChatResponse {
                        meta: DefaultResponse::error("30302", "Invalid recipient UUID"),
                        affected_rows: None,
                    };
                }
            }
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // For private chats (1 recipient): check if chat already exists
        if recipient_ids.len() == 1 {
            let other = recipient_ids[0];
            if let Some(existing) = state.chats.iter().find(|c| {
                c.name.is_none()
                    && c.participant_ids.len() == 2
                    && c.participant_ids.contains(&user_id)
                    && c.participant_ids.contains(&other)
            }) {
                return CreateChatResponse {
                    meta: DefaultResponse::success("11803", "Chat already exists"),
                    affected_rows: Some(CreateChatResult {
                        chatid: existing.id.to_string(),
                    }),
                };
            }
        }

        let chat_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        let mut participant_ids_all = vec![user_id];
        participant_ids_all.extend(recipient_ids);

        let is_group = participant_ids_all.len() > 2;
        let name = if is_group { Some(input.name) } else { None };
        let image = if is_group { input.image } else { None };

        let record = ChatRecord {
            id: chat_id,
            name,
            image,
            created_at: now.clone(),
            updated_at: now,
            participant_ids: participant_ids_all,
        };

        state.chats.push(record);

        CreateChatResponse {
            meta: DefaultResponse::success("11802", "Chat created"),
            affected_rows: Some(CreateChatResult {
                chatid: chat_id.to_string(),
            }),
        }
    }

    /// Send a message to a chat.
    async fn send_chat_message(
        &self,
        ctx: &Context<'_>,
        chatid: ID,
        content: String,
    ) -> SendMessageResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return SendMessageResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        if content.is_empty() {
            return SendMessageResponse {
                meta: DefaultResponse::error("30307", "Message cannot be empty"),
                affected_rows: None,
            };
        }
        if content.len() > 500 {
            return SendMessageResponse {
                meta: DefaultResponse::error("30306", "Message must be 500 characters or fewer"),
                affected_rows: None,
            };
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let chat_uuid = match chatid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return SendMessageResponse {
                    meta: DefaultResponse::error("30303", "Invalid chat UUID"),
                    affected_rows: None,
                };
            }
        };

        let chat = match state.chats.iter_mut().find(|c| c.id == chat_uuid) {
            Some(c) => c,
            None => {
                return SendMessageResponse {
                    meta: DefaultResponse::error("30304", "Chat not found"),
                    affected_rows: None,
                };
            }
        };

        if !chat.participant_ids.contains(&user_id) {
            return SendMessageResponse {
                meta: DefaultResponse::error("30305", "Not a participant in this chat"),
                affected_rows: None,
            };
        }

        let now = Utc::now().to_rfc3339();
        chat.updated_at = now.clone();

        let msg_id = Uuid::new_v4();
        let message_record = ChatMessageRecord {
            id: msg_id,
            sender_id: user_id,
            chat_id: chat_uuid,
            content: content.clone(),
            created_at: now.clone(),
        };

        state.chat_messages.push(message_record);

        let response_message = ChatMessage {
            id: msg_id.to_string(),
            senderid: user_id.to_string(),
            chatid: chat_uuid.to_string(),
            content,
            createdat: now,
        };

        SendMessageResponse {
            meta: DefaultResponse::success("11804", "Message sent"),
            affected_rows: Some(response_message),
        }
    }

    /// Mark a chat as read for the authenticated user at the current time.
    async fn mark_chat_read(&self, ctx: &Context<'_>, chatid: ID) -> MarkChatReadResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return MarkChatReadResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    last_read_at: None,
                };
            }
        };

        let chat_uuid = match chatid.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return MarkChatReadResponse {
                    meta: DefaultResponse::error("30303", "Invalid chat UUID"),
                    last_read_at: None,
                };
            }
        };

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        let chat = match state.chats.iter().find(|c| c.id == chat_uuid) {
            Some(c) => c,
            None => {
                return MarkChatReadResponse {
                    meta: DefaultResponse::error("30304", "Chat not found"),
                    last_read_at: None,
                };
            }
        };

        if !chat.participant_ids.contains(&user_id) {
            return MarkChatReadResponse {
                meta: DefaultResponse::error("30305", "Not a participant in this chat"),
                last_read_at: None,
            };
        }

        let now = Utc::now().to_rfc3339();
        state
            .chat_last_read_at
            .insert((user_id, chat_uuid), now.clone());

        MarkChatReadResponse {
            meta: DefaultResponse::success("11806", "Chat marked as read"),
            last_read_at: Some(now),
        }
    }
}
