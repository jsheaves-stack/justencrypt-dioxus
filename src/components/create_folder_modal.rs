use dioxus::prelude::*;

use crate::requests::folder::create_folder;

#[component]
pub fn CreateFolderModal() -> Element {
    let mut folder_name = use_signal::<String>(|| String::new());
    let mut app_state = use_context::<Signal<crate::AppState>>();
    let current_path = app_state.read().path.clone();

    rsx! {
        div { class: "fixed left-0 top-0 z-50 flex h-screen w-screen items-center justify-center bg-overlay text-text",
            div { class: "relative flex w-[300px] flex-col items-center justify-center rounded-base border-2 border-border bg-bg p-6 pt-12 font-base shadow-base",
                button {
                    class: "text-align-center w-18 absolute right-4 top-4 ml-2 flex cursor-pointer rounded-base border-2 border-black bg-main px-4 py-2 text-sm font-base shadow-base transition-all hover:translate-x-boxShadowX hover:translate-y-boxShadowY hover:shadow-none",
                    onclick: move |_| {
                        app_state.write().create_folder_modal = false;
                    },
                    "X"
                }
                div { class: "grid grid-rows-[2em,1fr] gap-2",
                    label { "Folder Name" }
                    input {
                        r#type: "text",
                        class: "h-12 w-full rounded-base border-2 border-black bg-bg p-[10px] font-base text-text outline-none ring-offset-white transition-all focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-slate-950 focus-visible:ring-offset-2",
                        value: "{folder_name.read()}",
                        onchange: move |e| {
                            folder_name.set(e.value());
                        },
                    }
                }
                button {
                    class: "text-align-center w-18 ml-2 mt-4 flex cursor-pointer rounded-base border-2 border-black bg-main px-4 py-2 text-sm font-base shadow-base transition-all hover:translate-x-boxShadowX hover:translate-y-boxShadowY hover:shadow-none",
                    onclick: move |_| {
                        let current_path = current_path.clone();

                        spawn(async move {
                            match create_folder(current_path, folder_name.read().to_string()).await {
                                Ok(_) => {
                                  app_state.write().create_folder_modal = false;
                                },
                                Err(_) => (),
                            }
                        });
                    },
                    "Ok"
                }
            }
        }
    }
}
