#[cfg(not(target_arch = "wasm32"))]
use cookie_store::CookieStore;
#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufReader, BufWriter};
#[cfg(all(not(target_arch = "wasm32"), unix))]
use std::os::unix::fs::PermissionsExt;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub api_url: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:8000".to_string(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "justencrypt-dioxus").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            let _ = fs::create_dir_all(config_dir);
        }
        config_dir.join("config.json")
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_config() -> AppConfig {
    let mut config = AppConfig::default();
    // Only re-save when the parsed config doesn't round-trip back to the
    // same bytes already on disk - i.e. the file was missing/unreadable, or
    // an old config was missing fields that `#[serde(default)]` just filled
    // in. Otherwise every startup would do a blocking write for nothing.
    let mut needs_save = true;

    if let Some(path) = get_config_path() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(parsed) = serde_json::from_str::<AppConfig>(&contents) {
                needs_save = serde_json::to_string_pretty(&parsed).ok().as_deref() != Some(contents.as_str());
                config = parsed;
            }
        }
    }

    if needs_save {
        save_config(&config);
    }
    config
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_config(config: &AppConfig) {
    if let Some(path) = get_config_path() {
        if let Ok(json) = serde_json::to_string_pretty(config) {
            let _ = fs::write(path, json);
        }
    }
}

/// Session cookies live in the data dir rather than the config dir, and get
/// 0600 permissions on Unix since they're effectively a bearer credential.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_cookie_store_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "justencrypt-dioxus").map(|proj_dirs| {
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            let _ = fs::create_dir_all(data_dir);
        }
        data_dir.join("session.json")
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_cookie_store() -> CookieStore {
    get_cookie_store_path()
        .and_then(|path| fs::File::open(path).ok())
        .and_then(|file| cookie_store::serde::json::load(BufReader::new(file)).ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_cookie_store(store: &CookieStore) {
    let Some(path) = get_cookie_store_path() else {
        return;
    };
    let Ok(file) = fs::File::create(&path) else {
        return;
    };

    #[cfg(unix)]
    let _ = file.set_permissions(fs::Permissions::from_mode(0o600));

    let mut writer = BufWriter::new(file);
    let _ = cookie_store::serde::json::save_incl_expired_and_nonpersistent(store, &mut writer);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_cookie_store() {
    if let Some(path) = get_cookie_store_path() {
        let _ = fs::remove_file(path);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use cookie_store::{CookieStore, RawCookie};
    use url::Url;

    #[test]
    fn session_cookie_with_no_expiry_survives_a_save_load_round_trip() {
        let mut store = CookieStore::default();
        let url = Url::parse("http://localhost:8000/").unwrap();
        let cookie = RawCookie::parse("session=abc123; Path=/").unwrap();
        store.store_response_cookies(std::iter::once(cookie), &url);
        assert!(store.get("localhost", "/", "session").is_some());

        let mut buf = Vec::new();
        cookie_store::serde::json::save_incl_expired_and_nonpersistent(&store, &mut buf).unwrap();

        let reloaded = cookie_store::serde::json::load(buf.as_slice()).unwrap();
        assert!(reloaded.get("localhost", "/", "session").is_some());
    }

    #[test]
    fn expired_cookie_is_dropped_on_load() {
        let mut store = CookieStore::default();
        let url = Url::parse("http://localhost:8000/").unwrap();
        let cookie =
            RawCookie::parse("session=abc123; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT")
                .unwrap();
        store.store_response_cookies(std::iter::once(cookie), &url);

        let mut buf = Vec::new();
        cookie_store::serde::json::save_incl_expired_and_nonpersistent(&store, &mut buf).unwrap();

        let reloaded = cookie_store::serde::json::load(buf.as_slice()).unwrap();
        assert!(reloaded.get("localhost", "/", "session").is_none());
    }
}
