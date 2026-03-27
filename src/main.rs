#![allow(non_snake_case)]

mod components;
mod requests;
pub mod state;

pub use state::*;

use components::{
    file_explorer::FileExplorer,
    home::Home,
    login_form::Login,
    ui::{context_menu::ContextMenuState, upload_modal::UploadModal},
};
use dioxus::prelude::*;
use reqwest::Client;
#[cfg(not(target_arch = "wasm32"))]
use std::env::var;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::Level;

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::DEBUG;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::INFO;

pub fn api_url() -> &'static str {
    static API_URL: OnceLock<String> = OnceLock::new();
    API_URL.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        {
            option_env!("API_URL")
                .unwrap_or("http://localhost:8000")
                .to_string()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            var("API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
        }
    })
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
            dioxus_desktop::Config::new().with_window(
                dioxus_desktop::WindowBuilder::new()
                    .with_resizable(true)
                    .with_title("JustEncrypt")
                    .with_inner_size(dioxus_desktop::LogicalSize::new(880.0, 600.0)),
            ),
        )
        .launch(App);
}

fn App() -> Element {
    #[cfg(not(target_arch = "wasm32"))]
    let client = {
        let jar = Arc::new(reqwest::cookie::Jar::default());
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
