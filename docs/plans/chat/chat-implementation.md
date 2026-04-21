# Chat Implementation Plan

**Feature:** Chat
**Status:** 🟡 Client Core Implemented — polling transport, unread, and search pending ([sprint plan](chat-completion-sprint.md)); backend persistence pending (Track A)
**Created:** 2026-04-12
**Plan Quality:** ⭐⭐⭐⭐ (4/5)
**Reviewed:** 2026-04-14
**Implementation Verified:** 2026-04-14
**Architecture note (2026-04-21):** Real-time transport is Postgres + polling for v1; GraphQL subscriptions are the preferred upgrade path. See [chat-completion-sprint.md § Blocker Resolution](chat-completion-sprint.md#blocker-resolution-2026-04-21) and [docs/adr-chat-realtime-transport.md](../../adr-chat-realtime-transport.md).

> **Reading note:** the code snippets below were drafted in 2026-04 against an earlier Leptos API (`create_signal`, `create_resource`, `create_effect`, `set_interval(..., Duration)`) and the pre-polling transport design. They are preserved as **historical design reference** — the authoritative source for current shape is the code under [peer-web/src/](../../../peer-web/src/) and the [sprint plan](chat-completion-sprint.md). Do not copy snippets from this file verbatim.

---

## What Isn't Here (2026-04-21 audit)

A repo-wide search of `peer_backend` confirmed the following are **absent**, not merely undocumented:

- No Firestore / Firebase Admin SDK dependency (`composer.json` ships only `firebase/php-jwt`).
- No `sendChatMessage` / `createChat` / `listChats` resolver or mapper in `peer_backend/src/`.
- No GraphQL chat schema file.
- No Firestore service-account credentials or config.

What **does** exist: the Postgres schema (`chats`, `chatmessages`, `chatparticipants`), input filters (`ValidateChatMessages`, `ValidateChatStructure`), and response-code copy in [json/response-codes-editable.json](../../../json/response-codes-editable.json). The feature is **stubbed**, not implemented, on the backend. The mock backend (`tests/mock_backend`) implements the full contract for client development.

---

## Overview

Implement the real-time chat system for the Leptos frontend. This is a complex feature combining GraphQL mutations for data persistence with Firebase Firestore for real-time message delivery. Supports both private (1:1) and group chats.

### Goals

1. Full parity with legacy `chat.php` user experience
2. Real-time message delivery via Firebase Firestore
3. Private (1:1) and group chat support
4. Contact/friends list for starting new chats
5. Message input with validation (500 char limit)
6. Chat history with lazy loading
7. Responsive design with mobile support

---

## Scope

### In Scope

- [x] Chat page UI (`/chat`)
- [x] Chat list component (sidebar)
- [x] Private/Group tab switching
- [x] Chat container with message display
- [x] Message input with send functionality
- [x] Friends list for starting new chats
- [x] Create private chat
- [x] Create group chat (multi-select, name, image)
- [ ] Real-time message updates (Firebase listener) — *polling fallback not yet implemented*
- [x] Message timestamp formatting (relative: Xm, Xh, Xd)
- [ ] Unread message indicators
- [ ] Chat search/filter — *UI present, logic not connected*
- [ ] User presence indicators
- [x] Loading states and skeletons
- [x] Error handling (connection lost, send failed)
- [x] Mobile-responsive layout
- [x] Message character limit (500)

Three unchecked items are owned by the [completion sprint](chat-completion-sprint.md) (Track C). See that plan for task breakdown, DoD, and cross-references to the ADR.

### Out of Scope (Future Work)

- Media messages (images, audio, video)
- Message reactions/emojis
- Message editing/deletion
- Typing indicators
- Read receipts
- Push notifications
- Group admin features (kick, promote)
- Chat archiving
- Message search within chat
- Voice/video calls

---

## Legacy Implementation Analysis

### Files

| File | Purpose |
|------|---------|
| `chat.php` | Page template with layout structure |
| `js/chat/index.js` | Initialization, entry point |
| `js/chat/state.js` | Shared state (selectedUsers, filterType, currentUserId) |
| `js/chat/api.js` | GraphQL fetch wrapper |
| `js/chat/graphql.js` | GraphQL query/mutation definitions |
| `js/chat/ui.js` | UI rendering, event handlers |
| `js/chat/loader.js` | Chat list loading and sidebar rendering |
| `js/chat/utils.js` | Utility functions (cookies, time formatting) |
| `template-parts/chat/chat-list.php` | Chat sidebar template with tabs |
| `template-parts/chat/chat-container.php` | Message display container template |
| `css/chat.css` | Chat-specific styles |
| `js/firebase_config.js` | Firebase/Firestore initialization |

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Chat System                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐         ┌─────────────────────────────┐   │
│  │  GraphQL API    │         │    Firebase Firestore       │   │
│  │  (Persistence)  │         │    (Real-time sync)         │   │
│  ├─────────────────┤         ├─────────────────────────────┤   │
│  │ • listChats     │───────► │ • Real-time listeners       │   │
│  │ • createChat    │         │ • Message subscriptions     │   │
│  │ • sendChatMessage│        │ • Presence tracking         │   │
│  │ • listFriends   │         │ • Optimistic updates        │   │
│  └─────────────────┘         └─────────────────────────────┘   │
│           │                             ▲                       │
│           ▼                             │                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Chat UI Layer                         │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │   │
│  │  │ Chat List    │  │ Chat Window  │  │ Message Input │  │   │
│  │  │ (Sidebar)    │  │ (Messages)   │  │ (Send)        │  │   │
│  │  └──────────────┘  └──────────────┘  └───────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Layout Structure (Desktop)

```
┌────────────────────────────────────────────────────────────────────────┐
│  HEADER: Logo + "Chat" title                                           │
├──────────────┬─────────────────────────────────────────┬───────────────┤
│              │                                         │               │
│  LEFT        │           MAIN CONTENT                  │  RIGHT        │
│  SIDEBAR     │                                         │  SIDEBAR      │
│  (search)    │  ┌─────────────────┬───────────────────┐│               │
│              │  │ CHAT LIST       │ CHAT CONTAINER    ││  - Profile    │
│              │  │ (36% width)     │ (64% width)       ││  - Main menu  │
│              │  │                 │                   ││  - New post   │
│              │  │ ┌─────────────┐ │ ┌───────────────┐ ││  - Version    │
│              │  │ │ Private     │ │ │ Chat Header   │ ││               │
│              │  │ │ Groups [+]  │ │ │ (name/avatar) │ ││               │
│              │  │ └─────────────┘ │ └───────────────┘ ││               │
│              │  │                 │                   ││               │
│              │  │ ┌─────────────┐ │ ┌───────────────┐ ││               │
│              │  │ │ Chat Item 1 │ │ │               │ ││               │
│              │  │ │ Chat Item 2 │ │ │  Messages     │ ││               │
│              │  │ │ Chat Item 3 │ │ │  (scrollable) │ ││               │
│              │  │ │ ...         │ │ │               │ ││               │
│              │  │ └─────────────┘ │ └───────────────┘ ││               │
│              │  │                 │                   ││               │
│              │  │                 │ ┌───────────────┐ ││               │
│              │  │                 │ │ Input + Send  │ ││               │
│              │  │                 │ └───────────────┘ ││               │
│              │  └─────────────────┴───────────────────┘│               │
│              │                                         │               │
├──────────────┴─────────────────────────────────────────┴───────────────┤
│  FOOTER (mobile navigation)                                            │
└────────────────────────────────────────────────────────────────────────┘
```

### Mobile Layout

```
┌──────────────────────────┐
│  [←] Chat           [⋮]  │
├──────────────────────────┤
│  [Private] [Groups] [+]  │
├──────────────────────────┤
│  ┌────────────────────┐  │
│  │ Chat Item 1        │  │
│  │   Preview message  │  │
│  ├────────────────────┤  │
│  │ Chat Item 2        │  │
│  │   Preview message  │  │
│  ├────────────────────┤  │
│  │ ...                │  │
│  └────────────────────┘  │
└──────────────────────────┘

          ↓ (tap chat)

┌──────────────────────────┐
│  [←] Username       [⋮]  │
├──────────────────────────┤
│                          │
│  ┌────────────────────┐  │
│  │ Their message      │  │
│  │              12:34 │  │
│  └────────────────────┘  │
│                          │
│  ┌────────────────────┐  │
│  │     My message     │  │
│  │ 12:35              │  │
│  └────────────────────┘  │
│                          │
├──────────────────────────┤
│ [📎] Write a message [→] │
└──────────────────────────┘
```

### Key Features

1. **Chat Type Tabs**
   - Private (1:1) chats: No name/image, identified by participant
   - Group chats: Has name and custom image
   - [+] button opens contacts overlay

2. **Contacts Overlay**
   - Shows friends list (`listFriends` query)
   - Private mode: Click to start chat immediately
   - Group mode: Checkbox multi-select → "Next" → name/image → "Create"

3. **Chat Item/Card**
   - Avatar (user avatar for private, group image for groups)
   - Name (username for private, group name for groups)
   - Last message preview (truncated)
   - Timestamp (relative: Xm, Xh, Xd)
   - Unread count badge

4. **Message Display**
   - Own messages: right-aligned, blue bubble
   - Other messages: left-aligned, with avatar
   - Timestamp under each message
   - Auto-scroll to newest
   - HTML entity decoding

5. **Message Input**
   - Textarea with placeholder
   - Enter key to send (no shift)
   - 500 character limit with error message
   - User avatar next to input
   - Send button (optional)

6. **State Management**
   - `filterType`: "private" | "group"
   - `selectedUsers`: array for group creation
   - `isInCreateOverlay`: boolean
   - `isInReviewScreen`: boolean (group creation step 2)
   - `currentUserId`: logged-in user ID
   - `fullContactList`: cached friends list

---

## Backend API Reference

> **⚠️ Status (2026-04-21):** The mutations and queries documented below exist in the **mock backend** ([tests/mock_backend](../../../tests/mock_backend), Phase 4) and in [json/response-codes-editable.json](../../../json/response-codes-editable.json) as planned surface. They are **not yet implemented** in `peer_backend`. The shapes below are the contract the real backend must honour — see [chat-completion-sprint.md § Blocker Resolution](chat-completion-sprint.md#blocker-resolution-2026-04-21) (Track A).

### `listChats` Query

```graphql
query ListChats($limit: Int, $offset: Int) {
  listChats(limit: $limit, offset: $offset) {
    affectedRows {
      id
      image
      name
      createdat
      updatedat
      chatmessages {
        id
        senderid
        chatid
        content
        createdat
      }
      chatparticipants {
        userid
        img
        username
        slug
        hasaccess
      }
    }
  }
}
```

#### Chat Type Detection
- **Private chat**: No `name` and no `image`
- **Group chat**: Has `name` or `image`

### `listFriends` Query

```graphql
query ListFriends {
  listFriends {
    affectedRows {
      userid
      img
      username
    }
  }
}
```

Friends are users who mutually follow each other.

### `sendChatMessage` Mutation

```graphql
mutation SendChatMessage($chatid: ID!, $content: String!) {
  sendChatMessage(chatid: $chatid, content: $content) {
    status
    ResponseCode
    affectedRows {
      content
      createdat
    }
  }
}
```

#### Validation
- Content max length: 500 characters
- Content required (non-empty)

### `createChat` Mutation

```graphql
mutation CreateChat($name: String!, $recipients: [String!]!, $image: String) {
  createChat(input: { name: $name, recipients: $recipients, image: $image }) {
    status
    ResponseCode
    affectedRows {
      chatid
    }
  }
}
```

#### Usage
- **Private chat**: `recipients: [userId]`, `name: username`, `image: null`
- **Group chat**: `recipients: [user1, user2, ...]`, `name: "Group Name"`, `image: "base64..."`

### `getProfile` Query

Used to fetch current user's ID and avatar for display:

```graphql
query GetProfile {
  getProfile {
    status
    ResponseCode
    affectedRows {
      id
      img
    }
  }
}
```

---

## Firebase Integration

> **⚠️ Architecture note (2026-04-21):** The Firestore structure below is **assumed / legacy** — it describes the shape the legacy `js/chat/loader.js` client expected, not a live architecture. The canonical write path in the new design is **Postgres** (`chatmessages` table). If a Firestore mirror is ever added, it becomes a **read-only projection** populated out-of-process (Cloud Function / outbox worker), never written directly by the request path. See [docs/adr-chat-realtime-transport.md](../../adr-chat-realtime-transport.md).

### Firestore Structure (Assumed)

```
/chats/{chatId}
  - name: string
  - image: string?
  - createdAt: timestamp
  - updatedAt: timestamp
  - participants: string[]

/chats/{chatId}/messages/{messageId}
  - senderId: string
  - content: string
  - createdAt: timestamp
  - readBy: string[]
```

### Real-time Listeners

```javascript
// Listen to chat list updates
db.collection('chats')
  .where('participants', 'array-contains', currentUserId)
  .orderBy('updatedAt', 'desc')
  .onSnapshot(snapshot => {
    // Update chat list
  });

// Listen to messages in active chat
db.collection('chats')
  .doc(activeChatId)
  .collection('messages')
  .orderBy('createdAt', 'asc')
  .onSnapshot(snapshot => {
    // Update messages
  });
```

### Leptos + Firebase Strategy

> **Decision (2026-04-21):** v1 ships **polling** against the GraphQL backend as the primary — and only — transport. GraphQL subscriptions are the preferred future upgrade path per the [ADR](../../adr-chat-realtime-transport.md); Firebase/Firestore JS-interop is **rejected** for this codebase (there is no production write path to mirror). The four-option menu below is preserved as historical context only. **No recommendation stands;** follow the ADR.

Since Leptos is Rust/WASM, earlier drafts considered:

1. **JavaScript Interop**: Call Firebase SDK from Rust via `wasm-bindgen`
2. **REST API**: Use Firebase REST API directly from Rust
3. **Server-Sent Events**: Implement polling/SSE on server side
4. **Hybrid Approach**: Use JS for real-time, Rust for UI

~~**Recommended**: JavaScript interop for Firebase listeners, Rust for GraphQL and UI.~~ — **Superseded.** See the ADR.

---

## Implementation Plan

### Phase 1: Core Infrastructure

#### 1.1 Chat Models (`src/models/chat.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Chat type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatType {
    Private,
    Group,
}

/// A chat participant
#[derive(Debug, Clone, Deserialize)]
pub struct ChatParticipant {
    pub userid: String,
    pub img: String,
    pub username: String,
    pub slug: Option<String>,
    pub hasaccess: Option<bool>,
}

/// A chat message
#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub senderid: String,
    pub chatid: String,
    pub content: String,
    pub createdat: String,
}

/// A chat room
#[derive(Debug, Clone, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub createdat: String,
    pub updatedat: String,
    pub chatmessages: Vec<ChatMessage>,
    pub chatparticipants: Vec<ChatParticipant>,
}

impl Chat {
    pub fn chat_type(&self) -> ChatType {
        if self.name.as_ref().map_or(true, |n| n.trim().is_empty())
            && self.image.is_none()
        {
            ChatType::Private
        } else {
            ChatType::Group
        }
    }

    pub fn display_name(&self, current_user_id: &str) -> String {
        match self.chat_type() {
            ChatType::Private => {
                self.chatparticipants
                    .iter()
                    .find(|p| p.userid != current_user_id)
                    .map(|p| p.username.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            }
            ChatType::Group => {
                self.name.clone().unwrap_or_else(|| "Group".to_string())
            }
        }
    }
}

/// Friend for starting new chats
#[derive(Debug, Clone, Deserialize)]
pub struct Friend {
    pub userid: String,
    pub img: String,
    pub username: String,
}

/// API response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct ListChatsResponse {
    #[serde(rename = "affectedRows")]
    pub affected_rows: Vec<Chat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListFriendsResponse {
    #[serde(rename = "affectedRows")]
    pub affected_rows: Vec<Friend>,
}
```

#### 1.2 Chat API (`src/api/chat.rs`)

```rust
use leptos::prelude::*;
use crate::models::chat::*;

/// Fetch user's chat list
#[server(ListChats, "/api")]
pub async fn list_chats(
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Chat>, ServerFnError>;

/// Fetch friends list for new chat creation
#[server(ListFriends, "/api")]
pub async fn list_friends() -> Result<Vec<Friend>, ServerFnError>;

/// Send a message to a chat
#[server(SendChatMessage, "/api")]
pub async fn send_chat_message(
    chatid: String,
    content: String,
) -> Result<ChatMessage, ServerFnError>;

/// Create a new chat (private or group)
#[server(CreateChat, "/api")]
pub async fn create_chat(
    name: String,
    recipients: Vec<String>,
    image: Option<String>,
) -> Result<String, ServerFnError>; // Returns chatid
```

GraphQL queries:

```rust
pub const LIST_CHATS_QUERY: &str = r#"
    query ListChats($limit: Int, $offset: Int) {
        listChats(limit: $limit, offset: $offset) {
            affectedRows {
                id
                image
                name
                createdat
                updatedat
                chatmessages {
                    id
                    senderid
                    chatid
                    content
                    createdat
                }
                chatparticipants {
                    userid
                    img
                    username
                    slug
                    hasaccess
                }
            }
        }
    }
"#;

pub const LIST_FRIENDS_QUERY: &str = r#"
    query ListFriends {
        listFriends {
            affectedRows {
                userid
                img
                username
            }
        }
    }
"#;

pub const SEND_MESSAGE_MUTATION: &str = r#"
    mutation SendChatMessage($chatid: ID!, $content: String!) {
        sendChatMessage(chatid: $chatid, content: $content) {
            status
            ResponseCode
            affectedRows {
                content
                createdat
            }
        }
    }
"#;

pub const CREATE_CHAT_MUTATION: &str = r#"
    mutation CreateChat($name: String!, $recipients: [String!]!, $image: String) {
        createChat(input: { name: $name, recipients: $recipients, image: $image }) {
            status
            ResponseCode
            affectedRows {
                chatid
            }
        }
    }
"#;
```

#### 1.3 Chat State (`src/state/chat.rs`)

```rust
use leptos::prelude::*;
use crate::models::chat::*;

#[derive(Clone)]
pub struct ChatContext {
    /// Current tab filter
    pub filter_type: RwSignal<ChatType>,
    
    /// List of chats
    pub chats: RwSignal<Vec<Chat>>,
    
    /// Currently selected chat
    pub active_chat: RwSignal<Option<Chat>>,
    
    /// Messages in active chat
    pub messages: RwSignal<Vec<ChatMessage>>,
    
    /// Friends list for new chat
    pub friends: RwSignal<Vec<Friend>>,
    
    /// Selected users for group creation
    pub selected_users: RwSignal<Vec<Friend>>,
    
    /// UI state
    pub is_create_overlay_open: RwSignal<bool>,
    pub is_review_screen: RwSignal<bool>,
    
    /// Loading states
    pub is_loading_chats: RwSignal<bool>,
    pub is_sending: RwSignal<bool>,
    
    /// Current user info
    pub current_user_id: RwSignal<Option<String>>,
    pub current_user_img: RwSignal<Option<String>>,
}

pub fn provide_chat_context();
pub fn use_chat() -> ChatContext;
```

### Phase 2: Chat Page UI

#### 2.1 Chat Page (`src/pages/chat.rs`)

```rust
#[component]
pub fn ChatPage() -> impl IntoView {
    // Initialize chat context
    let _ = provide_chat_context();
    
    // Fetch initial data
    let chats = create_resource(|| (), |_| list_chats(Some(20), Some(0)));
    
    view! {
        <Title text="Peer Network - Chat"/>
        
        <div class="site_layout">
            <Header title="Chat"/>
            
            <aside class="left-sidebar left-sidebar-chats">
                <div class="inner-scroll">
                    <SearchBar/>
                </div>
            </aside>
            
            <main class="site-main site-main-chats">
                <div class="main-chat">
                    <ChatList/>
                    <ChatContainer/>
                </div>
            </main>
            
            <aside class="right-sidebar right-sidebar-chats">
                <div class="inner-scroll">
                    <ProfileWidget/>
                    <MainMenu/>
                    <AddNewPostWidget/>
                    <VersionWidget/>
                </div>
            </aside>
            
            <Footer/>
        </div>
    }
}
```

#### 2.2 Chat List Component (`src/components/chat/chat_list.rs`)

```rust
#[component]
pub fn ChatList() -> impl IntoView {
    let chat_ctx = use_chat();
    
    view! {
        <div class="chat-list">
            // Tab switcher
            <div class="top-bar">
                <div class="chat-switch">
                    <ChatTabs/>
                    <button 
                        class="add-btn btn-blue"
                        on:click=move |_| {
                            chat_ctx.is_create_overlay_open.set(true);
                            // Fetch friends
                        }
                    >
                        "+"
                    </button>
                </div>
            </div>
            
            // Chat items or contacts overlay
            <div class="chat-pannel">
                <div class="chat-pannel-widget">
                    <Show
                        when=move || chat_ctx.is_create_overlay_open.get()
                        fallback=|| view! { <ChatItems/> }
                    >
                        <ContactsOverlay/>
                    </Show>
                </div>
            </div>
            
            // Empty state
            <EmptyState
                show=move || chat_ctx.chats.get().is_empty()
                message="No chats found..."
            />
        </div>
    }
}

#[component]
pub fn ChatTabs() -> impl IntoView {
    let chat_ctx = use_chat();
    
    let is_private = move || chat_ctx.filter_type.get() == ChatType::Private;
    let is_group = move || chat_ctx.filter_type.get() == ChatType::Group;
    
    view! {
        <button
            id="privateBtn"
            class=move || if is_private() { "tab active" } else { "tab" }
            on:click=move |_| chat_ctx.filter_type.set(ChatType::Private)
        >
            "Private"
        </button>
        <button
            id="groupBtn"
            class=move || if is_group() { "tab active" } else { "tab" }
            on:click=move |_| chat_ctx.filter_type.set(ChatType::Group)
        >
            "Groups"
        </button>
    }
}

#[component]
pub fn ChatItem(chat: Chat) -> impl IntoView {
    let chat_ctx = use_chat();
    let current_user_id = chat_ctx.current_user_id.get().unwrap_or_default();
    
    let display_name = chat.display_name(&current_user_id);
    let last_message = chat.chatmessages.last();
    let preview = last_message
        .map(|m| html_decode(&m.content))
        .unwrap_or_else(|| "Start chatting...".to_string());
    let time = last_message
        .map(|m| format_time_ago(&m.createdat))
        .unwrap_or_else(|| "—".to_string());
    
    let avatar_url = chat.chatparticipants
        .iter()
        .find(|p| p.userid != current_user_id)
        .map(|p| media_url(&p.img))
        .unwrap_or_else(|| "/svg/noname.svg".to_string());
    
    let is_active = move || {
        chat_ctx.active_chat.get()
            .map(|c| c.id == chat.id)
            .unwrap_or(false)
    };
    
    view! {
        <div
            class=move || if is_active() { "chat-item active-chat" } else { "chat-item" }
            data-chatid=chat.id.clone()
            on:click={
                let chat = chat.clone();
                move |_| chat_ctx.active_chat.set(Some(chat.clone()))
            }
        >
            <img class="avatar" src=avatar_url alt=display_name.clone()/>
            <div class="chat-details">
                <div class="chat-row">
                    <span class="name">{display_name}</span>
                    <span class="time">{time}</span>
                </div>
                <div class="message-preview">{preview}</div>
            </div>
        </div>
    }
}
```

#### 2.3 Chat Container (`src/components/chat/chat_container.rs`)

```rust
#[component]
pub fn ChatContainer() -> impl IntoView {
    let chat_ctx = use_chat();
    
    view! {
        <div class="chat-container">
            <Show
                when=move || chat_ctx.active_chat.get().is_some()
                fallback=|| view! { <NoChatSelected/> }
            >
                {move || {
                    let chat = chat_ctx.active_chat.get().unwrap();
                    view! {
                        <ChatHeader chat=chat.clone()/>
                        <ChatMessages chat=chat.clone()/>
                        <ChatInput/>
                    }
                }}
            </Show>
        </div>
    }
}

#[component]
pub fn ChatHeader(chat: Chat) -> impl IntoView {
    let chat_ctx = use_chat();
    let current_user_id = chat_ctx.current_user_id.get().unwrap_or_default();
    let display_name = chat.display_name(&current_user_id);
    
    let avatar_url = chat.chatparticipants
        .iter()
        .find(|p| p.userid != current_user_id)
        .map(|p| media_url(&p.img))
        .unwrap_or_else(|| "/svg/noname.svg".to_string());
    
    view! {
        <div class="chat-header">
            <div class="header-left">
                <img src=avatar_url alt="user" class="avatar"/>
                <span class="username">{display_name}</span>
            </div>
            <div class="header-right">
                // Search and menu buttons
            </div>
        </div>
    }
}

#[component]
pub fn ChatMessages(chat: Chat) -> impl IntoView {
    let chat_ctx = use_chat();
    let current_user_id = chat_ctx.current_user_id.get().unwrap_or_default();
    
    // Sort messages by timestamp
    let messages = chat.chatmessages.clone();
    let sorted: Vec<_> = {
        let mut msgs = messages;
        msgs.sort_by_key(|m| m.createdat.clone());
        msgs
    };
    
    view! {
        <div class="chat-messages">
            <For
                each=move || sorted.clone()
                key=|m| m.id.clone()
                children=move |message| {
                    let is_own = message.senderid == current_user_id;
                    view! {
                        <Message message=message is_own=is_own/>
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn Message(message: ChatMessage, is_own: bool) -> impl IntoView {
    let content = html_decode(&message.content);
    let time = format_time(&message.createdat);
    
    view! {
        <div class=if is_own { "message right" } else { "message" }>
            <Show when=move || !is_own>
                <div class="profile_avatar">
                    <img class="avatar" alt="Avatar"/>
                </div>
            </Show>
            <div class="message_content">
                <div class="bubble blue">
                    <span class="message-text">{content}</span>
                    <span class="time">{time}</span>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ChatInput() -> impl IntoView {
    let chat_ctx = use_chat();
    let (message, set_message) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    
    let send_message = move || {
        let content = message.get().trim().to_string();
        if content.is_empty() {
            return;
        }
        if content.len() > 500 {
            set_error.set(Some("Message must be 500 characters or fewer".to_string()));
            return;
        }
        
        let chat_id = chat_ctx.active_chat.get().map(|c| c.id);
        if let Some(chatid) = chat_id {
            chat_ctx.is_sending.set(true);
            spawn_local(async move {
                match send_chat_message(chatid, content).await {
                    Ok(_) => {
                        set_message.set(String::new());
                        set_error.set(None);
                        // Optimistic update handled by Firebase listener
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Send failed: {}", e)));
                    }
                }
                chat_ctx.is_sending.set(false);
            });
        }
    };
    
    view! {
        <div class="chat-input">
            <img 
                src=move || chat_ctx.current_user_img.get()
                    .map(|img| media_url(&img))
                    .unwrap_or_else(|| "/svg/noname.svg".to_string())
                class="avatar"
            />
            <textarea
                id="sendPrivateMessage"
                placeholder="Write a message ..."
                prop:value=message
                on:input=move |ev| set_message.set(event_target_value(&ev))
                on:keydown=move |ev| {
                    if ev.key() == "Enter" && !ev.shift_key() {
                        ev.prevent_default();
                        send_message();
                    }
                }
            />
            <Show when=move || error.get().is_some()>
                <div class="error-block">
                    <label>{error.get()}</label>
                </div>
            </Show>
        </div>
    }
}
```

#### 2.4 Contacts Overlay (`src/components/chat/contacts_overlay.rs`)

```rust
#[component]
pub fn ContactsOverlay() -> impl IntoView {
    let chat_ctx = use_chat();
    
    // Fetch friends on mount
    create_effect(move |_| {
        spawn_local(async move {
            if let Ok(friends) = list_friends().await {
                chat_ctx.friends.set(friends);
            }
        });
    });
    
    view! {
        <Show
            when=move || chat_ctx.is_review_screen.get()
            fallback=|| view! { <ContactsList/> }
        >
            <GroupReviewScreen/>
        </Show>
    }
}

#[component]
pub fn ContactsList() -> impl IntoView {
    let chat_ctx = use_chat();
    let friends = move || chat_ctx.friends.get();
    let is_group = move || chat_ctx.filter_type.get() == ChatType::Group;
    
    view! {
        <For
            each=friends
            key=|f| f.userid.clone()
            children=move |friend| {
                view! {
                    <ContactCard friend=friend/>
                }
            }
        />
        
        // Show "Next" button for group mode
        <Show when=is_group>
            <div class="chat_buttons selected">
                <span class="count-selected">
                    {move || format!("{} account selected", chat_ctx.selected_users.get().len())}
                </span>
                <a href="#" class="next-btn" on:click=move |e| {
                    e.prevent_default();
                    chat_ctx.is_review_screen.set(true);
                }>
                    "Next"
                </a>
            </div>
        </Show>
        
        // Empty state
        <Show when=move || friends().is_empty()>
            <div class="no_post_found active">
                "No friends found..."
            </div>
        </Show>
    }
}

#[component]
pub fn ContactCard(friend: Friend) -> impl IntoView {
    let chat_ctx = use_chat();
    let is_group = move || chat_ctx.filter_type.get() == ChatType::Group;
    
    let is_selected = move || {
        chat_ctx.selected_users.get()
            .iter()
            .any(|f| f.userid == friend.userid)
    };
    
    let toggle_selection = {
        let friend = friend.clone();
        move |_| {
            let mut selected = chat_ctx.selected_users.get();
            if selected.iter().any(|f| f.userid == friend.userid) {
                selected.retain(|f| f.userid != friend.userid);
            } else {
                selected.push(friend.clone());
            }
            chat_ctx.selected_users.set(selected);
        }
    };
    
    let start_private_chat = {
        let friend = friend.clone();
        move |_| {
            spawn_local(async move {
                match create_chat(
                    friend.username.clone(),
                    vec![friend.userid.clone()],
                    None,
                ).await {
                    Ok(chat_id) => {
                        // Close overlay and select new chat
                        chat_ctx.is_create_overlay_open.set(false);
                        // Refresh chat list
                    }
                    Err(e) => {
                        // Show error
                    }
                }
            });
        }
    };
    
    view! {
        <div class="chat-list-overlay" data-user-id=friend.userid.clone()>
            <div class="chat-list-item">
                <span class="profile-pic">
                    <img src=media_url(&friend.img) alt=friend.username.clone()/>
                </span>
                <span class="profile-name">{friend.username.clone()}</span>
                
                <Show when=is_group fallback=move || view! {
                    <img 
                        class="chat-icon" 
                        src="/svg/chats.svg" 
                        alt="chat"
                        on:click=start_private_chat.clone()
                    />
                }>
                    <input 
                        type="checkbox"
                        prop:checked=is_selected
                        on:change=toggle_selection.clone()
                    />
                </Show>
            </div>
        </div>
    }
}
```

### Phase 3: Firebase Real-time Integration

#### 3.1 Firebase JS Interop (`src/firebase/mod.rs`)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = firebase)]
    fn firestore() -> JsValue;
    
    // Add listeners, etc.
}

/// Set up real-time listener for messages
pub fn subscribe_to_chat(chat_id: &str, callback: impl Fn(Vec<ChatMessage>) + 'static) {
    // JS interop to set up onSnapshot listener
}

/// Clean up listener
pub fn unsubscribe_from_chat(chat_id: &str) {
    // Detach listener
}
```

#### 3.2 Polling Transport (PRIMARY for v1)

Polling is the chosen transport per the [ADR](../../adr-chat-realtime-transport.md). Implementation specifics — visibility-aware intervals, `since` semantics, optimistic-send dedup, connection-lost banner — are owned by [chat-completion-sprint.md § Task 1](chat-completion-sprint.md#task-1--polling-transport-primary).

### Phase 4: Polish & Edge Cases

#### 4.1 Error Handling

- Network disconnection → show reconnecting banner
- Send failure → show retry button
- Empty chat list → show "No chats" state
- No friends → show "No friends found" state

#### 4.2 Accessibility

- Keyboard navigation for chat items
- ARIA labels for buttons
- Focus management when selecting chats
- Screen reader announcements for new messages

#### 4.3 Performance

- Virtualized list for many chats
- Pagination/infinite scroll for chat list
- Debounced search input
- Message batching for history

---

## Testing Strategy

> Status legend: ✅ landed, 🔲 owned by [sprint](chat-completion-sprint.md)

### Unit Tests

| Test | Status | Description |
|------|--------|-------------|
| `chat_type_detection` | ✅ | Private vs group chat detection |
| `display_name_private` | ✅ | Username shown for private chats |
| `display_name_group` | ✅ | Group name shown for group chats |
| `time_formatting` | ✅ | Relative time (Xm, Xh, Xd) |
| `message_validation` | ✅ | 500 char limit, empty check |
| `html_decode` | ✅ | Entity decoding in messages |
| `message_status_failed_renders_retry` | 🔲 | Failed bubble shows Retry button (sprint Task 6) |

### Integration Tests

| Test | Description |
|------|-------------|
| `load_chat_list` | Fetches and displays chats |
| `switch_tabs` | Private/Group tab filtering |
| `select_chat` | Clicking chat loads messages |
| `send_message` | Message appears in chat |
| `create_private_chat` | Start chat with friend |
| `create_group_chat` | Multi-select, name, create |

### E2E Tests (Playwright)

```typescript
test('can send a message', async ({ page }) => {
  await page.goto('/chat');
  await page.click('.chat-item:first-child');
  await page.fill('#sendPrivateMessage', 'Hello world');
  await page.press('#sendPrivateMessage', 'Enter');
  await expect(page.locator('.message-text:last-child')).toContainText('Hello world');
});

test('can create group chat', async ({ page }) => {
  await page.goto('/chat');
  await page.click('#groupBtn');
  await page.click('.add-btn');
  await page.click('.chat-list-overlay:first-child input');
  await page.click('.chat-list-overlay:nth-child(2) input');
  await page.click('.next-btn');
  await page.fill('.input.title', 'Test Group');
  await page.click('.create-btn');
  await expect(page.locator('.chat-item .name')).toContainText('Test Group');
});
```

---

## Files Created

| File | Purpose | Status |
|------|---------|--------|
| `src/pages/chat.rs` | Chat page component | ✅ |
| `src/models/chat.rs` | Chat data models | ✅ |
| `src/api/chat.rs` | GraphQL queries/mutations | ✅ |
| `src/state/chat.rs` | Chat context/state | ✅ |
| `src/components/chat/mod.rs` | Chat components module | ✅ |
| `src/components/chat/chat_list.rs` | Chat sidebar | ✅ |
| `src/components/chat/chat_item.rs` | Single chat card | ✅ |
| `src/components/chat/chat_container.rs` | Message display | ✅ |
| `src/components/chat/chat_input.rs` | Message input | ✅ |
| `src/components/chat/chat_messages.rs` | Message bubbles + scroll | ✅ |
| `src/components/chat/contacts_overlay.rs` | Friends list | ✅ |
| `src/components/chat/group_review.rs` | Group creation | ✅ |
| `style/chat.scss` | Chat page styles | ✅ |
| `src/models/chat.rs` (tests) | Unit tests | ✅ |
| `end2end/tests/chat.spec.ts` | E2E tests | 🔲 owned by [sprint Task 7](chat-completion-sprint.md#task-7--e2e-tests) |

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Backend persistence (Track A) slips | High | Client Track C ships against the mock today; production ship is gated on Track A landing, flagged in the sprint's feature-level DoD. |
| Polling backend load at scale | Medium | Intervals env-configurable; visibility-pause on hidden tabs; `since`-exclusive keeps payloads small. |
| Real-time message ordering | Medium | Use timestamp sorting, handle duplicates via client-side id dedup (sprint Task 1). |
| Mobile responsiveness | Medium | Design mobile-first, test on devices |
| Large chat history | Low | Implement pagination, virtualization |

---

## Estimated Effort

Superseded by the [completion sprint](chat-completion-sprint.md), which tracks remaining effort per-task. Original 9-day estimate from the 2026-04-12 draft is retained below for historical context only.

| Phase | Effort (historical) |
|-------|--------|
| Phase 1: Infrastructure | 2 days |
| Phase 2: UI Components | 3 days |
| Phase 3: Firebase Real-time | 2 days — **superseded** by polling (sprint Task 1) |
| Phase 4: Polish | 1 day |
| Testing | 1 day |
| **Total (historical)** | **9 days** |

---

## Dependencies

- Auth system (complete ✅)
- Profile API (for current user info)
- Friends/follow system (for `listFriends`)
- Media URL helper (for avatars)
- Track A backend persistence (Postgres resolvers) — tracked in a sibling plan, required for production ship

---

## Changelog

### 2026-04-21 (Transport decision + doc refresh)
- Transport decision ratified in [docs/adr-chat-realtime-transport.md](../../adr-chat-realtime-transport.md): **polling for v1**, GraphQL subscriptions as the preferred upgrade path. Firebase/Firestore rejected for this codebase.
- Superseded stale priority and effort headers; added historical-reference note to the code snippets section.
- Rewrote §3.2 (polling) to point at the sprint; struck the Firebase recommendation under §Leptos + Firebase Strategy.
- Reconciled Testing Strategy table with the 2026-04-14 changelog (unit tests are landed ✅; E2E delegated to sprint Task 7).
- Added "What Isn't Here" audit pointing at [chat-completion-sprint.md § Blocker Resolution](chat-completion-sprint.md#blocker-resolution-2026-04-21).

### 2026-04-14 (Implementation Verified)
- Core implementation complete and compiles successfully
- All planned components created except E2E tests
- Unit tests present in `src/models/chat.rs` for chat type detection, display names, message truncation
- GraphQL queries/mutations integrated with auth tokens
- Known gaps:
  - Real-time updates not implemented (no Firebase or polling)
  - Chat search UI present but not functional
  - Sender avatars hardcoded in messages (uses `/svg/noname.svg`)
  - Group image upload UI placeholder only
  - No unread indicators or presence

### 2026-04-14 (Quality Review)
- Plan rated ⭐⭐⭐⭐ (4/5)
- Strengths: thorough legacy analysis, clear architecture diagrams, well-decomposed components, comprehensive API reference
- Gaps identified: Firebase JS interop under-specified, group review screen code missing, no dual-source deduplication strategy, sparse error handling stubs, no scroll-to-bottom implementation, no offline/reconnection detail

### 2026-04-12
- Initial planning document created
- Legacy implementation analyzed
- API contracts documented
- Component architecture designed
