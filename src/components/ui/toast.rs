use dioxus::prelude::*;
use std::time::Duration;

use crate::{util::sleep, ToastKind, ToastState};

const TOAST_DURATION: Duration = Duration::from_secs(5);

/// Queues a toast and schedules its automatic dismissal. This is the one
/// entry point for showing a toast - callers never touch `ToastState`
/// directly, so the auto-dismiss timer can't be forgotten at a call site.
pub fn show_toast(
    mut toast_state: Signal<ToastState>,
    message: impl Into<String>,
    kind: ToastKind,
) {
    let id = toast_state.write().push(message, kind);
    spawn(async move {
        sleep(TOAST_DURATION).await;
        toast_state.write().dismiss(id);
    });
}

#[component]
pub fn ToastContainer() -> Element {
    let mut toast_state = use_context::<Signal<ToastState>>();
    let toasts = toast_state.read().toasts.clone();

    if toasts.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "fixed top-4 right-4 z-[200] flex flex-col gap-2 w-[calc(100%-2rem)] sm:w-[380px]",
            for toast in toasts {
                div {
                    key: "{toast.id}",
                    class: format!(
                        "flex items-start gap-3 rounded-lg border p-4 shadow-lg animate-in slide-in-from-top-4 fade-in duration-200 {}",
                        match toast.kind {
                            ToastKind::Error => {
                                "bg-destructive text-destructive-foreground border-destructive/50"
                            }
                            ToastKind::Success => "bg-card text-foreground border-emerald-500/50",
                            ToastKind::Info => "bg-card text-foreground border-border",
                        },
                    ),
                    span { class: "flex-1 text-sm font-medium", "{toast.message}" }
                    button {
                        class: "shrink-0 text-lg leading-none opacity-70 hover:opacity-100 transition-opacity",
                        onclick: move |_| {
                            toast_state.write().dismiss(toast.id);
                        },
                        "✕"
                    }
                }
            }
        }
    }
}
