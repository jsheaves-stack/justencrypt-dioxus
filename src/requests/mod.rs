pub mod file;
pub mod folder;
pub mod session;

use crate::util::push_path_segments;
use reqwest::{RequestBuilder, StatusCode};

#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
    web_sys::{window, HtmlAnchorElement},
};

#[cfg(not(target_arch = "wasm32"))]
use {
    crate::ReqwestClient, dioxus::signals::Signal, tokio::fs::File as TokioFile,
    tokio::io::AsyncWriteExt,
};

pub trait RequestBuilderExt {
    fn with_credentials_if_wasm(self) -> Self;
}

impl RequestBuilderExt for RequestBuilder {
    #[cfg(target_arch = "wasm32")]
    fn with_credentials_if_wasm(self) -> Self {
        self.fetch_credentials_include()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn with_credentials_if_wasm(self) -> Self {
        self
    }
}

/// Builds `{api_url}/{endpoint}/{path...}/{item_name}`. `endpoint` is a fixed
/// route (e.g. `"file/upload/initiate"`) and is used as-is; every folder in
/// `current_path` and `item_name` is percent-encoded as its own segment.
pub fn build_url(endpoint: &str, current_path: &[String], item_name: Option<&str>) -> String {
    let mut url = format!("{}/{}", crate::api_url(), endpoint);
    push_path_segments(&mut url, current_path, item_name);
    url
}

/// Why a request failed, kept around just long enough to turn into a
/// user-facing message - callers that only need to know "did it work" can
/// still just check `.is_err()`.
///
/// The `String` payloads are only ever surfaced through `{:?}` in
/// `tracing::error!` logging at each call site, which rustc's `dead_code`
/// lint doesn't credit as a "read" - hence the explicit allow rather than
/// dropping detail that's genuinely used.
#[derive(Debug, Clone)]
pub enum RequestError {
    /// The server responded with a non-success status.
    Status(StatusCode),
    /// The request itself never got a response (network/TLS/timeout).
    #[allow(dead_code)]
    Network(String),
    /// A success response's body couldn't be read or parsed as expected.
    #[allow(dead_code)]
    Decode(String),
}

impl RequestError {
    /// A short, user-facing message for the given `action` (e.g. "rename
    /// file"), specialized for the handful of statuses worth explaining
    /// beyond "it failed".
    pub fn user_message(&self, action: &str) -> String {
        match self {
            RequestError::Status(StatusCode::CONFLICT) => {
                "An item with that name already exists".to_string()
            }
            RequestError::Status(StatusCode::NOT_FOUND) => {
                format!("Failed to {action}: item not found")
            }
            RequestError::Status(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => {
                "Your session has expired, please log in again".to_string()
            }
            RequestError::Status(StatusCode::TOO_MANY_REQUESTS) => {
                format!("Failed to {action}: the server is busy, try again shortly")
            }
            RequestError::Network(_) => {
                format!("Failed to {action}: could not reach the server")
            }
            RequestError::Status(_) | RequestError::Decode(_) => {
                format!("Failed to {action}")
            }
        }
    }
}

/// Sends `req` and maps it to a `RequestError` on any non-success outcome.
/// On both success and failure, the body is drained so the underlying
/// connection can be reused - callers that need the body call `.bytes()`/
/// `.text()`/`.json()` themselves on the returned response.
pub async fn send(req: RequestBuilder) -> Result<reqwest::Response, RequestError> {
    let resp = req
        .send()
        .await
        .map_err(|e| RequestError::Network(e.to_string()))?;

    if resp.status().is_success() {
        Ok(resp)
    } else {
        let status = resp.status();
        let _ = resp.bytes().await;
        Err(RequestError::Status(status))
    }
}

/// Like `send`, but also drains and discards the response body on success -
/// for calls where nothing needs the body back.
pub async fn send_and_discard(req: RequestBuilder) -> Result<(), RequestError> {
    let resp = send(req).await?;
    let _ = resp.bytes().await;
    Ok(())
}

/// Starts a browser download of `url` by clicking a temporary `<a download>`.
/// The browser fetches the URL itself (with the session cookie), so the body
/// streams straight to disk and never passes through wasm memory - but it
/// also means an error status shows up only as a failed download in the
/// browser, not as an `Err` here.
///
/// `download_name` is only a hint: an empty string defers to the server's
/// `Content-Disposition` filename, and browsers ignore the attribute
/// entirely for cross-origin URLs.
#[cfg(target_arch = "wasm32")]
pub fn trigger_browser_download(url: &str, download_name: &str) -> Result<(), RequestError> {
    tracing::debug!("Triggering WASM download for {}", url);

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

    a.set_href(url);
    a.set_download(download_name);

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

/// Asks the user where to save (suggesting `suggested_name`), then streams a
/// GET of `url` into that file. Returns `Ok` without doing anything if the
/// user cancels the dialog.
///
/// A failed or cut-short download (e.g. a folder archive that ends before
/// its `Content-Length`, which reqwest reports as a body error) deletes the
/// partial file rather than leaving a truncated copy that looks complete.
#[cfg(not(target_arch = "wasm32"))]
pub async fn save_download(
    reqwest_client_context: Signal<ReqwestClient>,
    url: &str,
    suggested_name: &str,
) -> Result<(), RequestError> {
    let Some(handle) = rfd::AsyncFileDialog::new()
        .set_file_name(suggested_name)
        .save_file()
        .await
    else {
        tracing::debug!("Download cancelled by user");
        return Ok(());
    };
    let path = handle.path();
    tracing::debug!("Downloading {} to {:?}", url, path);

    let mut resp = send(
        reqwest_client_context()
            .0
            .get(url)
            .with_credentials_if_wasm(),
    )
    .await
    .inspect_err(|e| tracing::error!("Download failed: {:?}", e))?;

    let mut file = TokioFile::create(path).await.map_err(|e| {
        tracing::error!("Failed to create local file: {}", e);
        RequestError::Decode(e.to_string())
    })?;

    let result = async {
        while let Some(chunk) = resp.chunk().await.map_err(|e| {
            tracing::error!("Failed to read chunk during download: {}", e);
            RequestError::Network(e.to_string())
        })? {
            file.write_all(&chunk).await.map_err(|e| {
                tracing::error!("Failed to write chunk to local file: {}", e);
                RequestError::Decode(e.to_string())
            })?;
        }
        // tokio's `File` buffers the last write; without this a late error
        // (e.g. disk full) would be lost and success reported anyway.
        file.flush().await.map_err(|e| {
            tracing::error!("Failed to flush local file: {}", e);
            RequestError::Decode(e.to_string())
        })
    }
    .await;

    match &result {
        Ok(()) => tracing::debug!("Download saved successfully"),
        Err(_) => {
            drop(file);
            if let Err(e) = tokio::fs::remove_file(path).await {
                tracing::error!("Failed to remove partial download {:?}: {}", path, e);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_base(url: &str) -> &str {
        url.strip_prefix(&crate::api_url())
            .expect("built URL should start with the API base")
    }

    #[test]
    fn build_url_with_no_path_or_name() {
        let url = build_url("folder", &[], None);
        assert_eq!(strip_base(&url), "/folder");
    }

    #[test]
    fn build_url_with_path_only() {
        let path = vec!["a".to_string(), "b".to_string()];
        let url = build_url("folder", &path, None);
        assert_eq!(strip_base(&url), "/folder/a/b");
    }

    #[test]
    fn build_url_with_path_and_name() {
        let path = vec!["a".to_string()];
        let url = build_url("file", &path, Some("report.txt"));
        assert_eq!(strip_base(&url), "/file/a/report.txt");
    }

    #[test]
    fn build_url_ignores_empty_item_name() {
        let url = build_url("folder", &[], Some(""));
        assert_eq!(strip_base(&url), "/folder");
    }

    #[test]
    fn build_url_encodes_reserved_characters_in_names() {
        // Regression test: a raw `#` truncated the URL at that point, so
        // deleting `notes#1.txt` sent `DELETE /file/notes`.
        let path = vec!["my docs".to_string(), "q?a".to_string()];
        let url = build_url("file", &path, Some("notes#1.txt"));
        assert_eq!(strip_base(&url), "/file/my%20docs/q%3Fa/notes%231.txt");
    }

    #[test]
    fn build_url_does_not_encode_the_endpoint() {
        let url = build_url("file/upload/initiate", &[], Some("a b.txt"));
        assert_eq!(strip_base(&url), "/file/upload/initiate/a%20b.txt");
    }

    #[test]
    fn build_url_survives_url_parsing_intact() {
        // What actually matters: after reqwest parses the URL, the name is
        // still the last path segment and nothing leaked into the query or
        // fragment.
        let url = build_url("file", &[], Some("invoice?2024#1.pdf"));
        let parsed = reqwest::Url::parse(&url).unwrap();
        assert_eq!(parsed.query(), None);
        assert_eq!(parsed.fragment(), None);
        assert_eq!(
            parsed.path_segments().unwrap().next_back(),
            Some("invoice%3F2024%231.pdf")
        );
    }

    #[test]
    fn user_message_specializes_conflict_and_auth_statuses() {
        assert_eq!(
            RequestError::Status(StatusCode::CONFLICT).user_message("create folder"),
            "An item with that name already exists"
        );
        assert_eq!(
            RequestError::Status(StatusCode::UNAUTHORIZED).user_message("rename file"),
            "Your session has expired, please log in again"
        );
        assert_eq!(
            RequestError::Status(StatusCode::INTERNAL_SERVER_ERROR).user_message("delete item"),
            "Failed to delete item"
        );
        assert_eq!(
            RequestError::Status(StatusCode::TOO_MANY_REQUESTS).user_message("download folder"),
            "Failed to download folder: the server is busy, try again shortly"
        );
    }
}
