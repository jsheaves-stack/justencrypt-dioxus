use dioxus::prelude::*;

use crate::{
    components::ui::button::{Button, ButtonVariant},
    requests::{file::delete_file, folder::delete_folder},
    AppState, ReqwestClient,
};

#[component]
pub fn DeleteModal(items: Vec<(String, bool)>, current_path: Vec<String>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut is_deleting = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);

    let items_clone = items.clone();
    let current_path_clone = current_path.clone();

    let items_count = items.len();
    let title = if items_count == 1 {
        let is_file = items[0].1;
        format!("Delete {}", if is_file { "File" } else { "Folder" })
    } else {
        format!("Delete {} Items", items_count)
    };

    let description_content = if items_count == 1 {
        let name = items[0].0.clone();
        let is_file = items[0].1;
        rsx! {
            "Are you sure you want to permanently delete the "
            {if is_file { "file" } else { "folder" }}
            " '"
            span { class: "font-semibold", {name.clone()} }
            "'? This action cannot be undone."
        }
    } else {
        rsx! {
            "Are you sure you want to permanently delete these "
            span { class: "font-semibold", "{items_count} items" }
            "? This action cannot be undone."
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none p-4 animate-in fade-in duration-200",
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" && !*is_deleting.read() {
                    app_state.write().delete_modal = None;
                }
            },
            onclick: move |_| {
                if !*is_deleting.read() {
                    app_state.write().delete_modal = None;
                }
            },
            div {
                class: "relative flex w-full max-w-[400px] flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg animate-in zoom-in-95 duration-200",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4",
                    h2 { class: "text-xl font-semibold text-destructive", {title.clone()} }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2",
                        disabled: is_deleting(),
                        onclick: move |_| {
                            if !*is_deleting.read() {
                                app_state.write().delete_modal = None;
                            }
                        },
                        "✕"
                    }
                }

                div { class: "flex flex-col gap-4 my-4",
                    p { {description_content} }
                    if let Some(err) = error_msg.read().as_ref() {
                        span { class: "text-sm text-destructive font-medium", {err.clone()} }
                    }
                }

                div { class: "flex justify-end mt-2 gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        autofocus: true,
                        disabled: is_deleting(),
                        onclick: move |_| {
                            app_state.write().delete_modal = None;
                        },
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        disabled: is_deleting(),
                        onclick: move |_| {
                            let current_path = current_path_clone.clone();
                            let items = items_clone.clone();

                            is_deleting.set(true);
                            spawn(async move {
                                let mut has_error = false;

                                for (target_name, is_file) in items.into_iter() {
                                    let result = if is_file {
                                        delete_file(
                                                reqwest_client_context,

                                                current_path.clone(),
                                                target_name,
                                            )
                                            .await
                                    } else {
                                        delete_folder(
                                                reqwest_client_context,
                                                current_path.clone(),
                                                target_name,
                                            )
                                            .await
                                    };
                                    if result.is_err() {
                                        has_error = true;
                                        break;
                                    }
                                }
                                is_deleting.set(false);
                                if has_error {
                                    error_msg
                                        .set(
                                            Some(
                                                if items_count == 1 {
                                                    "Failed to delete item.".to_string()
                                                } else {
                                                    "An error occurred while deleting items. Some items may not have been deleted."
                                                        .to_string()
                                                },
                                            ),
                                        );
                                } else {
                                    app_state.write().selected_files.clear();
                                    app_state.write().delete_modal = None;
                                    app_state.write().needs_refetch = true;
                                }
                            });
                        },
                        if is_deleting() {
                            "Deleting..."
                        } else {
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
