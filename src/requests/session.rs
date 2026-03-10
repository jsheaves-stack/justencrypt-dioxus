use crate::{components::login_form::LoginForm, ReqwestClient, api_url};
use dioxus::signals::Signal;
use super::RequestBuilderExt;

pub async fn create_session(
    reqwest_client_context: Signal<ReqwestClient>,
    login_form: LoginForm,
) -> Result<(), ()> {
    let create_session_url = format!("{}/session/create", api_url());

    let resp = reqwest_client_context()
        .0
        .post(create_session_url)
        .with_credentials_if_wasm()
        .json(&login_form)
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(_) => Err(()),
        Err(err) => {
            tracing::error!("Login failed: {}", err);
            Err(())
        }
    }
}

pub async fn destroy_session(reqwest_client_context: Signal<ReqwestClient>) -> Result<(), ()> {
    let destroy_session_url = format!("{}/session/destroy", api_url());

    let resp = reqwest_client_context()
        .0
        .post(destroy_session_url)
        .with_credentials_if_wasm()
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(_) => Err(()),
        Err(err) => {
            tracing::error!("Logout failed: {}", err);
            Err(())
        }
    }
}
