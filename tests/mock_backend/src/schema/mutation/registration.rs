use async_graphql::{Context, ID, Object};
use uuid::Uuid;

use crate::seed::mock_users;
use crate::state::{SharedState, User};
use crate::types::registration::{
    DefaultResponse, ReferralResponse, RegisterResponse, RegistrationInput, VerifyAccountResponse,
};

/// UUID regex for validation
const UUID_REGEX: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

#[derive(Default)]
pub struct RegistrationMutation;

#[Object]
impl RegistrationMutation {
    /// Verify a referral code is valid
    ///
    /// Response codes:
    /// - 11011: Valid referral
    /// - 31010: Invalid referral string
    async fn verify_referral_string(
        &self,
        ctx: &Context<'_>,
        referral_string: String,
    ) -> ReferralResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let state_read = state.read().await;

        // Validate UUID format
        let regex = regex::Regex::new(UUID_REGEX).unwrap();
        if !regex.is_match(&referral_string) {
            return ReferralResponse {
                status: "error".to_string(),
                response_code: "31010".to_string(),
                affected_rows: None,
                meta: DefaultResponse::error("31010", "Invalid referral string"),
            };
        }

        // Parse and check against known referrals
        let uuid = match Uuid::parse_str(&referral_string) {
            Ok(u) => u,
            Err(_) => {
                return ReferralResponse {
                    status: "error".to_string(),
                    response_code: "31010".to_string(),
                    affected_rows: None,
                    meta: DefaultResponse::error("31010", "Invalid referral string"),
                };
            }
        };

        if state_read.known_referrals.contains(&uuid) {
            ReferralResponse {
                status: "success".to_string(),
                response_code: "11011".to_string(),
                affected_rows: Some(vec![mock_users::referral_user()]),
                meta: DefaultResponse::success("11011", "Referral info fetched successfully"),
            }
        } else {
            ReferralResponse {
                status: "error".to_string(),
                response_code: "31010".to_string(),
                affected_rows: None,
                meta: DefaultResponse::error("31010", "Invalid referral string"),
            }
        }
    }

    /// Register a new user account
    ///
    /// Response codes:
    /// - 10601: Registration successful
    /// - 30601: Email already registered
    /// - 40601: Internal server error (simulated with fail@ prefix)
    async fn register(&self, ctx: &Context<'_>, input: RegistrationInput) -> RegisterResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Simulate internal error for fail@ emails (testing hook)
        if input.email.starts_with("fail@") {
            return RegisterResponse {
                status: "error".to_string(),
                response_code: "40601".to_string(),
                userid: None,
                meta: DefaultResponse::error("40601", "Internal server error"),
            };
        }

        // Check for duplicate email
        if state_write.registered_emails.contains(&input.email) {
            return RegisterResponse {
                status: "error".to_string(),
                response_code: "30601".to_string(),
                userid: None,
                meta: DefaultResponse::error("30601", "Email is already registered"),
            };
        }

        // Success: generate user ID and record email
        let userid = Uuid::new_v4();
        state_write.registered_emails.insert(input.email.clone());

        // Create user record so login can authenticate
        let slug_num = 10000 + state_write.users.len() as i32;
        state_write.users.insert(
            userid,
            User {
                uid: userid,
                email: input.email.clone(),
                username: input.username.clone(),
                slug: input.username.to_lowercase(),
                slug_num,
                role: 0,
                status: 0,
                img: None,
                biography: None,
                visibility_status: crate::state::ContentVisibilityState::Normal,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        );
        state_write
            .user_passwords
            .insert(userid, input.password.clone());

        RegisterResponse {
            status: "success".to_string(),
            response_code: "10601".to_string(),
            userid: Some(userid.to_string()),
            meta: DefaultResponse::success("10601", "User registered successfully"),
        }
    }

    /// Verify a newly registered account
    ///
    /// Response codes:
    /// - 10701: Account verified successfully
    /// - 30701: Account already verified
    async fn verify_account(&self, ctx: &Context<'_>, userid: ID) -> VerifyAccountResponse {
        let state = ctx.data_unchecked::<SharedState>();
        let mut state_write = state.write().await;

        // Parse the user ID
        let uuid = match Uuid::parse_str(&userid) {
            Ok(u) => u,
            Err(_) => {
                // Invalid UUID still gets "verified" in the mock for compatibility
                return VerifyAccountResponse {
                    status: "success".to_string(),
                    response_code: "10701".to_string(),
                    meta: DefaultResponse::success("10701", "Account verified successfully"),
                };
            }
        };

        // Check if already verified
        if state_write.verified_users.contains(&uuid) {
            return VerifyAccountResponse {
                status: "success".to_string(),
                response_code: "30701".to_string(),
                meta: DefaultResponse::success("30701", "Account is already verified"),
            };
        }

        // Mark as verified
        state_write.verified_users.insert(uuid);

        VerifyAccountResponse {
            status: "success".to_string(),
            response_code: "10701".to_string(),
            meta: DefaultResponse::success("10701", "Account verified successfully"),
        }
    }
}
