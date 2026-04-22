//! Wallet page.
//!
//! Displays the user's token balance, transfer functionality,
//! and transaction history with infinite scroll.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;

use crate::api::profile::get_profile;
use crate::api::wallet::get_balance;
use crate::components::auth_guard::AuthGuard;
use crate::components::layout::SiteShell;
use crate::components::wallet::{
    BalanceHeader, BalanceSkeleton, TransactionHistory, TransferModal, WalletViewerId,
};
use crate::components::widgets::{MainMenu, ProfileWidget, VersionWidget};

/// Wallet page - token balance and transaction history.
///
/// Requires authentication to access.
#[component]
pub fn WalletPage() -> impl IntoView {
    let show_transfer_modal = RwSignal::new(false);

    // Provide the viewer's user id so transaction rows can gate shop-only
    // affordances (e.g. the delivery panel). Resolved lazily from the
    // authenticated profile.
    let viewer_user_id = RwSignal::new(Option::<String>::None);
    provide_context(WalletViewerId(viewer_user_id));
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(profile) = get_profile(None, None).await {
                viewer_user_id.set(Some(profile.id));
            }
        });
    });

    // Balance resource with manual refresh trigger
    let refresh_trigger = RwSignal::new(0u32);
    let balance = Resource::new(
        move || refresh_trigger.get(),
        |_| async move { get_balance().await.unwrap_or_default() },
    );

    let refresh_balance = move || {
        refresh_trigger.update(|n| *n += 1);
    };

    view! {
        <Title text="Wallet - Peer Network"/>
        <AuthGuard>
            <SiteShell id="wallet-page" modifier="wallet-layout">
                <WalletHeader/>

                <aside class="left-sidebar left-sidebar-wallet">
                    <div class="inner-scroll">
                        // Empty left sidebar per design
                    </div>
                </aside>

                <main id="main" class="site-main site-main-wallet">
                    <div class="wallet_main">
                        <Suspense fallback=move || view! { <BalanceSkeleton/> }>
                            {move || {
                                balance.get().map(|bal| {
                                    view! {
                                        <BalanceHeader
                                            balance=bal
                                            on_transfer=move || show_transfer_modal.set(true)
                                            on_reload=move || refresh_balance()
                                        />
                                    }
                                })
                            }}
                        </Suspense>

                        <TransactionHistory />
                    </div>
                </main>

                <aside class="right-sidebar right-sidebar-wallet">
                    <div class="inner-scroll">
                        <ProfileWidget />
                        <MainMenu />
                        <NewPostButton />
                        <VersionWidget />
                    </div>
                </aside>
            </SiteShell>

            // Transfer modal
            <Show when=move || show_transfer_modal.get()>
                <TransferModal
                    on_close=move || show_transfer_modal.set(false)
                    on_success=move || {
                        show_transfer_modal.set(false);
                        refresh_balance();
                    }
                />
            </Show>
        </AuthGuard>
    }
}

/// Wallet page header with icon and title.
#[component]
fn WalletHeader() -> impl IntoView {
    view! {
        <header class="site-header header-wallet">
            <h1 id="h1">
                <i class="peer-icon peer-icon-wallet-filled" />
                " Wallet"
            </h1>
        </header>
    }
}

/// New post button for sidebar.
#[component]
fn NewPostButton() -> impl IntoView {
    view! {
        <div class="new-post-widget">
            <a href="/newpost" class="new-post-btn">
                <i class="peer-icon peer-icon-plus" />
                <span>"New Post"</span>
            </a>
        </div>
    }
}
