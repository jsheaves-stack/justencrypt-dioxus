use dioxus::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::{requests::RequestBuilderExt, ReqwestClient};
#[cfg(not(target_arch = "wasm32"))]
use base64::engine::{general_purpose, Engine};

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
            let resp = client.read().0.get(&url).with_credentials_if_wasm().send().await;

            match resp {
                Ok(r) => {
                    let mime = r
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("image/jpeg")
                        .to_string();

                    match r.bytes().await {
                        Ok(b) => Some((mime, b)),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }));

        let image_resource = image_data.read().clone();

        match image_resource {
            Some(Some((mime, bytes))) => {
                let b64 = general_purpose::STANDARD.encode(bytes);
                let data_url = format!("data:{};base64,{}", mime, b64);

                rsx! {
                    img {
                        src: "{data_url}",
                        class: "{class}",
                        alt: "{alt}",
                    }
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
