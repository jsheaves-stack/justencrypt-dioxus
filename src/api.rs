//! API base URL, config persistence, and the desktop cookie jar.
//!
//! On desktop the URL is user-configurable (see `ServerUrlInput` in the
//! login form) and persisted to disk via `config`; on wasm it's fixed at
//! build/runtime (an injected `window.API_URL` or the `API_URL` env var at
//! compile time, falling back to localhost for local dev).

use std::sync::OnceLock;

#[cfg(not(target_arch = "wasm32"))]
use std::{
    env::var,
    sync::{Arc, RwLock},
};

#[cfg(not(target_arch = "wasm32"))]
use reqwest_cookie_store::CookieStoreMutex;

#[cfg(not(target_arch = "wasm32"))]
use crate::config;

#[cfg(not(target_arch = "wasm32"))]
static API_URL: OnceLock<RwLock<String>> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn api_url_lock() -> &'static RwLock<String> {
    API_URL.get_or_init(|| {
        let cfg = config::load_config();
        // Allow env var override if present
        let url = var("API_URL").unwrap_or(cfg.api_url);
        RwLock::new(url)
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn api_url() -> String {
    api_url_lock().read().unwrap().clone()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_api_url(new_url: String) {
    // Update memory only
    if let Ok(mut writer) = api_url_lock().write() {
        *writer = new_url;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_api_url_to_disk() {
    let url = api_url();
    std::thread::spawn(move || {
        let mut cfg = config::load_config();
        cfg.api_url = url;
        config::save_config(&cfg);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn api_url() -> String {
    static API_URL: OnceLock<String> = OnceLock::new();
    API_URL
        .get_or_init(|| {
            let mut url = None;
            if let Some(win) = web_sys::window() {
                if let Ok(val) =
                    js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("API_URL"))
                {
                    if let Some(s) = val.as_string() {
                        if !s.is_empty() {
                            url = Some(s);
                        }
                    }
                }
            }
            url.or_else(|| option_env!("API_URL").map(|s| s.to_string()))
                .unwrap_or_else(|| "http://localhost:8000".to_string())
        })
        .clone()
}

/// The cookie jar backing the desktop HTTP client, shared so login/logout can
/// persist or clear it on disk. On wasm the browser owns cookies instead.
#[cfg(not(target_arch = "wasm32"))]
pub fn cookie_jar() -> Arc<CookieStoreMutex> {
    static JAR: OnceLock<Arc<CookieStoreMutex>> = OnceLock::new();
    JAR.get_or_init(|| Arc::new(CookieStoreMutex::new(config::load_cookie_store())))
        .clone()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn persist_session_cookies() {
    let jar = cookie_jar();
    let store = jar.lock().unwrap();
    config::save_cookie_store(&store);
}

#[cfg(target_arch = "wasm32")]
pub fn persist_session_cookies() {}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_session_cookies() {
    config::clear_cookie_store();
    *cookie_jar().lock().unwrap() = Default::default();
}

#[cfg(target_arch = "wasm32")]
pub fn clear_session_cookies() {}
