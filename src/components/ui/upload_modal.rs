use crate::{UploadState, UploadStatus};
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn UploadModal() -> Element {
    let mut upload_state = use_context::<Signal<UploadState>>();
    let mut auto_scroll = use_signal(|| true);
    let mut was_visible = use_signal(|| true);
    let mut check_pending = use_signal(|| false);
    let mut container_mounted = use_signal(|| None::<Rc<MountedData>>);
    let mut active_mounted = use_signal(|| None::<Rc<MountedData>>);

    let tasks = upload_state.read().tasks.clone();

    if tasks.is_empty() {
        return rsx! {};
    }

    let completed = tasks
        .iter()
        .filter(|t| t.status == UploadStatus::Done || t.status == UploadStatus::Error)
        .count();
    let total = tasks.len();

    // Auto-scroll effect - only triggers when auto_scroll or active_mounted changes
    use_effect(move || {
        if *auto_scroll.read() {
            if let Some(active) = active_mounted.read().clone() {
                spawn(async move {
                    let _ = active.scroll_to(ScrollBehavior::Smooth).await;
                });
            }
        }
    });

    rsx! {
        div {
            class: "fixed bottom-4 right-4 left-4 sm:left-auto sm:bottom-6 sm:right-6 w-auto sm:w-[650px] max-w-full bg-card border border-border rounded-lg shadow-xl z-[100] flex flex-col overflow-hidden",
            div {
                class: "bg-muted px-4 py-3 flex justify-between items-center border-b border-border shadow-sm",
                span { class: "font-medium text-sm text-foreground", "Uploading {completed}/{total} files..." }
                button {
                    class: "text-muted-foreground hover:text-foreground p-1 rounded transition-colors",
                    onclick: move |_| {
                        upload_state.write().tasks.clear();
                        auto_scroll.set(true);
                        was_visible.set(true);
                    },
                    "✕"
                }
            }
            div {
                id: "upload-modal-list",
                class: "max-h-[400px] overflow-y-auto p-2 flex flex-col gap-1 bg-background",
                onmounted: move |e| container_mounted.set(Some(e.data())),
                onwheel: move |_| auto_scroll.set(false),
                onmousedown: move |_| auto_scroll.set(false),
                ontouchstart: move |_| auto_scroll.set(false),
                onscroll: move |_| {
                    if *check_pending.read() {
                        return;
                    }
                    check_pending.set(true);
                    spawn(async move {
                        if let (Some(active), Some(container)) = (active_mounted.read().clone(), container_mounted.read().clone()) {
                            if let (Ok(a_rect), Ok(c_rect)) = (active.get_client_rect().await, container.get_client_rect().await) {
                                // Viewport coordinates check
                                let is_visible = (a_rect.origin.y + 5.0) < c_rect.max_y() && (a_rect.max_y() - 5.0) > c_rect.origin.y;

                                let prev_visible = *was_visible.read();
                                if is_visible && !prev_visible && !*auto_scroll.read() {
                                    // User scrolled back to the item, resume auto-scroll
                                    auto_scroll.set(true);
                                }
                                was_visible.set(is_visible);
                            }
                        }
                        check_pending.set(false);
                    });
                },
                for (i, task) in tasks.clone().into_iter().enumerate() {
                    div {
                        key: "{i}-{task.status == UploadStatus::Uploading}",
                        onmounted: move |e| {
                            if task.status == UploadStatus::Uploading {
                                active_mounted.set(Some(e.data()));
                            }
                        },
                        class: if task.status == UploadStatus::Uploading {
                            "flex flex-col text-sm px-3 py-2 rounded-md transition-colors is-uploading bg-muted/50"
                        } else {
                            "flex flex-col text-sm px-3 py-2 rounded-md transition-colors"
                        },
                        div { class: "flex items-center justify-between",
                            span { class: "truncate w-[450px] text-foreground font-medium", "{task.file_name}" }
                            span {
                                class: match task.status {
                                    UploadStatus::Pending => "text-muted-foreground text-xs whitespace-nowrap",
                                    UploadStatus::Uploading => "text-blue-500 animate-pulse text-xs whitespace-nowrap font-bold",
                                    UploadStatus::Done => "text-green-500 text-xs whitespace-nowrap font-bold",
                                    UploadStatus::Error => "text-red-500 text-xs whitespace-nowrap font-bold",
                                },
                                match task.status {
                                    UploadStatus::Pending => "Pending",
                                    UploadStatus::Uploading => "Uploading...",
                                    UploadStatus::Done => "Done",
                                    UploadStatus::Error => "Error",
                                }
                            }
                        }
                        if task.status == UploadStatus::Uploading {
                            div { class: "flex items-center gap-2 mt-2",
                                div { class: "flex-grow h-1.5 bg-muted rounded-full overflow-hidden",
                                    div {
                                        class: "h-full bg-blue-500 transition-all duration-300 ease-out",
                                        style: "width: {task.progress}%"
                                    }
                                }
                                span { class: "text-[10px] text-muted-foreground font-mono w-8 text-right", "{task.progress:.0}%" }
                            }
                        }
                    }
                }
            }
        }
    }
}
