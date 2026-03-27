use dioxus::prelude::*;

use crate::{
    components::ui::button::{Button, ButtonVariant},
    requests::folder::put_folder,
    ReqwestClient,
};

#[component]
pub fn CreateFolderModal(current_path: Vec<String>) -> Element {
    let mut folder_name = use_signal::<String>(String::new);
    let mut is_saving = use_signal(|| false);
    let mut app_state = use_context::<Signal<crate::AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none p-4",
            tabindex: "-1",
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" {
                    app_state.write().create_folder_modal = false;
                }
            },
            onclick: move |_| {
                app_state.write().create_folder_modal = false;
            },
            div {
                class: "relative flex w-full max-w-[400px] flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4",
                    h2 { class: "text-xl font-semibold", "Create Folder" }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2",
                        onclick: move |_| {
                            app_state.write().create_folder_modal = false;
                        },
                        "✕"
                    }
                }

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
                            },
                        }
                    }
                }

                div { class: "flex justify-end mt-2 gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            app_state.write().create_folder_modal = false;
                        },
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            if folder_name.read().trim().is_empty() {
                                return;
                            }
                            let current_path = current_path.clone();
                            is_saving.set(true);
                            spawn(async move {
                                match put_folder(
                                        reqwest_client_context,
                                        current_path,
                                        folder_name.read().to_string(),
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        is_saving.set(false);
                                        app_state.write().create_folder_modal = false;
                                        app_state.write().needs_refetch = true;
                                    }
                                    Err(_) => {
                                        is_saving.set(false);
                                    }
                                }
                            });
                        },
                        disabled: is_saving(),
                        if is_saving() {
                            "Saving..."
                        } else {
                            "Create"
                        }
                    }
                }
            }
        }
    }
}
