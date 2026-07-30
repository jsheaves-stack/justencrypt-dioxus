use dioxus::prelude::*;

use crate::{
    components::file_explorer::EDITABLE_FORMATS, models::FolderContent,
    requests::file::download_file, AppState, ReqwestClient, Route,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ContextMenuTarget {
    None,
    Item(FolderContent, Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuState {
    pub target: ContextMenuTarget,
    pub position: (f64, f64),
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            target: ContextMenuTarget::None,
            position: (0.0, 0.0),
        }
    }
}

#[component]
pub fn ContextMenu() -> Element {
    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let mut app_state = use_context::<Signal<AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    let router = router();

    let state = context_menu.read().clone();
    if state.target == ContextMenuTarget::None {
        return rsx! {};
    }

    let (x, y) = state.position;
    tracing::debug!("Rendering context menu at {}, {}", x, y);

    rsx! {
        div {
            class: "fixed inset-0 z-[9999]",
            onmousedown: move |e| {
                tracing::debug!("Overlay clicked, closing menu");
                e.stop_propagation();
                context_menu.write().target = ContextMenuTarget::None;
            },
            oncontextmenu: move |e| {
                tracing::debug!("Overlay right-clicked, closing menu");
                e.prevent_default();
                e.stop_propagation();
                context_menu.write().target = ContextMenuTarget::None;
            },
            div {
                class: "absolute bg-background text-foreground border border-border shadow-lg rounded-md min-w-[160px] z-[10000] overflow-hidden",
                style: "left: min({x}px, calc(100vw - 170px)); top: min({y}px, calc(100vh - 220px));",
                onmousedown: move |e| {
                    e.stop_propagation();
                },
                match state.target {

                    ContextMenuTarget::Item(item, path) => {
                        let name = item.file_name().to_string();
                        let is_file = item.is_file();

                        let rename_name = name.clone();
                        let rename_path = path.clone();

                        let delete_name = name.clone();
                        let delete_path = path.clone();

                        let move_name = name.clone();
                        let move_path = path.clone();

                        let details_item = item.clone();
                        let details_path = path.clone();

                        let is_root_folder = !is_file && path.is_empty() && name == "/";

                        let is_editable = if let FolderContent::File(file) = &item {
                            file.file_extension

                                // --- Folder Specific ---

                                // --- File Specific ---

                                // --- Shared (Rename, Delete, Details) ---

                                .as_deref()
                                .map(|ext| {
                                    EDITABLE_FORMATS.iter().any(|f| ext.eq_ignore_ascii_case(f))
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        rsx! {
                            div { class: "py-1 flex flex-col",
                                if !is_file && !is_root_folder {
                                    ContextMenuItem {
                                        label: "Open",
                                        onclick: {
                                            let open_name = name.clone();
                                            let open_path = path.clone();
                                            move |_| {
                                                let mut new_path = open_path.clone();
                                                new_path.push(open_name.clone());
                                                router



                                                    .push(Route::FileExplorer {
                                                        path: new_path,
                                                    });
                                                context_menu.write().target = ContextMenuTarget::None;
                                            }
                                        },
                                    }

                                    if is_file {
                                        if is_editable {
                                            ContextMenuItem {
                                                label: "Edit",
                                                onclick: {
                                                    let edit_name = name.clone();
                                                    let edit_path = path.clone();
                                                    move |_| {
                                                        app_state.write().active_modal = Some(
                                                            crate::AppModal::EditFile(edit_name.clone(), edit_path.clone()),
                                                        );
                                                        context_menu.write().target = ContextMenuTarget::None;
                                                    }
                                                },
                                            }
                                        }

                                        ContextMenuItem {
                                            label: "Download",
                                            onclick: {
                                                let dl_name = name.clone();
                                                let dl_path = path.clone();
                                                move |_| {
                                                    let name_clone = dl_name.clone();
                                                    let path_clone = dl_path.clone();
                                                    spawn(async move {
                                                        let _ = download_file(reqwest_client_context, path_clone, name_clone)
                                                            .await;
                                                    });
                                                    context_menu.write().target = ContextMenuTarget::None;
                                                }
                                            },
                                        }
                                    }
                                }

                                if !is_root_folder {
                                    ContextMenuItem {
                                        label: "Move",
                                        onclick: move |_| {
                                            let mut state = app_state.write();
                                            let mut items_to_move = vec![];
                                            if state.selected_files.contains(&(move_name.clone(), is_file)) {
                                                items_to_move = state.selected_files.iter().cloned().collect();
                                            } else {
                                                state.selected_files.clear();
                                                items_to_move.push((move_name.clone(), is_file));
                                            }
                                            state.active_modal = Some(
                                                crate::AppModal::Move(items_to_move, move_path.clone()),
                                            );
                                            context_menu.write().target = ContextMenuTarget::None;
                                        },
                                    }

                                    ContextMenuItem {
                                        label: "Rename",
                                        onclick: move |_| {
                                            app_state.write().active_modal = Some(
                                                crate::AppModal::Rename(rename_name.clone(), rename_path.clone(), is_file),
                                            );
                                            context_menu.write().target = ContextMenuTarget::None;
                                        },
                                    }

                                    ContextMenuItem {
                                        label: "Delete",
                                        onclick: move |_| {
                                            let mut state = app_state.write();
                                            let mut items_to_delete = vec![];
                                            if state.selected_files.contains(&(delete_name.clone(), is_file)) {
                                                items_to_delete = state.selected_files.iter().cloned().collect();
                                            } else {
                                                state.selected_files.clear();
                                                items_to_delete.push((delete_name.clone(), is_file));
                                            }
                                            state.active_modal = Some(
                                                crate::AppModal::Delete((items_to_delete, delete_path.clone())),
                                            );
                                            context_menu.write().target = ContextMenuTarget::None;
                                        },
                                    }
                                }

                                ContextMenuItem {
                                    label: "Details",
                                    onclick: move |_| {
                                        app_state.write().active_modal = Some(
                                            crate::AppModal::Details(details_item.clone(), details_path.clone()),
                                        );
                                        context_menu.write().target = ContextMenuTarget::None;
                                    },
                                }
                            }
                        }
                    }
                    ContextMenuTarget::None => rsx! {},
                }
            }
        }
    }
}

#[component]
fn ContextMenuItem(label: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "w-full text-left px-4 py-2 text-sm hover:bg-muted focus:bg-muted outline-none transition-colors",
            onclick: move |e| onclick.call(e),
            {label}
        }
    }
}
