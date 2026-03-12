use dioxus::prelude::*;

use crate::{
    components::{
        file_explorer::FolderItem,
        ui::{
            card::{Card, CardContent, CardFooter},
            context_menu::{ContextMenuState, ContextMenuTarget},
        },
    },
    Route,
};

static FOLDER_ICO: Asset = asset!("/assets/icons/folder-filled.svg");

#[component]
pub fn Folder(file: FolderItem, current_path: Vec<String>) -> Element {
    let router = router();
    let new_file = file.clone();

    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let file_clone = file.clone();
    let current_path_clone = current_path.clone();

    rsx! {
        div {
            class: "flex justify-center p-4 h-64 w-64 cursor-pointer",
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                tracing::info!("Folder right-clicked at {}, {}", coords.x, coords.y);
                let mut state = context_menu.write();
                state.target = ContextMenuTarget::Folder(file_clone.clone(), current_path_clone.clone());
                state.position = (coords.x, coords.y);
            },
            Card {
                class: "w-full h-full grid grid-rows-[1fr,2em]",
                onclick: move |_| {
                    let mut new_path = current_path.clone();
                    new_path.push(new_file.file_name.clone());
                    router
                        .push(Route::FileExplorer {
                            path: new_path,
                        });
                },
                CardContent {
                    img {
                        class: "object-contain w-1/2 h-full m-auto",
                        src: FOLDER_ICO,
                    }
                }
                CardFooter {
                    div { class: "text-sm w-full h-4",
                        span { class: "block w-[90%] overflow-hidden truncate text-ellipsis",
                            "{file.file_name}"
                        }
                    }
                }
            }
        }
    }
}
