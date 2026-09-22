use dioxus::prelude::*;

use crate::{
    components::{file_explorer::EDITABLE_FORMATS, ui::toast::show_toast},
    models::{FolderContent, ItemKey},
    requests::file::download_file,
    ModalState, ReqwestClient, Route, SelectionState, ToastKind, ToastState,
};

/// Resolves what a Move/Delete action applies to: the whole current
/// selection if `clicked` is part of it, otherwise just the clicked item on
/// its own (replacing whatever was selected).
fn resolve_selection_targets(
    selection_state: &mut Signal<SelectionState>,
    clicked: &ItemKey,
) -> Vec<ItemKey> {
    let mut selection = selection_state.write();
    if selection.selected_files.contains(clicked) {
        selection.selected_files.iter().cloned().collect()
    } else {
        selection.selected_files.clear();
        vec![clicked.clone()]
    }
}

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
    let mut modal_state = use_context::<Signal<ModalState>>();
    let mut selection_state = use_context::<Signal<SelectionState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    let toast_state = use_context::<Signal<ToastState>>();
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
                        let key = item.key();
                        let name = key.name.clone();
                        let is_file = key.is_file;

                        let rename_key = key.clone();
                        let rename_path = path.clone();

                        let delete_key = key.clone();
                        let delete_path = path.clone();

                        let move_key = key.clone();
                        let move_path = path.clone();

                        let details_item = item.clone();
                        let details_path = path.clone();

                        let is_root_folder = !is_file && path.is_empty() && name == "/";

                        // Folders never have an extension, so only files can be editable.
                        let is_editable = if let FolderContent::File(file) = &item {
                            file.file_extension
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
                                }



                                if is_file {
                                    if is_editable {
                                        ContextMenuItem {
                                            label: "Edit",
                                            onclick: {
                                                let edit_name = name.clone();
                                                let edit_path = path.clone();
                                                move |_| {
                                                    modal_state.write().active_modal = Some(
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
                                                    if let Err(e) = download_file(
                                                            reqwest_client_context,
                                                            path_clone,
                                                            name_clone,
                                                        )
                                                        .await
                                                    {
                                                        show_toast(
                                                            toast_state,
                                                            e.user_message("download file"),
                                                            ToastKind::Error,
                                                        );
                                                    }
                                                });
                                                context_menu.write().target = ContextMenuTarget::None;
                                            }
                                        },
                                    }
                                }

                                if !is_root_folder {
                                    ContextMenuItem {
                                        label: "Move",
                                        onclick: move |_| {
                                            let items_to_move = resolve_selection_targets(
                                                &mut selection_state,
                                                &move_key,
                                            );
                                            modal_state.write().active_modal = Some(
                                                crate::AppModal::Move(items_to_move, move_path.clone()),
                                            );
                                            context_menu.write().target = ContextMenuTarget::None;
                                        },
                                    }

                                    ContextMenuItem {
                                        label: "Rename",
                                        onclick: move |_| {
                                            modal_state.write().active_modal = Some(
                                                crate::AppModal::Rename(rename_key.clone(), rename_path.clone()),
                                            );
                                            context_menu.write().target = ContextMenuTarget::None;
                                        },
                                    }

                                    ContextMenuItem {
                                        label: "Delete",
                                        onclick: move |_| {
                                            let items_to_delete = resolve_selection_targets(
                                                &mut selection_state,
                                                &delete_key,
                                            );
                                            modal_state.write().active_modal = Some(
                                                crate::AppModal::Delete((items_to_delete, delete_path.clone())),
                                            );
                                            context_menu.write().target = ContextMenuTarget::None;
                                        },
                                    }
                                }

                                ContextMenuItem {
                                    label: "Details",
                                    onclick: move |_| {
                                        modal_state.write().active_modal = Some(
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
