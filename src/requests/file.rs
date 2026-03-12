use super::RequestBuilderExt;
use crate::{api_url, ReqwestClient};
use dioxus::prelude::*;
use dioxus::{html::FileData, signals::Signal};
use futures::FutureExt;

use serde::Serialize;

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
        Ok(r) if r.status().is_success() => Ok(()),
        _ => Err(()),
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

    let resp = reqwest_client_context()
        .0
        .delete(file_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        _ => Err(()),
    }
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

    let resp = reqwest_client_context()
        .0
        .get(file_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => r.text().await.map_err(|_| ()),
        _ => Err(()),
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

    let resp = reqwest_client_context()
        .0
        .put(file_url)
        .with_credentials_if_wasm()
        .body(content)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        _ => Err(()),
    }
}

#[cfg(target_arch = "wasm32")]
async fn upload_wasm(
    url: String,
    file: FileData,
    upload_state: Signal<crate::UploadState>,
    task_index: usize,
) -> Result<(), ()> {
    let xhr = XmlHttpRequest::new().map_err(|_| ())?;
    xhr.open("PUT", &url).map_err(|_| ())?;
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
        .unwrap()
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

    let bytes = file.read_bytes().await.map_err(|_| ())?;
    let array = js_sys::Uint8Array::from(&bytes[..]);
    xhr.send_with_opt_buffer_source(Some(&array))
        .map_err(|_| ())?;

    let res = futures::select! {
        res = rx.fuse() => res.map_err(|_| ()),
        res = err_rx.fuse() => res.map_err(|_| ()),
    };

    res?
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
    let file_path = file.path();
    let file_handle = tokio::fs::File::open(&file_path).await.map_err(|_| ())?;
    let metadata = file_handle.metadata().await.map_err(|_| ())?;
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
                    yield Ok::<_, std::io::Error>(buffer[..n].to_vec());
                }
                Err(e) => {
                    yield Err(e);
                    break;
                }
            }
        }
    };

    let body = reqwest::Body::wrap_stream(stream);
    let mut upload_fut = client.put(url).body(body).send().fuse();

    loop {
        futures::select! {
            res = upload_fut => {
                match res {
                    Ok(r) if r.status().is_success() => return Ok(()),
                    _ => return Err(()),
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
        let file_path_str = file_path.to_str().unwrap();

        {
            let mut state = upload_state.write();
            if task_index < state.tasks.len() {
                state.tasks[task_index].status = crate::UploadStatus::Uploading;
            } else {
                // If tasks were cleared, we should probably stop uploading further files
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

    overall_result
}
