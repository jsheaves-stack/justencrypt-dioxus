use crate::{
    components::{
        file_explorer::{FolderItem, IMAGE_FORMATS},
        ui::{
            card::{Card, CardContent, CardFooter},
            label::Label,
        },
    },
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
        div { class: "flex justify-center p-4 h-64 w-64",
            Card { class: "w-full h-full grid grid-rows-[1fr,2em]",
                CardContent { class: "p-4! pt-4 grid justify-items-center",
                    img {
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
