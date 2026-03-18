use super::RequestBuilderExt;
use crate::{api_url, components::login_form::LoginForm, ReqwestClient};
use dioxus::signals::Signal;

pub async fn create_session(
    reqwest_client_context: Signal<ReqwestClient>,
    login_form: LoginForm,
) -> Result<(), ()> {
    let create_session_url = format!("{}/session/create", api_url());
    tracing::debug!("Creating session at {}", create_session_url);

    let resp = reqwest_client_context()
        .0
        .post(create_session_url)
        .with_credentials_if_wasm()
        .json(&login_form)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::debug!("Session created successfully");
            Ok(())
        }
        Ok(r) => {
            tracing::error!("Login failed with status: {}", r.status());
            Err(())
        }
        Err(err) => {
            tracing::error!("Login request failed: {}", err);
            Err(())
        }
    }
}

pub async fn destroy_session(reqwest_client_context: Signal<ReqwestClient>) -> Result<(), ()> {
    let destroy_session_url = format!("{}/session/destroy", api_url());
    tracing::debug!("Destroying session at {}", destroy_session_url);

    let resp = reqwest_client_context()
        .0
        .post(destroy_session_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            tracing::debug!("Session destroyed successfully");
            Ok(())
        }
        Ok(r) => {
            tracing::error!("Logout failed with status: {}", r.status());
            Err(())
        }
        Err(err) => {
            tracing::error!("Logout request failed: {}", err);
            Err(())
        }
    }
}
