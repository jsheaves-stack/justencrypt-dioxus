use super::RequestBuilderExt;
use crate::{api_url, ReqwestClient};
use dioxus::{html::FileData, signals::Signal};

use serde::Serialize;

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

pub async fn put_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    files: Vec<FileData>,
) -> Result<(), ()> {
    for file in &files {
        let file_path = file.path();
        let file_path_str = file_path.to_str().unwrap();

        let file_url = format!(
            "{}/file/{}/{}",
            api_url(),
            current_path.join("/"),
            file_path_str
        );

        let file_contents = file.read_bytes().await.map_err(|_| ())?;

        let resp = reqwest_client_context()
            .0
            .put(file_url)
            .with_credentials_if_wasm()
            .body(file_contents)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => continue,
            _ => return Err(()),
        }
    }

    Ok(())
}
