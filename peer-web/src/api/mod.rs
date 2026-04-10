//! API layer for communicating with the Peer GraphQL backend.

pub mod graphql;

// Re-export commonly used items
pub use graphql::{
    GraphQLRequest, GraphQLResponse, RegisterData, VerifyAccountData, VerifyReferralData,
    REGISTER_MUTATION, VERIFY_ACCOUNT_MUTATION, VERIFY_REFERRAL_MUTATION,
};

#[cfg(feature = "ssr")]
pub use graphql::{mutate, query};
