use dioxus::prelude::*;

use crate::{
    components::{
        file_explorer::FolderItem,
        ui::button::Button,
    },
    AppState,
};

fn format_file_size(size_opt: Option<i64>) -> String {
    match size_opt {
        Some(size) if size >= 0 => {
            if size < 1024 {
                format!("{} bytes", size)
            } else if size < 1024 * 1024 {
                format!("{:.2} KB ({} bytes)", size as f64 / 1024.0, size)
            } else if size < 1024 * 1024 * 1024 {
                format!("{:.2} MB ({} bytes)", size as f64 / (1024.0 * 1024.0), size)
            } else {
                format!("{:.2} GB ({} bytes)", size as f64 / (1024.0 * 1024.0 * 1024.0), size)
            }
        }
        _ => "-".to_string(),
    }
}

fn format_date(date_opt: Option<&str>) -> String {
    match date_opt {
        Some(s) if !s.is_empty() => {
            let s = s.replace('T', " ");
            let s = s.trim_end_matches('Z');
            let main_part = s.split('.').next().unwrap_or(&s);
            format!("{} UTC", main_part)
        }
        _ => "-".to_string(),
    }
}

#[component]
pub fn DetailsModal(item: FolderItem, current_path: Vec<String>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let item_type = if item.is_file {
        format!(
            "File ({})",
            item.file_extension
                .as_deref()
                .unwrap_or("Unknown")
                .to_uppercase()
        )
    } else {
        "File Folder".to_string()
    };

    let location = if current_path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", current_path.join("/"))
    };

    let size_str = if item.is_file {
        format_file_size(item.file_size)
    } else {
        "-".to_string()
    };

    let created_str = if item.is_file {
        format_date(item.created_date.as_deref())
    } else {
        "-".to_string()
    };

    let modified_str = if item.is_file {
        format_date(item.last_modified_date.as_deref())
    } else {
        "-".to_string()
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none p-4",
            tabindex: "-1",
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" {
                    app_state.write().details_modal = None;
                }
            },
            onclick: move |_| {
                app_state.write().details_modal = None;
            },
            div {
                class: "relative flex w-full max-w-[500px] flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-6 pb-2 border-b border-border",
                    h2 { class: "text-xl font-semibold flex items-center gap-2", "Properties" }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2 transition-colors",
                        onclick: move |_| {
                            app_state.write().details_modal = None;
                        },
                        "✕"
                    }
                }

                div { class: "flex flex-col gap-3 my-2 text-sm",
                    div { class: "grid grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                        span { class: "font-medium text-muted-foreground", "Name:" }
                        span { class: "font-semibold break-all text-foreground", "{item.file_name}" }
                    }
                    div { class: "grid grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                        span { class: "font-medium text-muted-foreground", "Type:" }
                        span { class: "text-foreground", "{item_type}" }
                    }
                    div { class: "grid grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                        span { class: "font-medium text-muted-foreground", "Location:" }
                        span { class: "font-mono text-xs text-foreground break-all",
                            "{location}"
                        }
                    }
                    div { class: "grid grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                        span { class: "font-medium text-muted-foreground", "Size:" }
                        span { class: "text-foreground", "{size_str}" }
                    }
                    div { class: "grid grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                        span { class: "font-medium text-muted-foreground", "Created:" }
                        span { class: "text-foreground", "{created_str}" }
                    }
                    div { class: "grid grid-cols-[110px,1fr] gap-2 items-baseline py-1.5",
                        span { class: "font-medium text-muted-foreground", "Modified:" }
                        span { class: "text-foreground", "{modified_str}" }
                    }
                }

                div { class: "flex justify-end mt-6 pt-3 border-t border-border",
                    Button {
                        onclick: move |_| {
                            app_state.write().details_modal = None;
                        },
                        "Close"
                    }
                }
            }
        }
    }
}
