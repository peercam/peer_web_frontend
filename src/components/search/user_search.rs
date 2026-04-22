//! User search dropdown component.

use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::api::posts::search_users;
use crate::models::post::UserSearchResult;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::spawn_local;

/// User search input with dropdown results.
#[component]
pub fn UserSearch() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let results = RwSignal::new(Vec::<UserSearchResult>::new());
    let is_open = RwSignal::new(false);
    let is_loading = RwSignal::new(false);

    // Debounce timeout handle
    #[cfg(feature = "hydrate")]
    let timeout_handle = StoredValue::new(None::<i32>);

    // Debounced search effect
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;

        let q = query.get();

        // Clear previous timeout
        if let Some(handle) = timeout_handle.get_value()
            && let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(handle);
            }

        // Require at least 3 characters
        if q.len() < 3 {
            results.set(vec![]);
            is_open.set(false);
            return;
        }

        // Debounce: wait 300ms before searching
        let closure = Closure::once(Box::new(move || {
            is_loading.set(true);
            spawn_local(async move {
                match search_users(q, 20).await {
                    Ok(users) => {
                        results.set(users);
                        is_open.set(true);
                    }
                    Err(e) => {
                        leptos::logging::error!("User search failed: {:?}", e);
                        results.set(vec![]);
                    }
                }
                is_loading.set(false);
            });
        }) as Box<dyn FnOnce()>);

        if let Some(window) = web_sys::window()
            && let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                300,
            ) {
                timeout_handle.set_value(Some(handle));
            }

        closure.forget();
    });

    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        query.set(value);
    };

    let on_blur = move |_| {
        // Small delay to allow click on dropdown items
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::prelude::*;

            let closure = Closure::once(Box::new(move || {
                is_open.set(false);
            }) as Box<dyn FnOnce()>);

            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    200,
                );
            }

            closure.forget();
        }
    };

    let on_focus = move |_| {
        if !results.get().is_empty() {
            is_open.set(true);
        }
    };

    view! {
        <div class="search-box" class:active=move || is_open.get()>
            <input
                type="text"
                id="searchUser"
                class="search-input user-search-input"
                placeholder="@username"
                prop:value=move || query.get()
                on:input=on_input
                on:focus=on_focus
                on:blur=on_blur
            />

            <Show when=move || is_loading.get()>
                <div class="search-loading">
                    <span class="spinner"/>
                </div>
            </Show>

            <div
                id="userDropdown"
                class="dropdown"
                class:active=move || is_open.get()
                // Prevent blur when clicking dropdown
                on:mousedown=|ev: leptos::ev::MouseEvent| ev.prevent_default()
            >
                <For
                    each=move || results.get()
                    key=|user| user.id.clone()
                    children=move |user| {
                        view! {
                            <UserDropdownItem
                                user=user
                                on_select=move || {
                                    is_open.set(false);
                                    query.set(String::new());
                                }
                            />
                        }
                    }
                />

                <Show when=move || results.get().is_empty() && is_open.get()>
                    <div class="dropdown-empty">"No users found"</div>
                </Show>
            </div>
        </div>
    }
}

/// Individual user item in the search dropdown.
#[component]
fn UserDropdownItem(
    user: UserSearchResult,
    #[prop(into)] on_select: Callback<()>,
) -> impl IntoView {
    let profile_url = format!("/profile/{}", user.slug);
    let img_src = user
        .img
        .clone()
        .unwrap_or_else(|| "/svg/noname.svg".to_string());
    let username = user.username.clone();
    #[allow(unused)]
    let slug = user.slug.clone();

    let on_click = move |_| {
        on_select.run(());
        // Navigate to profile
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&format!("/profile/{}", slug));
            }
        }
    };

    view! {
        <a
            href=profile_url
            class="dropdown-item user-item"
            on:click=on_click
        >
            <img src=img_src alt="" class="user-avatar"/>
            <div class="user-info">
                <span class="username">{username}</span>
                <span class="slug">{"@"}{user.slug}</span>
            </div>
        </a>
    }
}
