use dioxus::prelude::*;

use crate::{
    components::ui::button::{Button, ButtonVariant},
    requests::file::put_file_text,
    components::file_explorer::EDITABLE_FORMATS,
    ReqwestClient, AppState,
};

#[component]
pub fn CreateFileModal(current_path: Vec<String>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    
    let mut file_name = use_signal::<String>(|| String::new());
    let mut file_extension = use_signal::<String>(|| EDITABLE_FORMATS[0].to_string());
    let mut file_content = use_signal::<String>(|| String::new());
    let mut is_saving = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);

    let current_path_clone = current_path.clone();

    rsx! {
        div { 
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none",
            tabindex: "-1",
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" {
                    app_state.write().create_file_modal = false;
                }
            },
            onclick: move |_| {
                app_state.write().create_file_modal = false;
            },
            div { 
                class: "relative flex w-11/12 max-w-5xl h-5/6 flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4",
                    h2 { class: "text-xl font-semibold", "Create New File" }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2",
                        onclick: move |_| {
                            app_state.write().create_file_modal = false;
                        },
                        "✕"
                    }
                }
                
                div { class: "flex items-center gap-2 mb-4",
                    label { class: "text-sm font-medium", "File Name:" }
                    input {
                        class: "h-10 flex-grow rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
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
                                value: "{ext}",
                                selected: *ext == file_extension.read().as_str(),
                                "{ext.to_lowercase()}"
                            }
                        }
                    }
                }

                if let Some(err) = error_msg.read().as_ref() {
                    span { class: "text-sm text-destructive mb-2", "{err}" }
                }
                
                div { class: "flex-grow flex flex-col min-h-0",
                    textarea {
                        class: "w-full h-full flex-grow resize-none rounded-md border border-input bg-background text-foreground px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 font-mono",
                        placeholder: "Write your file content here...",
                        value: "{file_content.read()}",
                        oninput: move |e| {
                            file_content.set(e.value());
                        }
                    }
                }

                div { class: "flex justify-end mt-4 gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            app_state.write().create_file_modal = false;
                        },
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
                                match put_file_text(reqwest_client_context, current_path, full_file_name, text).await {
                                    Ok(_) => {
                                        is_saving.set(false);
                                        app_state.write().create_file_modal = false;
                                        app_state.write().needs_refetch = true;
                                    }
                                    Err(_) => {
                                        is_saving.set(false);
                                        error_msg.set(Some("Failed to create file".to_string()));
                                    }
                                }
                            });
                        },
                        disabled: is_saving(),
                        if is_saving() { "Creating..." } else { "Create" }
                    }
                }
            }
        }
    }
}