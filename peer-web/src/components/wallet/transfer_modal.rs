//! Transfer modal component.
//!
//! Multi-step modal for transferring tokens to another user:
//! 1. Select recipient (friends list or search)
//! 2. Enter amount and optional message
//! 3. Confirm and submit transfer

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use rust_decimal::Decimal;

use crate::api::wallet::{
    get_balance, list_transfer_recipients, search_transfer_recipient, transfer_tokens,
};
use crate::models::post::UserSearchResult;
use crate::models::profile::BasicUserInfo;
use crate::models::transaction::{calculate_fees, calculate_total_with_fees, format_balance};

/// Transfer modal steps.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TransferStep {
    SelectUser,
    EnterAmount,
    Confirm,
    Success,
}

/// Multi-step transfer modal.
#[component]
pub fn TransferModal<F1, F2>(
    /// Callback when modal is closed.
    on_close: F1,
    /// Callback when transfer succeeds.
    on_success: F2,
) -> impl IntoView
where
    F1: Fn() + 'static + Clone + Send + Sync,
    F2: Fn() + 'static + Clone + Send + Sync,
{
    let step = RwSignal::new(TransferStep::SelectUser);
    let selected_user = RwSignal::new(Option::<RecipientUser>::None);
    let amount = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let balance = RwSignal::new(Decimal::ZERO);
    let submitting = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    // Load balance on mount
    spawn_local(async move {
        if let Ok(bal) = get_balance().await {
            balance.set(bal);
        }
    });

    // Wrap callbacks in Callback type for reuse in view
    let on_close_cb = Callback::new(move |_: ()| on_close());
    let on_close_click = on_close_cb;
    let on_close_backdrop = on_close_cb;

    // Handle user selection
    let on_select_user = Callback::new(move |user: RecipientUser| {
        selected_user.set(Some(user));
        step.set(TransferStep::EnterAmount);
    });

    // Handle back navigation
    let on_back = Callback::new(move |_: ()| match step.get() {
        TransferStep::EnterAmount => step.set(TransferStep::SelectUser),
        TransferStep::Confirm => step.set(TransferStep::EnterAmount),
        _ => {}
    });

    // Handle continue to confirmation
    let on_continue = Callback::new(move |_: ()| {
        error.set(None);

        // Validate amount
        let amt_str = amount.get();
        let amt: Decimal = match amt_str.parse() {
            Ok(d) => d,
            Err(_) => {
                error.set(Some("Please enter a valid amount".into()));
                return;
            }
        };

        let min_amount = Decimal::new(1, 6); // 0.000001
        if amt < min_amount {
            error.set(Some("Minimum amount is 0.000001".into()));
            return;
        }

        let total = calculate_total_with_fees(amt);
        if total > balance.get() {
            error.set(Some("Insufficient balance".into()));
            return;
        }

        // Validate message
        let msg = message.get();
        if msg.len() > 500 {
            error.set(Some("Message exceeds 500 characters".into()));
            return;
        }

        step.set(TransferStep::Confirm);
    });

    // Handle transfer submission
    let on_success_clone = on_success.clone();
    let submit_transfer = Callback::new(move |_: ()| {
        let Some(user) = selected_user.get() else {
            return;
        };
        let amt: Decimal = amount.get().parse().unwrap_or_default();
        let msg = message.get();

        submitting.set(true);
        error.set(None);

        let on_success = on_success_clone.clone();

        spawn_local(async move {
            let msg_opt = if msg.is_empty() { None } else { Some(msg) };

            match transfer_tokens(user.id.clone(), amt, msg_opt).await {
                Ok(resp) if resp.is_success() => {
                    step.set(TransferStep::Success);
                    // Delay before calling success callback
                    #[cfg(feature = "hydrate")]
                    gloo_timers::callback::Timeout::new(1500, move || {
                        on_success();
                    })
                    .forget();

                    #[cfg(not(feature = "hydrate"))]
                    on_success();
                }
                Ok(_) => {
                    error.set(Some("Transfer failed. Please try again.".into()));
                    submitting.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        });
    });

    // Clone callback for final use
    let on_close_final = on_close_cb;

    view! {
        <div class="transfer-backdrop" on:click=move |_| on_close_backdrop.run(()) />

        <div class="transfer-dropdown" id="transferDropdown">
            <div class="transfer-form-screen">
                // Header
                <div class="transfer-header">
                    <h2 class="xl_font_size">
                        {move || match step.get() {
                            TransferStep::Confirm => "Summary",
                            TransferStep::Success => "Completed",
                            _ => "Transfer",
                        }}
                    </h2>
                    <Show when=move || step.get() != TransferStep::Success>
                        <button
                            class="close-transfer"
                            on:click=move |_| on_close_click.run(())
                        >
                            "×"
                        </button>
                    </Show>
                </div>

                // Balance header (not shown on success)
                <Show when=move || step.get() != TransferStep::Success>
                    <div class=move || format!(
                        "balance-header {}",
                        if step.get() == TransferStep::Confirm { "summary-header" } else { "" }
                    )>
                        <span class="md_font_size txt-color-gray bal_label">
                            {move || if step.get() == TransferStep::Confirm {
                                "Remaining balance"
                            } else {
                                "Your Balance"
                            }}
                        </span>
                        <span class="xl_font_size bold tbalance">
                            {move || {
                                if step.get() == TransferStep::Confirm {
                                    let amt: Decimal = amount.get().parse().unwrap_or_default();
                                    let total = calculate_total_with_fees(amt);
                                    format_balance(balance.get() - total)
                                } else {
                                    format_balance(balance.get())
                                }
                            }}
                        </span>
                    </div>
                </Show>

                // Error display
                <Show when=move || error.get().is_some()>
                    <div class="transfer-error md_font_size">
                        {move || error.get().unwrap_or_default()}
                    </div>
                </Show>

                // Step 1: Select user
                <Show when=move || step.get() == TransferStep::SelectUser>
                    <UserSelector on_select=on_select_user />
                </Show>

                // Step 2: Enter amount and message
                <Show when=move || step.get() == TransferStep::EnterAmount>
                    <AmountForm
                        user=selected_user.get().unwrap()
                        amount=amount
                        message=message
                        balance=balance.get()
                        on_back=on_back
                        on_continue=on_continue
                    />
                </Show>

                // Step 3: Confirmation
                <Show when=move || step.get() == TransferStep::Confirm>
                    <ConfirmTransfer
                        user=selected_user.get().unwrap()
                        amount=amount.get()
                        message=message.get()
                        submitting=submitting.get()
                        on_back=on_back
                        on_submit=submit_transfer
                    />
                </Show>

                // Success state
                <Show when=move || step.get() == TransferStep::Success>
                    <SuccessScreen on_close=on_close_final />
                </Show>
            </div>
        </div>
    }
}

/// Simplified user type for recipient selection.
#[derive(Clone)]
pub struct RecipientUser {
    pub id: String,
    pub username: String,
    pub slug: String,
    pub img: Option<String>,
}

impl From<BasicUserInfo> for RecipientUser {
    fn from(u: BasicUserInfo) -> Self {
        Self {
            id: u.userid,
            username: u.username,
            slug: u.slug.to_string(),
            img: u.img,
        }
    }
}

impl From<UserSearchResult> for RecipientUser {
    fn from(u: UserSearchResult) -> Self {
        Self {
            id: u.id,
            username: u.username,
            slug: u.slug,
            img: u.img,
        }
    }
}

impl RecipientUser {
    fn avatar_url(&self) -> String {
        self.img
            .as_ref()
            .map(|img| {
                if img.starts_with("media/") {
                    format!("/media/{}", img.trim_start_matches("media/"))
                } else {
                    format!("/media/{}", img)
                }
            })
            .unwrap_or_else(|| "/svg/noname.svg".to_string())
    }
}

/// User selector component (friends list + search).
#[component]
fn UserSelector(on_select: Callback<RecipientUser>) -> impl IntoView {
    let search_query = RwSignal::new(String::new());
    let friends = RwSignal::new(Vec::<RecipientUser>::new());
    let search_results = RwSignal::new(Vec::<RecipientUser>::new());
    let loading = RwSignal::new(true);
    let searching = RwSignal::new(false);

    // Load friends on mount
    spawn_local(async move {
        match list_transfer_recipients().await {
            Ok(response) => {
                let users: Vec<RecipientUser> = response
                    .affected_rows
                    .into_iter()
                    .map(RecipientUser::from)
                    .collect();
                friends.set(users);
            }
            Err(e) => {
                leptos::logging::error!("Failed to load friends: {}", e);
            }
        }
        loading.set(false);
    });

    // Debounced search using gloo-timers
    #[cfg(feature = "hydrate")]
    let search_timeout = StoredValue::new_local(Option::<gloo_timers::callback::Timeout>::None);

    let on_search_input = move |query: String| {
        search_query.set(query.clone());

        // Cancel previous timeout
        #[cfg(feature = "hydrate")]
        search_timeout.update_value(|t| {
            if let Some(timeout) = t.take() {
                drop(timeout);
            }
        });

        if query.len() < 2 {
            search_results.set(vec![]);
            return;
        }

        searching.set(true);

        #[cfg(feature = "hydrate")]
        {
            let timeout = gloo_timers::callback::Timeout::new(300, move || {
                let q = query.clone();
                spawn_local(async move {
                    match search_transfer_recipient(q).await {
                        Ok(users) => {
                            let results: Vec<RecipientUser> =
                                users.into_iter().map(RecipientUser::from).collect();
                            search_results.set(results);
                        }
                        Err(e) => {
                            leptos::logging::error!("Search failed: {}", e);
                        }
                    }
                    searching.set(false);
                });
            });

            search_timeout.set_value(Some(timeout));
        }

        // Fallback for SSR (shouldn't trigger but for safety)
        #[cfg(not(feature = "hydrate"))]
        {
            let q = query;
            spawn_local(async move {
                match search_transfer_recipient(q).await {
                    Ok(users) => {
                        let results: Vec<RecipientUser> =
                            users.into_iter().map(RecipientUser::from).collect();
                        search_results.set(results);
                    }
                    Err(e) => {
                        leptos::logging::error!("Search failed: {}", e);
                    }
                }
                searching.set(false);
            });
        }
    };

    view! {
        <div class="search_wrapper">
            <label class="md_font_size txt-color-gray">"Recipient username"</label>
            <div class="search-input-wrapper">
                <i class="peer-icon peer-icon-search" />
                <input
                    type="text"
                    placeholder="Search user..."
                    class="search-input"
                    prop:value=move || search_query.get()
                    on:input=move |e| {
                        let value = event_target_value(&e);
                        on_search_input(value);
                    }
                />
                <Show when=move || searching.get()>
                    <div class="spinner small" />
                </Show>
            </div>
        </div>

        <div class="user-list">
            // Show search results if searching
            <Show
                when=move || { !search_query.get().is_empty() && (search_query.get().len() >= 2) }
                fallback=move || {
                    view! {
                        // Show friends list
                        <Show when=move || loading.get()>
                            <div class="loading-users">
                                <div class="spinner" />
                                <span>"Loading friends..."</span>
                            </div>
                        </Show>
                        <Show when=move || !loading.get() && friends.get().is_empty()>
                            <div class="empty-users txt-color-gray">
                                "No friends found. Search for a user above."
                            </div>
                        </Show>
                        <For
                            each=move || friends.get()
                            key=|u| u.id.clone()
                            children=move |user| {
                                let user_clone = user.clone();
                                view! {
                                    <UserListItem
                                        user=user
                                        on_click=Callback::new(move |_: ()| on_select.run(user_clone.clone()))
                                    />
                                }
                            }
                        />
                    }
                }
            >
                {
                    view! {
                        <Show when=move || search_results.get().is_empty() && !searching.get()>
                            <div class="empty-users txt-color-gray">
                                "No users found"
                            </div>
                        </Show>
                        <For
                            each=move || search_results.get()
                            key=|u| u.id.clone()
                            children=move |user| {
                                let user_clone = user.clone();
                                view! {
                                    <UserListItem
                                        user=user
                                        on_click=Callback::new(move |_: ()| on_select.run(user_clone.clone()))
                                    />
                                }
                            }
                        />
                    }
                }
            </Show>
        </div>
    }
}

/// Single user item in the list.
#[component]
fn UserListItem(user: RecipientUser, on_click: Callback<()>) -> impl IntoView {
    let avatar = user.avatar_url();
    let username = user.username.clone();
    let slug = user.slug.clone();

    view! {
        <div class="user-item" on:click=move |_| on_click.run(())>
            <img
                src=avatar.clone()
                alt="User avatar"
                on:error=|e| {
                    if let Some(target) = e.target()
                        && let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                            img.set_src("/svg/noname.svg");
                        }
                }
            />
            <div class="info">
                <span class="username bold">"@"{username.clone()}</span>
                <span class="slug txt-color-gray">"#"{slug.clone()}</span>
            </div>
        </div>
    }
}

/// Amount and message form.
#[component]
fn AmountForm(
    user: RecipientUser,
    amount: RwSignal<String>,
    message: RwSignal<String>,
    balance: Decimal,
    on_back: Callback<()>,
    on_continue: Callback<()>,
) -> impl IntoView {
    let avatar = user.avatar_url();
    let username = user.username.clone();
    let slug = user.slug.clone();

    // Calculate fees reactively
    let fee_total = move || {
        let amt: Decimal = amount.get().parse().unwrap_or_default();
        calculate_fees(amt).total
    };

    let total_amount = move || {
        let amt: Decimal = amount.get().parse().unwrap_or_default();
        calculate_total_with_fees(amt)
    };

    let message_count = move || message.get().len();

    let fee_expanded = RwSignal::new(false);

    view! {
        // Selected recipient
        <div class="recipient-info">
            <label class="md_font_size txt-color-gray">"Sending to"</label>
            <div class="info">
                <img
                    src=avatar
                    alt="User avatar"
                    on:error=|e| {
                        if let Some(target) = e.target()
                            && let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                                img.set_src("/svg/noname.svg");
                            }
                    }
                />
                <span class="username bold">"@"{username}</span>
                <span class="slug txt-color-gray">"#"{slug}</span>
                <button
                    class="edit_btn"
                    on:click=move |_| on_back.run(())
                    title="Change recipient"
                >
                    <i class="peer-icon peer-icon-edit" />
                </button>
            </div>
        </div>

        // Amount input
        <div class="amount-input">
            <label class="amtlabel md_font_size txt-color-gray">
                "Enter amount"
                <span class="available-balance txt-color-gray">
                    " (Available: " {format_balance(balance)} ")"
                </span>
            </label>
            <input
                type="number"
                step="0.00000001"
                min="0.000001"
                placeholder="0.00000000"
                prop:value=move || amount.get()
                on:input=move |e| {
                    amount.set(event_target_value(&e));
                }
            />
            <div class="fee-section" class:close=move || !fee_expanded.get()>
                <div
                    class="fee-title md_font_size"
                    on:click=move |_| fee_expanded.update(|e| *e = !*e)
                >
                    <span class="fee-label txt-color-gray">"Transfer fee"</span>
                    <span class="fee-total">{move || format_balance(fee_total())}</span>
                </div>
                <div class="fee-breakdowns">
                    <div class="fee-item md_font_size">
                        <span class="label txt-color-gray">"2% Platform fee"</span>
                        <span class="value">{move || {
                            let amt: Decimal = amount.get().parse().unwrap_or_default();
                            format_balance(amt * Decimal::new(2, 2))
                        }}</span>
                    </div>
                    <div class="fee-item md_font_size">
                        <span class="label txt-color-gray">"1% Burned"</span>
                        <span class="value">{move || {
                            let amt: Decimal = amount.get().parse().unwrap_or_default();
                            format_balance(amt * Decimal::new(1, 2))
                        }}</span>
                    </div>
                    <div class="fee-item md_font_size">
                        <span class="label txt-color-gray">"1% to Inviter"</span>
                        <span class="value">{move || {
                            let amt: Decimal = amount.get().parse().unwrap_or_default();
                            format_balance(amt * Decimal::new(1, 2))
                        }}</span>
                    </div>
                </div>
                <div class="total_amount md_font_size">
                    <span class="label">"Total amount"</span>
                    <span class="final-total bold">{move || format_balance(total_amount())}</span>
                </div>
            </div>
        </div>

        // Message input
        <div class="message-wrap">
            <div class="label md_font_size txt-color-gray">
                <span>"Add a message (optional)"</span>
                <span class="char-count">{message_count}"/500"</span>
            </div>
            <div class="message_area">
                <textarea
                    placeholder="Letters, numbers, emojis. No links."
                    maxlength="500"
                    prop:value=move || message.get()
                    on:input=move |e| {
                        message.set(event_target_value(&e));
                    }
                />
            </div>
            <p class="ins_label md_font_size txt-color-gray">
                "Letters, numbers, emojis. No links."
            </p>
        </div>

        // Actions
        <div class="modal-actions">
            <button class="btn btn-secondary" on:click=move |_| on_back.run(())>
                "Back"
            </button>
            <button class="btn btn-primary" on:click=move |_| on_continue.run(())>
                "Continue"
            </button>
        </div>
    }
}

/// Confirmation screen before submitting.
#[component]
fn ConfirmTransfer(
    user: RecipientUser,
    amount: String,
    message: String,
    submitting: bool,
    on_back: Callback<()>,
    on_submit: Callback<()>,
) -> impl IntoView {
    let avatar = user.avatar_url();
    let username = user.username.clone();
    let slug = user.slug.clone();

    let amt: Decimal = amount.parse().unwrap_or_default();
    let fees = calculate_fees(amt);
    let total = calculate_total_with_fees(amt);

    // Clone message for use in closures
    let message_for_check = message.clone();
    let message_for_display = message;

    view! {
        // Recipient info (read-only)
        <div class="recipient-info">
            <label class="md_font_size txt-color-gray">"Sending to"</label>
            <div class="info">
                <img
                    src=avatar
                    alt="User avatar"
                    on:error=|e| {
                        if let Some(target) = e.target()
                            && let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                                img.set_src("/svg/noname.svg");
                            }
                    }
                />
                <span class="username bold">"@"{username}</span>
                <span class="slug txt-color-gray">"#"{slug}</span>
            </div>
        </div>

        // Amount summary
        <div class="amount-input summary-amount">
            <div class="fee-section">
                <div class="total_amount md_font_size">
                    <span class="label">"Total amount"</span>
                    <span class="final-total bold">{format_balance(total)}</span>
                </div>
                <div class="fee-item md_font_size">
                    <span class="label txt-color-gray">"Amount to recipient"</span>
                    <span class="value">{format_balance(amt)}</span>
                </div>
                <div class="fee-item md_font_size">
                    <span class="label txt-color-gray">"Fee"</span>
                    <span class="value">{format_balance(fees.total)}</span>
                </div>
            </div>
        </div>

        // Message preview
        <Show when=move || !message_for_check.is_empty()>
            <div class="message-wrap summary-message">
                <div class="label md_font_size txt-color-gray">
                    <i class="peer-icon peer-icon-message" />
                    " Message"
                </div>
                <div class="message_area">
                    {message_for_display.clone()}
                </div>
            </div>
        </Show>

        // Actions
        <div class="modal-actions">
            <button
                class="btn btn-secondary"
                disabled=submitting
                on:click=move |_| on_back.run(())
            >
                "Back"
            </button>
            <button
                class="btn btn-primary"
                disabled=submitting
                on:click=move |_| on_submit.run(())
            >
                {if submitting {
                    view! { <span class="spinner" />" Sending..." }.into_any()
                } else {
                    view! { "Submit transfer" }.into_any()
                }}
            </button>
        </div>
    }
}

/// Success screen after transfer.
#[component]
fn SuccessScreen(on_close: Callback<()>) -> impl IntoView {
    view! {
        <div class="success-screen">
            <div class="success-icon">
                <i class="peer-icon peer-icon-check-circle" />
            </div>
            <h3 class="xl_font_size">"Transfer Complete!"</h3>
            <p class="md_font_size txt-color-gray">
                "Your transfer was sent successfully."
            </p>
            <button class="btn btn-primary" on:click=move |_| on_close.run(())>
                "OK"
            </button>
        </div>
    }
}
