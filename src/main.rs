#![allow(non_snake_case)]

mod components;
mod requests;

use components::{
    file_explorer::FileExplorer, home::Home, login_form::Login, ui::context_menu::ContextMenuState,
    ui::upload_modal::UploadModal,
};
use dioxus::prelude::*;
use reqwest::Client;
use tracing::Level;

const LOG_LEVEL: Level = Level::DEBUG;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Done,
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UploadTask {
    pub file_name: String,
    pub status: UploadStatus,
    pub progress: f64,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct UploadState {
    pub tasks: Vec<UploadTask>,
}

pub fn api_url() -> &'static str {
    static API_URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    API_URL.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(val) = js_sys::Reflect::get(
                &js_sys::global(),
                &wasm_bindgen::JsValue::from_str("API_URL"),
            ) {
                if let Some(s) = val.as_string() {
                    return s;
                }
            }
            "http://localhost:8000".to_string()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
        }
    })
}

pub struct AppState {
    pub _theme: String,
    pub authenticated: bool,
    pub create_folder_modal: bool,
    pub create_file_modal: bool,
    pub edit_file_modal: Option<(String, Vec<String>)>,
    pub rename_modal: Option<(String, Vec<String>, bool)>, // (old_name, path, is_file)
    pub delete_modal: Option<(String, Vec<String>, bool)>, // (name, path, is_file)
    pub photo_viewer_modal: Option<(String, Vec<String>)>, // (file_name, current_path)
    pub needs_refetch: bool,
}

static TAILWIND_CSS: Asset = asset!("assets/tailwind.css");
static MAIN_CSS: Asset = asset!("assets/main.css");
static FAVICON_ICO: Asset = asset!("assets/favicon.ico");

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},

    #[route("/login")]
    Login {},

    #[layout(Home)]
    #[route("/folder/:..path")]
    FileExplorer { path: Vec<String> },
}

#[derive(Clone)]
pub struct ReqwestClient(pub Client);

pub const APP_VERSION: &str = match option_env!("APP_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

fn main() {
    dioxus_logger::init(LOG_LEVEL).expect("failed to init logger");

    #[cfg(target_arch = "wasm32")]
    LaunchBuilder::web().launch(App);

    #[cfg(not(target_arch = "wasm32"))]
    LaunchBuilder::desktop()
        .with_cfg(
            dioxus_desktop::Config::new()
                .with_window(
                    dioxus_desktop::WindowBuilder::new()
                        .with_resizable(true)
                        .with_title("JustEncrypt")
                        .with_inner_size(dioxus_desktop::LogicalSize::new(880.0, 600.0)),
                )
        )
        .launch(App);
}

fn App() -> Element {
    #[cfg(not(target_arch = "wasm32"))]
    let client = {
        let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
        Client::builder().cookie_provider(jar).build().unwrap()
    };

    #[cfg(target_arch = "wasm32")]
    let client = Client::builder().build().unwrap();

    let app_state = AppState {
        _theme: "light".to_string(),
        authenticated: false,
        create_folder_modal: false,
        create_file_modal: false,
        edit_file_modal: None,
        rename_modal: None,
        delete_modal: None,
        photo_viewer_modal: None,
        needs_refetch: false,
    };

    use_context_provider(|| Signal::new(ReqwestClient(client)));
    use_context_provider(|| Signal::new(app_state));
    use_context_provider(|| Signal::new(ContextMenuState::default()));
    use_context_provider(|| Signal::new(UploadState::default()));

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: MAIN_CSS }
        document::Link { rel: "icon", href: FAVICON_ICO }

        div { class: "dark w-full h-full",
            Router::<Route> {}
            UploadModal {}
        }
    }
}
