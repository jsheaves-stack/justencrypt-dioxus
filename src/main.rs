#![allow(non_snake_case)]

const LOG_LEVEL: Level = Level::DEBUG;

mod global {
    pub use crate::components::{file_explorer::FileExplorer, home::Home, login_form::Login};
    pub use dioxus::prelude::*;
    pub use tracing::Level;
}

#[cfg(not(target_arch = "wasm32"))]
mod desktop_only {
    pub use dioxus_desktop::LogicalSize;
    pub use dioxus_desktop::{Config, WindowBuilder};
    pub use reqwest::{cookie::Jar, Client};
    pub use std::sync::Arc;
}

#[cfg(target_arch = "wasm32")]
mod web_only {
    pub use reqwest::Client;
}

#[cfg(not(target_arch = "wasm32"))]
use desktop_only::*;
#[cfg(target_arch = "wasm32")]
use web_only::*;

use global::*;

mod components;
mod requests;

pub const API_URL: &str = "http://localhost:8000";

struct AppState {
    _theme: String,
    authenticated: bool,
    create_folder_modal: bool,
}

static TAILWIND_CSS: Asset = asset!("./assets/tailwind.css");
static MAIN_CSS: Asset = asset!("./assets/main.css");
static HEADER_SVG: Asset = asset!("./assets/header.svg");
static FAVICON_ICO: Asset = asset!("./assets/favicon.ico");

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},

    #[route("/login")]
    Login {},

    #[layout(Home)]
    #[route("/folder/:..path")]
    FileExplorer { path: Vec<String> },
}

#[derive(Clone)]
struct ReqwestClient(Client);

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus_logger::init(LOG_LEVEL).expect("failed to init logger");

    LaunchBuilder::web().launch(App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    dioxus_logger::init(LOG_LEVEL).expect("failed to init logger");

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_resizable(true)
                        .with_title("JustEncrypt")
                        .with_inner_size(LogicalSize::new(880.0, 600.0)),
                )
                .with_resource_directory("assets"),
        )
        .launch(App)
}

#[cfg(not(target_arch = "wasm32"))]
fn App() -> Element {
    let jar = Arc::new(Jar::default());
    let client = Client::builder().cookie_provider(jar).build().unwrap();
    let app_state = AppState {
        _theme: "light".to_string(),
        authenticated: false,
        create_folder_modal: false,
    };

    use_context_provider(|| Signal::new(ReqwestClient(client)));
    use_context_provider(|| Signal::new(app_state));

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: HEADER_SVG }
        document::Stylesheet { href: FAVICON_ICO }

        Router::<Route> {}
    }
}

#[cfg(target_arch = "wasm32")]
fn App() -> Element {
    let client = Client::builder().build().unwrap();
    let app_state = AppState {
        _theme: "light".to_string(),
        authenticated: false,
        create_folder_modal: false,
    };

    use_context_provider(|| Signal::new(ReqwestClient(client)));
    use_context_provider(|| Signal::new(app_state));

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: HEADER_SVG }
        document::Stylesheet { href: FAVICON_ICO }

        Router::<Route> {}
    }
}
