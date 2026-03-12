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

    let is_media = match file.file_extension.as_deref() {
        Some(ext) => {
            let ext_upper = ext.to_uppercase();
            IMAGE_FORMATS.contains(&ext_upper.as_str())
                || VIDEO_FORMATS.contains(&ext_upper.as_str())
        }
        _ => false,
    };

    let thumbnail_url = match is_media {
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
            class: "flex justify-center p-4 h-64 w-64",
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                tracing::info!("File right-clicked at {}, {}", coords.x, coords.y);
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
                    if is_media {
                        Image {
                            class: "object-contain w-min h-full align-center",
                            src: "{thumbnail_url}",
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
                            class: "grid grid-cols-1 w-32 mx-auto",
                            span { class: "truncate min-w-0 text-center h-6", "{file.file_name}" }
                        }
                    }
                }
            }
        }
    }
}
