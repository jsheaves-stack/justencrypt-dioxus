use crate::{components::ui::spinner::Spinner, UploadState, UploadStatus};
use dioxus::prelude::*;

#[component]
pub fn UploadModal() -> Element {
    let mut upload_state = use_context::<Signal<UploadState>>();

    let tasks = upload_state.read().tasks.clone();

    if tasks.is_empty() {
        return rsx! {};
    }

    let completed = tasks
        .iter()
        .filter(|t| t.status == UploadStatus::Done || t.status == UploadStatus::Error)
        .count();
    let total = tasks.len();
    let overall_progress = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let active_tasks: Vec<_> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.status == UploadStatus::Uploading)
        .collect();
    let queued_tasks: Vec<_> = tasks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.status != UploadStatus::Uploading)
        .collect();

    rsx! {
        div { class: "fixed bottom-0 sm:bottom-6 sm:right-6 left-0 right-0 sm:left-auto w-full sm:w-[500px] bg-card border-t sm:border border-border sm:rounded-xl shadow-2xl z-[100] flex flex-col overflow-hidden animate-in slide-in-from-bottom-4 fade-in duration-300 max-h-[80vh]",
            // Header
            div { class: "bg-background/95 backdrop-blur px-4 py-4 border-b border-border shadow-sm shrink-0 flex items-start justify-between gap-4",
                div { class: "flex flex-col gap-2 w-full mt-1",
                    div { class: "flex items-center justify-between",
                        span { class: "font-semibold text-sm text-foreground tracking-tight",
                            "Uploaded {completed} of {total} files"
                        }
                        div { class: "flex items-center gap-2",
                            select {
                                class: "text-xs bg-background text-foreground border border-border rounded-md px-2 py-1 outline-none focus:ring-1 focus:ring-primary cursor-pointer transition-colors hover:bg-muted",
                                value: "{upload_state.read().max_concurrent}",
                                onchange: move |e| {
                                    if let Ok(val) = e.value().parse::<usize>() {
                                        upload_state.write().max_concurrent = val;
                                    }
                                },
                                option { value: "1", "1 concurrent" }
                                option { value: "5", "5 concurrent" }
                                option { value: "10", "10 concurrent" }
                                option { value: "20", "20 concurrent" }
                                option { value: "50", "50 concurrent" }
                            }
                        }
                    }
                    div { class: "w-full h-1.5 bg-muted rounded-full overflow-hidden mt-1",
                        div {
                            class: "h-full bg-primary transition-[width] duration-500 ease-out",
                            style: "width: {overall_progress}%",
                        }
                    }
                }
                button {
                    class: "text-muted-foreground hover:text-foreground p-1.5 rounded-lg hover:bg-muted transition-colors shrink-0",
                    onclick: move |_| {
                        upload_state.write().tasks.clear();
                    },
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "18",
                        height: "18",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        "stroke-width": "2",
                        "stroke-linecap": "round",
                        "stroke-linejoin": "round",
                        path { d: "M18 6 6 18" }
                        path { d: "m6 6 12 12" }
                    }
                }
            }

            // Body
            div { class: "flex flex-col flex-1 overflow-hidden bg-background/50",
                // Active Uploads (Sticky)
                if !active_tasks.is_empty() {
                    div { class: "flex flex-col gap-2 p-3 bg-card shrink-0 border-b border-border/50 shadow-sm z-10 max-h-[30vh] overflow-y-auto",
                        for (i , task) in active_tasks {
                            div {
                                key: "{i}-active",
                                class: "flex flex-col p-3 bg-muted/30 rounded-lg border border-border/50 shadow-sm",
                                div { class: "flex items-center justify-between gap-3 mb-2",
                                    span { class: "text-sm text-foreground font-medium truncate",
                                        "{task.file_name}"
                                    }
                                    div { class: "flex items-center gap-1.5 shrink-0",
                                        Spinner { class: "w-3 h-3 text-primary" }
                                        span { class: "text-[11px] text-primary font-bold uppercase tracking-wide",
                                            "Uploading"
                                        }
                                    }
                                }
                                div { class: "flex items-center gap-3",
                                    div { class: "flex-grow h-1.5 bg-muted rounded-full overflow-hidden",
                                        div {
                                            class: "h-full bg-primary transition-[width] duration-300 ease-out",
                                            style: "width: {task.progress}%",
                                        }
                                    }
                                    span { class: "text-[11px] text-muted-foreground font-mono w-9 text-right",
                                        "{task.progress:.0}%"
                                    }
                                }
                            }
                        }
                    }
                }

                // Queue and History (Scrollable)
                if !queued_tasks.is_empty() {
                    div { class: "flex flex-col gap-0.5 p-2 overflow-y-auto bg-muted/10",
                        for (i , task) in queued_tasks {
                            div {
                                key: "{i}-queued",
                                class: "flex items-center justify-between px-3 py-2.5 hover:bg-muted/50 rounded-md transition-colors group",
                                span { class: "text-sm text-muted-foreground group-hover:text-foreground transition-colors truncate pr-4",
                                    "{task.file_name}"
                                }
                                match task.status {
                                    UploadStatus::Pending => rsx! {
                                        span { class: "text-xs text-muted-foreground/70 shrink-0 uppercase tracking-wider font-medium text-[10px]",
                                            "Waiting"
                                        }
                                    },
                                    UploadStatus::Done => rsx! {
                                        span { class: "text-xs text-emerald-500 font-medium shrink-0 flex items-center gap-1.5",
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "14",
                                                height: "14",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                "stroke-width": "2.5",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                path { d: "M20 6 9 17l-5-5" }
                                            }
                                            "Done"
                                        }
                                    },
                                    UploadStatus::Error => rsx! {
                                        span { class: "text-xs text-red-500 font-medium shrink-0 flex items-center gap-1.5",
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "14",
                                                height: "14",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                "stroke-width": "2.5",
                                                "stroke-linecap": "round",
                                                "stroke-linejoin": "round",
                                                circle { cx: "12", cy: "12", r: "10" }
                                                path { d: "m15 9-6 6" }
                                                path { d: "m9 9 6 6" }
                                            }
                                            "Error"
                                        }
                                    },
                                    _ => rsx! {},
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
