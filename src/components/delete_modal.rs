use dioxus::prelude::*;

use crate::{
    components::{
        modal_action::finish_bulk_modal_action,
        ui::{
            button::{Button, ButtonVariant},
            error_message::ErrorMessage,
            modal::Modal,
        },
    },
    models::ItemKey,
    requests::{file::delete_file, folder::delete_folder},
    AppState, ModalState, ReqwestClient, SelectionState,
};

#[component]
pub fn DeleteModal(items: Vec<ItemKey>, current_path: Vec<String>) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut modal_state = use_context::<Signal<ModalState>>();
    let selection_state = use_context::<Signal<SelectionState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut is_deleting = use_signal(|| false);
    let error_msg = use_signal::<Option<String>>(|| None);

    let items_clone = items.clone();
    let current_path_clone = current_path.clone();

    let items_count = items.len();
    let title = if items_count == 1 {
        let is_file = items[0].is_file;
        format!("Delete {}", if is_file { "File" } else { "Folder" })
    } else {
        format!("Delete {} Items", items_count)
    };

    let description_content = if items_count == 1 {
        let name = items[0].name.clone();
        let is_file = items[0].is_file;
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
        Modal {
            title: rsx! {
                span { class: "text-destructive", {title.clone()} }
            },
            class: "max-w-[400px]",
            closable: !is_deleting(),
            on_close: move |_| modal_state.write().close_all_modals(),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Outline,
                    autofocus: true,
                    disabled: is_deleting(),
                    onclick: move |_| modal_state.write().close_all_modals(),
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
                            let results = futures_util::future::join_all(items.into_iter().map(|item| {
                                let current_path = current_path.clone();
                                async move {
                                    if item.is_file {
                                        delete_file(reqwest_client_context, current_path, item.name)
                                            .await
                                    } else {
                                        delete_folder(reqwest_client_context, current_path, item.name)
                                            .await
                                    }
                                }
                            }))
                            .await;

                            let first_error = results.into_iter().find_map(Result::err);
                            finish_bulk_modal_action(
                                app_state,
                                modal_state,
                                selection_state,
                                is_deleting,
                                error_msg,
                                first_error,
                                items_count,
                                "delete item",
                                "An error occurred while deleting items. Some items may not have been deleted.",
                            );
                        });
                    },
                    if is_deleting() {
                        "Deleting..."
                    } else {
                        "Delete"
                    }
                }
            },
            div { class: "flex flex-col gap-4 my-4",
                p { {description_content} }
                ErrorMessage { message: error_msg.read().clone() }
            }
        }
    }
}
