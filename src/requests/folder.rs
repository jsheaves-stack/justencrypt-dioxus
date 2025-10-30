use crate::{components::file_explorer::FolderItem, ReqwestClient, API_URL};
use dioxus::{hooks::use_context, signals::Signal};

pub async fn create_folder(current_path: Vec<String>, folder_name: String) -> Result<(), ()> {
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    let folder_url = format!(
        "{}/folder/{}/{}",
        API_URL,
        current_path.join("/"),
        folder_name
    );

    #[cfg(target_arch = "wasm32")]
    let resp = reqwest_client_context()
        .0
        .put(folder_url)
        .fetch_credentials_include()
        .send()
        .await;

    #[cfg(not(target_arch = "wasm32"))]
    let resp = reqwest_client_context().0.put(folder_url).send().await;

    match resp {
        Ok(r) => {
            if r.status().is_success() {
                Ok(())
            } else {
                Err(())
            }
        }
        Err(_err) => Err(()),
    }
}

pub async fn get_folder(current_path: Vec<String>) -> Result<Vec<FolderItem>, ()> {
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    let folder_url = format!("{}/folder/{}", API_URL, current_path.join("/"));

    #[cfg(target_arch = "wasm32")]
    let resp = reqwest_client_context()
        .0
        .get(folder_url)
        .fetch_credentials_include()
        .send()
        .await;

    #[cfg(not(target_arch = "wasm32"))]
    let resp = reqwest_client_context().0.get(folder_url).send().await;

    let folders = match resp {
        Ok(r) => {
            if r.status().is_success() {
                let response_string = r.text().await.unwrap();

                let folders: Vec<FolderItem> =
                    serde_json::from_str(response_string.as_str()).unwrap();

                Ok(folders)
            } else {
                Err(())
            }
        }
        Err(_err) => {
            tracing::error!("Fetch folder failed: {}", _err);
            Err(())
        }
    };

    match folders {
        Ok(f) => Ok(f),
        Err(_) => Err(()),
    }
}
