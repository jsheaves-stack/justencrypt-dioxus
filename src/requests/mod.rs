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
