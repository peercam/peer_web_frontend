//! Toast notification component for displaying feedback messages.

use leptos::prelude::*;
use std::time::Duration;

/// Toast notification types with corresponding styling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

impl ToastType {
    pub fn class(&self) -> &'static str {
        match self {
            ToastType::Success => "toast--success",
            ToastType::Error => "toast--error",
            ToastType::Info => "toast--info",
        }
    }
}

/// A single toast message.
#[derive(Clone, Debug)]
pub struct ToastMessage {
    pub id: u32,
    pub message: String,
    pub toast_type: ToastType,
}

/// Toast context for showing notifications.
#[derive(Clone, Copy)]
pub struct ToastContext {
    toasts: RwSignal<Vec<ToastMessage>>,
    next_id: RwSignal<u32>,
}

impl ToastContext {
    /// Show a toast notification that auto-dismisses after 3 seconds.
    pub fn show(&self, message: impl Into<String>, toast_type: ToastType) {
        let id = self.next_id.get();
        self.next_id.update(|n| *n += 1);

        let toast = ToastMessage {
            id,
            message: message.into(),
            toast_type,
        };

        self.toasts.update(|t| t.push(toast));

        // Auto-dismiss after 3 seconds
        let toasts = self.toasts;
        set_timeout(
            move || {
                toasts.update(|t| t.retain(|toast| toast.id != id));
            },
            Duration::from_secs(3),
        );
    }

    /// Manually dismiss a toast by ID.
    pub fn dismiss(&self, id: u32) {
        self.toasts.update(|t| t.retain(|toast| toast.id != id));
    }
}

/// Provide toast context to the component tree.
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let context = ToastContext {
        toasts: RwSignal::new(Vec::new()),
        next_id: RwSignal::new(0),
    };

    provide_context(context);

    view! {
        {children()}
        <ToastContainer toasts=context.toasts dismiss=move |id| context.dismiss(id) />
    }
}

/// Get the toast context from the nearest provider.
pub fn use_toast() -> ToastContext {
    expect_context::<ToastContext>()
}

/// Container that renders all active toasts.
#[component]
fn ToastContainer(
    toasts: RwSignal<Vec<ToastMessage>>,
    dismiss: impl Fn(u32) + Copy + Send + 'static,
) -> impl IntoView {
    view! {
        <div
            class="toast-container"
            aria-live="assertive"
            aria-atomic="true"
        >
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    let id = toast.id;
                    view! {
                        <div
                            class=format!("toast {}", toast.toast_type.class())
                            role="alert"
                        >
                            <span class="toast__message">{toast.message.clone()}</span>
                            <button
                                type="button"
                                class="toast__dismiss"
                                aria-label="Dismiss notification"
                                on:click=move |_| dismiss(id)
                            >
                                "\u{00d7}"
                            </button>
                        </div>
                    }
                }
            />
        </div>
    }
}
