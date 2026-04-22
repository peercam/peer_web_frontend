//! Transaction item component.
//!
//! Displays a single transaction with expandable details.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;

use crate::models::transaction::{
    DeliveryDetails, ShopItemSpecs, ShopOrderDetails, Transaction, TransactionCategory,
    format_balance,
};
use crate::utils::constants::is_shop_account;

/// Context value carrying the currently signed-in viewer's user id, used by
/// the wallet's transaction rows to gate shop-account-only affordances such as
/// the lazy-loaded delivery panel.
#[derive(Clone, Copy)]
pub struct WalletViewerId(pub RwSignal<Option<String>>);

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
                                if let Some(target) = e.target()
                                    && let Ok(img) = target.dyn_into::<leptos::web_sys::HtmlImageElement>() {
                                        img.set_src("/svg/noname.svg");
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
        return ().into_any();
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
        return ().into_any();
    };

    if !matches!(tx.category(), TransactionCategory::P2pTransfer) {
        return ().into_any();
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
    let tx_id = tx.transaction_id.clone();
    let is_shop_purchase = matches!(tx.category(), TransactionCategory::ShopPurchase);

    // Show the delivery panel only when the viewer is the Peer Shop account.
    let viewer = use_context::<WalletViewerId>();
    let viewer_is_shop = move || {
        viewer
            .as_ref()
            .map(|v| {
                v.0.with(|opt| opt.as_deref().map(is_shop_account).unwrap_or(false))
            })
            .unwrap_or(false)
    };
    let show_delivery = move || is_shop_purchase && viewer_is_shop();

    view! {
        <div class="transaction_detail">
            <div class="transaction_detail_inner">
                <div class="price_detail_row md_font_size">
                    <span class="price_label txt-color-gray">"Transaction amount"</span>
                    <span class="price bold">{format_balance(amount)}</span>
                </div>
                <div class="price_detail_row md_font_size">
                    <span class="price_label txt-color-gray">"Base amount"</span>
                    <span class="price bold">{format_balance(net_amount)}</span>
                </div>

                // Fee breakdown
                {tx.fees.as_ref().map(|fees| view! {
                    <div class="price_detail_row md_font_size">
                        <span class="price_label txt-color-gray">"Fees included"</span>
                        <span class="price bold">{format_balance(fees.total)}</span>
                    </div>
                    <div class="price_detail_row fee-subitem">
                        <span class="price_label txt-color-gray">"2% to Peer Bank (platform fee)"</span>
                        <span class="price txt-color-gray">{format_balance(fees.peer)}</span>
                    </div>
                    <div class="price_detail_row fee-subitem">
                        <span class="price_label txt-color-gray">"1% Burned (removed from supply)"</span>
                        <span class="price txt-color-gray">{format_balance(fees.burn)}</span>
                    </div>
                    {fees.inviter.map(|inv| view! {
                        <div class="price_detail_row fee-subitem">
                            <span class="price_label txt-color-gray">"1% to your Inviter"</span>
                            <span class="price txt-color-gray">{format_balance(inv)}</span>
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

                // Delivery panel (Peer Shop account only, lazy-loaded on first
                // render of this expanded view).
                <Show when=show_delivery fallback=|| ()>
                    <DeliveryPanel transaction_id=tx_id.clone() />
                </Show>
            </div>
        </div>
    }
}

/// Lazy-loaded delivery information panel for shop purchases.
///
/// Mounted only when the row is expanded and the viewer is the Peer Shop
/// account, so the network request fires on first expand and is reused for
/// the lifetime of the expanded row.
#[component]
fn DeliveryPanel(transaction_id: String) -> impl IntoView {
    let state = RwSignal::new(DeliveryFetch::Loading);

    let id_for_fetch = transaction_id.clone();
    spawn_local(async move {
        match crate::api::shop::get_shop_order_details(id_for_fetch).await {
            Ok(resp) => {
                let order = resp.affected_rows.and_then(|rows| rows.into_iter().next());
                match order {
                    Some(o) => state.set(DeliveryFetch::Loaded(Box::new(o))),
                    None => state.set(DeliveryFetch::Empty),
                }
            }
            Err(_) => state.set(DeliveryFetch::Errored),
        }
    });

    view! {
        {move || match state.get() {
            DeliveryFetch::Loading => view! {
                <div class="delivery_info_container">
                    <div class="price_detail_row md_font_size txt-color-gray">
                        "Loading delivery info…"
                    </div>
                </div>
            }.into_any(),
            DeliveryFetch::Empty => view! {
                <div class="delivery_info_container">
                    <div class="price_detail_row md_font_size txt-color-gray">
                        "Unable to load delivery info"
                    </div>
                </div>
            }.into_any(),
            DeliveryFetch::Errored => view! {
                <div class="delivery_info_container">
                    <div class="price_detail_row md_font_size txt-color-gray">
                        "Error loading delivery info"
                    </div>
                </div>
            }.into_any(),
            DeliveryFetch::Loaded(order) => render_delivery_panel(*order).into_any(),
        }}
    }
}

#[derive(Clone)]
enum DeliveryFetch {
    Loading,
    Empty,
    Errored,
    Loaded(Box<ShopOrderDetails>),
}

fn render_delivery_panel(order: ShopOrderDetails) -> impl IntoView {
    let item_display = format_item_display(&order.shop_item_id, order.shop_item_specs.as_ref());
    let delivery = order.delivery_details.unwrap_or(DeliveryDetails {
        name: None,
        email: None,
        addressline1: None,
        addressline2: None,
        city: None,
        zipcode: None,
        country: None,
    });

    let address = compose_address(&delivery);
    let na = || "N/A".to_string();
    let name = delivery
        .name
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(na);
    let email = delivery
        .email
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(na);

    view! {
        <div class="delivery_info_container">
            <div class="delivery_label md_font_size bold">
                <i class="peer-icon peer-icon-delivery-info" />
                " Delivery information"
            </div>
            <div class="price_detail_row md_font_size">
                <span class="price_label txt-color-gray">"Item"</span>
                <span class="price bold">{item_display}</span>
            </div>
            <div class="price_detail_row md_font_size">
                <span class="price_label txt-color-gray">"Name"</span>
                <span class="price bold">{name}</span>
            </div>
            <div class="price_detail_row md_font_size">
                <span class="price_label txt-color-gray">"Email"</span>
                <span class="price bold">{email}</span>
            </div>
            <div class="price_detail_row md_font_size">
                <span class="price_label txt-color-gray">"Address"</span>
                <span class="price bold">{address}</span>
            </div>
        </div>
    }
}

fn compose_address(d: &DeliveryDetails) -> String {
    let parts: Vec<String> = [
        d.addressline1.as_deref(),
        d.addressline2.as_deref(),
        d.city.as_deref(),
        d.zipcode.as_deref(),
        d.country.as_deref(),
    ]
    .into_iter()
    .filter_map(|opt| {
        opt.map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    })
    .collect();

    if parts.is_empty() {
        "N/A".to_string()
    } else {
        parts.join(", ")
    }
}

// TODO(peer-shop-firebase): replace with product lookup once the Peer Shop
// Firebase integration lands. See docs/plans/peer-shop/peer-shop-implementation.md.
fn format_item_display(shop_item_id: &str, specs: Option<&ShopItemSpecs>) -> String {
    let mut out = format!("Shop item #{shop_item_id}");
    if let Some(size) = specs
        .and_then(|s| s.size.as_deref())
        .filter(|s| !s.is_empty())
    {
        out.push_str(", size ");
        out.push_str(size);
    }
    out
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
