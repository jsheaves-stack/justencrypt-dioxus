use dioxus::prelude::*;

use crate::{
    components::ui::button::{Button, ButtonVariant},
    requests::{file::delete_file, folder::delete_folder},
    AppState, ReqwestClient,
};

#[component]
pub fn DeleteModal(name: String, current_path: Vec<String>, is_file: bool) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut is_deleting = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);

    let name_clone = name.clone();
    let current_path_clone = current_path.clone();
    let is_file_clone = is_file;

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none p-4",
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" {
                    app_state.write().delete_modal = None;
                }
            },
            onclick: move |_| {
                app_state.write().delete_modal = None;
            },
            div {
                class: "relative flex w-full max-w-[400px] flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4",
                    h2 { class: "text-xl font-semibold text-destructive", "Delete ", {if is_file_clone { "File" } else { "Folder" }} }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2",
                        onclick: move |_| {
                            app_state.write().delete_modal = None;
                        },
                        "✕"
                    }
                }

                div { class: "flex flex-col gap-4 my-4",
                    p {
                        "Are you sure you want to permanently delete the ",
                        {if is_file_clone { "file" } else { "folder" }},
                        " '",
                        span { class: "font-semibold", "{name}" },
                        "'? This action cannot be undone."
                    }
                    if let Some(err) = error_msg.read().as_ref() {
                        span { class: "text-sm text-destructive", "{err}" }
                    }
                }

                div { class: "flex justify-end mt-2 gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        autofocus: true,
                        onclick: move |_| {
                            app_state.write().delete_modal = None;
                        },
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| {
                            let current_path = current_path_clone.clone();
                            let target_name = name_clone.clone();
                            let is_file_flag = is_file_clone;

                            is_deleting.set(true);
                            spawn(async move {
                                let result = if is_file_flag {
                                    delete_file(reqwest_client_context, current_path, target_name).await
                                } else {
                                    delete_folder(reqwest_client_context, current_path, target_name).await
                                };

                                match result {
                                    Ok(_) => {
                                        is_deleting.set(false);
                                        app_state.write().delete_modal = None;
                                        app_state.write().needs_refetch = true;
                                    }
                                    Err(_) => {
                                        is_deleting.set(false);
                                        error_msg.set(Some(format!("Failed to delete {}", if is_file_flag { "file" } else { "folder" })));
                                    }
                                }
                            });
                        },
                        disabled: is_deleting(),
                        if is_deleting() { "Deleting..." } else { "Delete" }
                    }
                }
            }
        }
    }
}
