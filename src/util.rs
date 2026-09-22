//! Small cross-platform helpers shared by code that isn't itself request- or
//! UI-specific enough to belong under `requests` or `components`.

use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use tokio::time;

/// Sleeps for `duration` on either platform: `tokio::time::sleep` on
/// desktop, `gloo_timers` (backed by `setTimeout`) on wasm.
pub async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::sleep(duration).await;

    #[cfg(not(target_arch = "wasm32"))]
    time::sleep(duration).await;
}

/// Builds the `{path}/` prefix used when composing a media-proxy URL for a
/// file inside `current_path` - empty (not just `/`) when the folder is the
/// root, so a root-level file's URL doesn't gain a leading slash before its
/// name.
pub fn path_prefix(current_path: &[String]) -> String {
    if current_path.is_empty() {
        String::new()
    } else {
        format!("{}/", current_path.join("/"))
    }
}
