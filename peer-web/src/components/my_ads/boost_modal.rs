//! Boost Post modal component.
//!
//! Multi-step modal for promoting a post as a pinned advertisement.
//! Steps: Warning (if reported/hidden) → Preview → Eligibility/Pay.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::ads::advertise_post_pinned;
use crate::api::wallet::get_balance;
use crate::components::toast::use_toast;
use crate::models::post::Post;

/// Pinned ad cost in tokens.
const PINNED_AD_COST: f64 = 200.0;

/// Boost Post modal steps.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BoostStep {
    Warning,
    Preview,
    Eligibility,
}

/// Modal for boosting (promoting) a post as a pinned advertisement.
#[component]
pub fn BoostPostModal(
    /// The post to boost.
    post: Post,
    /// Whether the post has active reports or is hidden.
    #[prop(default = false)]
    is_reported: bool,
    /// Callback when the modal should close.
    on_close: Callback<()>,
    /// Callback on successful promotion.
    #[prop(optional)]
    on_success: Option<Callback<()>>,
) -> impl IntoView {
    let _toast = use_toast();
    let post_title = post.title.clone();
    let post_thumbnail = post.cover.clone().or_else(|| post.media.clone());

    // Start on warning step if post is reported/hidden, otherwise go to preview
    let current_step = RwSignal::new(if is_reported {
        BoostStep::Warning
    } else {
        BoostStep::Preview
    });

    let is_submitting = RwSignal::new(false);
    let balance = RwSignal::new(Option::<f64>::None);

    // Fetch balance when reaching eligibility step
    let fetch_balance = move || {
        spawn_local(async move {
            match get_balance().await {
                Ok(bal) => {
                    // Convert Decimal to f64
                    use std::str::FromStr;
                    let bal_f64 = f64::from_str(&bal.to_string()).unwrap_or(0.0);
                    balance.set(Some(bal_f64));
                }
                Err(e) => {
                    leptos::logging::error!("Failed to fetch balance: {:?}", e);
                    balance.set(Some(0.0));
                }
            }
        });
    };

    let go_to_eligibility = move || {
        current_step.set(BoostStep::Eligibility);
        fetch_balance();
    };

    let post_id_signal = RwSignal::new(post.id.clone());
    let close_triggered = RwSignal::new(false);

    // Effect to close when triggered
    Effect::new(move |_| {
        if close_triggered.get() {
            on_close.run(());
        }
    });

    let do_promote = Callback::new(move |_: ()| {
        if is_submitting.get() {
            return;
        }
        is_submitting.set(true);
        let pid = post_id_signal.get();

        spawn_local(async move {
            match advertise_post_pinned(pid).await {
                Ok(response) => {
                    if response.meta.status == "success" {
                        if let Some(cb) = on_success {
                            cb.run(());
                        }
                        close_triggered.set(true);
                    }
                }
                Err(_) => {}
            }
            is_submitting.set(false);
        });
    });

    view! {
        <div class="boost-modal-overlay" on:click=move |_| on_close.run(())>
            <div class="boost-modal" on:click=|e| e.stop_propagation()>
                <button class="boost-modal-close" on:click=move |_| on_close.run(())>
                    <i class="peer-icon peer-icon-close"></i>
                </button>

                // Step 1: Warning
                <Show when=move || current_step.get() == BoostStep::Warning>
                    <div class="boost-step">
                        <h2>"Warning"</h2>
                        <p class="boost-warning-text">
                            "Your post has been reported or is currently hidden. "
                            "Promoting it may not yield the best results."
                        </p>
                        <div class="boost-actions">
                            <button
                                class="button btn-blue"
                                on:click=move |_| current_step.set(BoostStep::Preview)
                            >
                                "Promote anyway"
                            </button>
                            <button
                                class="button btn-white"
                                on:click=move |_| on_close.run(())
                            >
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </Show>

                // Step 2: Preview
                <Show when=move || current_step.get() == BoostStep::Preview>
                    <div class="boost-step">
                        <h2>"Post Preview"</h2>
                        <div class="boost-preview-card">
                            <div class="boost-preview-image">
                                {match post_thumbnail.clone() {
                                    Some(url) => view! {
                                        <img src=url alt="Post preview"/>
                                    }.into_any(),
                                    None => view! {
                                        <div class="post-image-placeholder"></div>
                                    }.into_any(),
                                }}
                                <div class="pin-badge">
                                    <img src="/svg/pin-icon.svg" alt="Pinned"/>
                                </div>
                            </div>
                            <h3>{post_title.clone()}</h3>
                        </div>
                        <div class="boost-actions">
                            <button
                                class="button btn-blue"
                                on:click=move |_| go_to_eligibility()
                            >
                                "Next"
                            </button>
                        </div>
                    </div>
                </Show>

                // Step 3: Eligibility / Pay
                <Show when=move || current_step.get() == BoostStep::Eligibility>
                    <div class="boost-step">
                        <h2>"Promote Post"</h2>
                        <div class="boost-cost-info">
                            <p>"Pinned advertisement cost:"</p>
                            <div class="ads-tokens-count">
                                <img src="/svg/logo_sw.svg" alt="tokens"/>
                                <span class="bold xl-font-size">"200"</span>
                            </div>
                        </div>

                        {move || {
                            match balance.get() {
                                None => view! {
                                    <div class="loading-indicator">
                                        <img src="/svg/logo_farbe.svg" alt="Loading..." class="loading-spinner"/>
                                    </div>
                                }.into_any(),
                                Some(bal) if bal >= PINNED_AD_COST => view! {
                                    <div class="boost-eligible">
                                        <p class="boost-balance">
                                            "Your balance: "
                                            <span class="bold">{format!("{:.0}", bal)}</span>
                                            " tokens"
                                        </p>
                                        <div class="boost-actions">
                                            <button
                                                class="button btn-blue"
                                                disabled=move || is_submitting.get()
                                                on:click=move |_| do_promote.run(())
                                            >
                                                {move || if is_submitting.get() {
                                                    "Processing..."
                                                } else {
                                                    "Pay 200 tokens"
                                                }}
                                            </button>
                                        </div>
                                    </div>
                                }.into_any(),
                                Some(bal) => view! {
                                    <div class="boost-ineligible">
                                        <p class="boost-balance error">
                                            "Your balance: "
                                            <span class="bold">{format!("{:.0}", bal)}</span>
                                            " tokens — insufficient funds"
                                        </p>
                                        <div class="boost-actions">
                                            <a href="/wallet" class="button btn-white">
                                                "Go to Wallet"
                                            </a>
                                        </div>
                                    </div>
                                }.into_any(),
                            }
                        }}
                    </div>
                </Show>
            </div>
        </div>
    }
}
