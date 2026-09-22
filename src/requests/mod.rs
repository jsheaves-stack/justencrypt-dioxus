pub mod file;
pub mod folder;
pub mod session;

use reqwest::{RequestBuilder, StatusCode};

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

pub fn build_url(endpoint: &str, current_path: &[String], item_name: Option<&str>) -> String {
    let mut url = format!("{}/{}", crate::api_url(), endpoint);
    if !current_path.is_empty() {
        url.push('/');
        url.push_str(&current_path.join("/"));
    }
    if let Some(name) = item_name {
        if !name.is_empty() {
            url.push('/');
            url.push_str(name);
        }
    }
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
    }
}
