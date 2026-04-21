//! Main navigation menu widget.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::state::chat::try_use_chat;

/// Main navigation menu.
#[component]
pub fn MainMenu() -> impl IntoView {
    let location = use_location();
    let current_path = move || location.pathname.get();

    let chat_ctx = try_use_chat();
    let total_unread = move || {
        chat_ctx
            .map(|c| c.unread_counts.get().values().copied().sum::<u32>())
            .unwrap_or(0)
    };
    let unread_label = move || {
        let n = total_unread();
        if n > 99 {
            "99+".to_string()
        } else {
            n.to_string()
        }
    };

    let menu_items = [
        (
            "/dashboard",
            "Dashboard",
            "peer-icon-home-alt",
            "peer-icon-home",
            false,
        ),
        ("/chat", "Chat", "peer-icon-chat", "peer-icon-chat", true),
        (
            "/wallet",
            "Wallet",
            "peer-icon-wallet",
            "peer-icon-wallet-filled",
            false,
        ),
        ("/shop", "Shop", "peer-icon-shop", "peer-icon-shop", false),
        (
            "/settings",
            "Settings",
            "peer-icon-setting",
            "peer-icon-setting-filled",
            false,
        ),
    ];

    view! {
        <div class="widget widget-margin-bottom">
            <div class="widget-inner widget-type-box widget-main-menu">
                <ul class="menu">
                    {menu_items.iter().map(|(path, label, icon, icon_filled, is_chat)| {
                        let p = *path;
                        let is_active = Memo::new(move |_| current_path() == p);
                        let is_chat = *is_chat;
                        let show_badge = move || is_chat && total_unread() > 0;

                        view! {
                            <li class="menu-item" class:active=is_active>
                                <a href=*path>
                                    <i class=format!("peer-icon {}", icon)/>
                                    <i class=format!("filled peer-icon {}", icon_filled)/>
                                    {*label}
                                    <Show when=show_badge>
                                        <span class="menu-unread-badge" aria-label="Unread chat messages">
                                            {unread_label}
                                        </span>
                                    </Show>
                                </a>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}
