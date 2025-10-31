use std::sync::Arc;

use dioxus::{html::FileEngine, signals::Signal};

use crate::{ReqwestClient, API_URL};

pub async fn put_file(
    reqwest_client_context: Signal<ReqwestClient>,
    current_path: Vec<String>,
    file_engine: &Arc<dyn FileEngine + 'static>,
) -> Result<(), ()> {
    let files = file_engine.files();

    for file_name in &files {
        let file_url = format!("{}/file/{}/{}", API_URL, current_path.join("/"), file_name);
        let file_contents: Vec<u8> = match file_engine.read_file(file_name).await {
            Some(file_contents) => file_contents,
            None => {
                tracing::error!("Failed to read file: {}", file_name);
                continue;
            }
        };

        #[cfg(target_arch = "wasm32")]
        let resp = reqwest_client_context()
            .0
            .put(file_url)
            .body(file_contents)
            .fetch_credentials_include()
            .send()
            .await;

        #[cfg(not(target_arch = "wasm32"))]
        let resp = reqwest_client_context()
            .0
            .put(folder_url)
            .body(file_contents)
            .send()
            .await;

        let _ = match resp {
            Ok(r) => {
                if r.status().is_success() {
                    Ok(())
                } else {
                    Err(())
                }
            }
            Err(_err) => Err(()),
        };
    }

    Ok(())
}
