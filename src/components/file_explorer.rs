use crate::{
    components::{
        create_file_modal::CreateFileModal,
        create_folder_modal::CreateFolderModal,
        delete_modal::DeleteModal,
        details_modal::DetailsModal,
        edit_file_modal::EditFileModal,
        file::File,
        folder::Folder,
        photo_viewer_modal::PhotoViewerModal,
        rename_modal::RenameModal,
        ui::{button::Button, header::Header, spinner::Spinner},
    },
    requests::{file::put_file, folder::get_folder},
    ReqwestClient, Route,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, str::FromStr};

pub const IMAGE_FORMATS: [&str; 8] = ["APNG", "AVIF", "GIF", "JPG", "JPEG", "PNG", "SVG", "WebP"];
pub const VIDEO_FORMATS: [&str; 6] = ["MP4", "WEBM", "OGG", "MOV", "AVI", "MKV"];

pub const EDITABLE_FORMATS: [&str; 50] = [
    "TXT",
    "MD",
    "JSON",
    "YAML",
    "YML",
    "XML",
    "HTML",
    "CSS",
    "JS",
    "JSX",
    "TS",
    "TSX",
    "PY",
    "RB",
    "PHP",
    "JAVA",
    "C",
    "CPP",
    "H",
    "CS",
    "GO",
    "RS",
    "LUA",
    "PL",
    "SH",
    "BASH",
    "ZSH",
    "CONF",
    "CFG",
    "INI",
    "LOG",
    "SQL",
    "CSV",
    "TSV",
    "NIX",
    "PROPERTIES",
    "TOML",
    "DOCKERFILE",
    "MAKEFILE",
    "SCALA",
    "KOTLIN",
    "SWIFT",
    "DART",
    "GROOVY",
    "R",
    "PS1",
    "VTT",
    "SRT",
    "NFO",
    "ME",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct FolderItem {
    pub is_file: bool,
    #[serde(default)]
    pub file_extension: Option<String>,
    pub file_name: String,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub created_date: Option<String>,
    #[serde(default)]
    pub last_modified_date: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortCriteria {
    NameAsc,
    NameDesc,
    TypeAsc,
    TypeDesc,
    SizeAsc,
    SizeDesc,
}

impl FromStr for SortCriteria {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NameAsc" => Ok(SortCriteria::NameAsc),
            "NameDesc" => Ok(SortCriteria::NameDesc),
            "TypeAsc" => Ok(SortCriteria::TypeAsc),
            "TypeDesc" => Ok(SortCriteria::TypeDesc),
            "SizeAsc" => Ok(SortCriteria::SizeAsc),
            "SizeDesc" => Ok(SortCriteria::SizeDesc),
            _ => Err(()),
        }
    }
}

#[component]
pub fn FileExplorer(path: Vec<String>) -> Element {
    let router = router();
    let mut app_state = use_context::<Signal<crate::AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    let upload_state = use_context::<Signal<crate::UploadState>>();
    let create_folder_modal_state = app_state.read().create_folder_modal;
    let create_file_modal_state = app_state.read().create_file_modal;
    let needs_refetch = use_memo(move || app_state.read().needs_refetch);
    let file_upload_path = path.clone();
    let folder_upload_path = path.clone();

    let path_clone = path.clone();
    let mut folder_contents_resource =
        use_resource(use_reactive!(|(path_clone, reqwest_client_context)| {
            let path_clone = path_clone.clone();
            let client_clone = reqwest_client_context;

            async move {
                let result = get_folder(client_clone, path_clone.clone()).await;

                (result, path_clone)
            }
        }));

    let mut display_count = use_signal(|| 50);
    let mut sort_criteria = use_signal(|| SortCriteria::TypeAsc);
    let mut show_hidden = use_signal(|| false);

    let _path_for_reset = path.clone();
    use_effect(use_reactive!(|_path_for_reset| {
        display_count.set(50);
        let _ = document::eval("document.getElementById('file-explorer-main').scrollTop = 0;");
    }));

    use_effect(move || {
        if needs_refetch() {
            folder_contents_resource.restart();
            app_state.write().needs_refetch = false;
        }
    });

    let path_clone_memo = path.clone();
    let sorted_files = use_memo(move || {
        let resource_value = (*folder_contents_resource.value())()?;
        let (Ok(mut files), resource_path) = resource_value else {
            return None;
        };

        if resource_path != path_clone_memo {
            return None;
        }

        if !*show_hidden.read() {
            files.retain(|f| !f.file_name.starts_with('.'));
        }

        files.sort_by(|file_a, file_b| {
            let name_comparison = file_a
                .file_name
                .chars()
                .map(|c| c.to_ascii_lowercase())
                .cmp(file_b.file_name.chars().map(|c| c.to_ascii_lowercase()));
            let extension_comparison = file_a.file_extension.cmp(&file_b.file_extension);

            match *sort_criteria.read() {
                SortCriteria::NameAsc => name_comparison,
                SortCriteria::NameDesc => name_comparison.reverse(),
                SortCriteria::TypeAsc => match (file_a.is_file, file_b.is_file) {
                    (false, true) => Ordering::Less,
                    (true, false) => Ordering::Greater,
                    _ => extension_comparison.then(name_comparison),
                },
                SortCriteria::TypeDesc => match (file_a.is_file, file_b.is_file) {
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    _ => extension_comparison
                        .reverse()
                        .then(name_comparison.reverse()),
                },
                SortCriteria::SizeAsc => match (file_a.is_file, file_b.is_file) {
                    (false, true) => Ordering::Less,
                    (true, false) => Ordering::Greater,
                    _ => file_a
                        .file_size
                        .unwrap_or(0)
                        .cmp(&file_b.file_size.unwrap_or(0))
                        .then(name_comparison),
                },
                SortCriteria::SizeDesc => match (file_a.is_file, file_b.is_file) {
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    _ => file_a
                        .file_size
                        .unwrap_or(0)
                        .cmp(&file_b.file_size.unwrap_or(0))
                        .reverse()
                        .then(name_comparison.reverse()),
                },
            }
        });

        Some(files)
    });

    let loading_spinner = rsx! {
        div { class: "flex w-full justify-center p-12 col-span-full",
            Spinner { class: "h-8 w-8 text-muted-foreground" }
        }
    };

    let folder_contents_for_display = match (*folder_contents_resource.value())() {
        Some((Err(_), _)) => {
            vec![rsx! { div { class: "col-span-full text-destructive", "Error loading folder." } }]
        }
        Some((Ok(_), resource_path)) if resource_path == path => {
            if let Some(files) = sorted_files.read().as_ref() {
                files
                    .iter()
                    .take(*display_count.read())
                    .map(|file| {
                        rsx! {
                            if file.is_file {
                                File {
                                    file: file.clone(),
                                    current_path: path.clone(),
                                }
                            } else {
                                Folder {
                                    file: file.clone(),
                                    current_path: path.clone(),
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![loading_spinner]
            }
        }
        _ => vec![loading_spinner],
    };

    let path_clone_back = path.clone();

    rsx! {
        div {
            id: "file-explorer-main",
            class: "w-full h-full bg-background text-foreground overflow-scroll focus:outline-none",
            tabindex: "0",
            autofocus: true,
            onkeydown: move |evt| {
                if evt.key().to_string() == "." {
                    show_hidden.toggle();
                }
            },
            onscroll: move |evt| {
                let data = evt.data();
                if data.scroll_top() + (data.client_height() as f64)
                    >= (data.scroll_height() as f64) - 1000.0
                {
                    display_count += 50;
                }
            },
            Header {}
            div { class: "mt-4 flex flex-col md:flex-row w-full justify-between items-start md:items-center gap-4 px-4",
                div { class: "flex-shrink-0",
                    Button {
                        onclick: move |_| {
                            let mut new_path_vec = path_clone_back.clone();
                            if new_path_vec.pop().is_some() {
                                router
                                    .push(Route::FileExplorer {
                                        path: new_path_vec,
                                    });
                            }
                        },
                        span { "Back" }
                    }
                }
                div { class: "flex flex-wrap items-center gap-2 justify-start md:justify-end w-full md:w-auto",
                    select {
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2 w-full sm:w-auto",
                        onchange: move |e| {
                            if let Ok(val) = e.value().parse::<SortCriteria>() {
                                sort_criteria.set(val);
                            }
                        },
                        option { value: "TypeAsc", "Type (A-Z)" }
                        option { value: "TypeDesc", "Type (Z-A)" }
                        option { value: "NameAsc", "Name (A-Z)" }
                        option { value: "NameDesc", "Name (Z-A)" }
                        option { value: "SizeAsc", "Size (Smallest)" }
                        option { value: "SizeDesc", "Size (Largest)" }
                    }
                    Button {
                        class: "w-full sm:w-auto",
                        onclick: move |_| {
                            app_state.write().create_file_modal = true;
                        },
                        "Create File"
                    }
                    Button {
                        class: "w-full sm:w-auto",
                        onclick: move |_| {
                            app_state.write().create_folder_modal = true;
                        },
                        "Create Folder"
                    }
                    div { class: "relative w-full sm:w-auto",
                        input {
                            class: "hidden",
                            id: "file-upload",
                            r#type: "file",
                            multiple: true,
                            onchange: move |e| {
                                let current_path = file_upload_path.clone();
                                spawn(async move {
                                    let files = e.files();
                                    if !files.is_empty() {
                                        let _ = put_file(
                                                reqwest_client_context,
                                                upload_state,
                                                current_path.clone(),
                                                files,
                                            )
                                            .await;
                                        app_state.write().needs_refetch = true;
                                    }
                                });
                            },
                        }
                        Button { class: "w-full sm:w-auto", "Upload Files" }
                        label {
                            class: "absolute inset-0 cursor-pointer opacity-0",
                            r#for: "file-upload",
                        }
                    }
                    div { class: "relative w-full sm:w-auto",
                        input {
                            class: "hidden",
                            id: "folder-upload",
                            r#type: "file",
                            directory: true,
                            onchange: move |e| {
                                let current_path = folder_upload_path.clone();
                                spawn(async move {
                                    let files = e.files();
                                    if !files.is_empty() {
                                        let _ = put_file(
                                                reqwest_client_context,
                                                upload_state,
                                                current_path.clone(),
                                                files,
                                            )
                                            .await;
                                        app_state.write().needs_refetch = true;
                                    }
                                });
                            },
                        }
                        Button { class: "w-full sm:w-auto", "Upload Folder" }
                        label {
                            class: "absolute inset-0 cursor-pointer opacity-0",
                            r#for: "folder-upload",
                        }
                    }
                }
            }

            if create_folder_modal_state {
                CreateFolderModal { current_path: path.clone() }
            }

            if create_file_modal_state {
                CreateFileModal { current_path: path.clone() }
            }

            if let Some((file_name, file_path)) = app_state.read().edit_file_modal.clone() {
                EditFileModal { file_name, current_path: file_path }
            }

            if let Some((old_name, file_path, is_file)) = app_state.read().rename_modal.clone() {
                RenameModal { old_name, current_path: file_path, is_file }
            }

            if let Some((name, file_path, is_file)) = app_state.read().delete_modal.clone() {
                DeleteModal { name, current_path: file_path, is_file }
            }

            if let Some((file_name, file_path)) = app_state.read().photo_viewer_modal.clone() {
                PhotoViewerModal { file_name, current_path: file_path }
            }

            if let Some((item, file_path)) = app_state.read().details_modal.clone() {
                DetailsModal { item, current_path: file_path }
            }

            div { class: "flex flex-row flex-wrap h-min w-full px-4 py-4 gap-4 auto-rows-max",
                {folder_contents_for_display.into_iter()}
            }
        }
    }
}
