//! Version list component — clickable list of version buttons.

use leptos::prelude::*;

use crate::models::version::VersionRelease;

/// Displays a list of version buttons in the left panel.
#[component]
pub fn VersionList(
    versions: Vec<VersionRelease>,
    selected_idx: RwSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <div class="setting-menu left_versionHistory">
            {versions
                .into_iter()
                .enumerate()
                .map(|(idx, version)| {
                    let is_active = move || selected_idx.get() == Some(idx);
                    view! {
                        <a
                            class:active=is_active
                            on:click=move |_| selected_idx.set(Some(idx))
                        >
                            {version.version}
                        </a>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
