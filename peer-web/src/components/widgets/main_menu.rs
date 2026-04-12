//! Main navigation menu widget.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// Main navigation menu.
#[component]
pub fn MainMenu() -> impl IntoView {
    let location = use_location();
    let current_path = move || location.pathname.get();

    let menu_items = [
        ("/dashboard", "Dashboard", "peer-icon-home-alt", "peer-icon-home"),
        ("/wallet", "Wallet", "peer-icon-wallet", "peer-icon-wallet-filled"),
        ("/shop", "Shop", "peer-icon-shop", "peer-icon-shop"),
        ("/settings", "Settings", "peer-icon-setting", "peer-icon-setting-filled"),
    ];

    view! {
        <div class="widget widget-margin-bottom">
            <div class="widget-inner widget-type-box widget-main-menu">
                <ul class="menu">
                    {menu_items.iter().map(|(path, label, icon, icon_filled)| {
                        let p = *path;
                        let is_active = Memo::new(move |_| current_path() == p);

                        view! {
                            <li class="menu-item" class:active=is_active>
                                <a href=*path>
                                    <i class=format!("peer-icon {}", icon)/>
                                    <i class=format!("filled peer-icon {}", icon_filled)/>
                                    {*label}
                                </a>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}
