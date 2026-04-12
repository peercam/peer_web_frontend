//! Chat state and context provider.
//!
//! Provides `ChatContext` for managing chat state across components:
//! - Active chat selection
//! - Chat list filtering (private/group)
//! - Contact selection for new chats
//! - Loading and sending states

use leptos::prelude::*;

use crate::api::chat::{create_chat, list_chats, send_chat_message};
use crate::api::profile::list_friends;
use crate::models::chat::{Chat, ChatMessage, ChatType};
use crate::models::profile::BasicUserInfo;

/// Global chat context available to all chat components.
#[derive(Clone, Copy)]
pub struct ChatContext {
    /// Current tab filter (private or group).
    pub filter_type: RwSignal<ChatType>,

    /// List of all chats.
    pub chats: RwSignal<Vec<Chat>>,

    /// Currently selected/active chat.
    pub active_chat: RwSignal<Option<Chat>>,

    /// Messages for the active chat (may be updated in real-time).
    pub messages: RwSignal<Vec<ChatMessage>>,

    /// Friends list for starting new chats.
    pub friends: RwSignal<Vec<BasicUserInfo>>,

    /// Selected users for group chat creation.
    pub selected_users: RwSignal<Vec<BasicUserInfo>>,

    /// Whether the contacts overlay is open.
    pub is_create_overlay_open: RwSignal<bool>,

    /// Whether we're on the group review screen (step 2 of group creation).
    pub is_review_screen: RwSignal<bool>,

    /// Loading state for chat list.
    pub is_loading_chats: RwSignal<bool>,

    /// Loading state for friends list.
    pub is_loading_friends: RwSignal<bool>,

    /// Loading state for sending a message.
    pub is_sending: RwSignal<bool>,

    /// Error message to display.
    pub error: RwSignal<Option<String>>,

    /// Current user ID (from auth).
    pub current_user_id: RwSignal<Option<String>>,

    /// Current user avatar (from auth).
    pub current_user_img: RwSignal<Option<String>>,

    /// Group name for creation.
    pub group_name: RwSignal<String>,

    /// Group image (base64) for creation.
    pub group_image: RwSignal<Option<String>>,
}

impl ChatContext {
    /// Filter chats by the current filter type.
    pub fn filtered_chats(&self) -> Vec<Chat> {
        let filter = self.filter_type.get();
        self.chats
            .get()
            .into_iter()
            .filter(|chat| chat.chat_type() == filter)
            .collect()
    }

    /// Check if a chat is currently active.
    pub fn is_active(&self, chat_id: &str) -> bool {
        self.active_chat
            .get()
            .map(|c| c.id == chat_id)
            .unwrap_or(false)
    }

    /// Check if a user is selected for group creation.
    pub fn is_user_selected(&self, user_id: &str) -> bool {
        self.selected_users
            .get()
            .iter()
            .any(|u| u.userid == user_id)
    }

    /// Toggle user selection for group creation.
    pub fn toggle_user_selection(&self, user: BasicUserInfo) {
        let mut selected = self.selected_users.get();
        if let Some(pos) = selected.iter().position(|u| u.userid == user.userid) {
            selected.remove(pos);
        } else {
            selected.push(user);
        }
        self.selected_users.set(selected);
    }

    /// Clear all selections and close overlay.
    pub fn close_overlay(&self) {
        self.is_create_overlay_open.set(false);
        self.is_review_screen.set(false);
        self.selected_users.set(vec![]);
        self.group_name.set(String::new());
        self.group_image.set(None);
    }

    /// Get the current user ID or empty string.
    pub fn user_id_or_default(&self) -> String {
        self.current_user_id.get().unwrap_or_default()
    }
}

/// Provide the chat context at the page level.
///
/// Call this in the ChatPage component before rendering children.
pub fn provide_chat_context() -> ChatContext {
    let ctx = ChatContext {
        filter_type: RwSignal::new(ChatType::Private),
        chats: RwSignal::new(vec![]),
        active_chat: RwSignal::new(None),
        messages: RwSignal::new(vec![]),
        friends: RwSignal::new(vec![]),
        selected_users: RwSignal::new(vec![]),
        is_create_overlay_open: RwSignal::new(false),
        is_review_screen: RwSignal::new(false),
        is_loading_chats: RwSignal::new(true),
        is_loading_friends: RwSignal::new(false),
        is_sending: RwSignal::new(false),
        error: RwSignal::new(None),
        current_user_id: RwSignal::new(None),
        current_user_img: RwSignal::new(None),
        group_name: RwSignal::new(String::new()),
        group_image: RwSignal::new(None),
    };

    provide_context(ctx);
    ctx
}

/// Get the chat context from the nearest provider.
pub fn use_chat() -> ChatContext {
    expect_context::<ChatContext>()
}

/// Load the chat list from the API.
pub async fn load_chats(ctx: ChatContext) {
    ctx.is_loading_chats.set(true);
    ctx.error.set(None);

    match list_chats(Some(50), Some(0)).await {
        Ok(response) => {
            if response.is_success() {
                ctx.chats.set(response.affected_rows);
            } else {
                ctx.error.set(Some("Failed to load chats".to_string()));
            }
        }
        Err(e) => {
            ctx.error.set(Some(format!("Error loading chats: {}", e)));
        }
    }

    ctx.is_loading_chats.set(false);
}

/// Load the friends list from the API.
pub async fn load_friends(ctx: ChatContext) {
    ctx.is_loading_friends.set(true);

    match list_friends(None, 0, 100).await {
        Ok(response) => {
            if response.is_success() {
                ctx.friends.set(response.affected_rows);
            }
        }
        Err(_) => {
            // Silently fail, friends list is optional
        }
    }

    ctx.is_loading_friends.set(false);
}

/// Send a message to the active chat.
pub async fn send_message(ctx: ChatContext, content: String) -> Result<(), String> {
    let chat_id = ctx
        .active_chat
        .get()
        .map(|c| c.id.clone())
        .ok_or_else(|| "No chat selected".to_string())?;

    if content.is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    if content.len() > 500 {
        return Err("Message must be 500 characters or fewer".to_string());
    }

    ctx.is_sending.set(true);
    ctx.error.set(None);

    let result = send_chat_message(chat_id.clone(), content.clone()).await;

    ctx.is_sending.set(false);

    match result {
        Ok(response) => {
            if response.is_success() {
                // Optimistically add the message to the list
                if let Some(new_msg) = response.affected_rows {
                    let mut messages = ctx.messages.get();
                    messages.push(new_msg.clone());
                    ctx.messages.set(messages);

                    // Also update the chat's last message
                    if let Some(mut chat) = ctx.active_chat.get() {
                        chat.chatmessages.push(new_msg);
                        ctx.active_chat.set(Some(chat.clone()));

                        // Update in the chats list too
                        let mut chats = ctx.chats.get();
                        if let Some(pos) = chats.iter().position(|c| c.id == chat_id) {
                            chats[pos] = chat;
                        }
                        ctx.chats.set(chats);
                    }
                }
                Ok(())
            } else {
                Err("Failed to send message".to_string())
            }
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

/// Start a new private chat with a user.
pub async fn start_private_chat(ctx: ChatContext, user: BasicUserInfo) -> Result<String, String> {
    ctx.is_sending.set(true);

    let result = create_chat(user.username.clone(), vec![user.userid.clone()], None).await;

    ctx.is_sending.set(false);

    match result {
        Ok(response) => {
            if response.is_success() {
                // Close the overlay
                ctx.close_overlay();

                // Refresh the chat list
                load_chats(ctx).await;

                // Return the new chat ID
                response
                    .chat_id()
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Chat created but ID not returned".to_string())
            } else {
                Err("Failed to create chat".to_string())
            }
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

/// Create a new group chat with selected users.
pub async fn create_group_chat(ctx: ChatContext) -> Result<String, String> {
    let name = ctx.group_name.get();
    let image = ctx.group_image.get();
    let recipients: Vec<String> = ctx
        .selected_users
        .get()
        .iter()
        .map(|u| u.userid.clone())
        .collect();

    if name.trim().is_empty() {
        return Err("Group name is required".to_string());
    }

    if recipients.is_empty() {
        return Err("Select at least one member".to_string());
    }

    ctx.is_sending.set(true);

    let result = create_chat(name, recipients, image).await;

    ctx.is_sending.set(false);

    match result {
        Ok(response) => {
            if response.is_success() {
                // Close the overlay
                ctx.close_overlay();

                // Switch to groups tab
                ctx.filter_type.set(ChatType::Group);

                // Refresh the chat list
                load_chats(ctx).await;

                response
                    .chat_id()
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Group created but ID not returned".to_string())
            } else {
                Err("Failed to create group".to_string())
            }
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}

/// Select a chat and load its messages.
pub fn select_chat(ctx: ChatContext, chat: Chat) {
    // Sort messages by timestamp
    let mut messages = chat.chatmessages.clone();
    messages.sort_by_key(|m| m.createdat.clone());

    ctx.messages.set(messages);
    ctx.active_chat.set(Some(chat));
}
