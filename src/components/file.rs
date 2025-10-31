use crate::{
    components::file_explorer::{FolderItem, IMAGE_FORMATS},
    API_URL,
};
use dioxus::prelude::*;

#[component]
pub fn File(file: FolderItem, current_path: Vec<String>) -> Element {
    let (_image_url, thumbnail_url) = match file.file_extension.as_deref() {
        Some(ext) if IMAGE_FORMATS.contains(&ext.to_uppercase().as_str()) => (
            format!(
                "{}/file/{}{}",
                API_URL,
                current_path.join("/"),
                file.file_name
            ),
            format!(
                "{}/thumbnail/{}/{}",
                API_URL,
                current_path.join("/"),
                file.file_name
            ),
        ),
        _ => (String::new(), String::new()),
    };

    rsx! {
        div { class: "w-40 h-40 overflow-hidden rounded-base border-2 border-black bg-main font-base shadow-base",
            img { class: "w-full h-24", src: "{thumbnail_url}" }
            div { class: "text-sm w-full border-t-2 border-black p-2 truncate text-wrap text-ellipsis overflow-hidden",
                "{file.file_name}"
            }
        }
    }
}
