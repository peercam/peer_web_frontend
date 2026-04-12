//! Content type tabs sidebar.

use leptos::prelude::*;

use crate::models::post::CreateContentType;
use crate::pages::new_post::NewPostContext;

/// Left sidebar with content type tabs.
#[component]
pub fn ContentTypeTabs() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    let content_types = [
        CreateContentType::Text,
        CreateContentType::Image,
        CreateContentType::Audio,
        CreateContentType::Video,
    ];

    view! {
        <aside class="leftside" id="createPostFilter">
            <div class="sidebar-content">
                <nav class="content-type-nav">
                    <ul class="tab-list">
                        {content_types
                            .into_iter()
                            .map(|ct| {
                                let is_active = move || ctx.content_type.get() == ct;
                                view! {
                                    <li class="tab-item">
                                        <button
                                            type="button"
                                            class="tab-button"
                                            class:active=is_active
                                            on:click=move |_| {
                                                ctx.set_content_type.set(ct);
                                                ctx.reset();
                                            }
                                        >
                                            <i class=move || format!(
                                                "peer-icon {}",
                                                ct.icon_class()
                                            )/>
                                            <span class="tab-label">{ct.display_name()}</span>
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </nav>
            </div>
        </aside>
    }
}
