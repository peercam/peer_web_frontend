//! Transaction history component with infinite scroll.
//!
//! Displays the user's transaction history with lazy loading.

use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashSet;

use crate::api::wallet::get_transaction_history;
use crate::components::wallet::TransactionItem;
use crate::models::transaction::Transaction;

const LIMIT: i32 = 20;

/// Transaction history with infinite scroll.
#[component]
pub fn TransactionHistory() -> impl IntoView {
    let transactions = RwSignal::new(Vec::<Transaction>::new());
    let offset = RwSignal::new(0i32);
    let has_more = RwSignal::new(true);
    let loading = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);
    let seen_ids = RwSignal::new(HashSet::<String>::new());

    // Load more transactions
    let load_more = move || {
        if loading.get() || !has_more.get() {
            return;
        }

        loading.set(true);
        error.set(None);

        spawn_local(async move {
            match get_transaction_history(offset.get(), LIMIT).await {
                Ok(new_txs) => {
                    let count = new_txs.len() as i32;

                    // Filter duplicates using seen IDs
                    let mut ids = seen_ids.get();
                    let unique_txs: Vec<_> = new_txs
                        .into_iter()
                        .filter(|tx| ids.insert(tx.transaction_id.clone()))
                        .collect();
                    seen_ids.set(ids);

                    // Append unique transactions
                    transactions.update(|txs| txs.extend(unique_txs));
                    offset.update(|o| *o += count);

                    // Check if we've reached the end
                    if count < LIMIT {
                        has_more.set(false);
                    }
                }
                Err(e) => {
                    leptos::logging::error!("Failed to load transactions: {}", e);
                    error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        });
    };

    // Initial load
    Effect::new(move |_| {
        load_more();
    });

    // Setup intersection observer for infinite scroll (client-side only)
    let sentinel_ref = NodeRef::<leptos::html::Div>::new();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        use std::sync::{Arc, Mutex};
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;

        let Some(sentinel) = sentinel_ref.get() else {
            return;
        };

        let callback = Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
            for entry in entries.iter() {
                let entry: web_sys::IntersectionObserverEntry = entry.unchecked_into();
                if entry.is_intersecting() && !loading.get() && has_more.get() {
                    load_more();
                }
            }
        });

        let options = web_sys::IntersectionObserverInit::new();
        options.set_root_margin("100% 0px 100% 0px");
        options.set_threshold(&JsValue::from_f64(0.01));

        if let Ok(observer) = web_sys::IntersectionObserver::new_with_options(
            callback.as_ref().unchecked_ref(),
            &options,
        ) {
            observer.observe(&sentinel);
            callback.forget();

            let observer = Arc::new(Mutex::new(Some(observer)));
            on_cleanup(move || {
                if let Ok(mut guard) = observer.lock() {
                    if let Some(obs) = guard.take() {
                        obs.disconnect();
                    }
                }
            });
        }
    });

    view! {
        <div class="wallet_transactions">
            <h3 class="transaction_heading xl_font_size">"Transactions"</h3>
            <div id="history-container" class="transaction_lists">
                <For
                    each=move || transactions.get()
                    key=|tx| tx.transaction_id.clone()
                    children=move |tx| view! { <TransactionItem tx=tx /> }
                />

                // Error state
                <Show when=move || error.get().is_some()>
                    <div class="transaction-error md_font_size">
                        <p class="error-message">
                            {move || error.get().unwrap_or_default()}
                        </p>
                        <button
                            class="retry-btn"
                            on:click=move |_| load_more()
                        >
                            "Retry"
                        </button>
                    </div>
                </Show>

                // Empty state
                <Show when=move || !loading.get() && transactions.get().is_empty() && error.get().is_none()>
                    <div class="transaction-empty md_font_size txt-color-gray">
                        <i class="peer-icon peer-icon-wallet" />
                        <p>"No transactions yet"</p>
                    </div>
                </Show>

                // Sentinel for infinite scroll
                <div
                    id="history-sentinel"
                    node_ref=sentinel_ref
                    style="height: 20px; width: 100%"
                />

                // Loading indicator
                <Show when=move || loading.get()>
                    <div class="transaction-loading">
                        <div class="spinner" />
                        <span class="md_font_size">"Loading..."</span>
                    </div>
                </Show>

                // End of list indicator
                <Show when=move || !has_more.get() && !transactions.get().is_empty()>
                    <div class="transaction-end md_font_size txt-color-gray">
                        "No more transactions"
                    </div>
                </Show>
            </div>
        </div>
    }
}
