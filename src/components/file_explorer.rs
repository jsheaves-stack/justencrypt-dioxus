use std::path::PathBuf;

use dioxus::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ReqwestClient;

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        .map(|f| {
            let mut current_path = current_path.clone();

            rsx! {
                if f.is_file {
                    div { class: "w-32 h-32 px-2 py-2 bg-green-200", "{f.file_name}" }
                } else {

                    div { class: "w-32 h-32 px-2 py-2 bg-green-200",
                        button {
                            onclick: move |_| {
                                let path = sanitize_url(format!("{}/{}", current_path().display(), f.file_name));
                                current_path.set(PathBuf::from(path));
                            },
                            "{f.file_name}"
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    rsx! {
        button {
            class: "bg-gray-200 px-4 py-2",
            onclick: move |_| {
                let mut path = current_path();
                path.pop();
                current_path.set(path);
            },
            "Back"
        }
        div { class: "w-full px-4 py-4 gap-4 flex flex-row flex-wrap h-min items-center justify-items-center",
            {
                folder_contents_for_display.into_iter()
            }
        }
    }
}
