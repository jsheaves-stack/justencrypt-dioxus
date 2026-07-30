use crate::{
    components::{
        create_file_modal::CreateFileModal,
        create_folder_modal::CreateFolderModal,
        delete_modal::DeleteModal,
        details_modal::DetailsModal,
        edit_file_modal::EditFileModal,
        file::File as FileComponent,
        folder::Folder as FolderComponent,
        move_modal::MoveModal,
        photo_viewer_modal::PhotoViewerModal,
        rename_modal::RenameModal,
        ui::{
            button::{Button, ButtonVariant},
            context_menu::{ContextMenuState, ContextMenuTarget},
            header::Header,
            spinner::Spinner,
        },
    },
    models::{Folder, FolderContent},
    requests::{file::put_file, folder::get_folder},
    ReqwestClient, Route,
};
use dioxus::prelude::*;
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
    let mut context_menu = use_context::<Signal<ContextMenuState>>();
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

        let mut state = app_state.write();
        if state
            .photo_viewer_modal
            .as_ref()
            .is_some_and(|(_, p)| p != &_path_for_reset)
        {
            state.photo_viewer_modal = None;
        }
        if state
            .edit_file_modal
            .as_ref()
            .is_some_and(|(_, p)| p != &_path_for_reset)
        {
            state.edit_file_modal = None;
        }
        if state
            .rename_modal
            .as_ref()
            .is_some_and(|(_, p, _)| p != &_path_for_reset)
        {
            state.rename_modal = None;
        }
        if state
            .delete_modal
            .as_ref()
            .is_some_and(|(_, p)| p != &_path_for_reset)
        {
            state.delete_modal = None;
        }
        if state
            .details_modal
            .as_ref()
            .is_some_and(|(_, p)| p != &_path_for_reset)
        {
            state.details_modal = None;
        }
        if state
            .move_modal
            .as_ref()
            .is_some_and(|(_, p)| p != &_path_for_reset)
        {
            state.move_modal = None;
        }
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
            files.retain(|f| !f.file_name().starts_with('.'));
        }

        files.sort_by(|file_a, file_b| {
            let name_comparison = file_a
                .file_name()
                .chars()
                .map(|c| c.to_ascii_lowercase())
                .cmp(file_b.file_name().chars().map(|c| c.to_ascii_lowercase()));
            let extension_comparison = file_a.file_extension().cmp(&file_b.file_extension());

            match *sort_criteria.read() {
                SortCriteria::NameAsc => name_comparison,
                SortCriteria::NameDesc => name_comparison.reverse(),
                SortCriteria::TypeAsc => match (file_a.is_file(), file_b.is_file()) {
                    (false, true) => Ordering::Less,
                    (true, false) => Ordering::Greater,
                    _ => extension_comparison.then(name_comparison),
                },
                SortCriteria::TypeDesc => match (file_a.is_file(), file_b.is_file()) {
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    _ => extension_comparison
                        .reverse()
                        .then(name_comparison.reverse()),
                },
                SortCriteria::SizeAsc => match (file_a.is_file(), file_b.is_file()) {
                    (false, true) => Ordering::Less,
                    (true, false) => Ordering::Greater,
                    _ => file_a
                        .file_size()
                        .unwrap_or(0)
                        .cmp(&file_b.file_size().unwrap_or(0))
                        .then(name_comparison),
                },
                SortCriteria::SizeDesc => match (file_a.is_file(), file_b.is_file()) {
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    _ => file_a
                        .file_size()
                        .unwrap_or(0)
                        .cmp(&file_b.file_size().unwrap_or(0))
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
                let current_files = files.clone();

                files
                    .iter()
                    .enumerate()
                    .take(*display_count.read())
                    .map(|(index, item)| {
                        let current_files_clone = current_files.clone();
                        let on_select_handler =
                            move |(idx, is_shift, _is_ctrl): (usize, bool, bool)| {
                                let mut state = app_state.write();
                                let (item_name, is_file) = match &current_files_clone[idx] {
                                    FolderContent::File(f) => (f.file_name.clone(), true),
                                    FolderContent::Folder(f) => (f.file_name.clone(), false),
                                };
                                let item_key = (item_name, is_file);

                                if is_shift {
                                    if let Some(last_idx) = state.last_selected_index {
                                        let start = std::cmp::min(last_idx, idx);
                                        let end = std::cmp::max(last_idx, idx);

                                        for item in
                                            current_files_clone.iter().take(end + 1).skip(start)
                                        {
                                            let key = match item {
                                                FolderContent::File(f) => {
                                                    (f.file_name.clone(), true)
                                                }
                                                FolderContent::Folder(f) => {
                                                    (f.file_name.clone(), false)
                                                }
                                            };
                                            state.selected_files.insert(key);
                                        }
                                    } else {
                                        state.selected_files.insert(item_key);
                                        state.last_selected_index = Some(idx);
                                    }
                                } else {
                                    if state.selected_files.contains(&item_key) {
                                        state.selected_files.remove(&item_key);
                                    } else {
                                        state.selected_files.insert(item_key);
                                    }
                                    state.last_selected_index = Some(idx);
                                }
                            };

                        rsx! {
                            match item {
                                FolderContent::File(file) => {
                                    rsx! {
                                        FileComponent {
                                            file: file.clone(),
                                            current_path: path.clone(),
                                            index,
                                            on_select: on_select_handler.clone(),
                                        }
                                    }
                                }
                                FolderContent::Folder(folder) => rsx! {
                                    FolderComponent {
                                        file: folder.clone(),
                                        current_path: path.clone(),
                                        index,
                                        on_select: on_select_handler.clone(),
                                    }
                                },
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
    let path_clone_ctx = path.clone();

    let media_files_list = if let Some(files) = sorted_files.read().as_ref() {
        files
            .iter()
            .filter(|f| f.is_file())
            .filter_map(|f| {
                if let Some(ext) = f.file_extension() {
                    if IMAGE_FORMATS
                        .iter()
                        .any(|fmt| ext.eq_ignore_ascii_case(fmt))
                        || VIDEO_FORMATS
                            .iter()
                            .any(|fmt| ext.eq_ignore_ascii_case(fmt))
                    {
                        return Some(f.file_name().to_string());
                    }
                }
                None
            })
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let (file_count, folder_count) = if let Some(files) = sorted_files.read().as_ref() {
        let mut total_files = 0;
        let mut total_folders = 0;
        for item in files {
            match item {
                FolderContent::File(_) => total_files += 1,
                FolderContent::Folder(f) => {
                    total_folders += 1 + f.folder_count;
                    total_files += f.file_count;
                }
            }
        }
        (total_files, total_folders)
    } else {
        (0, 0)
    };
    let folder_size = sorted_files
        .read()
        .as_ref()
        .map(|files| files.iter().filter_map(|f| f.file_size()).sum::<i64>());

    rsx! {
        div {
            id: "file-explorer-main",
            class: "w-full h-full bg-background text-foreground overflow-scroll focus:outline-none",
            tabindex: "0",
            autofocus: true,
            oncontextmenu: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();
                app_state.write().selected_files.clear();
                let coords = evt.client_coordinates();
                let mut state = context_menu.write();
                let current_folder_name = path_clone_ctx
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "/".to_string());
                let parent_path = if path_clone_ctx.is_empty() {
                    vec![]
                } else {
                    path_clone_ctx[..path_clone_ctx.len() - 1].to_vec()
                };
                let folder = Folder {
                    file_name: current_folder_name,
                    file_size: folder_size,
                    created_date: None,
                    last_modified_date: None,
                    file_count,
                    folder_count,
                };
                state.target = ContextMenuTarget::Item(
                    FolderContent::Folder(folder),
                    parent_path,
                );
                state.position = (coords.x, coords.y);
            },
            onclick: move |_| {
                app_state.write().selected_files.clear();
            },
            onkeydown: move |evt| {
                if evt.key().to_string() == "." {
                    show_hidden.toggle();
                } else if evt.key().to_string() == "Escape" {
                    app_state.write().selected_files.clear();
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
                div { class: "grid grid-cols-2 sm:flex sm:flex-wrap items-center gap-2 justify-start sm:justify-end w-full md:w-auto",
                    Button {
                        variant: ButtonVariant::Outline,
                        class: "w-full sm:w-auto",
                        onclick: move |_| {
                            show_hidden.toggle();
                        },
                        if *show_hidden.read() {
                            "Hide Hidden"
                        } else {
                            "Show Hidden"
                        }
                    }
                    select {
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground shadow-sm h-10 px-4 py-2 w-full sm:w-auto active:scale-95 duration-200 cursor-pointer appearance-none",
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
                        Button {
                            class: "w-full sm:w-auto cursor-pointer",
                            label_for: "file-upload".to_string(),
                            "Upload Files"
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
                        Button {
                            class: "w-full sm:w-auto cursor-pointer",
                            label_for: "folder-upload".to_string(),
                            "Upload Folder"
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

            if let Some((items, file_path)) = app_state.read().delete_modal.clone() {
                DeleteModal { items, current_path: file_path }
            }

            if let Some((file_name, file_path)) = app_state.read().photo_viewer_modal.clone() {
                PhotoViewerModal {
                    file_name,
                    current_path: file_path,
                    media_files: media_files_list.clone(),
                }
            }

            if let Some((item, file_path)) = app_state.read().details_modal.clone() {
                DetailsModal { item, current_path: file_path }
            }

            if let Some((files_to_move, file_path)) = app_state.read().move_modal.clone() {
                MoveModal { files_to_move, original_path: file_path }
            }

            div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 h-min w-full px-4 py-4 gap-4",
                {folder_contents_for_display.into_iter()}
            }
        }
    }
}
