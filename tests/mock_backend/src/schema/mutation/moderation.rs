use async_graphql::{Context, ID, Object, Result};
use uuid::Uuid;

use crate::guards::require_moderator;
use crate::state::SharedState;
use crate::types::moderation::ModerationStatus;
use crate::types::registration::DefaultResponse;

#[derive(Default)]
pub struct ModerationMutation;

#[Object]
impl ModerationMutation {
    /// Perform a moderation action on a ticket.
    #[graphql(guard = "require_moderator()")]
    async fn perform_moderation(
        &self,
        ctx: &Context<'_>,
        moderation_ticket_id: ID,
        moderation_action: ModerationStatus,
    ) -> Result<DefaultResponse> {
        let current_user_id =
            crate::require_auth(ctx).map_err(|_| async_graphql::Error::new("Not authenticated"))?;

        let mut state = ctx.data::<SharedState>()?.write().await;

        // Validate action is not "waiting_for_review"
        if moderation_action == ModerationStatus::WaitingForReview {
            return Ok(DefaultResponse::error("32101", "Invalid moderation action"));
        }

        // Find ticket
        let ticket_id_str = moderation_ticket_id.to_string();
        let ticket_uuid = Uuid::parse_str(&ticket_id_str)
            .map_err(|_| async_graphql::Error::new("Invalid ticket ID"))?;

        let ticket = state
            .moderation_tickets
            .iter_mut()
            .find(|t| t.id == ticket_uuid);

        let ticket = match ticket {
            Some(t) => t,
            None => return Ok(DefaultResponse::error("22103", "Ticket not found")),
        };

        // Check if already processed with same action
        if ticket.status == moderation_action.as_str() {
            return Ok(DefaultResponse::error("32103", "Ticket already processed"));
        }

        // Update ticket
        ticket.status = moderation_action.as_str().to_string();
        ticket.moderated_by = Some(current_user_id);

        // Update content visibility
        let target_id = ticket.target_content_id;
        match moderation_action {
            ModerationStatus::Hidden => {
                state.content_visibility.insert(target_id, "HIDDEN".into());
            }
            ModerationStatus::Illegal => {
                state.content_visibility.insert(target_id, "ILLEGAL".into());
            }
            ModerationStatus::Restored => {
                state.content_visibility.insert(target_id, "NORMAL".into());
            }
            _ => {}
        }

        Ok(DefaultResponse::success(
            "12103",
            "Moderation action performed",
        ))
    }
}
