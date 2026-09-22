use dioxus::prelude::*;

use crate::{
    components::{
        file_explorer::is_video_ext,
        ui::{image::CROSSORIGIN, spinner::Spinner},
    },
    media_url,
    util::path_prefix,
    ModalState,
};

/// Fullscreen image/video gallery, opened from a media file's grid card or
/// context menu. `media_files` is the sibling image/video files in the same
/// folder, so the user can step through them with the arrow buttons or
/// arrow keys without leaving the gallery.
#[component]
pub fn Gallery(file_name: String, current_path: Vec<String>, media_files: Vec<String>) -> Element {
    let mut modal_state = use_context::<Signal<ModalState>>();

    let mut scale = use_signal(|| 1.0f64);
    let mut offset_x = use_signal(|| 0.0f64);
    let mut offset_y = use_signal(|| 0.0f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| (0.0f64, 0.0f64));
    // Tracks which full-resolution URL has actually finished loading (via the
    // hidden preload <img> below), so we know when to swap the visible
    // <img> from the thumbnail to the full image. Comparing the URL itself
    // (rather than a plain bool) means navigating to a new file
    // automatically falls back to that file's thumbnail until its own full
    // image loads, with no separate reset step needed.
    let mut full_loaded_url = use_signal(|| None::<String>);

    // We read the actual current file name from the global state to avoid any stale closure captures
    // during fast clicking. If it's missing, fallback to the prop.
    let active_file_name = {
        if let Some(crate::AppModal::Gallery(name, _)) = &modal_state.read().active_modal {
            name.clone()
        } else {
            file_name.clone()
        }
    };

    let (current_file, has_next, has_prev) = {
        let files = &media_files;
        let current_index = files
            .iter()
            .position(|f| f == &active_file_name)
            .unwrap_or(0);
        let current_file = files.get(current_index).cloned();

        let has_next = current_index + 1 < files.len();
        let has_prev = current_index > 0;

        (current_file, has_next, has_prev)
    };

    let is_video = current_file
        .as_ref()
        .and_then(|f| f.split('.').next_back())
        .is_some_and(is_video_ext);

    let path_str = path_prefix(&current_path);

    // media_url() points at the real API origin on wasm (the browser
    // already holds the session cookie) or a local proxy URL on desktop
    // that proxies through the app's own authenticated client - see
    // `media_proxy` in main.rs. Either way these are plain URLs the DOM
    // fetches directly; no Rust-side request needed to display them.
    let image_url = current_file.as_ref().map_or(String::new(), |f| {
        media_url(&format!("/file/{}{}", path_str, f))
    });

    let thumbnail_url = current_file.as_ref().map_or(String::new(), |f| {
        media_url(&format!("/thumbnail/{}{}", path_str, f))
    });

    let image_url_for_preload = image_url.clone();

    let full_ready = full_loaded_url.read().as_deref() == Some(image_url.as_str());

    let display_url = if is_video {
        None
    } else if full_ready {
        Some(image_url.clone())
    } else if !thumbnail_url.is_empty() {
        Some(thumbnail_url.clone())
    } else {
        None
    };

    let is_loading_full = !is_video && !full_ready;

    let transform_style = if is_dragging() {
        format!(
            "transform: translate({}px, {}px) scale({});",
            offset_x(),
            offset_y(),
            scale()
        )
    } else {
        format!(
            "transform: translate({}px, {}px) scale({}); transition: transform 0.2s ease-out;",
            offset_x(),
            offset_y(),
            scale()
        )
    };

    let current_file_name_display = current_file
        .clone()
        .unwrap_or_else(|| active_file_name.clone());

    // Steps `delta` positions through `media_files` from whichever file is
    // currently shown (read fresh from `modal_state` rather than a stale
    // closure capture, so fast repeated clicks/key-presses stay in sync),
    // resetting zoom/pan and updating the active modal. Shared by the
    // keyboard shortcut and both arrow buttons below rather than each
    // re-deriving the same "find current index, bounds-check, jump" logic.
    let media_files_for_nav = media_files.clone();
    let navigate = move |delta: i32| {
        let state = modal_state.read();
        let Some(crate::AppModal::Gallery(current_name, path)) = state.active_modal.clone() else {
            return;
        };
        let current_idx = media_files_for_nav
            .iter()
            .position(|f| f == &current_name)
            .unwrap_or(0);
        let new_idx = current_idx as i32 + delta;
        if new_idx < 0 || new_idx as usize >= media_files_for_nav.len() {
            return;
        }
        let new_name = media_files_for_nav[new_idx as usize].clone();
        drop(state);
        scale.set(1.0);
        offset_x.set(0.0);
        offset_y.set(0.0);
        modal_state.write().active_modal = Some(crate::AppModal::Gallery(new_name, path));
    };

    rsx! {
        div {
            class: "fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-black/90 backdrop-blur-md focus:outline-none animate-in fade-in duration-200",
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: {
                let mut navigate = navigate.clone();
                move |e| {
                    match e.key().to_string().as_str() {
                        "ArrowRight" => navigate(1),
                        "ArrowLeft" => navigate(-1),
                        "Escape" => modal_state.write().close_all_modals(),
                        _ => {}
                    }
                }
            },
            // Click anywhere on this backdrop closes the gallery; every
            // piece of chrome drawn on top of it (header, nav buttons,
            // media itself) stops propagation so only genuine background
            // clicks reach this handler - mirrors the shared `Modal`
            // component's backdrop-click convention.
            onclick: move |_| {
                modal_state.write().close_all_modals();
            },

            // Header bar
            div {
                class: "absolute top-0 left-0 right-0 p-4 flex justify-between items-center z-[10000] bg-gradient-to-b from-black/80 to-transparent",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-white text-lg font-semibold drop-shadow-md truncate max-w-[50%] md:max-w-[70%]",
                    {current_file_name_display.clone()}
                }
                div { class: "flex items-center gap-4",
                    if is_loading_full {
                        span { class: "text-white/70 text-sm animate-pulse",
                            "Loading full resolution..."
                        }
                    }
                    if !is_video {
                        div { class: "flex items-center bg-black/60 rounded-lg p-1 backdrop-blur-md",
                            button {
                                class: "p-2 text-white hover:bg-white/20 rounded",
                                onclick: move |_| {
                                    let current = scale();
                                    if current > 0.5 {
                                        let new_scale = current - 0.25;
                                        scale.set(new_scale);
                                        if new_scale <= 1.0 {
                                            offset_x.set(0.0);
                                            offset_y.set(0.0);
                                        }
                                    }
                                },
                                "➖"
                            }
                            span { class: "text-white text-sm min-w-[3ch] text-center",
                                "{(scale() * 100.0).round()}%"
                            }
                            button {
                                class: "p-2 text-white hover:bg-white/20 rounded",
                                onclick: move |_| {
                                    let current = scale();
                                    if current < 5.0 {
                                        scale.set(current + 0.25);
                                    }
                                },
                                "➕"
                            }
                        }
                    }
                    button {
                        class: "text-white/80 hover:text-white p-2 text-2xl font-bold bg-black/60 rounded-lg hover:bg-red-500/80 backdrop-blur-md transition-colors",
                        onclick: move |_| {
                            modal_state.write().close_all_modals();
                        },
                        "✕"
                    }
                }
            }

            // Previous button
            if has_prev {
                button {
                    class: "absolute left-2 md:left-6 top-1/2 -translate-y-1/2 w-10 h-10 md:w-16 md:h-16 flex items-center justify-center text-2xl md:text-3xl pb-1 bg-black/50 hover:bg-white/30 rounded-full text-white z-[10000] backdrop-blur-md transition border border-white/10 hover:scale-110",
                    onclick: {
                        let mut navigate = navigate.clone();
                        move |e| {
                            e.stop_propagation();
                            navigate(-1);
                        }
                    },
                    span { "◀" }
                }
            }

            // Next button
            if has_next {
                button {
                    class: "absolute right-2 md:right-6 top-1/2 -translate-y-1/2 w-10 h-10 md:w-16 md:h-16 flex items-center justify-center text-2xl md:text-3xl pb-1 pl-1 bg-black/50 hover:bg-white/30 rounded-full text-white z-[10000] backdrop-blur-md transition border border-white/10 hover:scale-110",
                    onclick: {
                        let mut navigate = navigate.clone();
                        move |e| {
                            e.stop_propagation();
                            navigate(1);
                        }
                    },
                    span { "▶" }
                }
            }

            // Image/Video container. No click-to-close handler here - clicks
            // that don't hit the media itself (which stops propagation)
            // bubble straight up to the backdrop's handler above.
            div {
                class: "w-full h-full flex items-center justify-center overflow-hidden",

                // Invisible preload of the full-resolution image, so we know
                // when it's ready to swap in for the thumbnail without
                // fetching its bytes through Rust. Not rendered for video -
                // the visible <video> element below loads that directly.
                if !is_video && !image_url.is_empty() {
                    img {
                        src: "{image_url}",
                        "aria-hidden": "true",
                        style: "position:absolute; width:1px; height:1px; opacity:0; pointer-events:none; overflow:hidden;",
                        onload: move |_| full_loaded_url.set(Some(image_url_for_preload.clone())),
                    }
                }

                if is_video {
                    video {
                        class: "max-w-full max-h-full outline-none",
                        src: image_url,
                        controls: true,
                        autoplay: true,
                        crossorigin: CROSSORIGIN,
                        onclick: move |e| {
                            e.stop_propagation();
                        },
                    }
                } else if let Some(url) = display_url {
                    img {
                        class: "max-w-full max-h-full object-contain cursor-grab active:cursor-grabbing select-none",
                        style: transform_style,
                        src: url,
                        alt: current_file_name_display.clone(),
                        draggable: "false",
                        crossorigin: CROSSORIGIN,
                        onclick: move |e| {
                            e.stop_propagation();
                        },
                        ondragstart: move |e| {
                            e.prevent_default();
                        },
                        onwheel: move |e| {
                            let delta = e.delta().strip_units().y;
                            let current = scale();
                            if delta > 0.0 && current > 0.5 {
                                let new_scale = current - 0.1;
                                scale.set(new_scale);
                                if new_scale <= 1.0 {
                                    offset_x.set(0.0);
                                    offset_y.set(0.0);
                                }
                            } else if delta < 0.0 && current < 5.0 {
                                scale.set(current + 0.1);
                            }
                        },
                        onmousedown: move |e| {
                            e.stop_propagation();
                            if scale() > 1.0 {
                                is_dragging.set(true);
                                drag_start.set((e.client_coordinates().x, e.client_coordinates().y));
                            }
                        },
                        onmousemove: move |e| {
                            if is_dragging() {
                                e.stop_propagation();
                                let dx = e.client_coordinates().x - drag_start().0;
                                let dy = e.client_coordinates().y - drag_start().1;
                                let zoom = scale();
                                // Divide by zoom factor so mouse movement maps 1:1 to image pixels
                                offset_x.set(offset_x() + (dx / zoom));
                                offset_y.set(offset_y() + (dy / zoom));
                                drag_start.set((e.client_coordinates().x, e.client_coordinates().y));
                            }
                        },
                        onmouseup: move |_| {
                            is_dragging.set(false);
                        },
                        onmouseleave: move |_| {
                            is_dragging.set(false);
                        },
                    }
                } else {
                    div { class: "flex flex-col items-center justify-center gap-4",
                        Spinner { class: "h-12 w-12 text-white" }
                        span { class: "text-white", "Loading..." }
                    }
                }
            }
        }
    }
}
