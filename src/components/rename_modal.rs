use dioxus::prelude::*;

use crate::{
    components::ui::button::{Button, ButtonVariant},
    requests::file::patch_file,
    ReqwestClient, AppState,
};

#[component]
pub fn RenameModal(old_name: String, current_path: Vec<String>, is_file: bool) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    
    let mut new_name = use_signal::<String>(|| old_name.clone());
    let mut is_saving = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);

    let old_name_clone = old_name.clone();
    let current_path_clone = current_path.clone();
    let is_file_clone = is_file.clone();

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm",
            div { class: "relative flex w-[400px] flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg",
                div { class: "flex justify-between items-center mb-4",
                    h2 { class: "text-xl font-semibold", "Rename ", {if is_file_clone { "File" } else { "Folder" }} }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2",
                        onclick: move |_| {
                            app_state.write().rename_modal = None;
                        },
                        "✕"
                    }
                }
                
                div { class: "flex flex-col gap-4 my-4",
                    div { class: "flex flex-col gap-2",
                        label { class: "text-sm font-medium", "New Name" }
                        input {
                            class: "h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                            r#type: "text",
                            value: "{new_name.read()}",
                            oninput: move |e| {
                                new_name.set(e.value());
                                error_msg.set(None);
                            },
                        }
                    }
                    if let Some(err) = error_msg.read().as_ref() {
                        span { class: "text-sm text-destructive", "{err}" }
                    }
                }

                div { class: "flex justify-end mt-2 gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            app_state.write().rename_modal = None;
                        },
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            let new_name_str = new_name.read().clone();
                            if new_name_str.trim().is_empty() {
                                error_msg.set(Some("Name cannot be empty".to_string()));
                                return;
                            }
                            if new_name_str == old_name_clone {
                                app_state.write().rename_modal = None;
                                return;
                            }

                            let current_path = current_path_clone.clone();
                            let old_name_str = old_name_clone.clone();
                            let is_file_flag = is_file_clone.clone();
                            
                            is_saving.set(true);
                            spawn(async move {
                                if is_file_flag {
                                    let new_parent_path = current_path.join("/");
                                    match patch_file(reqwest_client_context, current_path.clone(), old_name_str, new_parent_path, new_name_str).await {
                                        Ok(_) => {
                                            is_saving.set(false);
                                            app_state.write().rename_modal = None;
                                            app_state.write().needs_refetch = true;
                                        }
                                        Err(_) => {
                                            is_saving.set(false);
                                            error_msg.set(Some("Failed to rename file".to_string()));
                                        }
                                    }
                                } else {
                                    // API doesn't support folder rename yet based on the spec
                                    is_saving.set(false);
                                    error_msg.set(Some("Folder renaming is not supported by API yet".to_string()));
                                }
                            });
                        },
                        disabled: is_saving(),
                        if is_saving() { "Saving..." } else { "Save" }
                    }
                }
            }
        }
    }
}