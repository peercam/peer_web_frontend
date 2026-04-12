//! Tag list display component.

use leptos::prelude::*;

use crate::pages::new_post::NewPostContext;

/// Display list of selected tags with remove buttons.
#[component]
pub fn TagList() -> impl IntoView {
    let ctx = NewPostContext::use_context();

    let has_tags = move || !ctx.tags.get().is_empty();

    view! {
        <Show when=has_tags>
            <div class="tag-list">
                {move || {
                    ctx.tags.get().iter().map(|tag| {
                        let tag_clone = tag.clone();
                        let tag_display = tag.clone();
                        view! {
                            <span class="tag-chip">
                                <span class="tag-text">{"#"}{tag_display}</span>
                                <button
                                    type="button"
                                    class="tag-remove"
                                    on:click=move |_| ctx.remove_tag(&tag_clone)
                                    title="Remove tag"
                                >
                                    <i class="peer-icon peer-icon-x"/>
                                </button>
                            </span>
                        }
                    }).collect_view()
                }}
            </div>
        </Show>
    }
}
