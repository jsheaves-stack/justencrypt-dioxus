use super::RequestBuilderExt;
use crate::{api_url, models::FolderContent, ReqwestClient};
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
    tracing::debug!("Creating folder at {}", folder_url);

    let resp = reqwest_client_context()
        .0
        .put(folder_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::debug!("Folder created successfully");
            Ok(())
        }
        Ok(r) => {
            tracing::error!("Create folder failed with status: {}", r.status());
            Err(())
        }
        Err(e) => {
            tracing::error!("Create folder request failed: {}", e);
            Err(())
        }
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
    tracing::debug!("Deleting folder at {}", folder_url);

    let resp = reqwest_client_context()
        .0
        .delete(folder_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::debug!("Folder deleted successfully");
            Ok(())
        }
        Ok(r) => {
            tracing::error!("Delete folder failed with status: {}", r.status());
            Err(())
        }
        Err(e) => {
            tracing::error!("Delete folder request failed: {}", e);
            Err(())
        }
    }
}

pub async fn get_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
) -> Result<Vec<FolderContent>, ()> {
    let folder_url = format!("{}/folder/{}", api_url(), current_path.join("/"));
    tracing::debug!("Fetching folder contents from {}", folder_url);

    let resp = reqwest_client_context()
        .0
        .get(folder_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let response_string = r.text().await.map_err(|e| {
                tracing::error!("Failed to read folder response text: {}", e);
            })?;
            let folders: Vec<FolderContent> =
                serde_json::from_str(&response_string).map_err(|e| {
                    tracing::error!("Failed to parse folder JSON: {}", e);
                })?;
            tracing::debug!("Successfully fetched folder contents");
            Ok(folders)
        }
        Ok(r) => {
            tracing::error!("Fetch folder failed with status: {}", r.status());
            Err(())
        }
        Err(e) => {
            tracing::error!("Fetch folder request failed: {}", e);
            Err(())
        }
    }
}
