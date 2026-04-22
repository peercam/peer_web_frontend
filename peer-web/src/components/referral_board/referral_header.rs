//! Referral header component.
//!
//! Displays the referral program description and shareable link with copy-to-clipboard.

use leptos::prelude::*;

use crate::models::referral::ReferralInfoResponse;

/// Displays the referral program description and shareable link with copy button.
#[component]
pub fn ReferralHeader(info: ReferralInfoResponse) -> impl IntoView {
    let copied = RwSignal::new(false);

    let link_text = info.referral_link.clone().unwrap_or_default();

    let do_copy = {
        #[allow(unused_variables)]
        let text = link_text.clone();
        move || {
            #[cfg(feature = "hydrate")]
            {
                let text = text.clone();
                leptos::task::spawn_local(async move {
                    use wasm_bindgen_futures::JsFuture;
                    if let Some(clipboard) =
                        web_sys::window().and_then(|w| Some(w.navigator().clipboard()))
                    {
                        if JsFuture::from(clipboard.write_text(&text)).await.is_ok() {
                            copied.set(true);
                            // Auto-reset after 3 seconds
                            leptos::task::spawn_local(async move {
                                gloo_timers::future::TimeoutFuture::new(3000).await;
                                copied.set(false);
                            });
                        }
                    }
                });
            }
        }
    };

    let do_copy_click = do_copy.clone();
    let do_copy_keydown = do_copy;

    view! {
        <div class="referralBoard_header">
            <h1>"Referral Program"</h1>
            <p>
                "Invite a friend and earn "
                <em class="bold">"1% of their earnings"</em>
                " every time they transfer or cash out "
                <em class="bold">"forever"</em>
                ". The more you refer, the more you earn!"
            </p>
            <p>"Copy the code and share it with a friend. Make sure they enter it during registration."</p>
            <div
                class="referral_link_container"
                on:click=move |_| do_copy_click()
                role="button"
                tabindex="0"
                on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Enter" || ev.key() == " " {
                        do_copy_keydown();
                    }
                }
            >
                <span class="link_text">
                    {info.referral_link.clone().unwrap_or_else(|| "Loading...".into())}
                </span>
                <img src="/svg/refCopy.svg" alt="Copy icon" class="copy_icon"/>
            </div>
            <Show when=move || copied.get()>
                <div class="toast show" role="status" aria-live="polite">
                    <svg class="toast_icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"/>
                    </svg>
                    <span class="toast_message">"Link copied!"</span>
                </div>
            </Show>
        </div>
    }
}

/// Loading skeleton for the referral header.
#[component]
pub fn ReferralHeaderSkeleton() -> impl IntoView {
    view! {
        <div class="referralBoard_header referral-header-skeleton">
            <div class="skeleton-line skeleton-title"></div>
            <div class="skeleton-line skeleton-text"></div>
            <div class="skeleton-line skeleton-text short"></div>
            <div class="skeleton-line skeleton-link"></div>
        </div>
    }
}
