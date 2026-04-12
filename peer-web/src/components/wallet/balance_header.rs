//! Balance header component.
//!
//! Displays the user's current token balance with transfer and reload actions.

use leptos::prelude::*;
use rust_decimal::Decimal;

use crate::models::transaction::format_balance;

/// Balance header with transfer and reload buttons.
#[component]
pub fn BalanceHeader(
    /// Current balance to display.
    balance: Decimal,
    /// Callback when transfer button is clicked.
    on_transfer: impl Fn() + 'static + Clone,
    /// Callback when reload button is clicked.
    on_reload: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let on_transfer_clone = on_transfer.clone();
    let on_reload_clone = on_reload.clone();

    view! {
        <div class="wallet_header">
            <h2 class="wallet_heading xl_font_size">"Available balance"</h2>
            <div class="balance_transfer">
                <div class="wallet_balance">
                    <img src="/svg/logo_sw.svg" alt="peer token" class="logo" />
                    <span id="token" class="bold xxxl_font_size">
                        {format_balance(balance)}
                    </span>
                </div>

                <div class="wallet_transfer">
                    <a
                        href="#"
                        id="openTransferDropdown"
                        class="md_font_size"
                        on:click=move |e| {
                            e.prevent_default();
                            on_transfer_clone();
                        }
                    >
                        <span>"Transfer "<em>"to user"</em></span>
                        <i class="peer-icon peer-icon-arrow-right" />
                    </a>
                </div>

                <div class="wallet_reload">
                    <a
                        id="reloadTransactions"
                        href="#"
                        class="md_font_size"
                        on:click=move |e| {
                            e.prevent_default();
                            on_reload_clone();
                        }
                    >
                        "Reload "
                        <i class="peer-icon peer-icon-refresh-alt" />
                    </a>
                </div>
            </div>
        </div>
    }
}

/// Skeleton loading state for balance header.
#[component]
pub fn BalanceSkeleton() -> impl IntoView {
    view! {
        <div class="wallet_header skeleton">
            <h2 class="wallet_heading xl_font_size">"Available balance"</h2>
            <div class="balance_transfer">
                <div class="wallet_balance">
                    <img src="/svg/logo_sw.svg" alt="peer token" class="logo" />
                    <span class="bold xxxl_font_size skeleton-text">"----"</span>
                </div>
            </div>
        </div>
    }
}
