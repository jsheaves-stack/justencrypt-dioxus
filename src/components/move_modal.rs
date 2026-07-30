use dioxus::prelude::*;

use crate::{
    components::ui::{
        button::{Button, ButtonVariant},
        spinner::Spinner,
    },
    models::FolderContent,
    requests::{file::patch_file, folder::get_folder},
    AppState, ReqwestClient,
};

#[component]
pub fn MoveModal(files_to_move: Vec<String>, original_path: Vec<String>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut is_moving = use_signal(|| false);
    let mut error_msg = use_signal::<Option<String>>(|| None);

    // Track the destination path the user is browsing
    let mut current_destination_path = use_signal(|| original_path.clone());

    let path_clone_for_resource = current_destination_path.read().clone();
    let client_clone = reqwest_client_context;

    // Fetch the subfolders for the current destination path
    let mut folders_resource =
        use_resource(use_reactive!(|(path_clone_for_resource, client_clone)| {
            let path = path_clone_for_resource.clone();
            let client = client_clone;
            async move {
                let result = get_folder(client, path.clone()).await;
                (result, path)
            }
        }));

    // React to path changes to restart the fetch
    let _path_dep = current_destination_path.read().clone();
    use_effect(use_reactive!(|_path_dep| {
        folders_resource.restart();
    }));

    let files_to_move_clone = files_to_move.clone();
    let original_path_clone = original_path.clone();

    let folders = match (*folders_resource.value())() {
        Some((Ok(items), path)) if path == *current_destination_path.read() => items
            .into_iter()
            .filter_map(|item| {
                if let FolderContent::Folder(f) = item {
                    Some(f)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };

    let is_loading = (*folders_resource.value())().is_none()
        || matches!((*folders_resource.value())(), Some((_, p)) if p != *current_destination_path.read());

    rsx! {
        div {
            class: "fixed inset-0 z-[1000] flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none p-4 animate-in fade-in duration-200",
            tabindex: "-1",
            onkeydown: move |e| {
                if e.key().to_string() == "Escape" && !*is_moving.read() {
                    app_state.write().move_modal = None;
                }
            },
            onclick: move |_| {
                if !*is_moving.read() {
                    app_state.write().move_modal = None;
                }
            },
            div {
                class: "relative flex w-full max-w-[500px] flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg animate-in zoom-in-95 duration-200 max-h-[80vh]",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4 flex-shrink-0",
                    h2 { class: "text-xl font-semibold", "Move {files_to_move_clone.len()} item(s)" }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2",
                        onclick: move |_| {
                            if !*is_moving.read() {
                                app_state.write().move_modal = None;
                            }
                        },
                        "✕"
                    }
                }

                // Breadcrumbs
                div { class: "flex items-center gap-1 text-sm overflow-x-auto whitespace-nowrap mb-4 pb-2 border-b border-border flex-shrink-0",
                    button {
                        class: "hover:underline hover:text-primary font-medium cursor-pointer transition-colors px-1",
                        onclick: move |_| {
                            current_destination_path.set(vec![]);
                        },
                        "Home"
                    }
                    for (i , p) in current_destination_path.read().iter().enumerate() {
                        span { class: "text-muted-foreground", "/" }
                        button {
                            class: "hover:underline hover:text-primary font-medium cursor-pointer transition-colors px-1",
                            onclick: move |_| {
                                let new_path = current_destination_path.read()[0..=i].to_vec();
                                current_destination_path.set(new_path);
                            },
                            {p.clone()}
                        }
                    }
                }

                // Folder List
                div { class: "flex flex-col gap-1 overflow-y-auto min-h-[200px] mb-4 p-2 bg-muted/20 rounded-md border border-border",
                    if is_loading {
                        div { class: "flex w-full h-full items-center justify-center p-8",
                            Spinner { class: "h-6 w-6 text-muted-foreground" }
                        }
                    } else if folders.is_empty() {
                        div { class: "flex w-full h-full items-center justify-center p-8 text-muted-foreground text-sm",
                            "No subfolders"
                        }
                    } else {
                        for folder in folders {
                            button {
                                class: "flex items-center gap-2 p-2 w-full text-left rounded-md hover:bg-accent hover:text-accent-foreground transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                onclick: move |_| {
                                    let mut new_path = current_destination_path.read().clone();
                                    new_path.push(folder.file_name.clone());
                                    current_destination_path.set(new_path);
                                },
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "20",
                                    height: "20",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    "stroke-width": "2",
                                    "stroke-linecap": "round",
                                    "stroke-linejoin": "round",
                                    class: "text-yellow-500 flex-shrink-0",
                                    path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
                                }
                                span { class: "truncate", "{folder.file_name}" }
                            }
                        }
                    }
                }

                if let Some(err) = error_msg.read().as_ref() {
                    span { class: "text-sm text-destructive mb-4", {err.clone()} }
                }

                div { class: "flex justify-end mt-auto gap-2 flex-shrink-0",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            if !*is_moving.read() {
                                app_state.write().move_modal = None;
                            }
                        },
                        disabled: *is_moving.read(),
                        "Cancel"
                    }
                    Button {
                        onclick: move |_| {
                            let files = files_to_move_clone.clone();
                            let orig_path = original_path_clone.clone();
                            let dest_path = current_destination_path.read().clone();

                            if orig_path == dest_path {
                                error_msg

                                    .set(

                                        Some(
                                            "Destination cannot be the same as the current path".to_string(),
                                        ),
                                    );
                                return;
                            }
                            let dest_path_str = dest_path.join("/");
                            is_moving.set(true);
                            error_msg.set(None);
                            spawn(async move {
                                let mut has_error = false;
                                for file_name in files {
                                    match patch_file(
                                            reqwest_client_context,
                                            orig_path.clone(),
                                            file_name.clone(),
                                            dest_path_str.clone(),
                                            file_name.clone(),
                                        )
                                        .await
                                    {
                                        Ok(_) => {}
                                        Err(_) => {
                                            has_error = true;
                                            break;
                                        }
                                    }
                                }
                                is_moving.set(false);
                                if has_error {
                                    error_msg.set(Some("Failed to move one or more files".to_string()));
                                } else {
                                    let mut state = app_state.write();
                                    state.selected_files.clear();
                                    state.move_modal = None;
                                    state.needs_refetch = true;
                                }
                            });
                        },
                        disabled: *is_moving.read(),
                        if *is_moving.read() {
                            "Moving..."
                        } else {
                            "Move Here"
                        }
                    }
                }
            }
        }
    }
}
