use dioxus::prelude::*;
use std::{cmp, time::Duration};
#[cfg(not(target_arch = "wasm32"))]
use tokio::time;

use crate::{
    components::ui::{
        card::{Card, CardContent, CardFooter},
        context_menu::{ContextMenuState, ContextMenuTarget},
    },
    models::{Folder as ModelFolder, FolderContent},
    AppState, Route,
};

#[component]
pub fn Folder(file: ModelFolder, current_path: Vec<String>, index: usize) -> Element {
    let router = router();
    let new_file = file.clone();

    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let mut app_state = use_context::<Signal<AppState>>();
    let sorted_files = use_context::<Memo<Option<Vec<FolderContent>>>>();

    let mut on_select = move |(idx, is_shift, _is_ctrl): (usize, bool, bool)| {
        let mut state = app_state.write();
        if let Some(files) = sorted_files.read().as_ref() {
            let item = &files[idx];
            let item_key = (item.file_name().to_string(), item.is_file());

            if is_shift {
                if let Some(last_idx) = state.last_selected_index {
                    let start = cmp::min(last_idx, idx);
                    let end = cmp::max(last_idx, idx);

                    for item in files.iter().take(end + 1).skip(start) {
                        let key = (item.file_name().to_string(), item.is_file());
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
        }
    };

    let file_clone = file.clone();
    let current_path_clone = current_path.clone();

    let display_name = file.file_name.clone();

    let mut press_task = use_signal(|| None);
    let mut was_long_pressed = use_signal(|| false);
    let mut is_pressed = use_signal(|| false);

    rsx! {
        div {
            class: "flex justify-center h-40 sm:h-48 md:h-56 w-full cursor-pointer relative group/folder",
            onmousedown: move |_| {
                is_pressed.set(true);
                let handle = spawn(async move {
                    #[cfg(target_arch = "wasm32")]
                    gloo_timers::future::sleep(Duration::from_millis(500)).await;

                    #[cfg(not(target_arch = "wasm32"))]
                    time::sleep(Duration::from_millis(500)).await;

                    if *is_pressed.read() {
                        was_long_pressed.set(true);
                        on_select((index, false, false));
                    }
                });
                press_task.set(Some(handle));
            },
            onmouseup: move |_| {
                is_pressed.set(false);
                if let Some(handle) = press_task.take() {
                    handle.cancel();
                }
            },
            onmouseleave: move |_| {
                is_pressed.set(false);
                if let Some(handle) = press_task.take() {
                    handle.cancel();
                }
            },
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                tracing::debug!("Folder right-clicked at {}, {}", coords.x, coords.y);
                let mut state = context_menu.write();
                state.target = ContextMenuTarget::Item(
                    FolderContent::Folder(file_clone.clone()),
                    current_path_clone.clone(),
                );
                state.position = (coords.x, coords.y);
            },
            Card {
                class: format!(
                    "w-full h-full grid grid-rows-[1fr,auto] active:scale-95 transition-all duration-300 hover:shadow-lg group select-none {}",
                    if app_state.read().selected_files.contains(&(display_name.clone(), false)) {
                        "ring-2 ring-primary bg-primary/5"
                    } else {
                        ""
                    },
                ),
                onclick: move |e: Event<MouseData>| {
                    if *was_long_pressed.read() {
                        was_long_pressed.set(false);
                        e.stop_propagation();
                        return;
                    }

                    let has_selection = !app_state.read().selected_files.is_empty();

                    if e.modifiers().contains(Modifiers::SHIFT)
                        // Open the folder
                        || e.modifiers().contains(Modifiers::CONTROL)
                    {
                        e.stop_propagation();
                        on_select((
                            index,
                            e.modifiers().contains(Modifiers::SHIFT),
                            e.modifiers().contains(Modifiers::CONTROL),
                        ));
                    } else if has_selection {
                        e.stop_propagation();
                        on_select((index, false, false));
                    } else {
                        let mut new_path = current_path.clone();
                        new_path.push(new_file.file_name.clone());
                        router
                            .push(Route::FileExplorer {
                                path: new_path,
                            });
                    }
                },
                CardContent { class: "flex flex-col items-center justify-center p-0 pt-6",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        view_box: "0 0 24 24",
                        class: "w-20 h-20 sm:w-24 sm:h-24 drop-shadow-sm group-hover:drop-shadow-md group-hover:scale-110 transition-all duration-300",
                        defs {
                            linearGradient {
                                id: "folderGrad",
                                x1: "0%",
                                y1: "0%",
                                x2: "0%",
                                y2: "100%",
                                stop {
                                    offset: "0%",
                                    "stop-color": "#2563eb",
                                    "stop-opacity": "0.8",
                                }
                                stop {
                                    offset: "100%",
                                    "stop-color": "#1d4ed8",
                                    "stop-opacity": "0.8",
                                }
                            }
                        }
                        path {
                            d: "M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z",
                            fill: "url(#folderGrad)",
                        }
                    }
                }
                CardFooter { class: "overflow-hidden pb-4",
                    div { class: "text-sm w-full h-4 flex justify-center mt-2",
                        span { class: "block w-[90%] overflow-hidden truncate text-center font-medium text-foreground/90 group-hover:text-foreground transition-colors h-6",
                            "{file.file_name}"
                        }
                    }
                }
            }
            div {
                class: format!(
                    "absolute top-2 right-2 z-10 transition-opacity duration-200 {}",
                    if app_state.read().selected_files.contains(&(file.file_name.clone(), false)) {
                        "opacity-100"
                    } else {
                        "opacity-0"
                    },
                ),
                input {
                    r#type: "checkbox",
                    class: "w-5 h-5 rounded border-gray-300 text-primary focus:ring-primary cursor-pointer",
                    checked: app_state.read().selected_files.contains(&(file.file_name.clone(), false)),
                    onclick: move |e| {
                        e.stop_propagation();
                        on_select((index, false, false));
                    },
                }
            }
        }
    }
}
