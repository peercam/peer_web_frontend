//! Version detail component — displays changelog for the selected version.

use leptos::prelude::*;

use crate::models::version::VersionRelease;

/// Displays the details of the currently selected version.
#[component]
pub fn VersionDetail(
    versions: Vec<VersionRelease>,
    selected_idx: RwSignal<Option<usize>>,
) -> impl IntoView {
    let versions = StoredValue::new(versions);

    let selected_version = move || {
        selected_idx
            .get()
            .and_then(|idx| versions.with_value(|v| v.get(idx).cloned()))
    };

    view! {
        <div class="setting-content right_versionHistory">
            {move || match selected_version() {
                Some(release) => {
                    let changes = release.changes.clone();
                    let links = release.links.clone();
                    view! {
                        <div class="releaseVersion_header">
                            <h1 class="xxl_font_size bold">{release.version}</h1>
                            <span>{release.date}</span>
                        </div>

                        {changes
                            .into_iter()
                            .map(|change| {
                                let descriptions = change.description.clone();
                                view! {
                                    <div class="releaseVersion_content">
                                        <h2 class="md_font_size bold">{change.title}</h2>
                                        {descriptions
                                            .into_iter()
                                            .map(|desc| view! { <p>{desc}</p> })
                                            .collect::<Vec<_>>()}
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()}

                        <div class="releaseVersion_btns">
                            {links
                                .into_iter()
                                .map(|link| {
                                    view! {
                                        <a
                                            href=link.href
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            class="button btn-transparent Versionbtn-link"
                                        >
                                            {link.label}
                                        </a>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    }
                        .into_any()
                }
                None => {
                    view! { <p class="empty-state">"Select a version to view details"</p> }
                        .into_any()
                }
            }}
        </div>
    }
}
