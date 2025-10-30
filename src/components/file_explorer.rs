use crate::{
    components::{create_folder_modal::CreateFolderModal, file::File, folder::Folder},
    requests::folder::get_folder,
    Route,
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
    let create_folder_modal_state = app_state.read().create_folder_modal;

    let folder_contents_resource =
        use_resource(use_reactive!(
            |(path)| async move { get_folder(path).await }
        ));

    let folder_contents_for_display = match &*folder_contents_resource.read() {
        Some(Ok(files)) => files
            .into_iter()
            .map(|file| {
                rsx! {
                    if file.is_file {
                        File { file: file.clone(), current_path: path.clone() }
                    } else {
                        Folder { file: file.clone(), current_path: path.clone() }
                    }
                }
            })
            .collect::<Vec<_>>(),
        Some(Err(_)) => {
            vec![rsx! { div { "Error loading folder." } }]
        }
        None => {
            vec![rsx! { div { "Loading contents..." } }]
        }
    };

    let display_path = if path.is_empty() {
        "/"
    } else {
        &(String::from("/") + &path.join("/"))
    };

    rsx! {
        div { class: "h-full w-full bg-mainAccent h-16 border-b-4 border-black content-center",
            button {
                class: "flex text-align-center cursor-pointer items-center rounded-base border-2 border-black bg-main ml-3.5 w-18 px-4 py-2 my-auto text-sm font-base shadow-base transition-all hover:translate-x-boxShadowX hover:translate-y-boxShadowY hover:shadow-none",
                onclick: move |_| {
                    let mut new_path_vec = path.clone();
                    if new_path_vec.pop().is_some() {
                        router
                            .push(Route::FileExplorer {
                                path: new_path_vec,
                            });
                    }
                },
                "Back"
            }
            button {
                class: "flex text-align-center cursor-pointer items-center rounded-base border-2 border-black bg-main ml-3.5 w-18 px-4 py-2 my-auto text-sm font-base shadow-base transition-all hover:translate-x-boxShadowX hover:translate-y-boxShadowY hover:shadow-none",
                onclick: move |_| {
                    app_state.write().create_folder_modal = true;
                },
                "Create Folder"
            }
        }

        if create_folder_modal_state {
            CreateFolderModal {current_path: path.clone()}
        }

        div { class: "h-full w-full px-4 py-4 gap-4 flex flex-row flex-wrap h-min items-center justify-items-center",
            {folder_contents_for_display.into_iter()}
        }
    }
}
