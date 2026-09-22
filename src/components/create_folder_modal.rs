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
    requests::folder::put_folder,
    AppState, ModalState, ReqwestClient,
};

#[component]
pub fn CreateFolderModal(current_path: Vec<String>) -> Element {
    let mut folder_name = use_signal::<String>(String::new);
    let mut is_saving = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let app_state = use_context::<Signal<AppState>>();
    let mut modal_state = use_context::<Signal<ModalState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    rsx! {
        Modal {
            title: rsx! { "Create Folder" },
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
                        if folder_name.read().trim().is_empty() {
                            error_msg.set(Some("Folder name cannot be empty".to_string()));
                            return;
                        }
                        let current_path = current_path.clone();
                        is_saving.set(true);
                        spawn(async move {
                            let result = put_folder(
                                reqwest_client_context,
                                current_path,
                                folder_name.read().to_string(),
                            )
                            .await;
                            finish_modal_action(
                                app_state,
                                modal_state,
                                is_saving,
                                error_msg,
                                result,
                                "create folder",
                            );
                        });
                    },
                    disabled: is_saving(),
                    if is_saving() {
                        "Saving..."
                    } else {
                        "Create"
                    }
                }
            },
            div { class: "flex flex-col gap-4 my-4",
                div { class: "flex flex-col gap-2",
                    label { class: "text-sm font-medium", "Folder Name" }
                    input {
                        class: "h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                        r#type: "text",
                        autofocus: true,
                        onmounted: move |e| async move {
                            let _ = e.set_focus(true).await;
                        },
                        value: "{folder_name.read()}",
                        placeholder: "New Folder",
                        oninput: move |e| {
                            folder_name.set(e.value());
                            error_msg.set(None);
                        },
                    }
                }
                ErrorMessage { message: error_msg.read().clone() }
            }
        }
    }
}
