//! API layer for communicating with the Peer GraphQL backend.

pub mod auth;
pub mod graphql;
pub mod registration;

#[cfg(feature = "ssr")]
pub mod validation;

// Re-export commonly used items
pub use graphql::{
    GraphQLRequest, GraphQLResponse, RegisterData, VerifyAccountData, VerifyReferralData,
    REGISTER_MUTATION, VERIFY_ACCOUNT_MUTATION, VERIFY_REFERRAL_MUTATION,
};

#[cfg(feature = "ssr")]
pub use graphql::{mutate, query};

pub use registration::{verify_account, verify_referral, register_user};
pub use auth::{login, refresh_access_token, logout_user, check_session};
