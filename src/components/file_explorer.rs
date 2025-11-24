use crate::{
    components::{
        create_folder_modal::CreateFolderModal,
        file::File,
        folder::Folder,
        ui::{button::Button, header::Header},
    },
    requests::{file::put_file, folder::get_folder},
    ReqwestClient, Route,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub const IMAGE_FORMATS: [&str; 8] = ["APNG", "AVIF", "GIF", "JPG", "JPEG", "PNG", "SVG", "WebP"];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FolderItem {
    pub is_file: bool,
    pub file_extension: Option<String>,
    pub file_name: String,
}

#[component]
pub fn FileExplorer(path: Vec<String>) -> Element {
    let router = router();
    let mut app_state = use_context::<Signal<crate::AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    let create_folder_modal_state = app_state.read().create_folder_modal;
    let needs_refetch = use_memo(move || app_state.read().needs_refetch);
    let current_path = path.clone();
    let current_path2 = path.clone();

    let path_clone = path.clone();
    let mut folder_contents_resource =
        use_resource(use_reactive!(|(path_clone, reqwest_client_context)| {
            let path_clone = path_clone.clone();
            let client_clone = reqwest_client_context;

            async move {
                let result = get_folder(client_clone, path_clone.clone()).await;

                (result, path_clone)
            }
        }));

    use_effect(move || {
        if needs_refetch() {
            folder_contents_resource.restart();
            app_state.write().needs_refetch = false;
        }
    });

    let path_clone = path.clone();
    let folder_contents_for_display = match (&*folder_contents_resource.value())() {
        Some((Ok(files), resource_path)) => {
            if resource_path == path_clone {
                files
                    .into_iter()
                    .map(|file| {
                        rsx! {
                            if file.is_file {
                                File {
                                    file: file.clone(),
                                    current_path: path.clone(),
                                }
                            } else {
                                Folder {
                                    file: file.clone(),
                                    current_path: path.clone(),
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![rsx! { div { "Loading contents..." } }]
            }
        }
        Some((Err(_), _)) => {
            vec![rsx! { div { "Error loading folder." } }]
        }
        None => {
            vec![rsx! { div { "Loading contents..." } }]
        }
    };

    rsx! {
        div { class: "w-full h-full bg-background text-foreground overflow-scroll",
            Header {}
            div { class: "mt-4 grid h-min w-full auto-rows-min justify-items-end gap-x-4 gap-y-2 px-4",
                div { class: "col-start-1 row-start-1 flex justify-self-start",
                    Button {
                        onclick: move |_| {
                            let mut new_path_vec = path_clone.clone();
                            if new_path_vec.pop().is_some() {
                                router
                                    .push(Route::FileExplorer {
                                        path: new_path_vec,
                                    });
                            }
                        },
                        span { "Back" }
                    }
                }
                div { class: "col-start-2 row-start-1 flex flex-wrap items-center gap-x-4 gap-y-2 justify-self-end",
                    div {}
                    div {
                        Button {
                            onclick: move |_| {
                                app_state.write().create_folder_modal = true;
                            },
                            "Create Folder"
                        }
                    }
                    div {
                        input {
                            class: "hidden",
                            id: "file-upload",
                            r#type: "file",
                            multiple: true,
                            onchange: move |e| {
                                let current_path = current_path.clone();
                                async move {
                                    put_file(reqwest_client_context, current_path.clone(), e.files())
                                        .await
                                        .unwrap();
                                    app_state.write().needs_refetch = true;
                                }
                            },
                        }
                        Button { class: "fit-content",
                            label { r#for: "file-upload", "Upload Files" }
                        }
                    }
                    div {
                        input {
                            class: "hidden",
                            id: "folder-upload",
                            r#type: "file",
                            directory: true,
                            onchange: move |e| {
                                let current_path = current_path2.clone();
                                async move {
                                    put_file(reqwest_client_context, current_path.clone(), e.files())
                                        .await
                                        .unwrap();
                                    app_state.write().needs_refetch = true;
                                }
                            },
                        }
                        Button { class: "fit-content",
                            label { r#for: "folder-upload", "Upload Folder" }
                        }
                    }
                }
            }

            if create_folder_modal_state {
                CreateFolderModal { current_path: path.clone() }
            }

            div { class: "h-full w-full px-4 py-4 flex flex-row flex-wrap h-min items-center justify-items-center",
                {folder_contents_for_display.into_iter()}
            }
        }
    }
}
