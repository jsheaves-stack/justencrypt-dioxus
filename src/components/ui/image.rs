use dioxus::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::{requests::RequestBuilderExt, ReqwestClient};
#[cfg(not(target_arch = "wasm32"))]
use base64::engine::{general_purpose, Engine};
#[cfg(not(target_arch = "wasm32"))]
use tokio::task;

#[component]
pub fn Image(src: String, class: Option<String>, alt: Option<String>) -> Element {
    let class = class.unwrap_or_default();
    let alt = alt.unwrap_or_default();

    #[cfg(target_arch = "wasm32")]
    {
        rsx! {
            img {
                src: "{src}",
                class: "{class}",
                alt: "{alt}",
                crossorigin: "use-credentials",
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let client = use_context::<Signal<ReqwestClient>>();

        let image_data = use_resource(use_reactive(&src, move |url| async move {
            let resp = client
                .read()
                .0
                .get(&url)
                .with_credentials_if_wasm()
                .send()
                .await;

            match resp {
                Ok(r) => {
                    let mime = r
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("image/jpeg")
                        .to_string();

                    match r.bytes().await {
                        Ok(b) => {
                            let mime_clone = mime.clone();
                            let b64 =
                                task::spawn_blocking(move || general_purpose::STANDARD.encode(b))
                                    .await
                                    .unwrap_or_default();
                            Some(format!("data:{};base64,{}", mime_clone, b64))
                        }
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }));

        let image_resource = image_data.read().clone();

        match image_resource {
            Some(Some(data_url)) => {
                rsx! {
                    img { src: data_url, class, alt }
                }
            }
            Some(None) => rsx! {
                div { class: "{class} bg-gray-200 animate-pulse", "Error" }
            },
            None => rsx! {
                div { class: "{class} bg-gray-700 animate-pulse" }
            },
        }
    }
}
