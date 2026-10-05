use super::{build_url, send, send_and_discard, RequestBuilderExt, RequestError};
use crate::{models::FolderContent, ReqwestClient};
use dioxus::signals::Signal;

#[cfg(target_arch = "wasm32")]
use super::trigger_browser_download;

#[cfg(not(target_arch = "wasm32"))]
use super::save_download;

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

/// Builds the `GET /folder/{path}?download` URL for the folder `folder_name`
/// inside `current_path`, and the `.tar` filename to suggest for it. The
/// root folder is passed as an empty path with the name `"/"` (how the
/// explorer's background context menu represents it); the server names its
/// archive after the user, which the client doesn't know, so the root's
/// suggested name is empty and the caller picks a fallback.
fn folder_download_target(current_path: &[String], folder_name: &str) -> (String, String) {
    if current_path.is_empty() && folder_name == "/" {
        (
            format!("{}?download", build_url("folder", &[], None)),
            String::new(),
        )
    } else {
        (
            format!(
                "{}?download",
                build_url("folder", current_path, Some(folder_name))
            ),
            format!("{folder_name}.tar"),
        )
    }
}

/// Downloads a folder and everything below it as an uncompressed TAR
/// archive, which the server streams with its exact `Content-Length`.
#[cfg(target_arch = "wasm32")]
pub async fn download_folder(
    _reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    folder_name: String,
) -> Result<(), RequestError> {
    let (url, download_name) = folder_download_target(&current_path, &folder_name);
    // An empty name lets the browser use the server's `Content-Disposition`.
    trigger_browser_download(&url, &download_name)
}

/// Downloads a folder and everything below it as an uncompressed TAR
/// archive, which the server streams with its exact `Content-Length`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn download_folder(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    folder_name: String,
) -> Result<(), RequestError> {
    let (url, download_name) = folder_download_target(&current_path, &folder_name);
    let download_name = if download_name.is_empty() {
        "root.tar".to_string()
    } else {
        download_name
    };
    save_download(reqwest_client_context, &url, &download_name).await
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

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_base(url: &str) -> &str {
        url.strip_prefix(&crate::api_url())
            .expect("built URL should start with the API base")
    }

    #[test]
    fn folder_download_target_for_nested_folder() {
        let path = vec!["a b".to_string()];
        let (url, name) = folder_download_target(&path, "photos#1");
        assert_eq!(strip_base(&url), "/folder/a%20b/photos%231?download");
        assert_eq!(name, "photos#1.tar");
    }

    #[test]
    fn folder_download_target_for_root() {
        let (url, name) = folder_download_target(&[], "/");
        assert_eq!(strip_base(&url), "/folder?download");
        assert_eq!(name, "");
    }
}
