//! Chat-related data models.
//!
//! These structs mirror the GraphQL types used for real-time chat:
//! - `Chat` (chat room)
//! - `ChatMessage` (individual message)
//! - `ChatParticipant` (user in a chat)
//! - `ChatType` (private vs group)

use serde::{Deserialize, Serialize};

use super::common::DefaultResponse;

/// Chat type enumeration (private 1:1 vs group).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChatType {
    #[default]
    Private,
    Group,
}

/// A participant in a chat room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatParticipant {
    pub userid: String,
    pub img: Option<String>,
    pub username: String,
    pub slug: Option<String>,
    pub hasaccess: Option<bool>,
}

impl ChatParticipant {
    /// Get avatar URL with fallback.
    pub fn avatar_url(&self) -> String {
        self.img
            .clone()
            .unwrap_or_else(|| "/svg/noname.svg".to_string())
    }
}

/// Delivery status for a chat message, used for optimistic rendering
/// and retry UX. Not serialised to/from the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MessageStatus {
    /// Confirmed by the server (default for polled messages).
    #[default]
    Sent,
    /// Optimistic send in flight.
    Sending,
    /// Send attempt failed; awaits user retry.
    Failed,
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    pub id: String,
    pub senderid: String,
    pub chatid: String,
    pub content: String,
    pub createdat: String,
    /// Client-side delivery status. Skipped during (de)serialisation.
    #[serde(skip, default)]
    pub status: MessageStatus,
}

impl ChatMessage {
    /// Decode HTML entities in message content.
    pub fn decoded_content(&self) -> String {
        html_escape::decode_html_entities(&self.content).to_string()
    }
}

/// A chat room (private or group).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub createdat: String,
    pub updatedat: String,
    #[serde(default)]
    pub chatmessages: Vec<ChatMessage>,
    #[serde(default)]
    pub chatparticipants: Vec<ChatParticipant>,
    /// Unread message count for the viewer (seeded by the server).
    #[serde(default, rename = "unreadCount")]
    pub unread_count: u32,
    /// Viewer's last-read timestamp on this chat, as recorded by the server.
    #[serde(default, rename = "lastReadAt")]
    pub last_read_at: Option<String>,
}

impl Chat {
    /// Determine if this is a private (1:1) or group chat.
    ///
    /// Private chats have no name and no image.
    /// Group chats have a name or image.
    pub fn chat_type(&self) -> ChatType {
        let has_name = self.name.as_ref().is_some_and(|n| !n.trim().is_empty());
        let has_image = self.image.is_some();

        if has_name || has_image {
            ChatType::Group
        } else {
            ChatType::Private
        }
    }

    /// Get the display name for this chat.
    ///
    /// For private chats: returns the other participant's username.
    /// For group chats: returns the group name.
    pub fn display_name(&self, current_user_id: &str) -> String {
        match self.chat_type() {
            ChatType::Private => self
                .chatparticipants
                .iter()
                .find(|p| p.userid != current_user_id)
                .map(|p| p.username.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            ChatType::Group => self.name.clone().unwrap_or_else(|| "Group".to_string()),
        }
    }

    /// Get the avatar URL for this chat.
    ///
    /// For private chats: returns the other participant's avatar.
    /// For group chats: returns the group image.
    pub fn avatar_url(&self, current_user_id: &str) -> String {
        match self.chat_type() {
            ChatType::Private => self
                .chatparticipants
                .iter()
                .find(|p| p.userid != current_user_id)
                .map(|p| p.avatar_url())
                .unwrap_or_else(|| "/svg/noname.svg".to_string()),
            ChatType::Group => self
                .image
                .clone()
                .unwrap_or_else(|| "/svg/noname.svg".to_string()),
        }
    }

    /// Get the last message in this chat, if any.
    pub fn last_message(&self) -> Option<&ChatMessage> {
        self.chatmessages.last()
    }

    /// Get a preview of the last message (truncated).
    pub fn message_preview(&self) -> String {
        self.last_message()
            .map(|m| {
                let decoded = m.decoded_content();
                if decoded.len() > 50 {
                    format!("{}...", &decoded[..47])
                } else {
                    decoded
                }
            })
            .unwrap_or_else(|| "Start chatting...".to_string())
    }
}

/// Response wrapper for listChats query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListChatsResponse {
    pub meta: DefaultResponse,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Vec<Chat>,
}

impl ListChatsResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Response wrapper for sendChatMessage mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub meta: DefaultResponse,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Option<ChatMessage>,
}

impl SendMessageResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

/// Response wrapper for createChat mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatResponse {
    pub meta: DefaultResponse,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Option<CreateChatResult>,
}

/// Created chat result containing the new chat ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatResult {
    pub chatid: String,
}

impl CreateChatResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }

    pub fn chat_id(&self) -> Option<&str> {
        self.affected_rows.as_ref().map(|r| r.chatid.as_str())
    }
}

/// Response wrapper for listChatMessages query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListChatMessagesResponse {
    pub meta: DefaultResponse,
    #[serde(default, rename = "affectedRows")]
    pub affected_rows: Option<Vec<ChatMessage>>,
}

impl ListChatMessagesResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }

    pub fn messages(self) -> Vec<ChatMessage> {
        self.affected_rows.unwrap_or_default()
    }
}

/// Response wrapper for markChatRead mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkChatReadResponse {
    pub meta: DefaultResponse,
    #[serde(default, rename = "lastReadAt")]
    pub last_read_at: Option<String>,
}

impl MarkChatReadResponse {
    pub fn is_success(&self) -> bool {
        self.meta.status == "success"
    }
}

// ============================================================================
// Time formatting utilities
// ============================================================================

/// Format a timestamp as relative time (e.g., "5m", "2h", "3d").
pub fn format_relative_time(timestamp: &str) -> String {
    use chrono::{DateTime, Utc};

    let parsed: Result<DateTime<Utc>, _> = timestamp.parse();
    match parsed {
        Ok(dt) => {
            let now = Utc::now();
            let duration = now.signed_duration_since(dt);

            if duration.num_minutes() < 1 {
                "now".to_string()
            } else if duration.num_hours() < 1 {
                format!("{}m", duration.num_minutes())
            } else if duration.num_days() < 1 {
                format!("{}h", duration.num_hours())
            } else if duration.num_days() < 7 {
                format!("{}d", duration.num_days())
            } else {
                format!("{}w", duration.num_weeks())
            }
        }
        Err(_) => "—".to_string(),
    }
}

/// Format a timestamp for display in chat (HH:MM format).
pub fn format_message_time(timestamp: &str) -> String {
    use chrono::{DateTime, Local, Utc};

    let parsed: Result<DateTime<Utc>, _> = timestamp.parse();
    match parsed {
        Ok(dt) => {
            let local: DateTime<Local> = dt.into();
            local.format("%H:%M").to_string()
        }
        Err(_) => "—".to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_type_detection() {
        let private_chat = Chat {
            id: "1".to_string(),
            name: None,
            image: None,
            createdat: "2024-01-01T00:00:00Z".to_string(),
            updatedat: "2024-01-01T00:00:00Z".to_string(),
            chatmessages: vec![],
            chatparticipants: vec![],
            unread_count: 0,
            last_read_at: None,
        };
        assert_eq!(private_chat.chat_type(), ChatType::Private);

        let group_chat = Chat {
            id: "2".to_string(),
            name: Some("Test Group".to_string()),
            image: None,
            createdat: "2024-01-01T00:00:00Z".to_string(),
            updatedat: "2024-01-01T00:00:00Z".to_string(),
            chatmessages: vec![],
            chatparticipants: vec![],
            unread_count: 0,
            last_read_at: None,
        };
        assert_eq!(group_chat.chat_type(), ChatType::Group);
    }

    #[test]
    fn test_display_name_private() {
        let chat = Chat {
            id: "1".to_string(),
            name: None,
            image: None,
            createdat: "2024-01-01T00:00:00Z".to_string(),
            updatedat: "2024-01-01T00:00:00Z".to_string(),
            chatmessages: vec![],
            chatparticipants: vec![
                ChatParticipant {
                    userid: "current".to_string(),
                    img: None,
                    username: "Me".to_string(),
                    slug: None,
                    hasaccess: Some(true),
                },
                ChatParticipant {
                    userid: "other".to_string(),
                    img: None,
                    username: "Friend".to_string(),
                    slug: None,
                    hasaccess: Some(true),
                },
            ],
            unread_count: 0,
            last_read_at: None,
        };

        assert_eq!(chat.display_name("current"), "Friend");
    }

    #[test]
    fn test_message_preview_truncation() {
        let long_message = "A".repeat(100);
        let chat = Chat {
            id: "1".to_string(),
            name: None,
            image: None,
            createdat: "2024-01-01T00:00:00Z".to_string(),
            updatedat: "2024-01-01T00:00:00Z".to_string(),
            chatmessages: vec![ChatMessage {
                id: "m1".to_string(),
                senderid: "user1".to_string(),
                chatid: "1".to_string(),
                content: long_message,
                createdat: "2024-01-01T00:00:00Z".to_string(),
                status: MessageStatus::Sent,
            }],
            chatparticipants: vec![],
            unread_count: 0,
            last_read_at: None,
        };

        let preview = chat.message_preview();
        assert!(preview.len() <= 50);
        assert!(preview.ends_with("..."));
    }
}
