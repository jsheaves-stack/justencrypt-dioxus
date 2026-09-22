#![allow(non_snake_case)]

mod api;
mod components;
mod config;
mod media_proxy;
mod models;
mod requests;
pub mod state;
mod util;

pub use api::*;
pub use media_proxy::*;
pub use models::*;
pub use state::*;

use components::{
    file_explorer::FileExplorer,
    home::Home,
    login_form::Login,
    ui::{
        back_handler::BackHandler, context_menu::ContextMenuState, toast::ToastContainer,
        upload_modal::UploadModal,
    },
};
use dioxus::prelude::*;
use reqwest::Client;
use tracing::Level;

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::DEBUG;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::INFO;

const TAILWIND_CSS: &str = include_str!("../assets/tailwind.css");
const MAIN_CSS: &str = include_str!("../assets/main.css");

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
    // The Docker build always sets this env var, even to "" when no
    // --build-arg APP_VERSION=... is passed, so option_env! sees Some("")
    // rather than None in that case - treat that the same as unset.
    Some(v) if !v.is_empty() => v,
    _ => env!("CARGO_PKG_VERSION"),
};

fn main() {
    dioxus_logger::init(LOG_LEVEL).expect("failed to init logger");

    #[cfg(target_arch = "wasm32")]
    LaunchBuilder::web().launch(App);

    // WebKitGTK's DMA-BUF renderer (the default since roughly 2.40) is badly
    // broken on a wide range of GPU/Mesa/compositor combinations - not
    // anything this app does, but a widely-reported upstream issue - causing
    // severe lag across the *entire* UI that gets dramatically worse the
    // longer the app runs (confirmed on this exact build: disabling it took
    // a folder view from unusably laggy, worsening while scrolling, to
    // smooth). There's no in-app fix beyond steering WebKitGTK away from
    // that renderer path before it starts, which has to happen via this env
    // var before the webview is created.
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

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
                .with_menu(None),
        )
        .launch(App);
}

fn App() -> Element {
    #[cfg(not(target_arch = "wasm32"))]
    let client = Client::builder()
        .cookie_provider(cookie_jar())
        .build()
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    let client = Client::builder().build().unwrap();

    use_context_provider(|| Signal::new(ReqwestClient(client)));
    let mut app_state_sig = use_context_provider(|| Signal::new(AppState::default()));
    use_context_provider(|| Signal::new(ModalState::default()));
    use_context_provider(|| Signal::new(SelectionState::default()));
    use_context_provider(|| Signal::new(ContextMenuState::default()));
    use_context_provider(|| Signal::new(UploadState::default()));
    use_context_provider(|| Signal::new(ToastState::default()));

    use_effect(move || {
        spawn(async move {
            let mut eval = dioxus::document::eval(
                r#"
                let theme = window.localStorage.getItem('theme');
                dioxus.send(theme ? theme : 'dark');
            "#,
            );
            if let Ok(val) = eval.recv::<String>().await {
                if val == "light" || val == "dark" {
                    app_state_sig.write()._theme = val;
                }
            }
        });
    });

    rsx! {
        AppStyles {}

        div { class: if app_state_sig.read()._theme == "dark" { "dark w-full h-full" } else { "w-full h-full" },
            Router::<Route> {}
            UploadModal {}
            ToastContainer {}
            BackHandler {}
        }
    }
}

/// Split out from `App()` so this never re-renders after first mount: it
/// reads no signals, so unlike `App()` (which re-runs on every
/// `app_state_sig` write - login/logout, any create/delete/move/rename) it's
/// never marked dirty, and the 51KB embedded CSS is diffed exactly once
/// instead of on every unrelated app_state change.
fn AppStyles() -> Element {
    rsx! {
        style { "{TAILWIND_CSS}" }
        style { "{MAIN_CSS}" }
    }
}
