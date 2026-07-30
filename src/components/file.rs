use crate::{
    api_url,
    components::{
        file_explorer::{IMAGE_FORMATS, VIDEO_FORMATS},
        ui::{
            card::{Card, CardContent, CardFooter},
            context_menu::{ContextMenuState, ContextMenuTarget},
            image::Image,
            label::Label,
        },
    },
    models::{File as ModelFile, FolderContent},
    AppState,
};
use dioxus::prelude::*;

#[component]
pub fn File(
    file: ModelFile,
    current_path: Vec<String>,
    index: usize,
    on_select: EventHandler<(usize, bool, bool)>,
) -> Element {
    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let mut app_state = use_context::<Signal<AppState>>();

    let file_clone = file.clone();
    let current_path_clone = current_path.clone();
    let current_path_clone_click = current_path.clone();

    let is_image = file
        .file_extension
        .as_deref()
        .map(|ext| IMAGE_FORMATS.iter().any(|f| ext.eq_ignore_ascii_case(f)))
        .unwrap_or(false);

    let is_video = file
        .file_extension
        .as_deref()
        .map(|ext| VIDEO_FORMATS.iter().any(|f| ext.eq_ignore_ascii_case(f)))
        .unwrap_or(false);

    let is_media = is_image || is_video;

    let thumbnail_url = match is_image {
        true => {
            format!(
                "{}/thumbnail/{}{}",
                api_url(),
                if current_path.is_empty() {
                    String::new()
                } else {
                    format!("{}/", current_path.join("/"))
                },
                file.file_name
            )
        }
        false => String::new(),
    };

    let display_name = file.file_name.clone();
    let display_ext = file.file_extension.as_deref().unwrap_or("FILE").to_string();

    let mut press_task = use_signal(|| None);
    let mut was_long_pressed = use_signal(|| false);
    let mut is_pressed = use_signal(|| false);

    rsx! {
        div {
            class: "flex justify-center h-40 sm:h-48 md:h-56 w-full cursor-pointer relative group/file",
            onmousedown: move |_| {
                is_pressed.set(true);
                let handle = spawn(async move {
                    #[cfg(target_arch = "wasm32")]
                    gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;

                    #[cfg(not(target_arch = "wasm32"))]
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    if *is_pressed.read() {
                        was_long_pressed.set(true);
                        on_select.call((index, false, false));
                    }
                });
                press_task.set(Some(handle));
            },
            onmouseup: move |_| {
                is_pressed.set(false);
                if let Some(handle) = press_task.take() {
                    handle.cancel();
                }
            },
            onmouseleave: move |_| {
                is_pressed.set(false);
                if let Some(handle) = press_task.take() {
                    handle.cancel();
                }
            },
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                tracing::debug!("File right-clicked at {}, {}", coords.x, coords.y);
                let mut state = context_menu.write();
                state.target = ContextMenuTarget::Item(
                    FolderContent::File(file_clone.clone()),
                    current_path_clone.clone(),
                );
                state.position = (coords.x, coords.y);
            },
            Card {
                class: format!(
                    "w-full h-full grid grid-rows-[1fr,auto] active:scale-95 transition-all duration-300 hover:shadow-lg group select-none {}",
                    if app_state.read().selected_files.contains(&(display_name.clone(), true)) {
                        "ring-2 ring-primary bg-primary/5"
                    } else {
                        ""
                    },
                ),
                onclick: move |e: Event<MouseData>| {
                    if *was_long_pressed.read() {
                        was_long_pressed.set(false);
                        e.stop_propagation();
                        return;
                    }

                    let has_selection = !app_state.read().selected_files.is_empty();

                    if e.modifiers().contains(Modifiers::SHIFT)
                        || e.modifiers().contains(Modifiers::CONTROL)
                    {
                        e.stop_propagation();
                        on_select
                            .call((
                                index,
                                e.modifiers().contains(Modifiers::SHIFT),
                                e.modifiers().contains(Modifiers::CONTROL),
                            ));
                    } else if has_selection {
                        e.stop_propagation();
                        on_select.call((index, false, false));
                    } else if is_media {
                        e.stop_propagation();
                        app_state.write().active_modal = Some(
                            crate::AppModal::PhotoViewer(
                                display_name.clone(),
                                current_path_clone_click.clone(),
                            ),
                        );
                    }
                },
                CardContent { class: "p-4 flex flex-col justify-center items-center h-full w-full overflow-hidden",
                    if is_image {
                        Image {
                            class: "object-contain w-min h-full align-center rounded-md transition-transform duration-300 group-hover:scale-105 group-hover:shadow-md",
                            src: thumbnail_url,
                        }
                    } else if is_video {
                        div { class: "flex flex-col items-center justify-center w-16 h-20 sm:w-20 sm:h-24 bg-purple-500/10 rounded-xl border border-purple-500/20 text-purple-700 dark:text-purple-400/90 shadow-sm backdrop-blur-sm relative overflow-hidden group-hover:scale-110 group-hover:bg-purple-500/20 group-hover:shadow-[0_0_15px_rgba(168,85,247,0.2)] transition-all duration-300",
                            div { class: "absolute top-0 left-0 w-full h-[2px] bg-gradient-to-r from-purple-400/60 to-indigo-500/60" }
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "28",
                                height: "28",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                "stroke-width": "2",
                                "stroke-linecap": "round",
                                "stroke-linejoin": "round",
                                class: "mb-2 opacity-90 drop-shadow-sm",
                                path { d: "m15 10 4.553-2.276A1 1 0 0 1 21 8.618v6.764a1 1 0 0 1-1.447.894L15 14v-4z" }
                                rect {
                                    x: "3",
                                    y: "6",
                                    width: "12",
                                    height: "12",
                                    rx: "2",
                                    ry: "2",
                                }
                            }
                        }
                    } else {
                        div { class: "flex flex-col items-center justify-center w-16 h-20 sm:w-20 sm:h-24 bg-slate-500/10 rounded-xl border border-slate-500/20 text-slate-700 dark:text-slate-300 font-bold text-base sm:text-lg uppercase tracking-wider shadow-sm backdrop-blur-sm relative overflow-hidden group-hover:scale-110 group-hover:bg-slate-500/20 group-hover:shadow-[0_0_15px_rgba(100,116,139,0.2)] transition-all duration-300",
                            div { class: "absolute top-0 left-0 w-full h-[2px] bg-gradient-to-r from-slate-400/60 to-slate-300/60" }
                            {display_ext}
                        }
                    }
                }
                CardFooter { class: "overflow-hidden pb-4",
                    div { class: "text-sm w-full h-4 mt-2",
                        Label {
                            html_for: "{file.file_name}",
                            tooltip: "{file.file_name}",
                            class: "w-full mx-auto",
                            span { class: "block w-[90%] mx-auto overflow-hidden truncate text-center font-medium text-foreground/90 group-hover:text-foreground transition-colors h-6",
                                "{file.file_name}"
                            }
                        }
                    }
                }
            }
            div {
                class: format!(
                    "absolute top-2 right-2 z-10 transition-opacity duration-200 {}",
                    if app_state.read().selected_files.contains(&(file.file_name.clone(), true)) {
                        "opacity-100"
                    } else {
                        "opacity-0"
                    },
                ),
                input {
                    r#type: "checkbox",
                    class: "w-5 h-5 rounded border-gray-300 text-primary focus:ring-primary cursor-pointer",
                    checked: app_state.read().selected_files.contains(&(file.file_name.clone(), true)),
                    onclick: move |e| {
                        e.stop_propagation();
                        on_select.call((index, false, false));
                    },
                }
            }
        }
    }
}
