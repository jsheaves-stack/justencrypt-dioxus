use dioxus::prelude::*;

use crate::{
    components::file_explorer::{IMAGE_FORMATS, VIDEO_FORMATS},
    requests::folder::get_folder,
    ReqwestClient, AppState, api_url,
};
#[cfg(not(target_arch = "wasm32"))]
use base64::engine::{general_purpose, Engine};

#[component]
pub fn PhotoViewerModal(file_name: String, current_path: Vec<String>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let mut scale = use_signal(|| 1.0f64);
    
    // Fetch folder contents once when modal is opened
    let current_path_for_resource = current_path.clone();
    let folder_contents_data = use_resource(use_reactive!(|(current_path_for_resource, reqwest_client_context)| {
        let path_clone = current_path_for_resource.clone();
        let client = reqwest_client_context;
        async move {
            get_folder(client, path_clone).await
        }
    }));

    let media_files = use_memo(move || {
        let mut media = Vec::new();
        if let Some(Ok(files)) = &*folder_contents_data.read() {
            for file in files {
                if file.is_file {
                    if let Some(ext) = &file.file_extension {
                        let ext_upper = ext.to_uppercase();
                        if IMAGE_FORMATS.contains(&ext_upper.as_str()) || VIDEO_FORMATS.contains(&ext_upper.as_str()) {
                            media.push(file.clone());
                        }
                    }
                }
            }
        }
        media
    });

    // We read the actual current file name from the global state to avoid any stale closure captures
    // during fast clicking. If it's missing, fallback to the prop.
    let active_file_name = {
        if let Some((name, _)) = &app_state.read().photo_viewer_modal {
            name.clone()
        } else {
            file_name.clone()
        }
    };

    let (current_file, has_next, has_prev) = {
        let files = media_files.read();
        let current_index = files.iter().position(|f| f.file_name == active_file_name).unwrap_or(0);
        let current_file = files.get(current_index).cloned();
        
        let has_next = current_index + 1 < files.len();
        let has_prev = current_index > 0;
        
        (current_file, has_next, has_prev)
    };

    let is_video = current_file.as_ref().map_or(false, |f| {
        if let Some(ext) = &f.file_extension {
            return VIDEO_FORMATS.contains(&ext.to_uppercase().as_str());
        }
        false
    });

    let path_str = if current_path.is_empty() { String::new() } else { format!("{}/", current_path.join("/")) };
    
    let image_url = current_file.as_ref().map_or(String::new(), |f| {
        format!("{}/file/{}{}", api_url(), path_str, f.file_name)
    });

    let thumbnail_url = current_file.as_ref().map_or(String::new(), |f| {
        format!("{}/thumbnail/{}{}", api_url(), path_str, f.file_name)
    });

    let image_url_for_resource = image_url.clone();
    let full_image_data = use_resource(use_reactive!(|(image_url_for_resource, is_video, reqwest_client_context)| {
        let url = image_url_for_resource.clone();
        let client = reqwest_client_context;
        async move {
            if is_video || url.is_empty() {
                return None;
            }
            let resp = client.read().0.get(&url).send().await;
            match resp {
                Ok(r) => {
                    let mime = r.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("image/jpeg").to_string();
                    match r.bytes().await {
                        Ok(b) => Some((url, mime, b)),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
    }));

    let thumb_url_for_resource = thumbnail_url.clone();
    let thumb_image_data = use_resource(use_reactive!(|(thumb_url_for_resource, is_video, reqwest_client_context)| {
        let url = thumb_url_for_resource.clone();
        let client = reqwest_client_context;
        async move {
            if is_video || url.is_empty() {
                return None;
            }
            let resp = client.read().0.get(&url).send().await;
            match resp {
                Ok(r) => {
                    let mime = r.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("image/jpeg").to_string();
                    match r.bytes().await {
                        Ok(b) => Some((url, mime, b)),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
    }));

    // Data URLs
    let mut thumb_data_url = None;
    if let Some(Some((res_url, _mime, _bytes))) = &*thumb_image_data.read() {
        if res_url == &thumbnail_url {
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
    }

    let mut full_data_url = None;
    if let Some(Some((res_url, _mime, _bytes))) = &*full_image_data.read() {
        if res_url == &image_url {
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
    }

    let display_url = full_data_url.or(thumb_data_url);
    // If the image is not ready, display_url will be None, triggering the loading spinner.
    let is_loading_full = (display_url.is_none() || full_image_data.read().as_ref().map_or(true, |v| v.is_none())) && !is_video;
    
    let transform_style = format!("transform: scale({}); transition: transform 0.2s ease-out;", scale());
    
    let current_file_name_display = current_file.as_ref().map(|f| f.file_name.clone()).unwrap_or_else(|| active_file_name.clone());
    
    rsx! {
        div { 
            class: "fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-black/90 backdrop-blur-md focus:outline-none",
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: move |e| {
                let state = app_state.read();
                if let Some((current_name, path)) = state.photo_viewer_modal.clone() {
                    let files = media_files.read();
                    let current_idx = files.iter().position(|f| f.file_name == current_name).unwrap_or(0);
                    
                    match e.key().to_string().as_str() {
                        "ArrowRight" => {
                            if current_idx + 1 < files.len() {
                                let next_name = files[current_idx + 1].file_name.clone();
                                drop(state); // Drop the read lock before writing
                                scale.set(1.0);
                                app_state.write().photo_viewer_modal = Some((next_name, path));
                            }
                        },
                        "ArrowLeft" => {
                            if current_idx > 0 {
                                let prev_name = files[current_idx - 1].file_name.clone();
                                drop(state);
                                scale.set(1.0);
                                app_state.write().photo_viewer_modal = Some((prev_name, path));
                            }
                        },
                        "Escape" => {
                            drop(state);
                            app_state.write().photo_viewer_modal = None;
                        },
                        _ => {}
                    }
                }
            },
            
            // Header bar
            div { class: "absolute top-0 left-0 right-0 p-4 flex justify-between items-center z-[10000] bg-gradient-to-b from-black/80 to-transparent",
                h2 { class: "text-white text-lg font-semibold drop-shadow-md", "{current_file_name_display}" }
                div { class: "flex items-center gap-4",
                    if is_loading_full {
                        span { class: "text-white/70 text-sm animate-pulse", "Loading full resolution..." }
                    }
                    if !is_video {
                        div { class: "flex items-center bg-black/60 rounded-lg p-1 backdrop-blur-md",
                            button { 
                                class: "p-2 text-white hover:bg-white/20 rounded",
                                onclick: move |_| {
                                    let current = scale();
                                    if current > 0.5 { scale.set(current - 0.25); }
                                },
                                "➖" 
                            }
                            span { class: "text-white text-sm min-w-[3ch] text-center", "{(scale() * 100.0).round()}%" }
                            button { 
                                class: "p-2 text-white hover:bg-white/20 rounded",
                                onclick: move |_| {
                                    let current = scale();
                                    if current < 5.0 { scale.set(current + 0.25); }
                                },
                                "➕" 
                            }
                        }
                    }
                    button {
                        class: "text-white/80 hover:text-white p-2 text-2xl font-bold bg-black/60 rounded-lg hover:bg-red-500/80 backdrop-blur-md transition-colors",
                        onclick: move |_| {
                            app_state.write().photo_viewer_modal = None;
                        },
                        "✕"
                    }
                }
            }
            
            // Previous button
            if has_prev {
                button {
                    class: "absolute left-6 top-1/2 -translate-y-1/2 w-16 h-16 flex items-center justify-center text-3xl leading-none bg-black/50 hover:bg-white/30 rounded-full text-white z-[10000] backdrop-blur-md transition-all border border-white/10 hover:scale-110",
                    onclick: move |e| {
                        e.stop_propagation();
                        let state = app_state.read();
                        if let Some((current_name, path)) = state.photo_viewer_modal.clone() {
                            let files = media_files.read();
                            let current_idx = files.iter().position(|f| f.file_name == current_name).unwrap_or(0);
                            if current_idx > 0 {
                                let prev_name = files[current_idx - 1].file_name.clone();
                                drop(state);
                                scale.set(1.0);
                                app_state.write().photo_viewer_modal = Some((prev_name, path));
                            }
                        }
                    },
                    "◀"
                }
            }

            // Next button
            if has_next {
                button {
                    class: "absolute right-6 top-1/2 -translate-y-1/2 w-16 h-16 flex items-center justify-center text-3xl leading-none bg-black/50 hover:bg-white/30 rounded-full text-white z-[10000] backdrop-blur-md transition-all border border-white/10 hover:scale-110",
                    onclick: move |e| {
                        e.stop_propagation();
                        let state = app_state.read();
                        if let Some((current_name, path)) = state.photo_viewer_modal.clone() {
                            let files = media_files.read();
                            let current_idx = files.iter().position(|f| f.file_name == current_name).unwrap_or(0);
                            if current_idx + 1 < files.len() {
                                let next_name = files[current_idx + 1].file_name.clone();
                                drop(state);
                                scale.set(1.0);
                                app_state.write().photo_viewer_modal = Some((next_name, path));
                            }
                        }
                    },
                    "▶"
                }
            }

            // Image/Video container
            div { 
                class: "w-full h-full flex items-center justify-center overflow-hidden",
                onclick: move |_| {
                    app_state.write().photo_viewer_modal = None;
                },
                
                if is_video {
                    video {
                        class: "max-w-full max-h-full outline-none",
                        src: "{image_url}",
                        controls: true,
                        autoplay: true,
                        onclick: move |e| {
                            e.stop_propagation();
                        }
                    }
                } else if let Some(url) = display_url {
                    img {
                        class: "max-w-full max-h-full object-contain cursor-grab active:cursor-grabbing",
                        style: "{transform_style}",
                        src: "{url}",
                        alt: "{current_file_name_display}",
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