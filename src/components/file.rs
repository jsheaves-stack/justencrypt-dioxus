use crate::{
    api_url,
    components::{
        file_explorer::{FolderItem, IMAGE_FORMATS, VIDEO_FORMATS},
        ui::{
            card::{Card, CardContent, CardFooter},
            context_menu::{ContextMenuState, ContextMenuTarget},
            image::Image,
            label::Label,
        },
    },
    AppState,
};
use dioxus::prelude::*;

#[component]
pub fn File(file: FolderItem, current_path: Vec<String>) -> Element {
    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let mut app_state = use_context::<Signal<AppState>>();

    let file_clone = file.clone();
    let current_path_clone = current_path.clone();
    let current_path_clone_click = current_path.clone();

    let is_image = match file.file_extension.as_deref() {
        Some(ext) => {
            let ext_upper = ext.to_uppercase();
            IMAGE_FORMATS.contains(&ext_upper.as_str())
        }
        _ => false,
    };

    let is_video = match file.file_extension.as_deref() {
        Some(ext) => {
            let ext_upper = ext.to_uppercase();
            VIDEO_FORMATS.contains(&ext_upper.as_str())
        }
        _ => false,
    };

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

    rsx! {
        div {
            class: "flex justify-center p-2 sm:p-4 h-48 sm:h-64 w-[10rem] md:w-[18rem] cursor-pointer",
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                tracing::debug!("File right-clicked at {}, {}", coords.x, coords.y);
                let mut state = context_menu.write();
                state.target = ContextMenuTarget::File(file_clone.clone(), current_path_clone.clone());
                state.position = (coords.x, coords.y);
            },
            Card {
                class: if is_media { "w-full h-full grid grid-rows-[1fr,2em] cursor-pointer" } else { "w-full h-full grid grid-rows-[1fr,2em]" },
                onclick: move |_| {
                    if is_media {
                        app_state.write().photo_viewer_modal = Some((
                            display_name.clone(),
                            current_path_clone_click.clone(),
                        ));
                    }
                },
                CardContent { class: "p-4! pt-4 flex flex-col justify-center items-center h-full w-full overflow-hidden",
                    if is_image {
                        Image {
                            class: "object-contain w-min h-full align-center",
                            src: "{thumbnail_url}",
                        }
                    } else if is_video {
                        div { class: "flex flex-col items-center justify-center w-3/4 h-3/4 bg-secondary/20 rounded-xl border-2 border-border text-muted-foreground shadow-inner",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "48",
                                height: "48",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                "stroke-width": "2",
                                "stroke-linecap": "round",
                                "stroke-linejoin": "round",
                                class: "mb-2",
                                path { d: "m15 10 4.553-2.276A1 1 0 0 1 21 8.618v6.764a1 1 0 0 1-1.447.894L15 14v-4z" }
                                rect { x: "3", y: "6", width: "12", height: "12", rx: "2", ry: "2" }
                            }
                            span { class: "font-bold text-xl uppercase tracking-wider", "VIDEO" }
                        }
                    } else {
                        div { class: "flex items-center justify-center w-3/4 h-3/4 bg-secondary/20 rounded-xl border-2 border-border text-muted-foreground font-bold text-3xl uppercase tracking-wider shadow-inner",
                            "{file.file_extension.as_deref().unwrap_or(\"FILE\")}"
                        }
                    }
                }
                CardFooter {
                    div { class: "text-sm w-full h-8",
                        Label {
                            html_for: "{file.file_name}",
                            tooltip: "{file.file_name}",
                            class: "grid grid-cols-1 w-full mx-auto",
                            span { class: "truncate min-w-0 text-center h-6", "{file.file_name}" }
                        }
                    }
                }
            }
        }
    }
}
