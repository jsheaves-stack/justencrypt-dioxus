use super::{send_and_discard, RequestBuilderExt, RequestError};
use crate::{api_url, components::login_form::LoginForm, ReqwestClient};
use dioxus::signals::Signal;

pub async fn create_session(
    reqwest_client_context: Signal<ReqwestClient>,
    login_form: LoginForm,
) -> Result<(), RequestError> {
    let create_session_url = format!("{}/session", api_url());
    tracing::debug!("Creating session at {}", create_session_url);

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .post(create_session_url)
            .with_credentials_if_wasm()
            .json(&login_form),
    )
    .await;

    match &result {
        Ok(_) => {
            tracing::debug!("Session created successfully");
            crate::persist_session_cookies();
        }
        Err(e) => tracing::error!("Login failed: {:?}", e),
    }

    result
}

pub async fn destroy_session(
    reqwest_client_context: Signal<ReqwestClient>,
) -> Result<(), RequestError> {
    let destroy_session_url = format!("{}/session", api_url());
    tracing::debug!("Destroying session at {}", destroy_session_url);

    let result = send_and_discard(
        reqwest_client_context()
            .0
            .delete(destroy_session_url)
            .with_credentials_if_wasm(),
    )
    .await;

    match &result {
        Ok(_) => tracing::debug!("Session destroyed successfully"),
        Err(e) => tracing::error!("Logout failed: {:?}", e),
    }

    result
}
