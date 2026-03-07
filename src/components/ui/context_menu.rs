use dioxus::prelude::*;

use crate::components::file_explorer::{FolderItem, EDITABLE_FORMATS};
use crate::{AppState, Route};

#[derive(Clone, Debug, PartialEq)]
pub enum ContextMenuTarget {
    None,
    File(FolderItem, Vec<String>),
    Folder(FolderItem, Vec<String>),
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
    let router = router();

    let state = context_menu.read().clone();
    if state.target == ContextMenuTarget::None {
        return rsx! {};
    }

    let (x, y) = state.position;
    tracing::info!("Rendering context menu at {}, {}", x, y);

    rsx! {
        div {
            class: "fixed inset-0 z-[9999]",
            onmousedown: move |e| {
                tracing::info!("Overlay clicked, closing menu");
                e.stop_propagation();
                context_menu.write().target = ContextMenuTarget::None;
            },
            oncontextmenu: move |e| {
                tracing::info!("Overlay right-clicked, closing menu");
                e.prevent_default();
                e.stop_propagation();
                context_menu.write().target = ContextMenuTarget::None;
            },
            div {
                class: "absolute bg-background text-foreground border border-border shadow-lg rounded-md min-w-[150px] z-[10000] overflow-hidden",
                style: "left: {x}px; top: {y}px;",
                onmousedown: move |e| {
                    e.stop_propagation();
                },
                match state.target {
                    ContextMenuTarget::File(file, path) => {
                        let is_editable = file.file_extension
                            .as_deref()
                            .map(|ext| EDITABLE_FORMATS.contains(&ext.to_uppercase().as_str()))
                            .unwrap_or(false);
                            
                        let edit_file_name = file.file_name.clone();
                        let edit_path = path.clone();
                        let rename_file_name = file.file_name.clone();
                        let rename_path = path.clone();
                        let delete_file_name = file.file_name.clone();
                        let delete_path = path.clone();

                        rsx! {
                            div { class: "py-1 flex flex-col",
                                if is_editable {
                                    ContextMenuItem { 
                                        label: "Edit", 
                                        onclick: move |_| { 
                                            app_state.write().edit_file_modal = Some((edit_file_name.clone(), edit_path.clone()));
                                            context_menu.write().target = ContextMenuTarget::None; 
                                        } 
                                    }
                                }
                                ContextMenuItem { 
                                    label: "Rename", 
                                    onclick: move |_| { 
                                        app_state.write().rename_modal = Some((rename_file_name.clone(), rename_path.clone(), true));
                                        context_menu.write().target = ContextMenuTarget::None; 
                                    } 
                                }
                                ContextMenuItem { 
                                    label: "Delete", 
                                    onclick: move |_| { 
                                        app_state.write().delete_modal = Some((delete_file_name.clone(), delete_path.clone(), true));
                                        context_menu.write().target = ContextMenuTarget::None; 
                                    } 
                                }
                                ContextMenuItem { label: "Details", onclick: move |_| { context_menu.write().target = ContextMenuTarget::None; } }
                            }
                        }
                    },
                    ContextMenuTarget::Folder(folder, path) => {
                        let open_folder_name = folder.file_name.clone();
                        let open_path = path.clone();
                        let rename_folder_name = folder.file_name.clone();
                        let rename_path = path.clone();
                        let delete_folder_name = folder.file_name.clone();
                        let delete_path = path.clone();
                        rsx! {
                            div { class: "py-1 flex flex-col",
                                ContextMenuItem { 
                                    label: "Open", 
                                    onclick: move |_| { 
                                        let mut new_path = open_path.clone();
                                        new_path.push(open_folder_name.clone());
                                        router.push(Route::FileExplorer { path: new_path });
                                        context_menu.write().target = ContextMenuTarget::None; 
                                    } 
                                }
                                ContextMenuItem { 
                                    label: "Rename", 
                                    onclick: move |_| { 
                                        app_state.write().rename_modal = Some((rename_folder_name.clone(), rename_path.clone(), false));
                                        context_menu.write().target = ContextMenuTarget::None; 
                                    } 
                                }
                                ContextMenuItem { 
                                    label: "Delete", 
                                    onclick: move |_| { 
                                        app_state.write().delete_modal = Some((delete_folder_name.clone(), delete_path.clone(), false));
                                        context_menu.write().target = ContextMenuTarget::None; 
                                    } 
                                }
                            }
                        }
                    },
                    ContextMenuTarget::None => rsx! {}
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
            "{label}"
        }
    }
}
