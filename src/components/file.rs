use crate::{
    components::{
        file_explorer::{is_image_ext, is_video_ext},
        ui::{
            card::{CardContent, CardFooter},
            grid_item::SelectableGridItem,
            image::Image,
            label::Label,
        },
    },
    media_url,
    models::{File as ModelFile, FolderContent},
    util::path_prefix,
    ModalState,
};
use dioxus::prelude::*;

#[component]
pub fn File(file: ModelFile, current_path: Vec<String>, index: usize) -> Element {
    let mut modal_state = use_context::<Signal<ModalState>>();

    let is_image = file
        .file_extension
        .as_deref()
        .map(is_image_ext)
        .unwrap_or(false);

    let is_video = file
        .file_extension
        .as_deref()
        .map(is_video_ext)
        .unwrap_or(false);

    let is_media = is_image || is_video;

    let thumbnail_url = match is_image {
        true => media_url(&format!(
            "/thumbnail/{}{}",
            path_prefix(&current_path),
            file.file_name
        )),
        false => String::new(),
    };

    let display_name = file.file_name.clone();
    let display_ext = file.file_extension.as_deref().unwrap_or("FILE").to_string();
    let gallery_path = current_path.clone();

    rsx! {
        SelectableGridItem {
            content: FolderContent::File(file.clone()),
            current_path: current_path.clone(),
            index,
            on_open: move |_| {
                if is_media {
                    modal_state.write().active_modal = Some(
                        crate::AppModal::Gallery(display_name.clone(), gallery_path.clone()),
                    );
                }
            },
            CardContent { class: "p-4 flex flex-col justify-center items-center h-full w-full overflow-hidden",
                if is_image {
                    Image {
                        class: "object-contain w-min h-full align-center rounded-md transition-transform duration-300 group-hover:scale-105 group-hover:shadow-md",
                        src: thumbnail_url,
                    }
                } else if is_video {
                    div { class: "flex flex-col items-center justify-center w-16 h-20 sm:w-20 sm:h-24 bg-purple-500/10 rounded-xl border border-purple-500/20 text-purple-700 dark:text-purple-400/90 shadow-sm backdrop-blur-sm relative overflow-hidden group-hover:scale-110 group-hover:bg-purple-500/20 group-hover:shadow-[0_0_15px_rgba(168,85,247,0.2)] transition duration-300",
                        div { class: "absolute top-0 left-0 w-full h-[2px] bg-gradient-to-r from-purple-400/60 to-indigo-500/60" }
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "28",
                            height: "28",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            "stroke-width": "2",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            class: "mb-2 opacity-90 drop-shadow-sm",
                            path { d: "m15 10 4.553-2.276A1 1 0 0 1 21 8.618v6.764a1 1 0 0 1-1.447.894L15 14v-4z" }
                            rect {
                                x: "3",
                                y: "6",
                                width: "12",
                                height: "12",
                                rx: "2",
                                ry: "2",
                            }
                        }
                    }
                } else {
                    div { class: "flex flex-col items-center justify-center w-16 h-20 sm:w-20 sm:h-24 bg-slate-500/10 rounded-xl border border-slate-500/20 text-slate-700 dark:text-slate-300 font-bold text-base sm:text-lg uppercase tracking-wider shadow-sm backdrop-blur-sm relative overflow-hidden group-hover:scale-110 group-hover:bg-slate-500/20 group-hover:shadow-[0_0_15px_rgba(100,116,139,0.2)] transition duration-300",
                        div { class: "absolute top-0 left-0 w-full h-[2px] bg-gradient-to-r from-slate-400/60 to-slate-300/60" }
                        {display_ext}
                    }
                }
            }
            CardFooter { class: "overflow-hidden pb-4",
                div { class: "text-sm w-full h-4 mt-2",
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
