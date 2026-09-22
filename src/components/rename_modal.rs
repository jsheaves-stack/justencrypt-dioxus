use dioxus::prelude::*;

use crate::{
    components::{
        modal_action::finish_modal_action,
        ui::{
            button::{Button, ButtonVariant},
            error_message::ErrorMessage,
            modal::Modal,
        },
    },
    models::ItemKey,
    requests::{file::patch_file, folder::patch_folder},
    AppState, ModalState, ReqwestClient,
};

#[component]
pub fn RenameModal(item: ItemKey, current_path: Vec<String>) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut modal_state = use_context::<Signal<ModalState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let old_name = item.name.clone();
    let mut new_name = use_signal::<String>(|| old_name.clone());
    let mut is_saving = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);

    let old_name_clone = old_name.clone();
    let current_path_clone = current_path.clone();
    let is_file_clone = item.is_file;

    rsx! {
        Modal {
            title: rsx! {
                "Rename "
                {if is_file_clone { "File" } else { "Folder" }}
            },
            class: "max-w-[400px]",
            closable: !is_saving(),
            on_close: move |_| modal_state.write().close_all_modals(),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: is_saving(),
                    onclick: move |_| modal_state.write().close_all_modals(),
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
                            modal_state.write().close_all_modals();
                            return;
                        }

                        let current_path = current_path_clone.clone();
                        let old_name_str = old_name_clone.clone();
                        let is_file_flag = is_file_clone;

                        is_saving.set(true);
                        spawn(async move {
                            let new_parent_path = current_path.join("/");
                            let result = if is_file_flag {
                                patch_file(
                                        reqwest_client_context,
                                        current_path.clone(),
                                        old_name_str,
                                        new_parent_path,
                                        new_name_str,
                                    )
                                    .await
                            } else {
                                patch_folder(
                                        reqwest_client_context,
                                        current_path.clone(),
                                        old_name_str,
                                        new_parent_path,
                                        new_name_str,
                                    )
                                    .await
                            };

                            let action = if is_file_flag {
                                "rename file"
                            } else {
                                "rename folder"
                            };
                            finish_modal_action(
                                app_state,
                                modal_state,
                                is_saving,
                                error_msg,
                                result,
                                action,
                            );
                        });
                    },
                    disabled: is_saving(),
                    if is_saving() {
                        "Saving..."
                    } else {
                        "Save"
                    }
                }
            },
            div { class: "flex flex-col gap-4 my-4",
                div { class: "flex flex-col gap-2",
                    label { class: "text-sm font-medium", "New Name" }
                    input {
                        class: "h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                        r#type: "text",
                        autofocus: true,
                        onmounted: move |e| async move {
                            let _ = e.set_focus(true).await;
                        },
                        value: "{new_name.read()}",
                        oninput: move |e| {
                            new_name.set(e.value());
                            error_msg.set(None);
                        },
                    }
                }
                ErrorMessage { message: error_msg.read().clone() }
            }
        }
    }
}
