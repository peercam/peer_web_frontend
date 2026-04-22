//! Referral user grid component.
//!
//! Grid container displaying user cards or an empty state message.

use leptos::prelude::*;

use crate::components::referral_board::referral_user_card::ReferralUserCard;
use crate::models::referral::ReferralListUser;

/// Grid of user cards with empty state handling.
#[component]
pub fn ReferralUserGrid(
    /// The list of users to display.
    users: Vec<ReferralListUser>,
    /// Message to display when the user list is empty.
    #[prop(into)]
    empty_message: String,
) -> impl IntoView {
    view! {
        <div class="referral_list active" role="tabpanel">
            {if users.is_empty() {
                view! {
                    <div class="empty_state">
                        <p>{empty_message}</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="users_grid">
                        <For
                            each=move || users.clone()
                            key=|u| u.id.clone()
                            children=|user| view! { <ReferralUserCard user=user/> }
                        />
                    </div>
                }.into_any()
            }}
        </div>
    }
}
