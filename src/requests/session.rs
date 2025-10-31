use crate::{ReqwestClient, API_URL};
use dioxus::{html::FormData, prelude::Event, signals::Signal};
use std::collections::HashMap;

pub async fn create_session(
    event: Event<FormData>,
    reqwest_client_context: Signal<ReqwestClient>,
) -> Result<(), ()> {
    let username = event
        .data
        .values()
        .get("username")
        .and_then(|v| v.get(0))
        .unwrap()
        .clone();

    let password = event
        .data
        .values()
        .get("password")
        .and_then(|v| v.get(0))
        .unwrap()
        .clone();

    let mut login_form = HashMap::new();

    login_form.insert("username", username);
    login_form.insert("password", password);

    let create_session_url = format!("{}/session/create", API_URL);

    #[cfg(target_arch = "wasm32")]
    let resp = reqwest_client_context()
        .0
        .post(create_session_url)
        .fetch_credentials_include()
        .json(&login_form)
        .send()
        .await;

    #[cfg(not(target_arch = "wasm32"))]
    let resp = reqwest_client_context()
        .0
        .post(create_session_url)
        .json(&login_form)
        .send()
        .await;

    match resp {
        Ok(_) => Ok(()),
        Err(_err) => {
            tracing::error!("Login failed: {}", _err);
            Err(())
        }
    }
}
