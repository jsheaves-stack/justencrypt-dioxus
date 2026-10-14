use super::{
    build_url, folder::put_folder, send, send_and_discard, RequestBuilderExt, RequestError,
};

#[cfg(target_arch = "wasm32")]
use super::trigger_browser_download;

#[cfg(not(target_arch = "wasm32"))]
use super::save_download;
use crate::{api_url, ReqwestClient};
use dioxus::{html::FileData, prelude::*, signals::Signal};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    cmp,
    collections::{BTreeSet, HashSet},
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

#[cfg(target_arch = "wasm32")]
use {js_sys::Uint8Array, wasm_bindgen_futures::JsFuture, web_sys::File as WebFile};

#[cfg(not(target_arch = "wasm32"))]
use {std::path::PathBuf, tokio::fs::File as TokioFile, tokio::io::AsyncReadExt};

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
    trigger_browser_download(&file_url, &file_name)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn download_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_name: String,
) -> Result<(), RequestError> {
    let file_url = build_url("file", &current_path, Some(&file_name));
    save_download(reqwest_client_context, &file_url, &file_name).await
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
    let chunk_url = format!("{}/upload/chunk/{}?offset={}", api_url(), upload_id, offset);

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

/// One file to upload: where it goes, relative to the folder the upload was
/// started in, and where to read its bytes from.
struct UploadItem {
    /// Folders between the upload's target folder and this file, e.g.
    /// `["pixel", "2024"]` for `pixel/2024/IMG_002.jpg` from a folder
    /// upload. Empty for a plain file upload.
    dirs: Vec<String>,
    name: String,
    size: u64,
    #[cfg(not(target_arch = "wasm32"))]
    source: PathBuf,
    #[cfg(target_arch = "wasm32")]
    source: FileData,
}

impl UploadItem {
    /// The item's path below the upload's target folder, as shown in the
    /// upload panel - so same-named files from different subfolders can be
    /// told apart.
    fn display_name(&self) -> String {
        let mut parts = self.dirs.clone();
        parts.push(self.name.clone());
        parts.join("/")
    }
}

/// On web, a folder pick gives one `FileData` per file inside it, whose
/// `path()` is the browser's `webkitRelativePath` (e.g.
/// `pixel/2024/IMG_002.jpg`); a plain file pick's `path()` is just the file
/// name, so it lands directly in the target folder.
#[cfg(target_arch = "wasm32")]
async fn upload_items(files: Vec<FileData>) -> Vec<UploadItem> {
    files
        .into_iter()
        .map(|file| {
            let (dirs, name) =
                split_relative_path(&file.path()).unwrap_or_else(|| (Vec::new(), file.name()));
            UploadItem {
                dirs,
                name,
                size: file.size(),
                source: file,
            }
        })
        .collect()
}

/// On desktop, a folder pick gives a single `FileData` for the folder
/// itself rather than one per file inside it, so folders are walked here -
/// off the UI thread, since a large folder is a lot of blocking filesystem
/// calls.
#[cfg(not(target_arch = "wasm32"))]
async fn upload_items(files: Vec<FileData>) -> Vec<UploadItem> {
    let paths: Vec<PathBuf> = files.iter().map(FileData::path).collect();
    tokio::task::spawn_blocking(move || {
        let mut items = Vec::new();
        for path in &paths {
            add_path(&mut items, path, Vec::new());
        }
        items
    })
    .await
    .unwrap_or_else(|e| {
        tracing::error!("Collecting files to upload failed: {}", e);
        Vec::new()
    })
}

/// Adds `path` to `items` in folder `dirs`: as a single file, or, if it's a
/// folder, every file under it - inside a folder of the same name, so a
/// picked `pixel` is recreated as `pixel/...` rather than just its contents.
/// Symlinks and special files inside a folder are skipped (`file_type`
/// doesn't follow symlinks), so a link back up the tree can't loop forever.
#[cfg(not(target_arch = "wasm32"))]
fn add_path(items: &mut Vec<UploadItem>, path: &Path, mut dirs: Vec<String>) {
    let name = path
        .file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned();

    if !path.is_dir() {
        items.push(UploadItem {
            dirs,
            name,
            // A file whose size can't be read fails (and shows as an error)
            // when its upload tries to open it.
            size: path.metadata().map(|m| m.len()).unwrap_or(0),
            source: path.to_path_buf(),
        });
        return;
    }

    dirs.push(name);
    match std::fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|t| t.is_file() || t.is_dir()) {
                    add_path(items, &entry.path(), dirs.clone());
                }
            }
        }
        Err(e) => tracing::error!("Failed to read folder {:?} for upload: {}", path, e),
    }
}

/// Splits a relative path like `pixel/2024/IMG_002.jpg` into its folder
/// segments and file name. Empty, `.`, `..` and root components are
/// dropped, so a malformed path can't point outside the upload's target
/// folder. `None` if no file name is left.
///
/// Only web gets relative paths from the browser; desktop walks folders
/// itself (see `add_path`), so this is web-only outside of tests.
#[cfg(any(target_arch = "wasm32", test))]
fn split_relative_path(path: &Path) -> Option<(Vec<String>, String)> {
    use std::path::Component;

    let mut segments: Vec<String> = path
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let name = segments.pop()?;
    Some((segments, name))
}

/// Every folder the `items` sit in, plus all their parents, in the order
/// they have to be created: a `BTreeSet` of segment lists iterates parents
/// before their children (`["a"]` sorts before `["a", "b"]`).
fn folders_to_create(items: &[UploadItem]) -> BTreeSet<Vec<String>> {
    items
        .iter()
        .flat_map(|item| (1..=item.dirs.len()).map(|depth| item.dirs[..depth].to_vec()))
        .collect()
}

/// Creates every folder in `folders` under `current_path`, parents first.
/// The server reports a folder that already exists as created, so uploading
/// `pixel` into a folder that already has one merges into it. Returns the
/// folders that couldn't be created. Since parents are created first, a
/// folder inside a failed one fails too, so callers only need to check a
/// file's own folder.
async fn create_upload_folders(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: &[String],
    folders: &BTreeSet<Vec<String>>,
) -> HashSet<Vec<String>> {
    let mut failed = HashSet::new();
    for folder in folders {
        let Some((name, parent)) = folder.split_last() else {
            continue;
        };
        let parent_path = current_path.iter().chain(parent).cloned().collect();
        if put_folder(reqwest_client_context, parent_path, name.clone())
            .await
            .is_err()
        {
            failed.insert(folder.clone());
        }
    }
    failed
}

fn set_task_status(
    mut upload_state: Signal<crate::UploadState>,
    task_index: usize,
    status: crate::UploadStatus,
) {
    let mut state = upload_state.write();
    if let Some(task) = state.tasks.get_mut(task_index) {
        task.status = status;
        if status == crate::UploadStatus::Done {
            task.progress = 100.0;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn stream_and_upload_chunks(
    client: &reqwest::Client,
    upload_id: &str,
    item: &UploadItem,
    mut upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    let total_size = item.size;
    let mut offset = 0u64;
    const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8MB

    let mut f = TokioFile::open(&item.source).await.map_err(|e| {
        tracing::error!("Failed to open file {:?}: {}", item.source, e);
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
    item: &UploadItem,
    mut upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    let total_size = item.size;
    let mut offset = 0u64;
    const CHUNK_SIZE: usize = 8 * 1024 * 1024; // 8MB

    let web_file = item
        .source
        .inner()
        .downcast_ref::<WebFile>()
        .ok_or_else(|| {
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

/// Uploads `item` into `target_path` (the upload's target folder plus the
/// item's own `dirs`).
async fn upload_chunked(
    client: reqwest::Client,
    target_path: Vec<String>,
    item: &UploadItem,
    upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    let init_url = build_url("upload/initiate", &target_path, Some(&item.name));

    tracing::debug!("Initializing chunked upload: {}", init_url);

    let resp = send(client.post(&init_url).with_credentials_if_wasm())
        .await
        .map_err(|e| tracing::error!("Init upload failed: {:?}", e))?;

    let init_resp: InitUploadResponse = resp.json().await.map_err(|e| {
        tracing::error!("Failed to parse init response: {}", e);
    })?;
    let upload_id = init_resp.upload_id;

    stream_and_upload_chunks(&client, &upload_id, item, upload_state, task_index).await?;

    let finalize_url = format!(
        "{}/upload/finalize/{}?total_size={}",
        api_url(),
        upload_id,
        item.size
    );
    send_and_discard(client.post(&finalize_url).with_credentials_if_wasm())
        .await
        .map_err(|e| tracing::error!("Finalize upload failed: {:?}", e))?;

    Ok(())
}

/// Uploads the picked `files` into `current_path`. A picked folder is
/// recreated there, subfolders and all, before any of its files upload.
pub async fn put_file(
    reqwest_client_context: Signal<ReqwestClient>,
    mut upload_state: Signal<crate::UploadState>,
    current_path: Vec<String>,
    files: Vec<FileData>,
) -> Result<(), ()> {
    let mut items = upload_items(files).await;
    items.sort_by_key(|item| cmp::Reverse(item.size));
    let folders = folders_to_create(&items);

    tracing::debug!(
        "Preparing to upload {} file(s) and {} folder(s) to {:?}",
        items.len(),
        folders.len(),
        current_path
    );

    // Pre-populate every task as Pending up front so the panel shows the
    // whole batch straight away; item `i` is task `start_index + i`.
    let start_index = {
        let mut state = upload_state.write();
        let start = state.tasks.len();
        for item in &items {
            state.tasks.push(crate::UploadTask {
                file_name: item.display_name(),
                status: crate::UploadStatus::Pending,
                progress: 0.0,
            });
        }
        start
    };

    let failed_folders =
        create_upload_folders(reqwest_client_context, &current_path, &folders).await;
    let overall_success = Arc::new(AtomicBool::new(failed_folders.is_empty()));

    let active_count = Arc::new(AtomicUsize::new(0));
    // Each spawned upload task sends on completion so the loops below can
    // wait for a free slot (or for everything to finish) by awaiting the
    // channel instead of polling `active_count` on a timer. Uploads are
    // still dispatched as their own detached `dioxus::prelude::spawn` tasks
    // (rather than driven from a single combinator here) so they keep
    // running to completion even if the caller's own task is cancelled,
    // e.g. by navigating away mid-upload.
    let (done_tx, mut done_rx) = futures_channel::mpsc::unbounded::<()>();

    for (i, item) in items.into_iter().enumerate() {
        let task_index = start_index + i;

        if failed_folders.contains(&item.dirs) {
            tracing::error!(
                "Skipping upload of {}: its folder couldn't be created",
                item.display_name()
            );
            set_task_status(upload_state, task_index, crate::UploadStatus::Error);
            continue;
        }

        while active_count.load(Ordering::Relaxed) >= upload_state.read().max_concurrent {
            done_rx.next().await;
        }

        let target_path: Vec<String> = current_path.iter().chain(&item.dirs).cloned().collect();
        let overall_success = overall_success.clone();
        let active_count = active_count.clone();
        let done_tx = done_tx.clone();

        active_count.fetch_add(1, Ordering::Relaxed);

        dioxus::prelude::spawn(async move {
            if task_index < upload_state.read().tasks.len() {
                set_task_status(upload_state, task_index, crate::UploadStatus::Uploading);
            } else {
                tracing::warn!("Upload tasks were unexpectedly cleared, aborting upload task");
                overall_success.store(false, Ordering::Relaxed);
                active_count.fetch_sub(1, Ordering::Relaxed);
                let _ = done_tx.unbounded_send(());
                return;
            }

            let res = upload_chunked(
                reqwest_client_context().0.clone(),
                target_path,
                &item,
                upload_state,
                task_index,
            )
            .await;

            match res {
                Ok(_) => set_task_status(upload_state, task_index, crate::UploadStatus::Done),
                Err(_) => {
                    set_task_status(upload_state, task_index, crate::UploadStatus::Error);
                    overall_success.store(false, Ordering::Relaxed);
                }
            }

            active_count.fetch_sub(1, Ordering::Relaxed);
            let _ = done_tx.unbounded_send(());
        });
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn segments(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn split_relative_path_separates_folders_from_file_name() {
        assert_eq!(
            split_relative_path(Path::new("pixel/2024/IMG_002.jpg")),
            Some((segments(&["pixel", "2024"]), "IMG_002.jpg".to_string()))
        );
    }

    #[test]
    fn split_relative_path_of_plain_file_name_has_no_folders() {
        assert_eq!(
            split_relative_path(Path::new("IMG_001.jpg")),
            Some((Vec::new(), "IMG_001.jpg".to_string()))
        );
    }

    #[test]
    fn split_relative_path_drops_empty_dot_dotdot_and_root_parts() {
        assert_eq!(
            split_relative_path(Path::new("/../pixel//./a.jpg")),
            Some((segments(&["pixel"]), "a.jpg".to_string()))
        );
        assert_eq!(split_relative_path(Path::new("")), None);
        assert_eq!(split_relative_path(Path::new("../..")), None);
    }

    fn item(dirs: &[&str], name: &str) -> UploadItem {
        UploadItem {
            dirs: segments(dirs),
            name: name.to_string(),
            size: 0,
            source: PathBuf::from(name),
        }
    }

    #[test]
    fn folders_to_create_lists_every_parent_folder_parents_first() {
        let items = [
            item(&["pixel", "2024", "jan"], "a.jpg"),
            item(&["pixel"], "b.jpg"),
            item(&[], "c.jpg"),
        ];
        assert_eq!(
            folders_to_create(&items).into_iter().collect::<Vec<_>>(),
            vec![
                segments(&["pixel"]),
                segments(&["pixel", "2024"]),
                segments(&["pixel", "2024", "jan"]),
            ]
        );
    }

    #[test]
    fn display_name_joins_folders_and_name() {
        assert_eq!(item(&["a", "b"], "c.txt").display_name(), "a/b/c.txt");
        assert_eq!(item(&[], "c.txt").display_name(), "c.txt");
    }

    /// A scratch directory under the system temp dir, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let dir =
                std::env::temp_dir().join(format!("justencrypt-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn collect(path: &Path) -> Vec<UploadItem> {
        let mut items = Vec::new();
        add_path(&mut items, path, Vec::new());
        items
    }

    #[test]
    fn picked_folder_is_walked_and_recreated_under_its_own_name() {
        let tmp = TempDir::new();
        let pixel = tmp.0.join("pixel");
        std::fs::create_dir_all(pixel.join("2024")).unwrap();
        std::fs::write(pixel.join("a.jpg"), b"aaa").unwrap();
        std::fs::write(pixel.join("2024").join("a.jpg"), b"bb").unwrap();

        let items = collect(&pixel);

        let mut names: Vec<_> = items.iter().map(UploadItem::display_name).collect();
        names.sort();
        assert_eq!(names, vec!["pixel/2024/a.jpg", "pixel/a.jpg"]);

        let nested = items
            .iter()
            .find(|i| i.dirs == segments(&["pixel", "2024"]))
            .unwrap();
        assert_eq!(nested.size, 2);
        assert_eq!(nested.source, pixel.join("2024").join("a.jpg"));
    }

    #[test]
    fn picked_plain_file_has_no_folders() {
        let tmp = TempDir::new();
        let file = tmp.0.join("notes.txt");
        std::fs::write(&file, b"hello").unwrap();

        let items = collect(&file);

        assert_eq!(items.len(), 1);
        assert!(items[0].dirs.is_empty());
        assert_eq!(items[0].name, "notes.txt");
        assert_eq!(items[0].size, 5);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_inside_a_folder_are_skipped() {
        let tmp = TempDir::new();
        let pixel = tmp.0.join("pixel");
        std::fs::create_dir_all(&pixel).unwrap();
        std::fs::write(pixel.join("a.jpg"), b"a").unwrap();
        // A link back to its own parent would recurse forever if followed.
        std::os::unix::fs::symlink(&pixel, pixel.join("loop")).unwrap();
        std::os::unix::fs::symlink(pixel.join("a.jpg"), pixel.join("link.jpg")).unwrap();

        let items = collect(&pixel);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "a.jpg");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_file_names_are_still_uploaded() {
        // Regression test for the old "Failed to convert file path to
        // string" check, which rejected these outright.
        use std::os::unix::ffi::OsStrExt;
        let tmp = TempDir::new();
        let file = tmp.0.join(std::ffi::OsStr::from_bytes(b"caf\xe9.txt"));
        std::fs::write(&file, b"x").unwrap();

        let items = collect(&file);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].source, file);
    }
}
