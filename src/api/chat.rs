//! Chat API functions.
//!
//! Server functions for chat operations:
//! - List chats
//! - Send messages
//! - Create new chats (private and group)
//! - Poll for new chat messages (real-time transport v1)
//! - Mark a chat as read

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use serde::Serialize;

use crate::models::chat::{
    ChatMessage, CreateChatResponse, ListChatsResponse, SendMessageResponse,
};
#[cfg(feature = "ssr")]
use crate::models::chat::{ListChatMessagesResponse, MarkChatReadResponse};

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

/// Variables for the listChatMessages query.
#[cfg(feature = "ssr")]
#[derive(Debug, Serialize)]
struct ListChatMessagesVars {
    chatid: String,
    since: Option<String>,
}

/// Variables for the markChatRead mutation.
#[cfg(feature = "ssr")]
#[derive(Debug, Serialize)]
struct MarkChatReadVars {
    chatid: String,
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
    use crate::api::graphql::{LIST_CHATS_QUERY, ListChatsData, query};

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
    use crate::api::graphql::{SEND_CHAT_MESSAGE_MUTATION, SendChatMessageData, mutate};

    // Validate message length
    if content.is_empty() {
        return Err(ServerFnError::new("Message cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new(
            "Message must be 500 characters or fewer",
        ));
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
    use crate::api::graphql::{CREATE_CHAT_MUTATION, CreateChatData, mutate};

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

/// Fetch messages for a single chat, optionally filtered by an
/// exclusive `since` timestamp. Backs the polling transport.
///
/// # Arguments
///
/// * `chat_id` - The chat to fetch messages for
/// * `since` - If set, only messages with `createdat > since` are returned
#[server(ListChatMessages, "/api")]
pub async fn list_chat_messages(
    chat_id: String,
    since: Option<String>,
) -> Result<Vec<ChatMessage>, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{LIST_CHAT_MESSAGES_QUERY, ListChatMessagesData, query};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = ListChatMessagesVars {
        chatid: chat_id,
        since,
    };

    let data: ListChatMessagesData = query(LIST_CHAT_MESSAGES_QUERY, vars, Some(&token)).await?;

    let response: ListChatMessagesResponse = data.list_chat_messages;
    if !response.is_success() {
        return Err(ServerFnError::new(format!(
            "listChatMessages failed: {}",
            response.meta.response_message
        )));
    }

    Ok(response.messages())
}

/// Mark a chat as read for the current user at the server's clock.
///
/// Returns the timestamp recorded as the new last-read marker, if any.
#[server(MarkChatRead, "/api")]
pub async fn mark_chat_read(chat_id: String) -> Result<Option<String>, ServerFnError> {
    use crate::api::auth_fetch::get_access_token_from_cookies;
    use crate::api::graphql::{MARK_CHAT_READ_MUTATION, MarkChatReadData, mutate};

    let token = get_access_token_from_cookies()
        .await
        .map_err(|_| ServerFnError::new("Not authenticated"))?;

    let vars = MarkChatReadVars { chatid: chat_id };

    let data: MarkChatReadData = mutate(MARK_CHAT_READ_MUTATION, vars, Some(&token)).await?;

    let response: MarkChatReadResponse = data.mark_chat_read;
    if !response.is_success() {
        return Err(ServerFnError::new(format!(
            "markChatRead failed: {}",
            response.meta.response_message
        )));
    }

    Ok(response.last_read_at)
}
