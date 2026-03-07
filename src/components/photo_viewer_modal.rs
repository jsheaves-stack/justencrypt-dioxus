use dioxus::prelude::*;

use crate::{
    ReqwestClient, AppState,
};
#[cfg(not(target_arch = "wasm32"))]
use base64::engine::{general_purpose, Engine};

#[component]
pub fn PhotoViewerModal(file_name: String, thumbnail_url: String, image_url: String) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut scale = use_signal(|| 1.0f64);
    
    // Fetch full size image
    let full_image_data = use_resource(use_reactive(&image_url, move |url| async move {
        let resp = reqwest_client_context.read().0.get(&url).send().await;
        
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

    let full_image_resource = full_image_data.read().clone();
    
    // Also fetch thumbnail so it can be shown immediately if it's cached, or loaded while full image loads
    let thumb_image_data = use_resource(use_reactive(&thumbnail_url, move |url| async move {
        let resp = reqwest_client_context.read().0.get(&url).send().await;
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

    let thumb_image_resource = thumb_image_data.read().clone();

    // Data URLs
    let mut thumb_data_url = None;
    if let Some(Some((_mime, _bytes))) = thumb_image_resource {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let b64 = general_purpose::STANDARD.encode(_bytes);
            thumb_data_url = Some(format!("data:{};base64,{}", _mime, b64));
        }
        #[cfg(target_arch = "wasm32")]
        {
            thumb_data_url = Some(thumbnail_url.clone());
        }
    }

    let mut full_data_url = None;
    if let Some(Some((_mime, _bytes))) = full_image_resource {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let b64 = general_purpose::STANDARD.encode(_bytes);
            full_data_url = Some(format!("data:{};base64,{}", _mime, b64));
        }
        #[cfg(target_arch = "wasm32")]
        {
            full_data_url = Some(image_url.clone());
        }
    }

    let display_url = full_data_url.or(thumb_data_url);
    let is_loading_full = full_image_data.read().is_none();

    let transform_style = format!("transform: scale({}); transition: transform 0.2s ease-out;", scale());

    rsx! {
        div { 
            class: "fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-black/90 backdrop-blur-md",
            
            // Header bar
            div { class: "absolute top-0 left-0 right-0 p-4 flex justify-between items-center z-10 bg-gradient-to-b from-black/60 to-transparent",
                h2 { class: "text-white text-lg font-semibold drop-shadow-md", "{file_name}" }
                div { class: "flex items-center gap-4",
                    if is_loading_full {
                        span { class: "text-white/70 text-sm animate-pulse", "Loading full resolution..." }
                    }
                    div { class: "flex items-center bg-black/40 rounded-lg p-1 backdrop-blur-sm",
                        button { 
                            class: "p-2 text-white hover:bg-white/20 rounded",
                            onclick: move |_| {
                                let current = scale();
                                if current > 0.5 { scale.set(current - 0.25); }
                            },
                            "➖" 
                        }
                        span { class: "text-white text-sm min-w-[3ch] text-center", "{scale() * 100.0}%" }
                        button { 
                            class: "p-2 text-white hover:bg-white/20 rounded",
                            onclick: move |_| {
                                let current = scale();
                                if current < 5.0 { scale.set(current + 0.25); }
                            },
                            "➕" 
                        }
                    }
                    button {
                        class: "text-white/80 hover:text-white p-2 text-xl font-bold bg-black/40 rounded-lg hover:bg-white/20 backdrop-blur-sm",
                        onclick: move |_| {
                            app_state.write().photo_viewer_modal = None;
                        },
                        "✕"
                    }
                }
            }
            
            // Image container
            div { 
                class: "w-full h-full flex items-center justify-center overflow-hidden",
                // Close when clicking the background
                onclick: move |_| {
                    app_state.write().photo_viewer_modal = None;
                },
                
                if let Some(url) = display_url {
                    img {
                        class: "max-w-full max-h-full object-contain cursor-grab active:cursor-grabbing",
                        style: "{transform_style}",
                        src: "{url}",
                        alt: "{file_name}",
                        // Prevent closing when clicking the image itself
                        onclick: move |e| {
                            e.stop_propagation();
                        },
                        onwheel: move |e| {
                            let delta = e.delta().strip_units().y;
                            let current = scale();
                            if delta > 0.0 && current > 0.5 {
                                scale.set(current - 0.1);
                            } else if delta < 0.0 && current < 5.0 {
                                scale.set(current + 0.1);
                            }
                        }
                    }
                } else {
                    div { class: "flex flex-col items-center justify-center",
                        div { class: "w-12 h-12 border-4 border-white/20 border-t-white rounded-full animate-spin mb-4" }
                        span { class: "text-white", "Loading..." }
                    }
                }
            }
        }
    }
}