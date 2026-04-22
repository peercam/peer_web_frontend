//! API layer for communicating with the Peer GraphQL backend.

pub mod ads;
pub mod auth;
pub mod auth_fetch;
pub mod chat;
pub mod comments;
pub mod forgot_password;
pub mod graphql;
pub mod moderation;
pub mod posts;
pub mod profile;
pub mod referral;
pub mod registration;
pub mod settings;
pub mod shop;
pub mod version;
pub mod wallet;

#[cfg(feature = "ssr")]
pub mod validation;

// Re-export commonly used items
pub use graphql::{
    CREATE_CHAT_MUTATION, CREATE_COMMENT_MUTATION, DELETE_ACCOUNT_MUTATION, GET_POST_QUERY,
    GET_PROFILE_QUERY, GET_USER_QUERY, GUEST_POST_QUERY, GraphQLRequest, GraphQLResponse,
    LIKE_COMMENT_MUTATION, LIST_AD_POSTS_QUERY, LIST_CHATS_QUERY, LIST_CHILD_COMMENTS_QUERY,
    LIST_COMMENTS_QUERY, LIST_FOLLOW_RELATIONS_QUERY, LIST_FRIENDS_QUERY, LIST_POSTS_QUERY,
    LIST_USER_POSTS_QUERY, POST_ACTION_MUTATION, REGISTER_MUTATION, REPORT_USER_MUTATION,
    RegisterData, SEARCH_USERS_QUERY, SEND_CHAT_MESSAGE_MUTATION, TOGGLE_BLOCK_MUTATION,
    TOGGLE_FOLLOW_MUTATION, UNLIKE_COMMENT_MUTATION, UPDATE_BIO_MUTATION, UPDATE_EMAIL_MUTATION,
    UPDATE_PASSWORD_MUTATION, UPDATE_PREFERENCES_MUTATION, UPDATE_PROFILE_IMAGE_MUTATION,
    UPDATE_USERNAME_MUTATION, VERIFY_ACCOUNT_MUTATION, VERIFY_REFERRAL_MUTATION, VerifyAccountData,
    VerifyReferralData,
};

#[cfg(feature = "ssr")]
pub use graphql::{mutate, query};

pub use auth::{check_session, login, logout_user, refresh_access_token};
pub use auth_fetch::{auth_fetch, auth_fetch_api, is_unauthorized};
pub use chat::{create_chat, list_chat_messages, list_chats, mark_chat_read, send_chat_message};
pub use comments::{
    create_comment, get_post, guest_get_post, like_comment, list_child_comments, list_comments,
    unlike_comment,
};
pub use moderation::{get_moderation_items, get_moderation_stats, perform_moderation};
pub use posts::{get_user_info, list_ad_posts, list_posts, post_action, search_users};
pub use profile::{
    fetch_biography, get_profile, list_follow_relations, list_friends, list_user_posts,
    report_user, toggle_block, toggle_follow,
};
pub use referral::{get_referral_info, get_referral_list};
pub use registration::{register_user, verify_account, verify_referral};
pub use settings::{
    delete_account, update_bio, update_content_preferences, update_email, update_password,
    update_profile_image, update_username,
};
