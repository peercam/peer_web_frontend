//! API layer for communicating with the Peer GraphQL backend.

pub mod ads;
pub mod auth;
pub mod auth_fetch;
pub mod chat;
pub mod comments;
pub mod forgot_password;
pub mod graphql;
pub mod posts;
pub mod profile;
pub mod referral;
pub mod registration;
pub mod settings;
pub mod shop;
pub mod wallet;

#[cfg(feature = "ssr")]
pub mod validation;

// Re-export commonly used items
pub use graphql::{
    GraphQLRequest, GraphQLResponse, RegisterData, VerifyAccountData, VerifyReferralData,
    REGISTER_MUTATION, VERIFY_ACCOUNT_MUTATION, VERIFY_REFERRAL_MUTATION,
    LIST_POSTS_QUERY, LIST_AD_POSTS_QUERY, POST_ACTION_MUTATION, SEARCH_USERS_QUERY, GET_USER_QUERY,
    GUEST_POST_QUERY, GET_POST_QUERY, LIST_COMMENTS_QUERY, LIST_CHILD_COMMENTS_QUERY,
    CREATE_COMMENT_MUTATION, LIKE_COMMENT_MUTATION, UNLIKE_COMMENT_MUTATION,
    GET_PROFILE_QUERY, LIST_FOLLOW_RELATIONS_QUERY, LIST_FRIENDS_QUERY, LIST_USER_POSTS_QUERY,
    TOGGLE_FOLLOW_MUTATION, TOGGLE_BLOCK_MUTATION, REPORT_USER_MUTATION,
    LIST_CHATS_QUERY, SEND_CHAT_MESSAGE_MUTATION, CREATE_CHAT_MUTATION,
    UPDATE_PROFILE_IMAGE_MUTATION, UPDATE_BIO_MUTATION, UPDATE_USERNAME_MUTATION,
    UPDATE_PASSWORD_MUTATION, UPDATE_EMAIL_MUTATION, UPDATE_PREFERENCES_MUTATION,
    DELETE_ACCOUNT_MUTATION,
};

#[cfg(feature = "ssr")]
pub use graphql::{mutate, query};

pub use registration::{verify_referral, verify_account, register_user};
pub use auth::{login, refresh_access_token, logout_user, check_session};
pub use auth_fetch::{auth_fetch, auth_fetch_api, is_unauthorized};
pub use posts::{list_posts, list_ad_posts, post_action, search_users, get_user_info};
pub use comments::{guest_get_post, get_post, list_comments, list_child_comments, create_comment, like_comment, unlike_comment};
pub use profile::{get_profile, list_follow_relations, list_friends, list_user_posts, toggle_follow, toggle_block, report_user, fetch_biography};
pub use chat::{list_chats, send_chat_message, create_chat, refresh_chat_messages};
pub use referral::{get_referral_info, get_referral_list};
pub use settings::{
    update_profile_image, update_bio, update_username, update_password,
    update_email, update_content_preferences, delete_account,
};
