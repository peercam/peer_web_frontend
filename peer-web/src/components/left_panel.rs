//! Shared left panel component for login/register pages.
//!
//! Renders the phone mockup with an image and logo, matching
//! the legacy PHP layout.

use leptos::prelude::*;

/// Left panel with animated phone mockup and logo.
///
/// Used on both the login and registration pages.
#[component]
pub fn LeftPanel(
    /// The image path to display in the phone mockup (e.g. "/img/register.webp").
    #[prop(into)]
    image_src: String,
    /// Alt text for the image.
    #[prop(into)]
    image_alt: String,
) -> impl IntoView {
    view! {
        <div class="container_left">
            <div class="phone">
                <div class="screen">
                    <img
                        src=image_src
                        alt=image_alt
                        width="612"
                        height="612"
                    />
                </div>
                <div class="home-button">
                    <img
                        src="/svg/logo_sw.svg"
                        alt="Peer Logo"
                        width="96"
                        height="96"
                    />
                </div>
            </div>
            <img
                class="logo"
                src="/svg/logo_farbe.svg"
                alt="Peer logo"
                width="96"
                height="96"
            />
        </div>
    }
}
