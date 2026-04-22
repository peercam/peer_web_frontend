//! Chat state and context provider.
//!
//! Provides `ChatContext` for managing chat state across components:
//! - Active chat selection, chat list filtering (private/group)
//! - Contact selection for new chats
//! - Polling transport (primary transport for v1) with visibility-aware
//!   intervals, optimistic id-swap, and client-side message dedup
//! - Unread counts + last-read persistence
//! - Connection state (Connected / Lost)
//! - Search query
//! - Send-failure retry
//!
//! Real-time transport decision: see
//! `docs/adr-chat-realtime-transport.md` (polling for v1, GraphQL
//! subscriptions as the preferred future upgrade path).

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::chat::{
    create_chat, list_chat_messages, list_chats, mark_chat_read, send_chat_message,
};
use crate::api::profile::list_friends;
use crate::models::chat::{Chat, ChatMessage, ChatType, MessageStatus};
use crate::models::profile::BasicUserInfo;

/// Aggregate connection state for the chat transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Connected,
    Lost,
}

/// Global chat context available to all chat components.
#[derive(Clone, Copy)]
pub struct ChatContext {
    /// Current tab filter (private or group).
    pub filter_type: RwSignal<ChatType>,

    /// List of all chats.
    pub chats: RwSignal<Vec<Chat>>,

    /// Currently selected/active chat.
    pub active_chat: RwSignal<Option<Chat>>,

    /// Messages for the active chat (polling + optimistic sends).
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

    /// Transient error message to display.
    pub error: RwSignal<Option<String>>,

    /// Current user ID (from auth).
    pub current_user_id: RwSignal<Option<String>>,

    /// Current user avatar (from auth).
    pub current_user_img: RwSignal<Option<String>>,

    /// Group name for creation.
    pub group_name: RwSignal<String>,

    /// Group image (base64) for creation.
    pub group_image: RwSignal<Option<String>>,

    /// Per-chat unread counts (chat_id -> count).
    pub unread_counts: RwSignal<HashMap<String, u32>>,

    /// Per-chat last-read timestamps (chat_id -> RFC3339 string).
    pub last_read_at: RwSignal<HashMap<String, String>>,

    /// Search query string for the sidebar filter.
    pub search_query: RwSignal<String>,

    /// Transport connection state.
    pub connection_state: RwSignal<ConnectionState>,

    /// Consecutive poll failure counter (for banner debouncing).
    pub consecutive_poll_failures: RwSignal<u32>,
}

impl ChatContext {
    /// Apply the active tab filter and the search query to the chat list.
    ///
    /// Search is case-insensitive and matches against display name, every
    /// participant username, and the last message preview.
    pub fn filtered_chats(&self) -> Vec<Chat> {
        let filter = self.filter_type.get();
        let query = self.search_query.get().trim().to_lowercase();
        let uid = self.user_id_or_default();

        self.chats
            .get()
            .into_iter()
            .filter(|chat| chat.chat_type() == filter)
            .filter(|chat| {
                if query.is_empty() {
                    return true;
                }
                let name = chat.display_name(&uid).to_lowercase();
                if name.contains(&query) {
                    return true;
                }
                if chat
                    .chatparticipants
                    .iter()
                    .any(|p| p.username.to_lowercase().contains(&query))
                {
                    return true;
                }
                chat.message_preview().to_lowercase().contains(&query)
            })
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

    /// Sum of per-chat unread counts; used for the global nav badge.
    pub fn total_unread(&self) -> u32 {
        self.unread_counts.get().values().copied().sum()
    }
}

/// Provide the chat context at the page level.
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
        unread_counts: RwSignal::new(HashMap::new()),
        last_read_at: RwSignal::new(HashMap::new()),
        search_query: RwSignal::new(String::new()),
        connection_state: RwSignal::new(ConnectionState::Connected),
        consecutive_poll_failures: RwSignal::new(0),
    };

    provide_context(ctx);
    ctx
}

/// Get the chat context from the nearest provider.
pub fn use_chat() -> ChatContext {
    expect_context::<ChatContext>()
}

/// Best-effort optional lookup used from components mounted outside
/// the chat page (e.g. the global nav badge).
pub fn try_use_chat() -> Option<ChatContext> {
    use_context::<ChatContext>()
}

// ============================================================================
// localStorage helpers for last-read persistence (WASM only).
// ============================================================================

const LAST_READ_KEY_PREFIX: &str = "chat:last-read:";

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

#[cfg(target_arch = "wasm32")]
fn read_last_read_local(chat_id: &str) -> Option<String> {
    let key = format!("{}{}", LAST_READ_KEY_PREFIX, chat_id);
    local_storage().and_then(|s| s.get_item(&key).ok().flatten())
}

#[cfg(target_arch = "wasm32")]
fn write_last_read_local(chat_id: &str, ts: &str) {
    let key = format!("{}{}", LAST_READ_KEY_PREFIX, chat_id);
    if let Some(s) = local_storage() {
        let _ = s.set_item(&key, ts);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_last_read_local(_chat_id: &str) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn write_last_read_local(_chat_id: &str, _ts: &str) {
    let _ = LAST_READ_KEY_PREFIX;
}

// ============================================================================
// Data loading
// ============================================================================

/// Load the chat list from the API.
///
/// Seeds `unread_counts` and `last_read_at` from the server's per-chat
/// fields, reconciling with locally-persisted last-read timestamps.
pub async fn load_chats(ctx: ChatContext) {
    ctx.is_loading_chats.set(true);
    ctx.error.set(None);

    match list_chats(Some(50), Some(0)).await {
        Ok(response) => {
            if response.is_success() {
                let chats = response.affected_rows;

                let mut unread: HashMap<String, u32> = HashMap::new();
                let mut last_read: HashMap<String, String> = HashMap::new();

                for chat in &chats {
                    let server_lr = chat.last_read_at.clone();
                    let local_lr = read_last_read_local(&chat.id);
                    let effective = match (local_lr.as_deref(), server_lr.as_deref()) {
                        (Some(a), Some(b)) => {
                            Some(if a > b { a.to_string() } else { b.to_string() })
                        }
                        (Some(a), None) => Some(a.to_string()),
                        (None, Some(b)) => Some(b.to_string()),
                        (None, None) => None,
                    };

                    if let Some(ts) = &effective {
                        last_read.insert(chat.id.clone(), ts.clone());
                        write_last_read_local(&chat.id, ts);
                    }

                    // If local was ahead of server, heal the server.
                    if let (Some(l), Some(s)) = (local_lr.as_deref(), server_lr.as_deref())
                        && l > s
                    {
                        let id = chat.id.clone();
                        spawn_local(async move {
                            let _ = mark_chat_read(id).await;
                        });
                    }

                    unread.insert(chat.id.clone(), chat.unread_count);
                }

                ctx.chats.set(chats);
                ctx.unread_counts.set(unread);
                ctx.last_read_at.set(last_read);
                note_poll_success(ctx);
            } else {
                ctx.error.set(Some("Failed to load chats".to_string()));
            }
        }
        Err(e) => {
            ctx.error.set(Some(format!("Error loading chats: {}", e)));
            note_poll_failure(ctx);
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
            // Silently fail — friends list is optional.
        }
    }

    ctx.is_loading_friends.set(false);
}

// ============================================================================
// Poll transport helpers
// ============================================================================

/// Record a successful poll. Resets failure streak and clears `Lost` state.
pub fn note_poll_success(ctx: ChatContext) {
    if ctx.consecutive_poll_failures.get() != 0 {
        ctx.consecutive_poll_failures.set(0);
    }
    if ctx.connection_state.get() != ConnectionState::Connected {
        ctx.connection_state.set(ConnectionState::Connected);
    }
}

/// Record a failed poll. Flips to `Lost` only after ≥ 2 consecutive failures
/// to avoid banner flicker on one-off blips.
pub fn note_poll_failure(ctx: ChatContext) {
    let n = ctx.consecutive_poll_failures.get().saturating_add(1);
    ctx.consecutive_poll_failures.set(n);
    if n >= 2 && ctx.connection_state.get() != ConnectionState::Lost {
        ctx.connection_state.set(ConnectionState::Lost);
    }
}

/// Refresh the chat list. Driven by the background poll timer.
pub async fn refresh_chat_list(ctx: ChatContext) {
    match list_chats(Some(50), Some(0)).await {
        Ok(response) if response.is_success() => {
            merge_chat_list(ctx, response.affected_rows);
            note_poll_success(ctx);
        }
        Ok(_) => {
            // "No chats" is still a success from the server's perspective.
            note_poll_success(ctx);
        }
        Err(_) => {
            note_poll_failure(ctx);
        }
    }
}

/// Merge the refreshed chat list into state. Unread counts are taken from
/// the server snapshot, except for the active chat (kept at 0).
fn merge_chat_list(ctx: ChatContext, fresh: Vec<Chat>) {
    let current_active = ctx.active_chat.get().map(|c| c.id);

    let mut unread: HashMap<String, u32> = HashMap::new();
    for c in &fresh {
        let count = if current_active.as_deref() == Some(c.id.as_str()) {
            0
        } else {
            c.unread_count
        };
        unread.insert(c.id.clone(), count);
    }

    ctx.chats.set(fresh);
    ctx.unread_counts.set(unread);
}

/// Poll for new messages on the active chat and merge them into state.
pub async fn poll_active_chat(ctx: ChatContext) {
    let chat_id = match ctx.active_chat.get().map(|c| c.id) {
        Some(id) => id,
        None => return,
    };

    let since = ctx
        .messages
        .get()
        .iter()
        .filter(|m| !m.id.starts_with("tmp:"))
        .map(|m| m.createdat.clone())
        .max();

    match list_chat_messages(chat_id.clone(), since).await {
        Ok(new_messages) => {
            if !new_messages.is_empty() {
                merge_new_messages(ctx, &chat_id, new_messages);
            }
            note_poll_success(ctx);
        }
        Err(_) => {
            note_poll_failure(ctx);
        }
    }
}

/// Merge polled messages into the active chat's message list. Handles the
/// optimistic → canonical id swap for messages the viewer just sent.
fn merge_new_messages(ctx: ChatContext, chat_id: &str, incoming: Vec<ChatMessage>) {
    let uid = ctx.user_id_or_default();
    let mut any_from_peer = false;

    ctx.messages.update(|msgs| {
        for msg in incoming {
            // 1. Same id already present → update in place.
            if let Some(existing) = msgs.iter_mut().find(|m| m.id == msg.id) {
                existing.content = msg.content.clone();
                existing.createdat = msg.createdat.clone();
                existing.status = MessageStatus::Sent;
                continue;
            }

            // 2. Match against an optimistic send (same sender + content) →
            //    swap the id.
            if let Some(optimistic) = msgs.iter_mut().find(|m| {
                m.id.starts_with("tmp:") && m.senderid == msg.senderid && m.content == msg.content
            }) {
                optimistic.id = msg.id.clone();
                optimistic.createdat = msg.createdat.clone();
                optimistic.status = MessageStatus::Sent;
                continue;
            }

            // 3. Brand new message from the server.
            if msg.senderid != uid {
                any_from_peer = true;
            }
            msgs.push(msg);
        }
        msgs.sort_by(|a, b| a.createdat.cmp(&b.createdat));
    });

    // Messages arriving while the chat is open are read in real-time, so
    // keep the server's last-read marker in sync.
    if any_from_peer {
        let id = chat_id.to_string();
        spawn_local(async move {
            let _ = mark_chat_read(id).await;
        });
    }
}

// ============================================================================
// Sending messages (optimistic + retry)
// ============================================================================

/// Send a message to the active chat with optimistic rendering.
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

    let uid = ctx.user_id_or_default();
    let now = chrono::Utc::now().to_rfc3339();
    let tmp_id = format!("tmp:{}", unique_suffix());

    let optimistic = ChatMessage {
        id: tmp_id.clone(),
        senderid: uid,
        chatid: chat_id.clone(),
        content: content.clone(),
        createdat: now,
        status: MessageStatus::Sending,
    };

    ctx.messages.update(|msgs| {
        msgs.push(optimistic);
        msgs.sort_by(|a, b| a.createdat.cmp(&b.createdat));
    });

    ctx.is_sending.set(true);
    ctx.error.set(None);

    let send_result = send_chat_message(chat_id.clone(), content.clone()).await;
    ctx.is_sending.set(false);

    match send_result {
        Ok(response) if response.is_success() => {
            if let Some(server_msg) = response.affected_rows {
                ctx.messages.update(|msgs| {
                    if let Some(m) = msgs.iter_mut().find(|m| m.id == tmp_id) {
                        m.id = server_msg.id.clone();
                        m.createdat = server_msg.createdat.clone();
                        m.status = MessageStatus::Sent;
                    }
                });
            } else {
                ctx.messages.update(|msgs| {
                    if let Some(m) = msgs.iter_mut().find(|m| m.id == tmp_id) {
                        m.status = MessageStatus::Sent;
                    }
                });
            }
            Ok(())
        }
        Ok(response) => {
            mark_tmp_failed(ctx, &tmp_id);
            Err(response.meta.response_message.clone())
        }
        Err(e) => {
            mark_tmp_failed(ctx, &tmp_id);
            Err(format!("Error: {}", e))
        }
    }
}

fn mark_tmp_failed(ctx: ChatContext, tmp_id: &str) {
    ctx.messages.update(|msgs| {
        if let Some(m) = msgs.iter_mut().find(|m| m.id == tmp_id) {
            m.status = MessageStatus::Failed;
        }
    });
}

/// Retry a send for a bubble currently in `Failed` state.
pub async fn retry_message(ctx: ChatContext, tmp_id: String) -> Result<(), String> {
    let (content, chat_id) = {
        let msgs = ctx.messages.get();
        let m = msgs
            .iter()
            .find(|m| m.id == tmp_id)
            .ok_or_else(|| "Message no longer available".to_string())?;
        (m.content.clone(), m.chatid.clone())
    };

    ctx.messages.update(|msgs| {
        if let Some(m) = msgs.iter_mut().find(|m| m.id == tmp_id) {
            m.status = MessageStatus::Sending;
        }
    });

    let result = send_chat_message(chat_id, content).await;

    match result {
        Ok(response) if response.is_success() => {
            if let Some(server_msg) = response.affected_rows {
                ctx.messages.update(|msgs| {
                    if let Some(m) = msgs.iter_mut().find(|m| m.id == tmp_id) {
                        m.id = server_msg.id.clone();
                        m.createdat = server_msg.createdat.clone();
                        m.status = MessageStatus::Sent;
                    }
                });
            }
            Ok(())
        }
        Ok(response) => {
            mark_tmp_failed(ctx, &tmp_id);
            Err(response.meta.response_message.clone())
        }
        Err(e) => {
            mark_tmp_failed(ctx, &tmp_id);
            Err(format!("Error: {}", e))
        }
    }
}

/// Short, process-local id suffix for optimistic messages. Uniqueness is
/// only needed within a single client session.
fn unique_suffix() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let now = js_sys::Date::now() as u64;
        let rand = (js_sys::Math::random() * 1.0e9) as u64;
        format!("{:x}-{:x}", now, rand)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("{:x}", ms)
    }
}

// ============================================================================
// Chat creation
// ============================================================================

/// Start a new private chat with a user.
pub async fn start_private_chat(ctx: ChatContext, user: BasicUserInfo) -> Result<String, String> {
    ctx.is_sending.set(true);

    let result = create_chat(user.username.clone(), vec![user.userid.clone()], None).await;

    ctx.is_sending.set(false);

    match result {
        Ok(response) => {
            if response.is_success() {
                ctx.close_overlay();
                load_chats(ctx).await;

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
                ctx.close_overlay();
                ctx.filter_type.set(ChatType::Group);
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

/// Select a chat, clear its unread badge, and persist the last-read marker.
///
/// Ordering matters: bump `last_read_at` **before** clearing the unread
/// count, so any poll-merge that happens mid-open is correctly suppressed.
pub fn select_chat(ctx: ChatContext, chat: Chat) {
    let chat_id = chat.id.clone();
    let now = chrono::Utc::now().to_rfc3339();

    let mut messages = chat.chatmessages.clone();
    messages.sort_by_key(|m| m.createdat.clone());

    ctx.messages.set(messages);
    ctx.active_chat.set(Some(chat));

    // 1. Bump high-water mark first.
    ctx.last_read_at.update(|m| {
        m.insert(chat_id.clone(), now.clone());
    });
    write_last_read_local(&chat_id, &now);

    // 2. Clear unread.
    ctx.unread_counts.update(|m| {
        m.insert(chat_id.clone(), 0);
    });

    // 3. Persist on the server.
    let id = chat_id;
    spawn_local(async move {
        let _ = mark_chat_read(id).await;
    });
}
