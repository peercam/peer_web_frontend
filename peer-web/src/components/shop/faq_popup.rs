//! FAQ popup with shop policies.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;

/// FAQ modal overlay with shop policies (order processing, delivery, returns, etc.).
#[component]
pub fn FaqPopup(is_open: RwSignal<bool>) -> impl IntoView {
    let on_close = move |_| is_open.set(false);

    let on_backdrop_click = move |ev: leptos::ev::MouseEvent| {
        // Close when clicking the backdrop itself, not content
        if let Some(target) = ev.target()
            && let Ok(el) = target.dyn_into::<web_sys::HtmlElement>()
            && el.class_list().contains("faq-overlay")
        {
            is_open.set(false);
        }
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="faq-overlay" on:click=on_backdrop_click>
                <div class="faq-popup">
                    <div class="faq-content">
                        <div class="faq-header">
                            <h2 class="faq-title">"FAQ's"</h2>
                            <button class="close-checkout" on:click=on_close>
                                "\u{00D7}"
                            </button>
                        </div>
                        <div class="faq-scroll">
                            // Order Processing
                            <section class="faq-section">
                                <h3 class="bold">"Order Processing"</h3>
                                <p class="txt-color-gray">
                                    "Your order is processed as soon as your payment is confirmed. You\u{2019}ll receive a confirmation email with delivery details within 1\u{2013}3 days."
                                </p>
                            </section>

                            // Delivery Information & Time (highlighted)
                            <section class="faq-section faq-highlighted">
                                <h3 class="bold">"Delivery information & time"</h3>
                                <ul>
                                    <li>"Delivery costs are already included in the item price."</li>
                                    <li>"Shipping times depend on the delivery provider and your location. You\u{2019}ll receive full delivery details by email once your order has been dispatched."</li>
                                </ul>
                            </section>

                            // Return Policy
                            <section class="faq-section">
                                <h3 class="bold">"Return policy"</h3>
                                <ul class="txt-color-gray">
                                    <li>"If you\u{2019}d like to return an item, we can offer an exchange."</li>
                                    <li>"Please note: Peer Tokens cannot be refunded."</li>
                                </ul>
                            </section>

                            // Delivery Area
                            <section class="faq-section">
                                <h3 class="bold">"Delivery area"</h3>
                                <p class="txt-color-gray">
                                    "Delivery is currently available only within Germany."
                                </p>
                            </section>

                            // Need Help?
                            <section class="faq-section">
                                <h3 class="bold">"Need help?"</h3>
                                <p class="txt-color-gray">
                                    "If you have any issues with your order, delivery, or product, feel free to reach out to us:"
                                </p>
                                <ul class="txt-color-gray">
                                    <li>
                                        "Email: "
                                        <a href="mailto:help.peernetwork@gmail.com" class="faq-link">
                                            "help.peernetwork@gmail.com"
                                        </a>
                                    </li>
                                </ul>
                            </section>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
