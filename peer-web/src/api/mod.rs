//! API layer for communicating with the Peer GraphQL backend.

pub mod auth;
pub mod auth_fetch;
pub mod comments;
pub mod graphql;
pub mod posts;
pub mod registration;

#[cfg(feature = "ssr")]
pub mod validation;

// Re-export commonly used items
pub use graphql::{
    GraphQLRequest, GraphQLResponse, RegisterData, VerifyAccountData, VerifyReferralData,
    REGISTER_MUTATION, VERIFY_ACCOUNT_MUTATION, VERIFY_REFERRAL_MUTATION,
    LIST_POSTS_QUERY, LIST_AD_POSTS_QUERY, POST_ACTION_MUTATION, SEARCH_USERS_QUERY, GET_USER_QUERY,
    GUEST_POST_QUERY, GET_POST_QUERY, LIST_COMMENTS_QUERY, LIST_CHILD_COMMENTS_QUERY,
    CREATE_COMMENT_MUTATION, LIKE_COMMENT_MUTATION, UNLIKE_COMMENT_MUTATION,
};

#[cfg(feature = "ssr")]
pub use graphql::{mutate, query};

pub use registration::{verify_referral, verify_account, register_user};
pub use auth::{login, refresh_access_token, logout_user, check_session};
pub use auth_fetch::{auth_fetch, auth_fetch_api, is_unauthorized};
pub use posts::{list_posts, list_ad_posts, post_action, search_users, get_user_info};
pub use comments::{guest_get_post, get_post, list_comments, list_child_comments, create_comment, like_comment, unlike_comment};
