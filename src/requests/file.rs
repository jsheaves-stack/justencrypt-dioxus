use super::{build_url, send, send_and_discard, RequestBuilderExt, RequestError};
use crate::{api_url, ReqwestClient};
use dioxus::{html::FileData, prelude::*, signals::Signal};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

#[cfg(target_arch = "wasm32")]
use {
    js_sys::Uint8Array,
    wasm_bindgen::JsCast,
    wasm_bindgen_futures::JsFuture,
    web_sys::{window, File as WebFile, HtmlAnchorElement},
};

#[cfg(not(target_arch = "wasm32"))]
use {
    tokio::fs::File as TokioFile,
    tokio::io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Serialize)]
struct PatchFileRequest {
    parent_folder_path: String,
    file_name: String,
}

#[derive(Deserialize)]
struct InitUploadResponse {
    upload_id: String,
}

pub async fn patch_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    old_file_name: String,
    new_parent_folder_path: String,
    new_file_name: String,
) -> Result<(), RequestError> {
    let file_url = build_url("file", &current_path, Some(&old_file_name));
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

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .patch(file_url)
            .with_credentials_if_wasm()
            .json(&payload),
    )
    .await;

    match &result {
        Ok(_) => tracing::debug!("File patched successfully"),
        Err(e) => tracing::error!("Patch file failed: {:?}", e),
    }

    result
}

pub async fn delete_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<(), RequestError> {
    let file_url = build_url("file", &current_path, Some(&file_name));
    tracing::debug!("Deleting file at {}", file_url);

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .delete(file_url)
            .with_credentials_if_wasm(),
    )
    .await;

    match &result {
        Ok(_) => tracing::debug!("File deleted successfully"),
        Err(e) => tracing::error!("Delete file failed: {:?}", e),
    }

    result
}

#[cfg(target_arch = "wasm32")]
pub async fn download_file(
    _reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<(), RequestError> {
    let file_url = build_url("file", &current_path, Some(&file_name));
    tracing::debug!("Triggering WASM download for {}", file_url);

    fn to_err<E: std::fmt::Debug>(context: &'static str) -> impl Fn(E) -> RequestError {
        move |e| {
            tracing::error!("WASM download failed to {}: {:?}", context, e);
            RequestError::Decode(format!("browser download failed to {context}"))
        }
    }

    let window = window().ok_or_else(|| {
        tracing::error!("WASM download failed: no window object");
        RequestError::Decode("no window object".to_string())
    })?;
    let document = window.document().ok_or_else(|| {
        tracing::error!("WASM download failed: no document object");
        RequestError::Decode("no document object".to_string())
    })?;
    let a = document
        .create_element("a")
        .map_err(to_err("create anchor"))?;
    let a: HtmlAnchorElement = a.dyn_into().map_err(to_err("cast anchor"))?;

    a.set_href(&file_url);
    a.set_download(&file_name);

    let body = document.body().ok_or_else(|| {
        tracing::error!("WASM download failed: no body object");
        RequestError::Decode("no body object".to_string())
    })?;
    body.append_child(&a).map_err(to_err("append anchor"))?;

    a.click();

    body.remove_child(&a).map_err(to_err("remove anchor"))?;

    tracing::debug!("WASM download triggered successfully");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn download_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<(), RequestError> {
    if let Some(path) = rfd::AsyncFileDialog::new()
        .set_file_name(&file_name)
        .save_file()
        .await
    {
        let file_url = build_url("file", &current_path, Some(&file_name));
        tracing::debug!("Downloading file from {} to {:?}", file_url, path.path());

        let mut resp = send(
            reqwest_client_context()
                .0
                .get(&file_url)
                .with_credentials_if_wasm(),
        )
        .await
        .inspect_err(|e| tracing::error!("Download failed: {:?}", e))?;

        let mut file = TokioFile::create(path.path()).await.map_err(|e| {
            tracing::error!("Failed to create local file: {}", e);
            RequestError::Decode(e.to_string())
        })?;
        while let Some(chunk) = resp.chunk().await.map_err(|e| {
            tracing::error!("Failed to read chunk during download: {}", e);
            RequestError::Network(e.to_string())
        })? {
            file.write_all(&chunk).await.map_err(|e| {
                tracing::error!("Failed to write chunk to local file: {}", e);
                RequestError::Decode(e.to_string())
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
) -> Result<String, RequestError> {
    let file_url = build_url("file", &current_path, Some(&file_name));
    tracing::debug!("Getting file text for {}", file_url);

    let resp = send(
        reqwest_client_context()
            .0
            .get(file_url)
            .with_credentials_if_wasm(),
    )
    .await
    .inspect_err(|e| tracing::error!("Get file text failed: {:?}", e))?;

    let text = resp.text().await.map_err(|e| {
        tracing::error!("Failed to read file text response: {}", e);
        RequestError::Decode(e.to_string())
    })?;

    tracing::debug!("Successfully retrieved file text");
    Ok(text)
}

pub async fn put_file_text(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
    content: String,
) -> Result<(), RequestError> {
    let file_url = build_url("file", &current_path, Some(&file_name));
    tracing::debug!("Putting file text to {}", file_url);

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .put(file_url)
            .with_credentials_if_wasm()
            .body(content),
    )
    .await;

    match &result {
        Ok(_) => tracing::debug!("Successfully updated file text"),
        Err(e) => tracing::error!("Put file text failed: {:?}", e),
    }

    result
}

async fn upload_chunk_with_retry(
    client: &reqwest::Client,
    upload_id: &str,
    offset: u64,
    chunk_data: bytes::Bytes,
) -> Result<(), ()> {
    let chunk_url = format!(
        "{}/file/upload/chunk/{}?offset={}",
        api_url(),
        upload_id,
        offset
    );

    let mut attempt = 0;
    const MAX_RETRIES: u32 = 5;

    loop {
        let result = send_and_discard(
            client
                .put(&chunk_url)
                .with_credentials_if_wasm()
                .body(chunk_data.clone()),
        )
        .await;

        if result.is_ok() {
            return Ok(());
        }
        tracing::error!(
            "Chunk upload failed at offset {}: {:?} (attempt {}/{})",
            offset,
            result.unwrap_err(),
            attempt + 1,
            MAX_RETRIES
        );

        attempt += 1;
        if attempt >= MAX_RETRIES {
            tracing::error!("Max retries reached for chunk at offset {}", offset);
            return Err(());
        }

        let delay = Duration::from_secs(1 << attempt.min(3));
        crate::util::sleep(delay).await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn stream_and_upload_chunks(
    client: &reqwest::Client,
    upload_id: &str,
    file: &FileData,
    mut upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    let total_size = file.size();
    let mut offset = 0u64;
    const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8MB

    let path = file.path();
    let mut f = TokioFile::open(&path).await.map_err(|e| {
        tracing::error!("Failed to open file {:?}: {}", path, e);
    })?;

    loop {
        let mut chunk_data = vec![0; CHUNK_SIZE];
        let mut bytes_read = 0;
        while bytes_read < CHUNK_SIZE {
            let n = f.read(&mut chunk_data[bytes_read..]).await.map_err(|e| {
                tracing::error!("Failed to read chunk: {}", e);
            })?;
            if n == 0 {
                break;
            }
            bytes_read += n;
        }
        if bytes_read == 0 {
            break;
        }
        chunk_data.truncate(bytes_read);

        upload_chunk_with_retry(client, upload_id, offset, bytes::Bytes::from(chunk_data)).await?;

        offset += bytes_read as u64;
        let mut state = upload_state.write();
        if task_index < state.tasks.len() {
            state.tasks[task_index].progress = (offset as f64 / total_size as f64) * 100.0;
        }
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn stream_and_upload_chunks(
    client: &reqwest::Client,
    upload_id: &str,
    file: &FileData,
    mut upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    let total_size = file.size();
    let mut offset = 0u64;
    const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8MB

    let web_file = file.inner().downcast_ref::<WebFile>().ok_or_else(|| {
        tracing::error!("Failed to downcast to WebFile");
    })?;

    while offset < total_size {
        let end = (offset + CHUNK_SIZE as u64).min(total_size);
        let blob = web_file
            .slice_with_f64_and_f64(offset as f64, end as f64)
            .map_err(|e| {
                tracing::error!("Failed to slice blob: {:?}", e);
            })?;

        let array_buffer = JsFuture::from(blob.array_buffer()).await.map_err(|e| {
            tracing::error!("Failed to get array buffer: {:?}", e);
        })?;

        let uint8_array = Uint8Array::new(&array_buffer);
        let chunk_data = uint8_array.to_vec();

        upload_chunk_with_retry(client, upload_id, offset, bytes::Bytes::from(chunk_data)).await?;

        offset += end - offset;
        let mut state = upload_state.write();
        if task_index < state.tasks.len() {
            state.tasks[task_index].progress = (offset as f64 / total_size as f64) * 100.0;
        }
    }
    Ok(())
}

async fn upload_chunked(
    client: reqwest::Client,
    current_path: Vec<String>,
    file: FileData,
    upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    let file_name = file.name();

    let init_url = build_url("file/upload/initiate", &current_path, Some(&file_name));

    tracing::debug!("Initializing chunked upload: {}", init_url);

    let resp = send(client.post(&init_url).with_credentials_if_wasm())
        .await
        .map_err(|e| tracing::error!("Init upload failed: {:?}", e))?;

    let init_resp: InitUploadResponse = resp.json().await.map_err(|e| {
        tracing::error!("Failed to parse init response: {}", e);
    })?;
    let upload_id = init_resp.upload_id;

    stream_and_upload_chunks(&client, &upload_id, &file, upload_state, task_index).await?;

    let finalize_url = format!(
        "{}/file/upload/finalize/{}?total_size={}",
        api_url(),
        upload_id,
        file.size()
    );
    send_and_discard(client.post(&finalize_url).with_credentials_if_wasm())
        .await
        .map_err(|e| tracing::error!("Finalize upload failed: {:?}", e))?;

    Ok(())
}

pub async fn put_file(
    reqwest_client_context: Signal<ReqwestClient>,
    mut upload_state: Signal<crate::UploadState>,
    current_path: Vec<String>,
    mut files: Vec<FileData>,
) -> Result<(), ()> {
    files.sort_by_key(|a| cmp::Reverse(a.size()));

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

    let overall_success = Arc::new(AtomicBool::new(true));
    let active_count = Arc::new(AtomicUsize::new(0));
    // Each spawned upload task sends on completion so the loops below can
    // wait for a free slot (or for everything to finish) by awaiting the
    // channel instead of polling `active_count` on a timer. Uploads are
    // still dispatched as their own detached `dioxus::prelude::spawn` tasks
    // (rather than driven from a single combinator here) so they keep
    // running to completion even if the caller's own task is cancelled,
    // e.g. by navigating away mid-upload.
    let (done_tx, mut done_rx) = futures_channel::mpsc::unbounded::<()>();

    let mut i = 0;
    while i < files.len() {
        while active_count.load(Ordering::Relaxed) >= upload_state.read().max_concurrent {
            done_rx.next().await;
        }

        let file = files[i].clone();
        let task_index = start_index + i;
        let overall_success = overall_success.clone();
        let current_path = current_path.clone();
        let active_count = active_count.clone();
        let done_tx = done_tx.clone();

        active_count.fetch_add(1, Ordering::Relaxed);

        dioxus::prelude::spawn(async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file_path = file.path();
                if file_path.to_str().is_none() {
                    tracing::error!("Failed to convert file path to string for upload");
                    let mut state = upload_state.write();
                    if task_index < state.tasks.len() {
                        state.tasks[task_index].status = crate::UploadStatus::Error;
                    }
                    overall_success.store(false, Ordering::Relaxed);
                    active_count.fetch_sub(1, Ordering::Relaxed);
                    let _ = done_tx.unbounded_send(());
                    return;
                }
            }

            {
                let mut state = upload_state.write();
                if task_index < state.tasks.len() {
                    state.tasks[task_index].status = crate::UploadStatus::Uploading;
                } else {
                    tracing::warn!("Upload tasks were unexpectedly cleared, aborting upload task");
                    overall_success.store(false, Ordering::Relaxed);
                    active_count.fetch_sub(1, Ordering::Relaxed);
                    let _ = done_tx.unbounded_send(());
                    return;
                }
            }

            let res = upload_chunked(
                reqwest_client_context().0.clone(),
                current_path,
                file,
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
                    overall_success.store(false, Ordering::Relaxed);
                }
            }

            active_count.fetch_sub(1, Ordering::Relaxed);
            let _ = done_tx.unbounded_send(());
        });

        i += 1;
    }

    while active_count.load(Ordering::Relaxed) > 0 {
        done_rx.next().await;
    }

    if !overall_success.load(Ordering::Relaxed) {
        tracing::error!("One or more file uploads failed");
        Err(())
    } else {
        tracing::debug!("All file uploads completed successfully");
        Ok(())
    }
}
