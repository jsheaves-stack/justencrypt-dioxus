use super::RequestBuilderExt;
use crate::{api_url, ReqwestClient};
use dioxus::{html::FileData, prelude::*, signals::Signal};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

#[cfg(target_arch = "wasm32")]
use {
    js_sys::{Promise, Uint8Array},
    wasm_bindgen::JsCast,
    wasm_bindgen_futures::JsFuture,
    web_sys::{window, File as WebFile, HtmlAnchorElement},
};

#[cfg(not(target_arch = "wasm32"))]
use {
    tokio::fs::File as TokioFile,
    tokio::io::{AsyncReadExt, AsyncWriteExt},
    tokio::time::sleep,
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
            let _ = r.bytes().await;
            tracing::debug!("File patched successfully");
            Ok(())
        }
        Ok(r) => {
            let status = r.status();
            let _ = r.bytes().await;
            tracing::error!("Patch file failed with status: {}", status);
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
            let _ = r.bytes().await;
            tracing::debug!("File deleted successfully");
            Ok(())
        }
        Ok(r) => {
            let status = r.status();
            let _ = r.bytes().await;
            tracing::error!("Delete file failed with status: {}", status);
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

    let window = window().ok_or_else(|| {
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
    let a: HtmlAnchorElement = a.dyn_into().map_err(|e| {
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

        let mut file = TokioFile::create(path.path()).await.map_err(|e| {
            tracing::error!("Failed to create local file: {}", e);
        })?;
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
            let _ = r.bytes().await;
            tracing::debug!("Successfully updated file text");
            Ok(())
        }
        Ok(r) => {
            let status = r.status();
            let _ = r.bytes().await;
            tracing::error!("Put file text failed with status: {}", status);
            Err(())
        }
        Err(e) => {
            tracing::error!("Put file text request failed: {}", e);
            Err(())
        }
    }
}

async fn async_sleep(delay: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    sleep(delay).await;

    #[cfg(target_arch = "wasm32")]
    {
        let _ = JsFuture::from(Promise::new(&mut |resolve, _| {
            window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    &resolve,
                    delay.as_millis() as i32,
                )
                .unwrap();
        }))
        .await;
    }
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
        let resp = client
            .put(&chunk_url)
            .with_credentials_if_wasm()
            .body(chunk_data.clone())
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let _ = r.bytes().await;
                return Ok(());
            }
            Ok(r) => {
                let status = r.status();
                let _ = r.bytes().await;
                tracing::error!(
                    "Chunk upload failed at offset {} with status: {} (attempt {}/{})",
                    offset,
                    status,
                    attempt + 1,
                    MAX_RETRIES
                );
            }
            Err(e) => {
                tracing::error!(
                    "Chunk upload failed at offset {}: {} (attempt {}/{})",
                    offset,
                    e,
                    attempt + 1,
                    MAX_RETRIES
                );
            }
        }

        attempt += 1;
        if attempt >= MAX_RETRIES {
            tracing::error!("Max retries reached for chunk at offset {}", offset);
            return Err(());
        }

        let delay = Duration::from_secs(1 << attempt.min(3));
        async_sleep(delay).await;
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
        ()
    })?;

    while offset < total_size {
        let end = (offset + CHUNK_SIZE as u64).min(total_size);
        let blob = web_file
            .slice_with_f64_and_f64(offset as f64, end as f64)
            .map_err(|e| {
                tracing::error!("Failed to slice blob: {:?}", e);
                ()
            })?;

        let array_buffer = JsFuture::from(blob.array_buffer()).await.map_err(|e| {
            tracing::error!("Failed to get array buffer: {:?}", e);
            ()
        })?;

        let uint8_array = Uint8Array::new(&array_buffer);
        let chunk_data = uint8_array.to_vec();

        upload_chunk_with_retry(client, upload_id, offset, bytes::Bytes::from(chunk_data)).await?;

        offset += (end - offset) as u64;
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
    let file_name = file.path().display().to_string();

    let file_path_str = current_path.join("/");
    let init_url = format!(
        "{}/file/upload/initiate/{}/{}",
        api_url(),
        file_path_str,
        file_name
    );

    tracing::debug!("Initializing chunked upload: {}", init_url);

    let resp = client
        .post(&init_url)
        .with_credentials_if_wasm()
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Init upload failed: {}", e);
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let _ = resp.bytes().await;
        tracing::error!("Init upload failed with status: {}", status);
        return Err(());
    }

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
    let resp = client
        .post(&finalize_url)
        .with_credentials_if_wasm()
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Finalize upload failed: {}", e);
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let _ = resp.bytes().await;
        tracing::error!("Finalize upload failed with status: {}", status);
        return Err(());
    }

    let _ = resp.bytes().await;
    Ok(())
}

pub async fn put_file(
    reqwest_client_context: Signal<ReqwestClient>,
    mut upload_state: Signal<crate::UploadState>,
    current_path: Vec<String>,
    mut files: Vec<FileData>,
) -> Result<(), ()> {
    files.sort_by_key(|a| std::cmp::Reverse(a.size()));

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

    let mut i = 0;
    while i < files.len() {
        loop {
            let max_concurrent = upload_state.read().max_concurrent;
            if active_count.load(Ordering::Relaxed) < max_concurrent {
                break;
            }
            async_sleep(Duration::from_millis(50)).await;
        }

        let file = files[i].clone();
        let task_index = start_index + i;
        let overall_success = overall_success.clone();
        let current_path = current_path.clone();
        let active_count = active_count.clone();

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
        });

        i += 1;
    }

    loop {
        if active_count.load(Ordering::Relaxed) == 0 {
            break;
        }
        async_sleep(Duration::from_millis(100)).await;
    }

    if !overall_success.load(Ordering::Relaxed) {
        tracing::error!("One or more file uploads failed");
        Err(())
    } else {
        tracing::debug!("All file uploads completed successfully");
        Ok(())
    }
}
