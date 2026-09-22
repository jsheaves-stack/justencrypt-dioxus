use dioxus::prelude::*;

use crate::{
    components::{
        file_explorer::EDITABLE_FORMATS,
        modal_action::finish_modal_action,
        ui::{
            button::{Button, ButtonVariant},
            error_message::ErrorMessage,
            modal::Modal,
        },
    },
    requests::file::put_file_text,
    AppState, ModalState, ReqwestClient,
};

#[component]
pub fn CreateFileModal(current_path: Vec<String>) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut modal_state = use_context::<Signal<ModalState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut file_name = use_signal::<String>(String::new);
    let mut file_extension = use_signal::<String>(|| EDITABLE_FORMATS[0].to_string());
    let mut file_content = use_signal::<String>(String::new);
    let mut is_saving = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);

    let current_path_clone = current_path.clone();

    rsx! {
        Modal {
            title: rsx! { "Create New File" },
            class: "max-w-5xl max-h-[85vh] sm:h-5/6",
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
                        let name = file_name.read().clone();
                        if name.trim().is_empty() {
                            error_msg.set(Some("File name cannot be empty".to_string()));
                            return;
                        }

                        let ext = file_extension.read().clone().to_lowercase();
                        let full_file_name = format!("{}.{}", name.trim(), ext);
                        let text = file_content.read().clone();
                        let current_path = current_path_clone.clone();

                        is_saving.set(true);
                        spawn(async move {
                            let result = put_file_text(
                                reqwest_client_context,
                                current_path,
                                full_file_name,
                                text,
                            )
                            .await;
                            finish_modal_action(
                                app_state,
                                modal_state,
                                is_saving,
                                error_msg,
                                result,
                                "create file",
                            );
                        });
                    },
                    disabled: is_saving(),
                    if is_saving() {
                        "Creating..."
                    } else {
                        "Create"
                    }
                }
            },
            div { class: "flex flex-col sm:flex-row items-start sm:items-center gap-2 w-full mb-4",
                label { class: "text-sm font-medium", "File Name:" }
                input {
                    class: "h-10 w-full sm:flex-grow rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                    r#type: "text",
                    autofocus: true,
                    onmounted: move |e| async move {
                        let _ = e.set_focus(true).await;
                    },
                    placeholder: "my_new_file",
                    value: "{file_name.read()}",
                    oninput: move |e| {
                        file_name.set(e.value());
                        error_msg.set(None);
                    },
                }
                span { class: "text-sm font-medium", "." }
                select {
                    class: "h-10 w-[120px] rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 cursor-pointer",
                    onchange: move |e| {
                        file_extension.set(e.value());
                    },
                    for ext in EDITABLE_FORMATS.iter() {
                        option {
                            value: *ext,
                            selected: *ext == file_extension.read().as_str(),
                            "{ext.to_lowercase()}"
                        }
                    }
                }
            }

            ErrorMessage { message: error_msg.read().clone(), class: "mb-2" }

            div { class: "flex-grow flex flex-col min-h-0",
                textarea {
                    class: "w-full h-full flex-grow resize-none rounded-md border border-input bg-background text-foreground px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 font-mono",
                    placeholder: "Write your file content here...",
                    value: "{file_content.read()}",
                    oninput: move |e| {
                        file_content.set(e.value());
                    },
                }
            }
        }
    }
}
