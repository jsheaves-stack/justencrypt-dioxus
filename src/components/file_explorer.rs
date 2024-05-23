use std::path::PathBuf;

use dioxus::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ReqwestClient;

const IMAGE_FORMATS: [&str; 8] = ["APNG", "AVIF", "GIF", "JPG", "JPEG", "PNG", "SVG", "WebP"];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GetFolder {
    is_file: bool,
    file_extension: Option<String>,
    file_name: String,
}

async fn get_folder(current_path: PathBuf) -> Result<Vec<GetFolder>, ()> {
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let resp = reqwest_client_context()
        .0
        .get(format!(
            "http://localhost:8000/folder/{}",
            current_path.display()
        ))
        .send()
        .await;

    let folders = match resp {
        // Parse data from here, such as storing a response token
        Ok(r) => {
            if r.status().is_success() {
                let response_string = r.text().await.unwrap();

                let folders: Vec<GetFolder> =
                    serde_json::from_str(response_string.as_str()).unwrap();

                Ok(folders)
            } else {
                Err(())
            }
        }
        //Handle any errors from the fetch here
        Err(_err) => {
            tracing::error!("Fetch user manifest failed: {}", _err);

            Err(())
        }
    };

    match folders {
        Ok(f) => Ok(f),
        Err(_) => Err(()),
    }
}

fn sanitize_url(url: String) -> String {
    // Create a regex to match multiple slashes
    let re = Regex::new(r"/+").unwrap();

    // Replace multiple slashes with a single slash
    let sanitized = re.replace_all(url.as_str(), "/");

    // Convert the result to a String and return
    sanitized.to_string()
}

#[component]
pub fn FileExplorer() -> Element {
    let mut current_path = use_signal(|| PathBuf::from(""));
    let folder_contents_resource =
        use_resource(move || get_folder(current_path().try_into().unwrap()));

    let folder_contents = folder_contents_resource.unwrap().unwrap();

    let folder_contents_for_display = folder_contents
        .into_iter()
        .map(|file| {
            rsx! {
                if file.is_file {
                    File { file, current_path }
                } else {
                    Folder { file, current_path }
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        div { class: "bg-mainAccent h-16 border-b-4 border-black content-center",
            button {
                class: "flex text-align-center cursor-pointer items-center rounded-base border-2 border-black bg-main ml-3.5 w-18 px-4 py-2 my-auto text-sm font-base shadow-base transition-all hover:translate-x-boxShadowX hover:translate-y-boxShadowY hover:shadow-none",
                onclick: move |_| {
                    let mut path = current_path();
                    path.pop();
                    current_path.set(path);
                },
                "Back"
            }
        }
        div { class: "w-full px-4 py-4 gap-4 flex flex-row flex-wrap h-min items-center justify-items-center",
            {
                folder_contents_for_display.into_iter()
            }
        }
    }
}

#[component]
pub fn File(file: GetFolder, current_path: Signal<PathBuf>) -> Element {
    let image_url = match file.file_extension.as_deref() {
        Some(ext) if IMAGE_FORMATS.contains(&ext.to_uppercase().as_str()) => {
            format!(
                "http://localhost:8000/file/{}",
                sanitize_url(format!(
                    "{}{}",
                    current_path().display().to_string(),
                    file.file_name
                ))
            )
        }
        _ => String::new(),
    };

    rsx! {
        div { class: "w-40 h-40 overflow-hidden rounded-base border-2 border-black bg-main font-base shadow-base",
            img { class: "w-full h-24", src: "{image_url}" }
            div { class: "text-sm w-full border-t-2 border-black p-2 truncate text-wrap text-ellipsis overflow-hidden", "{file.file_name}" }
        }
    }
}

#[component]
pub fn Folder(file: GetFolder, current_path: Signal<PathBuf>) -> Element {
    rsx! {
        div {
            class: "w-40 h-40 overflow-hidden cursor-pointer rounded-base border-2 border-black bg-main font-base shadow-base",
            onclick: move |_| {
                let path = sanitize_url(
                    format!("{}/{}", current_path().display(), file.file_name),
                );
                current_path.set(PathBuf::from(path));
            },
            img { class: "w-full h-24", src: "" }
            div { class: "text-sm w-full border-t-2 border-black p-2 truncate text-wrap text-ellipsis overflow-hidden", "{file.file_name}" }
        }
    }
}
