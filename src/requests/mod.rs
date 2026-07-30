pub mod file;
pub mod folder;
pub mod session;

use reqwest::RequestBuilder;

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
