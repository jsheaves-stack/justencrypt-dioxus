use crate::{ReqwestClient, API_URL};
use dioxus::{html::FileData, signals::Signal};

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
            API_URL,
            current_path.join("/"),
            file_path_str
        );

        #[cfg(target_arch = "wasm32")]
        let resp = {
            let file_contents = file.read_bytes().await.unwrap();

            reqwest_client_context()
                .0
                .put(file_url)
                .body(file_contents)
                .fetch_credentials_include()
                .send()
                .await
        };

        #[cfg(not(target_arch = "wasm32"))]
        let resp = {
            let byte_stream = file.read_bytes().await.unwrap();

            reqwest_client_context()
                .0
                .put(file_url)
                .body(byte_stream)
                .send()
                .await
        };

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
