//! Chat API functions.
//!
//! Server functions for chat operations:
//! - List chats
//! - Send messages
//! - Create new chats (private and group)

use leptos::prelude::*;
use serde::Serialize;

use crate::models::chat::{ChatMessage, CreateChatResponse, ListChatsResponse, SendMessageResponse};

/// Variables for the listChats query.
#[cfg(feature = "ssr")]
#[derive(Debug, Serialize)]
struct ListChatsVars {
    limit: Option<i32>,
    offset: Option<i32>,
}

/// Variables for the sendChatMessage mutation.
#[cfg(feature = "ssr")]
#[derive(Debug, Serialize)]
struct SendMessageVars {
    chatid: String,
    content: String,
}

/// Variables for the createChat mutation.
#[cfg(feature = "ssr")]
#[derive(Debug, Serialize)]
struct CreateChatVars {
    name: String,
    recipients: Vec<String>,
    image: Option<String>,
}

/// Fetch the user's chat list.
///
/// # Arguments
///
/// * `limit` - Maximum number of chats to return
/// * `offset` - Pagination offset
#[server(ListChats, "/api")]
pub async fn list_chats(
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<ListChatsResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{query, ListChatsData, LIST_CHATS_QUERY};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ListChatsVars { limit, offset };

    let data: ListChatsData = query(LIST_CHATS_QUERY, vars, Some(&token)).await?;

    Ok(data.list_chats)
}

/// Send a message to a chat.
///
/// # Arguments
///
/// * `chatid` - The chat ID to send to
/// * `content` - The message content (max 500 characters)
#[server(SendChatMessage, "/api")]
pub async fn send_chat_message(
    chatid: String,
    content: String,
) -> Result<SendMessageResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, SendChatMessageData, SEND_CHAT_MESSAGE_MUTATION};

    // Validate message length
    if content.is_empty() {
        return Err(ServerFnError::new("Message cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Message must be 500 characters or fewer"));
    }

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = SendMessageVars { chatid, content };

    let data: SendChatMessageData = mutate(SEND_CHAT_MESSAGE_MUTATION, vars, Some(&token)).await?;

    Ok(data.send_chat_message)
}

/// Create a new chat (private or group).
///
/// # Arguments
///
/// * `name` - Chat name (username for private, group name for groups)
/// * `recipients` - List of user IDs to add to the chat
/// * `image` - Optional base64 image for group chats
#[server(CreateChat, "/api")]
pub async fn create_chat(
    name: String,
    recipients: Vec<String>,
    image: Option<String>,
) -> Result<CreateChatResponse, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{mutate, CreateChatData, CREATE_CHAT_MUTATION};

    if recipients.is_empty() {
        return Err(ServerFnError::new("At least one recipient is required"));
    }

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = CreateChatVars {
        name,
        recipients,
        image,
    };

    let data: CreateChatData = mutate(CREATE_CHAT_MUTATION, vars, Some(&token)).await?;

    Ok(data.create_chat)
}

/// Refresh chat messages (polling strategy for real-time updates).
///
/// Fetches the latest messages for a specific chat.
#[server(RefreshChatMessages, "/api")]
pub async fn refresh_chat_messages(
    chat_id: String,
) -> Result<Vec<ChatMessage>, ServerFnError> {
    // For now, we refetch the full chat list and extract messages
    // In a production app, you'd have a dedicated endpoint for this
    let response = list_chats(Some(50), Some(0)).await?;

    let chat = response
        .affected_rows
        .into_iter()
        .find(|c| c.id == chat_id);

    match chat {
        Some(c) => Ok(c.chatmessages),
        None => Ok(vec![]),
    }
}
