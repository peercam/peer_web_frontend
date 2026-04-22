//! Multi-step checkout popup for Peer Shop purchases.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;

use crate::api::shop::perform_shop_order;
use crate::models::post::Post;
use crate::models::shop::{
    DeliveryFormData, DeliveryFormErrors, ShopProduct, validate_delivery_form,
};
use crate::models::transaction::calculate_fees;

/// Checkout step state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckoutStep {
    DeliveryForm,
    ReviewPayment,
}

/// Payment state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaymentState {
    Idle,
    Processing,
    Success,
    Failed,
}

/// Multi-step checkout overlay for purchasing shop products.
#[component]
pub fn CheckoutPopup(
    is_open: RwSignal<bool>,
    post: Post,
    product: ShopProduct,
    product_price: u32,
) -> impl IntoView {
    let step = RwSignal::new(CheckoutStep::DeliveryForm);
    let payment_state = RwSignal::new(PaymentState::Idle);

    // Form data signals
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let address_line_1 = RwSignal::new(String::new());
    let address_line_2 = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let zipcode = RwSignal::new(String::new());
    let selected_size = RwSignal::new(None::<String>);
    let errors = RwSignal::new(DeliveryFormErrors::default());
    let show_fees = RwSignal::new(false);

    let has_sizes = product.has_sizes();
    let sizes_data = StoredValue::new(product.sizes.clone().unwrap_or_default());

    let post_title = StoredValue::new(post.title.clone());
    let post_desc = StoredValue::new(post.mediadescription.clone().unwrap_or_default());
    let post_id = StoredValue::new(post.id.clone());

    // Get first image from media JSON
    let post_image = {
        let media_str = post.media.clone().unwrap_or_default();
        if let Ok(parsed) = serde_json::from_str::<Vec<serde_json::Value>>(&media_str) {
            parsed
                .first()
                .and_then(|v| v.get("path"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string())
        } else {
            post.cover.clone().or(post.media.clone())
        }
    }
    .unwrap_or_default();
    let post_image = StoredValue::new(post_image);

    let price_display = StoredValue::new(format!("{}", product_price));
    let price_display_review = StoredValue::new(format!("{}.00", product_price));

    // Fee breakdown
    let fees = calculate_fees(rust_decimal::Decimal::from(product_price));
    let fee_total = StoredValue::new(format!("{:.2}", fees.total));
    let fee_burn = StoredValue::new(format!("{:.2}", fees.burn));
    let fee_peer = StoredValue::new(format!("{:.2}", fees.peer));
    let fee_inviter = StoredValue::new(
        fees.inviter
            .map(|i| format!("{:.2}", i))
            .unwrap_or_else(|| "0.00".to_string()),
    );

    let reset_state = move || {
        is_open.set(false);
        step.set(CheckoutStep::DeliveryForm);
        payment_state.set(PaymentState::Idle);
    };

    let on_close = move |_: leptos::ev::MouseEvent| {
        reset_state();
    };

    let on_backdrop_click = move |ev: leptos::ev::MouseEvent| {
        if let Some(target) = ev.target()
            && let Ok(el) = target.dyn_into::<web_sys::HtmlElement>()
                && el.class_list().contains("checkout-overlay") {
                    reset_state();
                }
    };

    let size_required = has_sizes;

    let do_next = move || {
        let form_data = DeliveryFormData {
            name: name.get(),
            email: email.get(),
            address_line_1: address_line_1.get(),
            address_line_2: address_line_2.get(),
            city: city.get(),
            zipcode: zipcode.get(),
        };
        let validation = validate_delivery_form(&form_data, size_required, &selected_size.get());
        if validation.has_errors() {
            errors.set(validation);
        } else {
            errors.set(DeliveryFormErrors::default());
            step.set(CheckoutStep::ReviewPayment);
        }
    };

    let on_next_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        do_next();
    };

    let on_next_click = move |_: leptos::ev::MouseEvent| {
        do_next();
    };

    let on_back = move |_: leptos::ev::MouseEvent| match step.get() {
        CheckoutStep::ReviewPayment => step.set(CheckoutStep::DeliveryForm),
        CheckoutStep::DeliveryForm => reset_state(),
    };

    let on_pay = move |_: leptos::ev::MouseEvent| {
        payment_state.set(PaymentState::Processing);
        let token_amount = price_display.get_value();
        let shop_item_id = post_id.get_value();
        let name_val = name.get();
        let email_val = email.get();
        let addr1_val = address_line_1.get();
        let addr2_val = address_line_2.get();
        let city_val = city.get();
        let zip_val = zipcode.get();
        let size_val = selected_size.get();

        spawn_local(async move {
            let addr2 = if addr2_val.trim().is_empty() {
                None
            } else {
                Some(addr2_val)
            };
            let result = perform_shop_order(
                token_amount,
                shop_item_id,
                name_val,
                email_val,
                addr1_val,
                addr2,
                city_val,
                zip_val,
                size_val,
            )
            .await;

            match result {
                Ok(response) if response.is_success() => {
                    payment_state.set(PaymentState::Success);
                }
                _ => {
                    payment_state.set(PaymentState::Failed);
                }
            }
        });
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="checkout-overlay" on:click=on_backdrop_click>
                <div class="checkout-popup">
                    // Success state
                    <Show when=move || payment_state.get() == PaymentState::Success>
                        <div class="checkout-result checkout-success">
                            <i class="peer-icon peer-icon-order-success"></i>
                            <h2>"Order successfully placed!"</h2>
                            <p>"Your order has been confirmed. You\u{2019}ll receive an email with delivery details within 1\u{2013}3 days."</p>
                            <a href="/wallet" class="button btn-blue">"View Wallet"</a>
                        </div>
                    </Show>

                    // Failure state
                    <Show when=move || payment_state.get() == PaymentState::Failed>
                        <div class="checkout-result checkout-failed">
                            <i class="peer-icon peer-icon-order-failed"></i>
                            <h2>"Order Failed"</h2>
                            <p>"Something went wrong. Please try again in a moment."</p>
                            <button
                                class="button btn-blue"
                                on:click=move |_| {
                                    payment_state.set(PaymentState::Idle);
                                    step.set(CheckoutStep::ReviewPayment);
                                }
                            >
                                "Try again"
                            </button>
                        </div>
                    </Show>

                    // Main checkout flow
                    <Show when=move || payment_state.get() == PaymentState::Idle || payment_state.get() == PaymentState::Processing>
                        <div class="checkout-form-screen">
                            // Header
                            <div class="checkout-header">
                                <h2>"Shipping"</h2>
                                <button class="close-checkout" on:click=on_close>"\u{00D7}"</button>
                            </div>

                            // Product Header
                            <div class="product_header">
                                <div class="product_media">
                                    <img src=move || post_image.get_value() alt=move || post_title.get_value()/>
                                </div>
                                <div class="productinfo">
                                    <h3 class="bold">{move || post_title.get_value()}</h3>
                                    <p class="txt-color-gray">{move || post_desc.get_value()}</p>
                                    <div class="product_price bold">
                                        <span class="product_price_label txt-color-gray">"Price"</span>
                                        {" "}{move || price_display.get_value()}
                                    </div>
                                    // Show selected size in step 2
                                    <Show when=move || step.get() == CheckoutStep::ReviewPayment && selected_size.get().is_some()>
                                        <div class="selected_size">
                                            <span class="product_price_label txt-color-gray">"Size"</span>
                                            {" "}{move || selected_size.get().unwrap_or_default()}
                                        </div>
                                    </Show>
                                </div>
                            </div>

                            // Step 1: Size Selection + Delivery Form
                            <Show when=move || step.get() == CheckoutStep::DeliveryForm>
                                // Size selection
                                {has_sizes.then(move || {
                                    let sizes_list: Vec<(String, u32)> = sizes_data.get_value().into_iter().collect();
                                    view! {
                                        <div class="product_size">
                                            <h3>"Select size"</h3>
                                            <div class="product_sizes">
                                                {sizes_list.into_iter().map(|(size, stock)| {
                                                    let s2 = size.clone();
                                                    let in_stock = stock > 0;
                                                    view! {
                                                        <label
                                                            class="psize_label"
                                                            class:out_of_stock=!in_stock
                                                        >
                                                            <input
                                                                type="radio"
                                                                name="product_size"
                                                                value=size.clone()
                                                                disabled=!in_stock
                                                                on:change=move |_| {
                                                                    selected_size.set(Some(s2.clone()));
                                                                }
                                                            />
                                                            <span>{size}</span>
                                                        </label>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                            <Show when=move || errors.get().size.is_some()>
                                                <span class="response_msg error">
                                                    {move || errors.get().size.clone().unwrap_or_default()}
                                                </span>
                                            </Show>
                                        </div>
                                    }
                                })}

                                // Delivery info collapsible
                                <DeliveryInfoCollapsible/>

                                // Delivery form
                                <form class="checkout-form" on:submit=on_next_submit>
                                    <div class="form_field">
                                        <input
                                            type="text"
                                            placeholder="Full name"
                                            prop:value=move || name.get()
                                            on:input=move |ev| name.set(event_target_value(&ev))
                                        />
                                        <Show when=move || errors.get().name.is_some()>
                                            <span class="response_msg error">
                                                {move || errors.get().name.clone().unwrap_or_default()}
                                            </span>
                                        </Show>
                                    </div>
                                    <div class="form_field">
                                        <input
                                            type="email"
                                            placeholder="Email address"
                                            prop:value=move || email.get()
                                            on:input=move |ev| email.set(event_target_value(&ev))
                                        />
                                        <Show when=move || errors.get().email.is_some()>
                                            <span class="response_msg error">
                                                {move || errors.get().email.clone().unwrap_or_default()}
                                            </span>
                                        </Show>
                                    </div>
                                    <div class="form_field">
                                        <input
                                            type="text"
                                            placeholder="Address line 1"
                                            prop:value=move || address_line_1.get()
                                            on:input=move |ev| address_line_1.set(event_target_value(&ev))
                                        />
                                        <Show when=move || errors.get().address_line_1.is_some()>
                                            <span class="response_msg error">
                                                {move || errors.get().address_line_1.clone().unwrap_or_default()}
                                            </span>
                                        </Show>
                                    </div>
                                    <div class="form_field">
                                        <input
                                            type="text"
                                            placeholder="Address line 2 (optional)"
                                            prop:value=move || address_line_2.get()
                                            on:input=move |ev| address_line_2.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="city_zip">
                                        <div class="form_field">
                                            <input
                                                type="text"
                                                placeholder="City"
                                                prop:value=move || city.get()
                                                on:input=move |ev| city.set(event_target_value(&ev))
                                            />
                                            <Show when=move || errors.get().city.is_some()>
                                                <span class="response_msg error">
                                                    {move || errors.get().city.clone().unwrap_or_default()}
                                                </span>
                                            </Show>
                                        </div>
                                        <div class="form_field">
                                            <input
                                                type="text"
                                                placeholder="ZIP"
                                                prop:value=move || zipcode.get()
                                                on:input=move |ev| zipcode.set(event_target_value(&ev))
                                            />
                                            <Show when=move || errors.get().zipcode.is_some()>
                                                <span class="response_msg error">
                                                    {move || errors.get().zipcode.clone().unwrap_or_default()}
                                                </span>
                                            </Show>
                                        </div>
                                    </div>
                                </form>
                            </Show>

                            // Step 2: Review & Payment
                            <Show when=move || step.get() == CheckoutStep::ReviewPayment>
                                <div class="scroll_wrap">
                                    // Delivery info review
                                    <div class="delivery_info_verify">
                                        <h3 class="bold">"Delivery information"</h3>
                                        <div class="delivery_verify_list">
                                            <div class="verify_row">
                                                <span class="verify_label txt-color-gray">"Name"</span>
                                                <span class="verify_value">{move || name.get()}</span>
                                            </div>
                                            <div class="verify_row">
                                                <span class="verify_label txt-color-gray">"E-mail"</span>
                                                <span class="verify_value">{move || email.get()}</span>
                                            </div>
                                            <div class="verify_row">
                                                <span class="verify_label txt-color-gray">"Address line 1"</span>
                                                <span class="verify_value">{move || address_line_1.get()}</span>
                                            </div>
                                            <Show when=move || !address_line_2.get().trim().is_empty()>
                                                <div class="verify_row">
                                                    <span class="verify_label txt-color-gray">"Address line 2"</span>
                                                    <span class="verify_value">{move || address_line_2.get()}</span>
                                                </div>
                                            </Show>
                                            <div class="verify_row">
                                                <span class="verify_label txt-color-gray">"City"</span>
                                                <span class="verify_value">{move || city.get()}</span>
                                            </div>
                                            <div class="verify_row">
                                                <span class="verify_label txt-color-gray">"ZIP code"</span>
                                                <span class="verify_value">{move || zipcode.get()}</span>
                                            </div>
                                            <div class="verify_row">
                                                <span class="verify_label txt-color-gray">"Country"</span>
                                                <span class="verify_value">"Germany"</span>
                                            </div>
                                        </div>
                                    </div>

                                    // Paying to
                                    <div class="paying_to">
                                        <span class="paying_to_label bold">"Paying to"</span>
                                        <span class="paying_to_store">
                                            "@Peer_Shop"
                                            <span class="store_slug txt-color-gray">" #12445"</span>
                                        </span>
                                    </div>

                                    // Amount breakdown
                                    <div class="amount_detail">
                                        <div class="feePanel">
                                            <div class="fee-section" class:close=move || !show_fees.get()>
                                                <div class="total_amount bold">
                                                    "Total amount "
                                                    <span class="final-total bold">{move || price_display_review.get_value()}</span>
                                                </div>
                                                <div class="product-price">
                                                    <div class="price-item txt-color-gray">
                                                        <span class="label">"Item Price"</span>
                                                        <span class="value bold">{move || price_display_review.get_value()}</span>
                                                    </div>
                                                </div>
                                                <div
                                                    class="fee-title txt-color-gray"
                                                    on:click=move |_| show_fees.update(|v| *v = !*v)
                                                    style="cursor:pointer"
                                                >
                                                    "Fees included "
                                                    <span class="fee-total bold">{move || fee_total.get_value()}</span>
                                                </div>
                                                <Show when=move || show_fees.get()>
                                                    <div class="fee-breakdowns">
                                                        <div class="fee-item txt-color-gray">
                                                            <span class="label">"Peer Bank"</span>
                                                            <span class="value">{move || fee_peer.get_value()}</span>
                                                        </div>
                                                        <div class="fee-item txt-color-gray">
                                                            <span class="label">"Burns"</span>
                                                            <span class="value">{move || fee_burn.get_value()}</span>
                                                        </div>
                                                        <div class="fee-item txt-color-gray">
                                                            <span class="label">"Inviter"</span>
                                                            <span class="value">{move || fee_inviter.get_value()}</span>
                                                        </div>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </Show>

                            // Action buttons
                            <div class="checkout-actions">
                                <button class="btn-back btn-transparent" on:click=on_back>
                                    <i class="peer-icon peer-icon-arrow-left"></i>
                                    " Back"
                                </button>
                                <Show
                                    when=move || step.get() == CheckoutStep::ReviewPayment
                                    fallback=move || view! {
                                        <button class="btn-next btn-blue bold" on:click=on_next_click>
                                            "Next "
                                            <i class="peer-icon peer-icon-arrow-right"></i>
                                        </button>
                                    }
                                >
                                    <button
                                        class="btn-next btn-blue btn-pay bold"
                                        on:click=on_pay
                                        disabled=move || payment_state.get() == PaymentState::Processing
                                    >
                                        {move || {
                                            if payment_state.get() == PaymentState::Processing {
                                                "Processing... "
                                            } else {
                                                "Pay "
                                            }
                                        }}
                                        <i
                                            class="peer-icon"
                                            class:peer-icon-arrow-right=move || payment_state.get() != PaymentState::Processing
                                            class:peer-icon-loader=move || payment_state.get() == PaymentState::Processing
                                            class:spin=move || payment_state.get() == PaymentState::Processing
                                        ></i>
                                    </button>
                                </Show>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}

/// Collapsible delivery information panel for Step 1.
#[component]
fn DeliveryInfoCollapsible() -> impl IntoView {
    let is_open = RwSignal::new(false);

    view! {
        <div class="delivery_info" class:close=move || !is_open.get()>
            <h3
                class="dtitle"
                on:click=move |_| is_open.update(|v| *v = !*v)
                style="cursor:pointer"
            >
                "Delivery information"
                <span class="txt-color-gray">
                    " \u{2014} 1-3 working days, "
                    <strong>"only within Germany"</strong>
                </span>
            </h3>
            <Show when=move || is_open.get()>
                <div class="delivery_message txt-color-gray">
                    "We\u{2019}ll email your delivery details within 1\u{2013}3 days after your payment is confirmed. Please make sure your email address is correct; they can\u{2019}t be changed after you place the order. Delivery is available only within Germany."
                </div>
            </Show>
        </div>
    }
}
