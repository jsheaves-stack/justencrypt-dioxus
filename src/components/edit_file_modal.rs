use dioxus::prelude::*;

use crate::{
    components::{
        modal_action::finish_modal_action,
        ui::{
            button::{Button, ButtonVariant},
            error_message::ErrorMessage,
            modal::Modal,
            spinner::Spinner,
        },
    },
    requests::file::{get_file_text, put_file_text},
    AppState, ModalState, ReqwestClient,
};

#[component]
pub fn EditFileModal(file_name: String, current_path: Vec<String>) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut modal_state = use_context::<Signal<ModalState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut file_content = use_signal::<Option<String>>(|| None);
    let mut is_saving = use_signal(|| false);
    let error_msg = use_signal::<Option<String>>(|| None);

    let current_path_clone = current_path.clone();
    let file_name_clone = file_name.clone();

    use_effect(use_reactive!(|(current_path_clone, file_name_clone)| {
        spawn(async move {
            if let Ok(text) =
                get_file_text(reqwest_client_context, current_path_clone, file_name_clone).await
            {
                file_content.set(Some(text));
            } else {
                file_content.set(Some("Error loading file.".to_string()));
            }
        });
    }));

    rsx! {
        Modal {
            title: rsx! { "Editing: {file_name}" },
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
                        let text = file_content.read().clone().unwrap_or_default();
                        let current_path = current_path.clone();
                        let file_name = file_name.clone();
                        is_saving.set(true);
                        spawn(async move {
                            let result =
                                put_file_text(reqwest_client_context, current_path, file_name, text)
                                    .await;
                            finish_modal_action(
                                app_state,
                                modal_state,
                                is_saving,
                                error_msg,
                                result,
                                "save file",
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
            ErrorMessage { message: error_msg.read().clone(), class: "mb-2" }
            div { class: "flex-grow flex flex-col min-h-0",
                if let Some(content) = file_content.read().as_ref() {
                    textarea {
                        class: "w-full h-full flex-grow resize-none rounded-md border border-input bg-background text-foreground px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 font-mono",
                        autofocus: true,
                        onmounted: move |e| async move {
                            let _ = e.set_focus(true).await;
                        },
                        value: content.clone(),
                        oninput: move |e| {
                            file_content.set(Some(e.value()));
                        },
                    }
                } else {
                    div { class: "flex items-center justify-center h-full",
                        Spinner { class: "h-8 w-8 text-primary" }
                    }
                }
            }
        }
    }
}
