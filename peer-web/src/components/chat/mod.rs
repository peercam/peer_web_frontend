//! Chat-related components.
//!
//! This module contains all components for the chat feature:
//! - `ChatList`: Sidebar with chat items and tabs
//! - `ChatContainer`: Message display and input
//! - `ContactsOverlay`: Friends list for starting new chats

mod chat_container;
mod chat_input;
mod chat_item;
mod chat_list;
mod chat_messages;
mod contacts_overlay;
mod group_review;

pub use chat_container::ChatContainer;
pub use chat_input::ChatInput;
pub use chat_item::ChatItem;
pub use chat_list::ChatList;
pub use chat_messages::ChatMessages;
pub use contacts_overlay::ContactsOverlay;
pub use group_review::GroupReviewScreen;
