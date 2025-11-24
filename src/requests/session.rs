use crate::{components::login_form::LoginForm, ReqwestClient, API_URL};
use dioxus::signals::Signal;

pub async fn create_session(
    reqwest_client_context: Signal<ReqwestClient>,
    login_form: LoginForm,
) -> Result<(), ()> {
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

pub async fn destroy_session(reqwest_client_context: Signal<ReqwestClient>) -> Result<(), ()> {
    let destroy_session_url = format!("{}/session/destroy", API_URL);

    #[cfg(target_arch = "wasm32")]
    let resp = reqwest_client_context()
        .0
        .post(destroy_session_url)
        .fetch_credentials_include()
        .send()
        .await;

    #[cfg(not(target_arch = "wasm32"))]
    let resp = reqwest_client_context()
        .0
        .post(destroy_session_url)
        .send()
        .await;

    match resp {
        Ok(_) => Ok(()),
        Err(_err) => {
            tracing::error!("Logout failed: {}", _err);
            Err(())
        }
    }
}
