//! Builds `<img>`/`<video>` `src` URLs that the DOM/webview will fetch
//! directly (as opposed to URLs fetched from Rust via `ReqwestClient`), and
//! - on desktop - the local authenticated HTTP proxy server that backs them.
//!
//! On wasm this just points at the real API origin: the browser already
//! holds the session cookie (set by a `fetch`/XHR login request made from
//! that same origin), so it attaches it automatically to any same-origin
//! `<img>`/`<video>` request.
//!
//! On desktop there is no such shared cookie jar. Dioxus serves the app's
//! own UI from a `dioxus://` origin, while the session cookie set at login
//! lives only in the Rust-side `reqwest` cookie jar (`crate::api::cookie_jar`),
//! so a `<video src="https://api...">` pointed straight at the API is a
//! cross-origin request the webview's own network stack has no cookie for.
//! `media_server_port` runs a small HTTP proxy on `127.0.0.1` instead; see
//! its docs for why that (rather than a wry custom URI scheme, which was
//! tried first) is what actually fixes video playback. Because that proxy
//! forwards to the real, authenticated API on behalf of whoever asks, every
//! URL it's given also carries a per-launch random token that only this
//! process knows (see `proxy_token`) - otherwise any other local process
//! could use it as an open tunnel to the logged-in user's files.

/// Builds `/{endpoint}/{path...}/{name}`, with every folder and the file name
/// percent-encoded as its own segment (see `util::encode_segment`). A name
/// containing e.g. `?` or `#` (both valid on Linux, e.g. `invoice?2024.pdf`)
/// would otherwise be truncated at that character by URL parsing - on
/// desktop that also shifts the rest of the string into the query string,
/// hiding the real `?token=...` and making `split_path_and_token` fail to
/// find it.
fn media_path(endpoint: &str, current_path: &[String], name: &str) -> String {
    let mut path = format!("/{endpoint}");
    crate::util::push_path_segments(&mut path, current_path, Some(name));
    path
}

/// Builds a URL for file `name` in `current_path` under `endpoint` (e.g.
/// `"thumbnail"` or `"file"`) that the DOM can fetch directly. This is the
/// only place that should construct such a URL - going through it (rather
/// than `media_origin`/`proxy_token` directly) means neither the name
/// encoding nor the desktop auth token can be forgotten at a call site.
#[cfg(target_arch = "wasm32")]
pub fn media_url(endpoint: &str, current_path: &[String], name: &str) -> String {
    format!(
        "{}{}",
        media_origin(),
        media_path(endpoint, current_path, name)
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub fn media_url(endpoint: &str, current_path: &[String], name: &str) -> String {
    format!(
        "{}{}?token={}",
        media_origin(),
        media_path(endpoint, current_path, name),
        proxy_token()
    )
}

#[cfg(target_arch = "wasm32")]
fn media_origin() -> String {
    crate::api_url()
}

#[cfg(not(target_arch = "wasm32"))]
fn media_origin() -> String {
    format!("http://127.0.0.1:{}", media_server_port())
}

/// Per-launch random token the media proxy requires on every request. It's
/// the proxy's only access control: the loopback socket itself accepts
/// connections from any local process, so without this, anything running as
/// the same OS user could use the proxy as an unauthenticated tunnel to the
/// logged-in user's files for as long as the app is open. A fresh UUID is
/// generated once per process and never persisted - only this running
/// instance (and the URLs it hands to its own webview) ever knows it.
#[cfg(not(target_arch = "wasm32"))]
fn proxy_token() -> &'static str {
    use std::sync::OnceLock;
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| uuid::Uuid::new_v4().to_string())
}

/// The blocking reqwest client backing the media proxy server. Separate from
/// the async `ReqwestClient` context value used elsewhere in the app (the
/// proxy server runs on plain OS threads, not the tokio runtime - see
/// `start_media_server` - so it needs the blocking client, not the async
/// one), but built on the same `cookie_jar()` singleton, so both see the
/// same session cookie.
#[cfg(not(target_arch = "wasm32"))]
fn media_proxy_client() -> reqwest::blocking::Client {
    use std::sync::OnceLock;
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::blocking::Client::builder()
                .cookie_provider(crate::api::cookie_jar())
                .build()
                .expect("failed to build media proxy client")
        })
        .clone()
}

/// Port of the local media proxy server, started on first use and then fixed
/// for the process lifetime.
#[cfg(not(target_arch = "wasm32"))]
fn media_server_port() -> u16 {
    use std::sync::OnceLock;
    static PORT: OnceLock<u16> = OnceLock::new();
    *PORT.get_or_init(start_media_server)
}

/// Starts the local media proxy server on a loopback port and returns that
/// port. `<img>`/`<video>` elements in the desktop webview load through this
/// instead of hitting the API directly, for two reasons:
///
/// - The session cookie set at login lives only in the Rust-side `reqwest`
///   cookie jar (`cookie_jar`), never in WebKitGTK's own cookie store, so a
///   cross-origin `<video src="https://api...">` request has no way to
///   authenticate.
/// - A wry/WebKitGTK custom URI scheme
///   (`Config::with_asynchronous_custom_protocol`) was tried first and does
///   fix the auth problem, but on Linux WebKitGTK never forwards the
///   incoming `Range` header into a custom-scheme request - confirmed by
///   wry's own example (`examples/streaming.rs`): "if the webview sent a
///   range header, we need to send a 206 in return - Actually only macOS and
///   Windows are supported. Linux will ALWAYS return empty headers." A
///   `<video>` element routed through a custom scheme therefore has no way
///   to make a partial request and downloads the entire file before
///   anything plays, which for anything but a few seconds of video looks
///   exactly like a hang.
///
/// A real loopback HTTP server has neither problem: WebKitGTK's ordinary
/// networking/GStreamer stack talks to it exactly like it would any other
/// HTTP server - real `Range` requests, streamed responses - and we
/// authenticate server-side with the same cookie jar the rest of the app
/// uses. It only binds `127.0.0.1`, so nothing outside this machine can
/// reach it over the network - see `proxy_token` for why that alone isn't
/// enough and requests still need to prove they came from this app.
#[cfg(not(target_arch = "wasm32"))]
fn start_media_server() -> u16 {
    let server =
        tiny_http::Server::http("127.0.0.1:0").expect("failed to start local media proxy server");
    let port = match server.server_addr() {
        tiny_http::ListenAddr::IP(addr) => addr.port(),
        // We always bind an IP address (above), so this can't happen -
        // tiny_http only returns the Unix-socket variant for servers started
        // with `Server::http_unix`.
        tiny_http::ListenAddr::Unix(_) => unreachable!("media proxy bound a unix socket"),
    };

    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            // Each request gets its own thread so a slow/streaming video
            // response can't block other concurrent requests (e.g. thumbnails
            // loading in the same folder view).
            std::thread::spawn(move || serve_media_request(request));
        }
    });

    port
}

/// Splits a request target like `/thumbnail/foo.jpg?token=abc` into its path
/// and the value of its `token` query parameter, if any.
#[cfg(not(target_arch = "wasm32"))]
fn split_path_and_token(url: &str) -> (&str, Option<&str>) {
    match url.split_once('?') {
        Some((path, query)) => {
            let token = query.split('&').find_map(|kv| kv.strip_prefix("token="));
            (path, token)
        }
        None => (url, None),
    }
}

/// Checks a request target's `token` query parameter against
/// `proxy_token()`. This is the proxy's only access control (see the module
/// docs), so every request must pass through here before anything is
/// forwarded - deliberately with no further restriction on which path a
/// validated request can reach: a valid token *is* being treated as fully
/// authenticated, the same as the cookie it stands in for.
///
/// Returns the path with the `token` parameter stripped (the real API
/// doesn't need it) if the token matches, or `None` if it's missing or wrong.
#[cfg(not(target_arch = "wasm32"))]
fn authenticate(url: &str) -> Option<&str> {
    let (path, token) = split_path_and_token(url);
    (token == Some(proxy_token())).then_some(path)
}

/// Proxies one request to `{api_url()}{path}` through `media_proxy_client()`
/// and relays the response back - status, the `Content-Type`/
/// `Content-Range`/`Accept-Ranges`/`Cache-Control`/`ETag`/`Last-Modified`
/// headers, and the body streamed (not buffered) straight from the backend
/// response into the reply - including forwarding an incoming `Range`
/// header so video seeking works, and `If-None-Match`/`If-Modified-Since` so
/// the webview can revalidate a cached thumbnail/image with a cheap 304
/// instead of always re-fetching the full body through the proxy.
#[cfg(not(target_arch = "wasm32"))]
fn serve_media_request(request: tiny_http::Request) {
    let url = request.url().to_string();
    let Some(path) = authenticate(&url) else {
        let _ = request.respond(tiny_http::Response::empty(403));
        return;
    };
    let target = format!("{}{}", crate::api_url(), path);

    let mut req = media_proxy_client().get(&target);
    for name in ["Range", "If-None-Match", "If-Modified-Since"] {
        if let Some(h) = request.headers().iter().find(|h| h.field.equiv(name)) {
            req = req.header(name, h.value.as_str());
        }
    }

    let resp = match req.send() {
        Ok(resp) => resp,
        Err(_) => {
            let _ = request.respond(tiny_http::Response::empty(502));
            return;
        }
    };

    let status = resp.status().as_u16();
    let mut headers = Vec::new();
    for name in [
        "content-type",
        "content-range",
        "accept-ranges",
        "cache-control",
        "etag",
        "last-modified",
    ] {
        if let Some(v) = resp.headers().get(name).and_then(|v| v.to_str().ok()) {
            if let Ok(header) = tiny_http::Header::from_bytes(name.as_bytes(), v.as_bytes()) {
                headers.push(header);
            }
        }
    }
    let content_length = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());

    // tiny_http defaults to chunked transfer (dropping Content-Length) for
    // any response over 32KB, *even when the length is known* - see
    // `chunked_threshold` in its docs. Virtually every video and many
    // images are over that, so without this override every such response
    // loses its Content-Length, which is very likely why the first version
    // of this proxy still didn't fix playback: video needs a known length
    // upfront to determine duration/seekability.
    let response = tiny_http::Response::new(status.into(), headers, resp, content_length, None)
        .with_chunked_threshold(usize::MAX);
    let _ = request.respond(response);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn path(segments: &[&str]) -> Vec<String> {
        segments.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn media_path_for_root_level_file_has_no_empty_segment() {
        assert_eq!(
            media_path("thumbnail", &[], "foo.jpg"),
            "/thumbnail/foo.jpg"
        );
    }

    #[test]
    fn media_path_includes_folders() {
        assert_eq!(
            media_path("file", &path(&["a", "b"]), "foo.jpg"),
            "/file/a/b/foo.jpg"
        );
    }

    #[test]
    fn media_path_encodes_folder_and_file_names() {
        assert_eq!(
            media_path(
                "thumbnail",
                &path(&["100% real", "a#b"]),
                "invoice?2024.pdf"
            ),
            "/thumbnail/100%25%20real/a%23b/invoice%3F2024.pdf"
        );
    }

    #[test]
    fn a_filename_containing_a_question_mark_still_authenticates() {
        // Regression test: before path escaping, a literal `?` in the
        // filename made split_path_and_token split at the wrong place, so
        // the real `?token=...` was never found and authenticate() failed.
        let target = format!(
            "{}?token={}",
            media_path("thumbnail", &[], "invoice?2024.pdf"),
            proxy_token()
        );
        assert_eq!(authenticate(&target), Some("/thumbnail/invoice%3F2024.pdf"));
    }

    #[test]
    fn split_path_and_token_splits_on_query() {
        assert_eq!(
            split_path_and_token("/thumbnail/foo.jpg?token=abc"),
            ("/thumbnail/foo.jpg", Some("abc"))
        );
    }

    #[test]
    fn split_path_and_token_handles_no_query() {
        assert_eq!(
            split_path_and_token("/file/foo.jpg"),
            ("/file/foo.jpg", None)
        );
    }

    #[test]
    fn split_path_and_token_finds_token_among_other_params() {
        assert_eq!(
            split_path_and_token("/file/foo.jpg?a=1&token=abc&b=2"),
            ("/file/foo.jpg", Some("abc"))
        );
    }

    #[test]
    fn split_path_and_token_is_none_when_token_param_absent() {
        assert_eq!(
            split_path_and_token("/file/foo.jpg?a=1"),
            ("/file/foo.jpg", None)
        );
    }

    #[test]
    fn authenticate_accepts_the_real_token_and_strips_it() {
        let url = format!("/thumbnail/foo.jpg?token={}", proxy_token());
        assert_eq!(authenticate(&url), Some("/thumbnail/foo.jpg"));
    }

    #[test]
    fn authenticate_rejects_wrong_or_missing_token() {
        assert_eq!(authenticate("/thumbnail/foo.jpg?token=wrong"), None);
        assert_eq!(authenticate("/thumbnail/foo.jpg"), None);
    }

    #[test]
    fn authenticate_does_not_restrict_which_path_a_valid_token_can_reach() {
        // Deliberate: a valid token is full authentication, same as the
        // cookie it stands in for - see `authenticate`'s docs.
        let url = format!("/anything/at/all?token={}", proxy_token());
        assert_eq!(authenticate(&url), Some("/anything/at/all"));
    }
}
