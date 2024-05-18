#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use reqwest::{cookie::Jar, Client};
use tracing::Level;

mod components; // Import the module containing the LoginForm component

use crate::components::login_form::Login;

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},

    #[route("/login")]
    Login{},
}
#[derive(Clone)]
struct ReqwestClient(Client);

fn main() {
    // Init logger
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");

    launch(App);
}

fn App() -> Element {
    let jar = Arc::new(Jar::default());

    let client = Client::builder()
        .cookie_provider(jar.clone())
        .build()
        .unwrap();

    use_context_provider(|| Signal::new(ReqwestClient(client)));

    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {}
    }
}
