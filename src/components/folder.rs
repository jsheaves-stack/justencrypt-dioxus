use dioxus::prelude::*;

use crate::{components::file_explorer::FolderItem, Route};

#[component]
pub fn Folder(file: FolderItem, current_path: Vec<String>) -> Element {
    let router = router();

    rsx! {
        div {
            class: "w-40 h-40 overflow-hidden cursor-pointer rounded-base border-2 border-black bg-main font-base shadow-base",
            onclick: move |_| {
                let mut new_path = current_path.clone();
                new_path.push(file.file_name.clone());
                router
                    .push(Route::FileExplorer {
                        path: new_path,
                    });
            },
            img { class: "w-full h-24", src: "" }
            div { class: "text-sm w-full border-t-2 border-black p-2 truncate text-wrap text-ellipsis overflow-hidden",
                "{file.file_name}"
            }
        }
    }
}
