//! Product price badge component for post cards.

use leptos::prelude::*;

/// Price badge overlay shown on shop product post cards.
#[component]
pub fn ProductPriceBadge(
    /// The token price to display.
    #[prop(into)]
    price: String,
) -> impl IntoView {
    view! {
        <div class="product_price bold">
            <span>{price}</span>
        </div>
    }
}
