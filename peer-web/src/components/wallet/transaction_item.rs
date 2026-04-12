//! Transaction item component.
//!
//! Displays a single transaction with expandable details.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

use crate::models::transaction::{format_decimal, Transaction, TransactionCategory};

/// Single transaction item with expandable details.
#[component]
pub fn TransactionItem(
    /// The transaction to display.
    tx: Transaction,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let tx_for_detail = tx.clone();

    let is_incoming = tx.is_incoming();
    let direction_class = if is_incoming { "trans_in" } else { "trans_out" };
    let display_amount = tx.display_amount();
    let category = tx.category();
    let title = category.display_title(is_incoming);

    // Format date
    let date_display = format_date(&tx.createdat);

    view! {
        <div
            class=move || format!(
                "tarnsaction_item {} {}",
                direction_class,
                if expanded.get() { "open" } else { "" }
            )
        >
            <div
                class="transaction_record"
                on:click=move |_| expanded.update(|e| *e = !*e)
            >
                <div class="transaction_info">
                    <TransactionMedia tx=tx.clone() />
                    <div class="transaction_content">
                        <div class="tinfo md_font_size">
                            <span class="title bold">{title}</span>
                            <TransactionUserInfo tx=tx.clone() />
                        </div>
                        <ShortMessage tx=tx.clone() />
                    </div>
                </div>
                <div class="transaction_date md_font_size txt-color-gray">
                    {date_display}
                </div>
                <div class="transaction_price xl_font_size bold">
                    {display_amount}
                </div>
            </div>

            <Show when=move || expanded.get()>
                <TransactionDetail tx=tx_for_detail.clone() />
            </Show>
        </div>
    }
}

/// Transaction media/icon component.
#[component]
fn TransactionMedia(tx: Transaction) -> impl IntoView {
    let category = tx.category();

    match category {
        TransactionCategory::P2pTransfer => {
            let user = tx.counterparty();
            let avatar_url = user.avatar_url();
            let is_hidden = user.is_hidden();

            view! {
                <div class=format!(
                    "transaction_media {}",
                    if is_hidden { "profile_status_hidden" } else { "" }
                )>
                    <span class="wrap_img">
                        <img
                            class="userimg"
                            src=avatar_url.clone()
                            alt="User avatar"
                            on:error=|e| {
                                if let Some(target) = e.target() {
                                    if let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                                        img.set_src("/svg/noname.svg");
                                    }
                                }
                            }
                        />
                    </span>
                </div>
            }
            .into_any()
        }
        _ => {
            let icon_class = category.icon_class();
            view! {
                <div class="transaction_media">
                    <i class=format!("peer-icon {}", icon_class) />
                </div>
            }
            .into_any()
        }
    }
}

/// User info display for P2P transfers.
#[component]
fn TransactionUserInfo(tx: Transaction) -> impl IntoView {
    if !matches!(tx.category(), TransactionCategory::P2pTransfer) {
        return view! {}.into_any();
    }

    let user = tx.counterparty();
    let username = user.username.clone();
    let slug = user.slug.clone();

    view! {
        <span class="user_name bold italic">"@"{username}</span>
        " "
        <span class="user_slug txt-color-gray">"#"{slug}</span>
    }
    .into_any()
}

/// Short message preview for transactions.
#[component]
fn ShortMessage(tx: Transaction) -> impl IntoView {
    let Some(short_msg) = tx.short_message() else {
        return view! {}.into_any();
    };

    if !matches!(tx.category(), TransactionCategory::P2pTransfer) {
        return view! {}.into_any();
    }

    view! {
        <div class="message txt-color-gray">
            <i class="peer-icon peer-icon-message" />
            {short_msg}
        </div>
    }
    .into_any()
}

/// Expanded transaction detail view.
#[component]
fn TransactionDetail(tx: Transaction) -> impl IntoView {
    let amount = tx.amount();
    let net_amount = tx.net_amount();

    view! {
        <div class="transaction_detail">
            <div class="transaction_detail_inner">
                <div class="price_detail_row md_font_size">
                    <span class="price_label txt-color-gray">"Transaction amount"</span>
                    <span class="price bold">{format_decimal(amount)}</span>
                </div>
                <div class="price_detail_row md_font_size">
                    <span class="price_label txt-color-gray">"Base amount"</span>
                    <span class="price bold">{format_decimal(net_amount)}</span>
                </div>

                // Fee breakdown
                {tx.fees.as_ref().map(|fees| view! {
                    <div class="price_detail_row md_font_size">
                        <span class="price_label txt-color-gray">"Fees included"</span>
                        <span class="price bold">{format_decimal(fees.total)}</span>
                    </div>
                    <div class="price_detail_row fee-subitem">
                        <span class="price_label txt-color-gray">"2% to Peer Bank (platform fee)"</span>
                        <span class="price txt-color-gray">{format_decimal(fees.peer)}</span>
                    </div>
                    <div class="price_detail_row fee-subitem">
                        <span class="price_label txt-color-gray">"1% Burned (removed from supply)"</span>
                        <span class="price txt-color-gray">{format_decimal(fees.burn)}</span>
                    </div>
                    {fees.inviter.map(|inv| view! {
                        <div class="price_detail_row fee-subitem">
                            <span class="price_label txt-color-gray">"1% to your Inviter"</span>
                            <span class="price txt-color-gray">{format_decimal(inv)}</span>
                        </div>
                    })}
                })}

                // Full message
                {tx.message.as_ref().filter(|m| !m.is_empty()).map(|msg| view! {
                    <div class="message_row">
                        <span class="message_label md_font_size txt-color-gray">
                            <i class="peer-icon peer-icon-message" />
                            " Message:"
                        </span>
                        <span class="message_body">{msg.clone()}</span>
                    </div>
                })}
            </div>
        </div>
    }
}

/// Format a timestamp string for display.
fn format_date(timestamp: &str) -> String {
    // Try to parse ISO 8601 timestamp and format nicely
    // Format: "10 Jun 2025, 04:20"
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%d %b %Y, %H:%M").to_string()
    } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.f")
    {
        dt.format("%d %b %Y, %H:%M").to_string()
    } else {
        // Fallback: return as-is or try simple parse
        timestamp.split('T').next().unwrap_or(timestamp).to_string()
    }
}
