use super::RequestBuilderExt;
use crate::{api_url, components::file_explorer::FolderItem, ReqwestClient};
use dioxus::signals::Signal;

pub async fn put_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    folder_name: String,
) -> Result<(), ()> {
    let folder_url = format!(
        "{}/folder/{}/{}",
        api_url(),
        current_path.join("/"),
        folder_name
    );

    let resp = reqwest_client_context()
        .0
        .put(folder_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        _ => Err(()),
    }
}

pub async fn delete_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    folder_name: String,
) -> Result<(), ()> {
    let folder_url = format!(
        "{}/folder/{}/{}",
        api_url(),
        current_path.join("/"),
        folder_name
    );

    let resp = reqwest_client_context()
        .0
        .delete(folder_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        _ => Err(()),
    }
}

pub async fn get_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
) -> Result<Vec<FolderItem>, ()> {
    let folder_url = format!("{}/folder/{}", api_url(), current_path.join("/"));

    let resp = reqwest_client_context()
        .0
        .get(folder_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let response_string = r.text().await.map_err(|_| ())?;
            let folders: Vec<FolderItem> =
                serde_json::from_str(&response_string).map_err(|_| ())?;
            Ok(folders)
        }
        Ok(_) => Err(()),
        Err(e) => {
            tracing::error!("Fetch folder failed: {}", e);
            Err(())
        }
    }
}
