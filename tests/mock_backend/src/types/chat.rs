use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

use super::registration::DefaultResponse;

// ============================================================================
// GraphQL Objects
// ============================================================================

/// A participant in a chat room.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ChatParticipant {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: Option<String>,
    pub hasaccess: Option<bool>,
}

/// A single chat message.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub senderid: String,
    pub chatid: String,
    pub content: String,
    pub createdat: String,
}

/// A chat room (private or group).
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct Chat {
    pub id: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub createdat: String,
    pub updatedat: String,
    pub chatmessages: Vec<ChatMessage>,
    pub chatparticipants: Vec<ChatParticipant>,
    /// Number of messages not yet seen by the viewer.
    pub unread_count: i32,
    /// Viewer's last-read timestamp for this chat (RFC3339), if any.
    pub last_read_at: Option<String>,
}

/// Response for `listChatMessages` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListChatMessagesResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<ChatMessage>>,
}

/// Response for `markChatRead` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct MarkChatReadResponse {
    pub meta: DefaultResponse,
    /// Timestamp recorded as the new last-read marker.
    pub last_read_at: Option<String>,
}

/// Response for `listChats` query.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct ListChatsResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<Vec<Chat>>,
}

/// Response for `sendChatMessage` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct SendMessageResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<ChatMessage>,
}

/// Response for `createChat` mutation.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateChatResponse {
    pub meta: DefaultResponse,
    #[graphql(name = "affectedRows")]
    pub affected_rows: Option<CreateChatResult>,
}

/// Created chat result containing the new chat ID.
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateChatResult {
    pub chatid: String,
}
