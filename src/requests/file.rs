use super::RequestBuilderExt;
use crate::{api_url, ReqwestClient};
use dioxus::{html::FileData, prelude::*, signals::Signal};
use futures::FutureExt;

#[cfg(not(target_arch = "wasm32"))]
use futures::StreamExt;

use serde::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::XmlHttpRequest;

#[derive(Serialize)]
struct PatchFileRequest {
    parent_folder_path: String,
    file_name: String,
}

pub async fn patch_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    old_file_name: String,
    new_parent_folder_path: String,
    new_file_name: String,
) -> Result<(), ()> {
    let file_url = format!(
        "{}/file/{}/{}",
        api_url(),
        current_path.join("/"),
        old_file_name
    );
    tracing::debug!(
        "Patching file at {} to new name {} in folder {}",
        file_url,
        new_file_name,
        new_parent_folder_path
    );

    let payload = PatchFileRequest {
        parent_folder_path: new_parent_folder_path,
        file_name: new_file_name,
    };

    let resp = reqwest_client_context()
        .0
        .patch(file_url)
        .with_credentials_if_wasm()
        .json(&payload)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::debug!("File patched successfully");
            Ok(())
        }
        Ok(r) => {
            tracing::error!("Patch file failed with status: {}", r.status());
            Err(())
        }
        Err(e) => {
            tracing::error!("Patch file request failed: {}", e);
            Err(())
        }
    }
}

pub async fn delete_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<(), ()> {
    let file_url = format!(
        "{}/file/{}/{}",
        api_url(),
        current_path.join("/"),
        file_name
    );
    tracing::debug!("Deleting file at {}", file_url);

    let resp = reqwest_client_context()
        .0
        .delete(file_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::debug!("File deleted successfully");
            Ok(())
        }
        Ok(r) => {
            tracing::error!("Delete file failed with status: {}", r.status());
            Err(())
        }
        Err(e) => {
            tracing::error!("Delete file request failed: {}", e);
            Err(())
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn download_file(
    _reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<(), ()> {
    let file_url = format!(
        "{}/file/{}/{}",
        crate::api_url(),
        current_path.join("/"),
        file_name
    );
    tracing::debug!("Triggering WASM download for {}", file_url);

    let window = web_sys::window().ok_or_else(|| {
        tracing::error!("WASM download failed: no window object");
        ()
    })?;
    let document = window.document().ok_or_else(|| {
        tracing::error!("WASM download failed: no document object");
        ()
    })?;
    let a = document.create_element("a").map_err(|e| {
        tracing::error!("WASM download failed to create anchor: {:?}", e);
        ()
    })?;
    let a: web_sys::HtmlAnchorElement = a.dyn_into().map_err(|e| {
        tracing::error!("WASM download failed to cast to HtmlAnchorElement: {:?}", e);
        ()
    })?;

    a.set_href(&file_url);
    a.set_download(&file_name);

    let body = document.body().ok_or_else(|| {
        tracing::error!("WASM download failed: no body object");
        ()
    })?;
    body.append_child(&a).map_err(|e| {
        tracing::error!("WASM download failed to append anchor: {:?}", e);
        ()
    })?;

    a.click();

    body.remove_child(&a).map_err(|e| {
        tracing::error!("WASM download failed to remove anchor: {:?}", e);
        ()
    })?;

    tracing::debug!("WASM download triggered successfully");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn download_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<(), ()> {
    if let Some(path) = rfd::AsyncFileDialog::new()
        .set_file_name(&file_name)
        .save_file()
        .await
    {
        let file_url = format!(
            "{}/file/{}/{}",
            crate::api_url(),
            current_path.join("/"),
            file_name
        );
        tracing::debug!("Downloading file from {} to {:?}", file_url, path.path());
        let mut resp = reqwest_client_context()
            .0
            .get(&file_url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Download request failed: {}", e);
            })?;

        if !resp.status().is_success() {
            tracing::error!("Download failed with status: {}", resp.status());
            return Err(());
        }

        let mut file = tokio::fs::File::create(path.path()).await.map_err(|e| {
            tracing::error!("Failed to create local file: {}", e);
        })?;
        use tokio::io::AsyncWriteExt;
        while let Some(chunk) = resp.chunk().await.map_err(|e| {
            tracing::error!("Failed to read chunk during download: {}", e);
        })? {
            file.write_all(&chunk).await.map_err(|e| {
                tracing::error!("Failed to write chunk to local file: {}", e);
            })?;
        }
        tracing::debug!("File downloaded successfully");
    } else {
        tracing::debug!("Download cancelled by user");
    }
    Ok(())
}

pub async fn get_file_text(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<String, ()> {
    let file_url = format!(
        "{}/file/{}/{}",
        api_url(),
        current_path.join("/"),
        file_name
    );
    tracing::debug!("Getting file text for {}", file_url);

    let resp = reqwest_client_context()
        .0
        .get(file_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let text = r.text().await.map_err(|e| {
                tracing::error!("Failed to read file text response: {}", e);
            })?;
            tracing::debug!("Successfully retrieved file text");
            Ok(text)
        }
        Ok(r) => {
            tracing::error!("Get file text failed with status: {}", r.status());
            Err(())
        }
        Err(e) => {
            tracing::error!("Get file text request failed: {}", e);
            Err(())
        }
    }
}

pub async fn put_file_text(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
    content: String,
) -> Result<(), ()> {
    let file_url = format!(
        "{}/file/{}/{}",
        api_url(),
        current_path.join("/"),
        file_name
    );
    tracing::debug!("Putting file text to {}", file_url);

    let resp = reqwest_client_context()
        .0
        .put(file_url)
        .with_credentials_if_wasm()
        .body(content)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::debug!("Successfully updated file text");
            Ok(())
        }
        Ok(r) => {
            tracing::error!("Put file text failed with status: {}", r.status());
            Err(())
        }
        Err(e) => {
            tracing::error!("Put file text request failed: {}", e);
            Err(())
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn upload_wasm(
    url: String,
    file: FileData,
    upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    tracing::debug!("Starting WASM upload to {}", url);
    let xhr = XmlHttpRequest::new().map_err(|e| {
        tracing::error!("Failed to create XmlHttpRequest: {:?}", e);
        ()
    })?;
    xhr.open("PUT", &url).map_err(|e| {
        tracing::error!("Failed to open XmlHttpRequest: {:?}", e);
        ()
    })?;
    xhr.set_with_credentials(true);

    let mut upload_state_clone = upload_state;
    let on_progress = Closure::wrap(Box::new(move |e: web_sys::ProgressEvent| {
        if e.length_computable() {
            let progress = (e.loaded() / e.total()) * 100.0;
            let mut state = upload_state_clone.write();
            if task_index < state.tasks.len() {
                state.tasks[task_index].progress = progress;
            }
        }
    }) as Box<dyn FnMut(_)>);

    xhr.upload()
        .map_err(|e| {
            tracing::error!("Failed to get XMLHttpRequest upload object: {:?}", e);
            ()
        })?
        .set_onprogress(Some(on_progress.as_ref().unchecked_ref()));
    on_progress.forget();

    let (tx, rx) = futures::channel::oneshot::channel();
    let mut tx = Some(tx);
    let on_load = Closure::wrap(Box::new(move || {
        if let Some(t) = tx.take() {
            let _ = t.send(Ok(()));
        }
    }) as Box<dyn FnMut()>);
    xhr.set_onload(Some(on_load.as_ref().unchecked_ref()));
    on_load.forget();

    let (err_tx, err_rx) = futures::channel::oneshot::channel();
    let mut err_tx = Some(err_tx);
    let on_error = Closure::wrap(Box::new(move || {
        if let Some(t) = err_tx.take() {
            let _ = t.send(Err(()));
        }
    }) as Box<dyn FnMut()>);
    xhr.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

    let web_file = file
        .inner()
        .downcast_ref::<web_sys::File>()
        .ok_or_else(|| {
            tracing::error!("Failed to downcast FileData to web_sys::File");
            ()
        })?
        .clone();
    let blob: &web_sys::Blob = web_file.as_ref();
    xhr.send_with_opt_blob(Some(blob)).map_err(|e| {
        tracing::error!("Failed to send XmlHttpRequest blob: {:?}", e);
        ()
    })?;

    let res = futures::select! {
        res = rx.fuse() => res.map_err(|_| ()),
        res = err_rx.fuse() => res.map_err(|_| ()),
    };

    match res {
        Ok(_) => {
            tracing::debug!("WASM upload to {} completed successfully", url);
            Ok(())
        }
        Err(_) => {
            tracing::error!("WASM upload to {} failed or was rejected", url);
            Err(())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn upload_native(
    client: reqwest::Client,
    url: String,
    file: FileData,
    mut upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    use tokio::io::AsyncReadExt;
    tracing::debug!("Starting native upload to {}", url);
    let file_path = file.path();
    let file_handle = tokio::fs::File::open(&file_path).await.map_err(|e| {
        tracing::error!("Failed to open local file for upload: {:?}", e);
    })?;
    let metadata = file_handle.metadata().await.map_err(|e| {
        tracing::error!("Failed to read local file metadata: {:?}", e);
    })?;
    let file_len = metadata.len();

    let (mut tx, mut rx) = futures::channel::mpsc::unbounded();
    let mut reader = tokio::io::BufReader::new(file_handle);

    let stream = async_stream::stream! {
        let mut total_read = 0;
        let mut buffer = vec![0u8; 256 * 1024]; // 256 KB chunks
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    total_read += n as u64;
                    let progress = if file_len > 0 {
                        (total_read as f64 / file_len as f64) * 100.0
                    } else {
                        100.0
                    };
                    let _ = tx.start_send(progress);
                    yield Ok::<_, Error>(buffer[..n].to_vec());
                }
                Err(e) => {
                    tracing::error!("Error reading local file chunk: {:?}", e);
                    yield Err(e);
                    break;
                }
            }
        }
    };

    let body = reqwest::Body::wrap_stream(stream);
    let mut upload_fut = client
        .put(&url)
        .header(reqwest::header::CONTENT_LENGTH, file_len)
        .body(body)
        .send()
        .fuse();

    loop {
        futures::select! {
            res = upload_fut => {
                match res {
                    Ok(r) if r.status().is_success() => {
                        tracing::debug!("Native upload to {} completed successfully", url);
                        return Ok(());
                    }
                    Ok(r) => {
                        tracing::error!("Native upload failed with status: {}", r.status());
                        return Err(());
                    }
                    Err(e) => {
                        tracing::error!("Native upload request failed: {}", e);
                        return Err(());
                    }
                }
            },
            p = rx.next() => {
                if let Some(progress) = p {
                    let mut state = upload_state.write();
                    if task_index < state.tasks.len() {
                        state.tasks[task_index].progress = progress;
                    }
                }
            }
        }
    }
}

pub async fn put_file(
    reqwest_client_context: Signal<ReqwestClient>,
    mut upload_state: Signal<crate::UploadState>,
    current_path: Vec<String>,
    files: Vec<FileData>,
) -> Result<(), ()> {
    // Pre-populate all tasks as Pending
    tracing::debug!(
        "Preparing to upload {} file(s) to {:?}",
        files.len(),
        current_path
    );
    let start_index = {
        let mut state = upload_state.write();
        let start = state.tasks.len();
        for file in &files {
            let file_name = file.name();
            state.tasks.push(crate::UploadTask {
                file_name,
                status: crate::UploadStatus::Pending,
                progress: 0.0,
            });
        }
        start
    };

    let mut overall_result = Ok(());

    for (i, file) in files.iter().enumerate() {
        let task_index = start_index + i;
        let file_path = file.path();
        let file_path_str = match file_path.to_str() {
            Some(path) => path,
            None => {
                tracing::error!("Failed to convert file path to string for upload");
                let mut state = upload_state.write();
                if task_index < state.tasks.len() {
                    state.tasks[task_index].status = crate::UploadStatus::Error;
                }
                overall_result = Err(());
                continue;
            }
        };

        {
            let mut state = upload_state.write();
            if task_index < state.tasks.len() {
                state.tasks[task_index].status = crate::UploadStatus::Uploading;
            } else {
                // If tasks were cleared, we should probably stop uploading further files
                tracing::warn!("Upload tasks were unexpectedly cleared, aborting upload loop");
                return overall_result;
            }
        }

        let file_url = format!(
            "{}/file/{}/{}",
            api_url(),
            current_path.join("/"),
            file_path_str
        );

        #[cfg(target_arch = "wasm32")]
        let res = {
            let _ = reqwest_client_context; // suppress unused
            upload_wasm(file_url, file.clone(), upload_state, task_index).await
        };

        #[cfg(not(target_arch = "wasm32"))]
        let res = upload_native(
            reqwest_client_context().0.clone(),
            file_url,
            file.clone(),
            upload_state,
            task_index,
        )
        .await;

        match res {
            Ok(_) => {
                let mut state = upload_state.write();
                if task_index < state.tasks.len() {
                    state.tasks[task_index].status = crate::UploadStatus::Done;
                    state.tasks[task_index].progress = 100.0;
                }
            }
            Err(_) => {
                let mut state = upload_state.write();
                if task_index < state.tasks.len() {
                    state.tasks[task_index].status = crate::UploadStatus::Error;
                }
                overall_result = Err(());
            }
        }
    }

    if overall_result.is_err() {
        tracing::error!("One or more file uploads failed");
    } else {
        tracing::debug!("All file uploads completed successfully");
    }
    overall_result
}
