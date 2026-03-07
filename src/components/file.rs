use crate::{
    components::{
        file_explorer::{FolderItem, IMAGE_FORMATS},
        ui::{
            card::{Card, CardContent, CardFooter},
            context_menu::{ContextMenuState, ContextMenuTarget},
            image::Image,
            label::Label,
        },
    },
    API_URL, AppState,
};
use dioxus::prelude::*;

#[component]
pub fn File(file: FolderItem, current_path: Vec<String>) -> Element {
    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let mut app_state = use_context::<Signal<AppState>>();
    
    let file_clone = file.clone();
    let current_path_clone = current_path.clone();

    let (image_url, thumbnail_url) = match file.file_extension.as_deref() {
        Some(ext) if IMAGE_FORMATS.contains(&ext.to_uppercase().as_str()) => (
            format!(
                "{}/file/{}{}",
                API_URL,
                if current_path.is_empty() { String::new() } else { format!("{}/", current_path.join("/")) },
                file.file_name
            ),
            format!(
                "{}/thumbnail/{}{}",
                API_URL,
                if current_path.is_empty() { String::new() } else { format!("{}/", current_path.join("/")) },
                file.file_name
            ),
        ),
        _ => (String::new(), String::new()),
    };

    let is_image = !image_url.is_empty();
    let image_url_clone = image_url.clone();
    let thumbnail_url_clone = thumbnail_url.clone();
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
                class: if is_image { "w-full h-full grid grid-rows-[1fr,2em] cursor-pointer" } else { "w-full h-full grid grid-rows-[1fr,2em]" },
                onclick: move |_| {
                    if is_image {
                        app_state.write().photo_viewer_modal = Some((
                            display_name.clone(),
                            thumbnail_url_clone.clone(),
                            image_url_clone.clone()
                        ));
                    }
                },
                CardContent { class: "p-4! pt-4 grid justify-items-center",
                    Image {
                        class: "object-contain w-min h-full align-center",
                        src: "{thumbnail_url}",
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
