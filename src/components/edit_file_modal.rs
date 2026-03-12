use dioxus::prelude::*;

use crate::{
    components::ui::button::{Button, ButtonVariant},
    requests::file::{get_file_text, put_file_text},
    AppState, ReqwestClient,
};

#[component]
pub fn EditFileModal(file_name: String, current_path: Vec<String>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut file_content = use_signal::<Option<String>>(|| None);
    let mut is_saving = use_signal(|| false);

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
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none",
            tabindex: "-1",
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" {
                    app_state.write().edit_file_modal = None;
                }
            },
            onclick: move |_| {
                app_state.write().edit_file_modal = None;
            },
            div {
                class: "relative flex w-11/12 max-w-5xl h-5/6 flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4",
                    h2 { class: "text-xl font-semibold", "Editing: {file_name}" }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2",
                        onclick: move |_| {
                            app_state.write().edit_file_modal = None;
                        },
                        "✕"
                    }
                }

                div { class: "flex-grow flex flex-col min-h-0",
                    if let Some(content) = file_content.read().as_ref() {
                        textarea {
                            class: "w-full h-full flex-grow resize-none rounded-md border border-input bg-background text-foreground px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 font-mono",
                            autofocus: true,
                            onmounted: move |e| async move {
                                let _ = e.set_focus(true).await;
                            },
                            value: "{content}",
                            oninput: move |e| {
                                file_content.set(Some(e.value()));
                            }
                        }
                    } else {
                        div { class: "flex items-center justify-center h-full",
                            span { class: "text-muted-foreground animate-pulse", "Loading..." }
                        }
                    }
                }

                div { class: "flex justify-end mt-4 gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            app_state.write().edit_file_modal = None;
                        },
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            let text = file_content.read().clone().unwrap_or_default();
                            let current_path = current_path.clone();
                            let file_name = file_name.clone();
                            is_saving.set(true);
                            spawn(async move {
                                let _ = put_file_text(reqwest_client_context, current_path, file_name, text).await;
                                is_saving.set(false);
                                app_state.write().edit_file_modal = None;
                                app_state.write().needs_refetch = true;
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
