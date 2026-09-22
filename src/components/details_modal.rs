use dioxus::prelude::*;

use crate::{
    components::ui::{button::Button, modal::Modal},
    models::{File, Folder, FolderContent},
    ModalState,
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
                format!(
                    "{:.2} GB ({} bytes)",
                    size as f64 / (1024.0 * 1024.0 * 1024.0),
                    size
                )
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
            let main_part = s.split('.').next().unwrap_or(s);
            format!("{} UTC", main_part)
        }
        _ => "-".to_string(),
    }
}

#[component]
pub fn DetailsModal(item: FolderContent, current_path: Vec<String>) -> Element {
    let mut modal_state = use_context::<Signal<ModalState>>();

    let (item_type, name, size_str, created_str, modified_str, contains) = match &item {
        FolderContent::File(File {
            file_name,
            file_extension,
            file_size,
            created_date,
            last_modified_date,
        }) => {
            let item_type = format!(
                "File ({})",
                file_extension
                    .as_deref()
                    .unwrap_or("Unknown")
                    .to_uppercase()
            );
            let size_str = format_file_size(*file_size);
            let created_str = format_date(created_date.as_deref());
            let modified_str = format_date(last_modified_date.as_deref());
            (
                item_type,
                file_name.clone(),
                size_str,
                created_str,
                modified_str,
                None,
            )
        }
        FolderContent::Folder(Folder {
            file_name,
            file_size,
            created_date,
            last_modified_date,
            file_count,
            folder_count,
        }) => {
            let item_type = "File Folder".to_string();
            let size_str = format_file_size(*file_size);
            let created_str = format_date(created_date.as_deref());
            let modified_str = format_date(last_modified_date.as_deref());
            let contains = Some(format!("{} files, {} folders", file_count, folder_count));
            (
                item_type,
                file_name.clone(),
                size_str,
                created_str,
                modified_str,
                contains,
            )
        }
    };

    let location = if current_path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", current_path.join("/"))
    };

    rsx! {
        Modal {
            title: rsx! { "Properties" },
            class: "w-[90vw] max-w-[500px] sm:w-fit sm:min-w-[400px]",
            on_close: move |_| modal_state.write().close_all_modals(),
            footer: rsx! {
                Button { onclick: move |_| modal_state.write().close_all_modals(), "Close" }
            },
            div { class: "flex flex-col gap-3 my-2 text-sm",
                div { class: "grid grid-cols-[80px,1fr] sm:grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                    span { class: "font-medium text-muted-foreground", "Name:" }
                    span { class: "font-semibold break-all text-foreground", {name} }
                }
                div { class: "grid grid-cols-[80px,1fr] sm:grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                    span { class: "font-medium text-muted-foreground", "Type:" }
                    span { class: "text-foreground", {item_type} }
                }
                div { class: "grid grid-cols-[80px,1fr] sm:grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                    span { class: "font-medium text-muted-foreground", "Location:" }
                    span { class: "font-mono text-xs text-foreground break-all", {location} }
                }
                div { class: "grid grid-cols-[80px,1fr] sm:grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                    span { class: "font-medium text-muted-foreground", "Size:" }
                    span { class: "text-foreground", {size_str} }
                }
                if let Some(contains_str) = contains {
                    div { class: "grid grid-cols-[80px,1fr] sm:grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                        span { class: "font-medium text-muted-foreground", "Contains:" }
                        span { class: "text-foreground", {contains_str} }
                    }
                }
                div { class: "grid grid-cols-[80px,1fr] sm:grid-cols-[110px,1fr] gap-2 items-baseline py-1.5 border-b border-border/40",
                    span { class: "font-medium text-muted-foreground", "Created:" }
                    span { class: "text-foreground", {created_str} }
                }
                div { class: "grid grid-cols-[80px,1fr] sm:grid-cols-[110px,1fr] gap-2 items-baseline py-1.5",
                    span { class: "font-medium text-muted-foreground", "Modified:" }
                    span { class: "text-foreground", {modified_str} }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_file_size_handles_each_unit_bracket() {
        assert_eq!(format_file_size(Some(500)), "500 bytes");
        assert_eq!(format_file_size(Some(2048)), "2.00 KB (2048 bytes)");
        assert_eq!(
            format_file_size(Some(5 * 1024 * 1024)),
            "5.00 MB (5242880 bytes)"
        );
        assert_eq!(
            format_file_size(Some(3 * 1024 * 1024 * 1024)),
            "3.00 GB (3221225472 bytes)"
        );
    }

    #[test]
    fn format_file_size_handles_missing_or_negative() {
        assert_eq!(format_file_size(None), "-");
        assert_eq!(format_file_size(Some(-1)), "-");
    }

    #[test]
    fn format_date_strips_time_and_zone_markers() {
        assert_eq!(
            format_date(Some("2024-01-02T03:04:05.123Z")),
            "2024-01-02 03:04:05 UTC"
        );
    }

    #[test]
    fn format_date_handles_missing_or_empty() {
        assert_eq!(format_date(None), "-");
        assert_eq!(format_date(Some("")), "-");
    }
}
