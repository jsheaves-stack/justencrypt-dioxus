use dioxus::prelude::*;

// On wasm, `<img>` fetches the real (cross-origin) API URL directly, and
// needs `crossorigin="use-credentials"` so the browser attaches the session
// cookie to that cross-origin request. On desktop, `src` instead points at
// the local media proxy (`media_url()`), which already authenticates
// server-side using its own cookie jar - the request needs no credentials
// forwarded to it at all. Setting `crossorigin` there anyway forces the
// webview into CORS-with-credentials fetch mode, which requires an
// `Access-Control-Allow-Origin`/`Access-Control-Allow-Credentials` response
// the proxy doesn't send, so the webview silently discards an otherwise
// successful response - a plain broken-image icon despite the proxy itself
// working (verified: curling the exact same URL returns a valid image).
#[cfg(target_arch = "wasm32")]
pub const CROSSORIGIN: Option<&str> = Some("use-credentials");
#[cfg(not(target_arch = "wasm32"))]
pub const CROSSORIGIN: Option<&str> = None;

#[derive(Clone, Copy, PartialEq)]
enum LoadState {
    Loading,
    Loaded,
    Error,
}

#[component]
pub fn Image(src: String, class: Option<String>, alt: Option<String>) -> Element {
    let class = class.unwrap_or_default();
    let alt = alt.unwrap_or_default();

    let mut state = use_signal(|| LoadState::Loading);

    // A given `Image` instance gets reused for a different `src` (e.g. a
    // grid item scrolled to a new file), so its load state has to reset
    // rather than keep showing the previous file's result.
    use_effect(use_reactive((&src,), move |_| {
        state.set(LoadState::Loading);
    }));

    rsx! {
        if state() == LoadState::Loading {
            div { class: "{class} bg-gray-200 dark:bg-gray-700 animate-pulse" }
        } else if state() == LoadState::Error {
            div { class: "{class} bg-gray-200 dark:bg-gray-700 flex items-center justify-center text-xs text-gray-500 dark:text-gray-400",
                "Failed to load"
            }
        }
        img {
            src: "{src}",
            class: if state() == LoadState::Loaded { "{class}" } else { "hidden" },
            alt: "{alt}",
            crossorigin: CROSSORIGIN,
            onload: move |_| state.set(LoadState::Loaded),
            onerror: move |_| state.set(LoadState::Error),
        }
    }
}
