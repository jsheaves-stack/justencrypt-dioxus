use super::{build_url, send, send_and_discard, RequestBuilderExt, RequestError};
use crate::{models::FolderContent, ReqwestClient};
use dioxus::signals::Signal;

pub async fn put_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    folder_name: String,
) -> Result<(), RequestError> {
    let folder_url = build_url("folder", &current_path, Some(&folder_name));
    tracing::debug!("Creating folder at {}", folder_url);

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .put(folder_url)
            .with_credentials_if_wasm(),
    )
    .await;

    match &result {
        Ok(_) => tracing::debug!("Folder created successfully"),
        Err(e) => tracing::error!("Create folder failed: {:?}", e),
    }

    result
}

#[derive(serde::Serialize)]
struct PatchFolderRequest {
    parent_folder_path: String,
    folder_name: String,
}

pub async fn patch_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    old_folder_name: String,
    new_parent_folder_path: String,
    new_folder_name: String,
) -> Result<(), RequestError> {
    let folder_url = build_url("folder", &current_path, Some(&old_folder_name));
    tracing::debug!(
        "Patching folder at {} to new name {} in folder {}",
        folder_url,
        new_folder_name,
        new_parent_folder_path
    );

    let payload = PatchFolderRequest {
        parent_folder_path: new_parent_folder_path,
        folder_name: new_folder_name,
    };

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .patch(folder_url)
            .with_credentials_if_wasm()
            .json(&payload),
    )
    .await;

    match &result {
        Ok(_) => tracing::debug!("Folder patched successfully"),
        Err(e) => tracing::error!("Patch folder failed: {:?}", e),
    }

    result
}

pub async fn delete_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    folder_name: String,
) -> Result<(), RequestError> {
    let folder_url = build_url("folder", &current_path, Some(&folder_name));
    tracing::debug!("Deleting folder at {}", folder_url);

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .delete(folder_url)
            .with_credentials_if_wasm(),
    )
    .await;

    match &result {
        Ok(_) => tracing::debug!("Folder deleted successfully"),
        Err(e) => tracing::error!("Delete folder failed: {:?}", e),
    }

    result
}

pub async fn get_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
) -> Result<Vec<FolderContent>, RequestError> {
    let folder_url = build_url("folder", &current_path, None);
    tracing::debug!("Fetching folder contents from {}", folder_url);

    let resp = send(
        reqwest_client_context()
            .0
            .get(folder_url)
            .with_credentials_if_wasm(),
    )
    .await
    .inspect_err(|e| tracing::error!("Fetch folder request failed: {:?}", e))?;

    let response_string = resp.text().await.map_err(|e| {
        tracing::error!("Failed to read folder response text: {}", e);
        RequestError::Decode(e.to_string())
    })?;
    let folders: Vec<FolderContent> = serde_json::from_str(&response_string).map_err(|e| {
        tracing::error!("Failed to parse folder JSON: {}", e);
        RequestError::Decode(e.to_string())
    })?;

    tracing::debug!("Successfully fetched folder contents");
    Ok(folders)
}
