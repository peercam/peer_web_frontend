use async_graphql::{Context, Object, ID};
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::require_auth;
use crate::state::{
    SharedState, TransactionFeesRecord, TransactionRecord, BURN_FEE_RATE, INVITER_FEE_RATE,
    PEER_FEE_RATE, SYSTEM_BURN_ACCOUNT, SYSTEM_MINT_ACCOUNT, SYSTEM_PEER_ACCOUNT,
    SYSTEM_SHOP_ACCOUNT,
};
use crate::types::registration::DefaultResponse;
use crate::types::wallet::*;

#[derive(Default)]
pub struct WalletMutation;

#[Object]
impl WalletMutation {
    /// Transfer tokens to another user.
    async fn resolve_transfer_v2(
        &self,
        ctx: &Context<'_>,
        recipient: ID,
        numberoftokens: Decimal,
        message: Option<String>,
    ) -> TransferTokenResponse {
        let user_id = match require_auth(ctx) {
            Ok(uid) => uid,
            Err(_) => {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("60501", "Not authenticated"),
                    affected_rows: None,
                };
            }
        };

        // Validate amount
        let min_amount = Decimal::new(1, 6); // 0.000001
        if numberoftokens < min_amount {
            return TransferTokenResponse {
                meta: DefaultResponse::error("30264", "Amount must be at least 0.000001"),
                affected_rows: None,
            };
        }

        // Parse recipient UUID
        let recipient_uuid = match recipient.to_string().parse::<Uuid>() {
            Ok(u) => u,
            Err(_) => {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("30201", "Invalid recipient UUID"),
                    affected_rows: None,
                };
            }
        };

        // Cannot transfer to self
        if recipient_uuid == user_id {
            return TransferTokenResponse {
                meta: DefaultResponse::error("31202", "Cannot transfer to self"),
                affected_rows: None,
            };
        }

        // Cannot transfer to system accounts
        let system_accounts = [
            SYSTEM_BURN_ACCOUNT,
            SYSTEM_PEER_ACCOUNT,
            SYSTEM_SHOP_ACCOUNT,
            SYSTEM_MINT_ACCOUNT,
        ];
        if system_accounts.contains(&recipient_uuid) {
            return TransferTokenResponse {
                meta: DefaultResponse::error("31203", "Cannot transfer to system account"),
                affected_rows: None,
            };
        }

        // Validate message
        if let Some(ref msg) = message {
            if msg.len() > 500 {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("30270", "Message exceeds 500 characters"),
                    affected_rows: None,
                };
            }
            if msg.contains("://") || msg.contains("www.") {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("30271", "URLs not allowed in messages"),
                    affected_rows: None,
                };
            }
            if msg.chars().any(|c| c.is_control() && c != '\n') {
                return TransferTokenResponse {
                    meta: DefaultResponse::error("30271", "Control characters not allowed"),
                    affected_rows: None,
                };
            }
        }

        let mut state = ctx.data_unchecked::<SharedState>().write().await;

        // Validate recipient exists
        if !state.users.contains_key(&recipient_uuid) {
            return TransferTokenResponse {
                meta: DefaultResponse::error("31007", "Recipient not found"),
                affected_rows: None,
            };
        }

        // Calculate fees
        let burn_fee = numberoftokens * BURN_FEE_RATE;
        let peer_fee = numberoftokens * PEER_FEE_RATE;
        let inviter_fee = numberoftokens * INVITER_FEE_RATE;
        let total_fee = burn_fee + peer_fee + inviter_fee;
        let total_deduction = numberoftokens + total_fee;

        // Check balance
        let sender_balance = state.wallets.get(&user_id).copied().unwrap_or(Decimal::ZERO);
        if sender_balance < total_deduction {
            return TransferTokenResponse {
                meta: DefaultResponse::error("51301", "Insufficient balance"),
                affected_rows: None,
            };
        }

        // Execute transfer
        let now = Utc::now().to_rfc3339();
        let operation_id = Uuid::new_v4();

        // Deduct from sender
        *state.wallets.entry(user_id).or_insert(Decimal::ZERO) -= total_deduction;
        // Credit recipient
        *state
            .wallets
            .entry(recipient_uuid)
            .or_insert(Decimal::ZERO) += numberoftokens;
        // Credit fee accounts
        *state
            .wallets
            .entry(SYSTEM_BURN_ACCOUNT)
            .or_insert(Decimal::ZERO) += burn_fee;
        *state
            .wallets
            .entry(SYSTEM_PEER_ACCOUNT)
            .or_insert(Decimal::ZERO) += peer_fee;

        // Record transaction
        let fees_record = TransactionFeesRecord {
            total: total_fee,
            burn: burn_fee,
            peer: peer_fee,
            inviter: Some(inviter_fee),
        };

        state.transactions.push(TransactionRecord {
            id: Uuid::new_v4(),
            operation_id,
            category: Some(TransactionCategory::P2pTransfer),
            transaction_type: "CREDIT".into(),
            sender_id: user_id,
            recipient_id: recipient_uuid,
            token_amount: numberoftokens,
            net_token_amount: numberoftokens,
            message: message.clone(),
            fees: Some(fees_record),
            created_at: now.clone(),
        });

        TransferTokenResponse {
            meta: DefaultResponse::success("11211", "Transfer successful"),
            affected_rows: Some(TransferToken {
                token_send_formatted: numberoftokens.to_string(),
                tokens_substracted_from_wallet_formatted: total_deduction.to_string(),
                createdat: Some(now),
            }),
        }
    }
}
