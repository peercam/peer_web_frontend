//! Screen reader step announcer.
//!
//! Provides a visually-hidden aria-live region that announces
//! step transitions and status messages to assistive technology.

use leptos::prelude::*;

/// Reactive context for announcing messages to screen readers.
///
/// Child components can obtain this via `use_context::<StepAnnouncerContext>()`
/// and call `announce()` to push messages to the aria-live region.
#[derive(Clone, Copy)]
pub struct StepAnnouncerContext {
    message: RwSignal<String>,
}

impl StepAnnouncerContext {
    /// Announce a message to screen readers.
    ///
    /// The message replaces the current content of the aria-live region.
    /// Screen readers will read the new content based on the politeness level.
    pub fn announce(&self, msg: impl Into<String>) {
        self.message.set(msg.into());
    }
}

/// Visually-hidden aria-live region for step announcements.
///
/// Place this once inside the `RegisterPage` component.
/// Child components can announce messages via `use_context::<StepAnnouncerContext>()`.
#[component]
pub fn StepAnnouncer(
    /// A signal that provides the announcement text.
    /// Changes to this signal trigger a screen reader announcement.
    announcement: Signal<String>,
) -> impl IntoView {
    let message = RwSignal::new(String::new());
    let ctx = StepAnnouncerContext { message };
    provide_context(ctx);

    // Auto-announce when the external announcement signal changes
    Effect::new(move |_| {
        let text = announcement.get();
        if !text.is_empty() {
            message.set(text);
        }
    });

    view! {
        <div
            id="step-announcer"
            class="sr-only"
            aria-live="polite"
            aria-atomic="true"
            role="status"
        >
            {move || message.get()}
        </div>
    }
}
