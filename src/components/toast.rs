//! Reusable toast notification component.
//!
//! Provides a `ToastProvider` context and `ToastContainer` renderer.
//! Any child component can trigger toasts via `use_context::<ToastContext>()`.

use leptos::prelude::*;
use std::time::Duration;

use crate::utils::response_codes::user_friendly_msg;

/// The visual style of a toast notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastType {
    Info,
    Success,
    Error,
}

impl ToastType {
    /// Returns the CSS class suffix for this toast type.
    fn css_class(&self) -> &'static str {
        match self {
            ToastType::Info => "",
            ToastType::Success => "success",
            ToastType::Error => "error",
        }
    }
}

/// A single toast entry in the notification queue.
#[derive(Clone)]
struct ToastEntry {
    id: u64,
    message: String,
    toast_type: ToastType,
    visible: RwSignal<bool>,
}

/// Handle for showing toasts, obtained via `use_toast()`.
#[derive(Clone, Copy)]
pub struct ToastContext {
    toasts: RwSignal<Vec<ToastEntry>>,
    next_id: RwSignal<u64>,
}

impl ToastContext {
    /// Show a toast with a custom message and type.
    ///
    /// Removes any existing toast first (only one toast visible at a time),
    /// then creates a new one that auto-dismisses after 3 seconds.
    pub fn show(&self, message: impl Into<String>, toast_type: ToastType) {
        let message = message.into();
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);

        let visible = RwSignal::new(false);

        let entry = ToastEntry {
            id,
            message,
            toast_type,
            visible,
        };

        // Replace any existing toasts (only one at a time)
        self.toasts.set(vec![entry]);

        // Trigger slide-in after a short delay (matches JS: setTimeout 100ms)
        set_timeout(
            move || {
                visible.set(true);
            },
            Duration::from_millis(100),
        );

        // Auto-dismiss after 3 seconds
        let toasts = self.toasts;
        set_timeout(
            move || {
                // Start slide-out
                visible.set(false);

                // Remove from DOM after slide-out animation (300ms)
                set_timeout(
                    move || {
                        toasts.update(|t| t.retain(|e| e.id != id));
                    },
                    Duration::from_millis(300),
                );
            },
            Duration::from_millis(3000),
        );
    }

    /// Show a toast by response code.
    ///
    /// Looks up the code in the compiled response-code map and infers
    /// the toast type from the code prefix.
    pub fn show_code(&self, code: &str) {
        let message = user_friendly_msg(code).to_string();
        let toast_type = toast_type_from_code(code);
        self.show(message, toast_type);
    }
}

/// Infer the toast type from a response code prefix.
///
/// Codes starting with `1` are successes, `2` are informational,
/// and `3`/`4` are errors (validation failures, server errors).
pub fn toast_type_from_code(code: &str) -> ToastType {
    match code.chars().next() {
        Some('1') => ToastType::Success,
        Some('2') => ToastType::Info,
        _ => ToastType::Error,
    }
}

/// Wraps children with toast context. Place this near the top of your
/// component tree (e.g. inside `App` or `RegisterPage`).
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let toasts = RwSignal::new(Vec::<ToastEntry>::new());
    let next_id = RwSignal::new(0u64);

    let ctx = ToastContext { toasts, next_id };
    provide_context(ctx);

    view! {
        {children()}
        <ToastContainer toasts=toasts />
    }
}

/// Get the toast context from the nearest provider.
pub fn use_toast() -> ToastContext {
    expect_context::<ToastContext>()
}

/// Renders the active toast notifications.
#[component]
fn ToastContainer(toasts: RwSignal<Vec<ToastEntry>>) -> impl IntoView {
    view! {
        <For
            each=move || toasts.get()
            key=|entry| entry.id
            children=move |entry| {
                let type_class = entry.toast_type.css_class().to_string();
                let visible = entry.visible;

                view! {
                    <div
                        class=move || {
                            let mut classes = String::from("toast");
                            if !type_class.is_empty() {
                                classes.push(' ');
                                classes.push_str(&type_class);
                            }
                            if visible.get() {
                                classes.push_str(" show");
                            }
                            classes
                        }
                        role="alert"
                        aria-live="assertive"
                    >
                        {entry.message.clone()}
                    </div>
                }
            }
        />
    }
}

#[cfg(test)]
mod toast_type_tests {
    use super::*;

    #[test]
    fn success_codes_start_with_1() {
        assert_eq!(toast_type_from_code("10601"), ToastType::Success);
        assert_eq!(toast_type_from_code("10701"), ToastType::Success);
        assert_eq!(toast_type_from_code("11011"), ToastType::Success);
    }

    #[test]
    fn info_codes_start_with_2() {
        assert_eq!(toast_type_from_code("21002"), ToastType::Info);
        assert_eq!(toast_type_from_code("21003"), ToastType::Info);
    }

    #[test]
    fn validation_errors_start_with_3() {
        assert_eq!(toast_type_from_code("30601"), ToastType::Error);
        assert_eq!(toast_type_from_code("31010"), ToastType::Error);
    }

    #[test]
    fn server_errors_start_with_4() {
        assert_eq!(toast_type_from_code("40601"), ToastType::Error);
        assert_eq!(toast_type_from_code("40701"), ToastType::Error);
    }

    #[test]
    fn empty_code_defaults_to_error() {
        assert_eq!(toast_type_from_code(""), ToastType::Error);
    }
}
