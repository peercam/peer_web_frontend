//! Share modal component.
//!
//! Provides share functionality via copy link, WhatsApp, Telegram, etc.

use leptos::prelude::*;

/// URL-encode a string for use in URLs.
fn url_encode(s: &str) -> String {
    // Simple URL encoding for common characters
    s.replace(' ', "%20")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('=', "%3D")
        .replace('#', "%23")
}

/// Get the base URL for sharing.
fn get_base_url() -> String {
    #[cfg(feature = "hydrate")]
    {
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "https://peer-network.com".to_string())
    }
    #[cfg(not(feature = "hydrate"))]
    {
        std::env::var("BASE_URL").unwrap_or_else(|_| "https://peer-network.com".to_string())
    }
}

/// Share modal with copy link, WhatsApp, Telegram.
#[component]
pub fn ShareModal<F>(post_id: String, on_close: F) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    let post_url = format!("{}/post/{}", get_base_url(), post_id);
    let post_url_for_input = post_url.clone();
    let copied = RwSignal::new(false);
    let on_close_clone = on_close.clone();

    let copy_to_clipboard = {
        #[allow(unused_variables)]
        let url_for_copy = post_url.clone();
        move |_| {
            #[cfg(feature = "hydrate")]
            {
                let url = url_for_copy.clone();
                leptos::task::spawn_local(async move {
                    use wasm_bindgen_futures::JsFuture;
                    if let Some(clipboard) =
                        web_sys::window().map(|w| w.navigator().clipboard())
                        && JsFuture::from(clipboard.write_text(&url)).await.is_ok() {
                            copied.set(true);
                            // Reset after 2 seconds
                            leptos::task::spawn_local(async move {
                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                copied.set(false);
                            });
                        }
                });
            }
        }
    };

    let whatsapp_url = format!(
        "https://api.whatsapp.com/send?text={}",
        url_encode(&post_url)
    );
    let telegram_url = format!("https://t.me/share/url?url={}", url_encode(&post_url));

    view! {
        <div class="share-modal-overlay" on:click=move |_| on_close_clone()>
            <div id="share-link-box" class="share-post-box" on:click=|ev| ev.stop_propagation()>
                <div
                    id="closeSharebox"
                    class="btClose"
                    on:click=move |_| on_close()
                >
                    <i class="peer-icon peer-icon-cancel"/>
                </div>

                <div class="share-post-subbox">
                    <h3 class="xl_font_size">"Share"</h3>
                    <div class="share-post-apps">
                        <a
                            href=whatsapp_url
                            target="_blank"
                            rel="noopener noreferrer"
                            class="share-app-link"
                        >
                            <span class="share-app whatsapp">
                                <span class="app-icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="24" height="24">
                                        <path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.741.982.998-3.648-.235-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.893 6.994c-.003 5.45-4.437 9.884-9.885 9.884m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.335.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.305-1.654a11.882 11.882 0 005.683 1.448h.005c6.554 0 11.89-5.335 11.893-11.893a11.821 11.821 0 00-3.48-8.413z"/>
                                    </svg>
                                </span>
                                <span class="app-name">"WhatsApp"</span>
                            </span>
                        </a>
                        <a
                            href=telegram_url
                            target="_blank"
                            rel="noopener noreferrer"
                            class="share-app-link"
                        >
                            <span class="share-app telegram">
                                <span class="app-icon">
                                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" width="24" height="24">
                                        <path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/>
                                    </svg>
                                </span>
                                <span class="app-name">"Telegram"</span>
                            </span>
                        </a>
                    </div>
                </div>

                <div class="share-post-subbox">
                    <h3 class="xl_font_size">"Link"</h3>
                    <div class="share-post-link">
                        <input
                            type="text"
                            class="share-link-input"
                            value=post_url_for_input
                            readonly
                        />
                        <button class="copy-link-btn btn-white" on:click=copy_to_clipboard>
                            <span>{move || if copied.get() { "Copied!" } else { "Copy" }}</span>
                            <i class="peer-icon peer-icon-copy-alt"/>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
