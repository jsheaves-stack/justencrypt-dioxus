use dioxus::prelude::*;

use crate::{
    components::ui::{
        card::{CardContent, CardFooter},
        grid_item::SelectableGridItem,
        label::Label,
    },
    models::{Folder as ModelFolder, FolderContent},
    Route,
};

#[component]
pub fn Folder(file: ModelFolder, current_path: Vec<String>, index: usize) -> Element {
    let router = router();
    let file_name = file.file_name.clone();
    let target_path = current_path.clone();

    rsx! {
        SelectableGridItem {
            content: FolderContent::Folder(file.clone()),
            current_path: current_path.clone(),
            index,
            on_open: move |_| {
                let mut new_path = target_path.clone();
                new_path.push(file_name.clone());
                router.push(Route::FileExplorer { path: new_path });
            },
            CardContent { class: "flex flex-col items-center justify-center p-0 pt-6",
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    class: "w-20 h-20 sm:w-24 sm:h-24 drop-shadow-sm group-hover:drop-shadow-md group-hover:scale-110 transition duration-300",
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
    }
}
