use dioxus::prelude::*;

use crate::{
    components::{
        file_explorer::FolderItem,
        ui::card::{Card, CardContent, CardFooter},
    },
    Route,
};

static FOLDER_ICO: Asset = asset!("/assets/icons/folder-filled.svg");

#[component]
pub fn Folder(file: FolderItem, current_path: Vec<String>) -> Element {
    let router = router();
    let new_file = file.clone();

    rsx! {
        div { class: "flex justify-center p-4 h-64 w-64 cursor-pointer",
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
                        class: "object-contain w-min h-full align-center",
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
