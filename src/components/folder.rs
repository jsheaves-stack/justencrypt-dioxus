use dioxus::prelude::*;

use crate::{
    components::ui::{
        card::{Card, CardContent, CardFooter},
        context_menu::{ContextMenuState, ContextMenuTarget},
    },
    models::{Folder as ModelFolder, FolderContent},
    Route,
};

#[component]
pub fn Folder(file: ModelFolder, current_path: Vec<String>) -> Element {
    let router = router();
    let new_file = file.clone();

    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let file_clone = file.clone();
    let current_path_clone = current_path.clone();

    rsx! {
        div {
            class: "flex justify-center h-40 sm:h-48 md:h-56 w-full cursor-pointer",
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                tracing::debug!("Folder right-clicked at {}, {}", coords.x, coords.y);
                let mut state = context_menu.write();
                state.target = ContextMenuTarget::Item(
                    FolderContent::Folder(file_clone.clone()),
                    current_path_clone.clone(),
                );
                state.position = (coords.x, coords.y);
            },
            Card {
                class: "w-full h-full grid grid-rows-[1fr,auto] active:scale-95 transition-all duration-300 hover:shadow-lg group",
                onclick: move |_| {
                    let mut new_path = current_path.clone();
                    new_path.push(new_file.file_name.clone());
                    router
                        .push(Route::FileExplorer {
                            path: new_path,
                        });
                },
                CardContent { class: "flex flex-col items-center justify-center p-0 pt-6",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 24 24",
                        class: "w-20 h-20 sm:w-24 sm:h-24 drop-shadow-sm group-hover:drop-shadow-md group-hover:scale-110 transition-all duration-300",
                        defs {
                            linearGradient {
                                id: "folderGrad",
                                x1: "0%",
                                y1: "0%",
                                x2: "0%",
                                y2: "100%",
                                stop {
                                    offset: "0%",
                                    "stop-color": "#2563eb",
                                    "stop-opacity": "0.8",
                                }
                                stop {
                                    offset: "100%",
                                    "stop-color": "#1d4ed8",
                                    "stop-opacity": "0.8",
                                }
                            }
                        }
                        path {
                            d: "M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z",
                            fill: "url(#folderGrad)",
                        }
                    }
                }
                CardFooter { class: "overflow-hidden pb-4",
                    div { class: "text-sm w-full h-4 flex justify-center mt-2",
                        span { class: "block w-[90%] overflow-hidden truncate text-center font-medium text-foreground/90 group-hover:text-foreground transition-colors h-6",
                            "{file.file_name}"
                        }
                    }
                }
            }
        }
    }
}
