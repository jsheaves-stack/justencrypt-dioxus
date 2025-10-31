use crate::{requests::session::create_session, AppState, ReqwestClient, Route};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    let reqwest_client_context: Signal<ReqwestClient> = use_context::<Signal<ReqwestClient>>();
    let mut app_state: Signal<AppState> = use_context::<Signal<AppState>>();
    let router: RouterContext = router();

    let onsubmit = move |event: FormEvent| async move {
        event.prevent_default();

        match create_session(event, reqwest_client_context).await {
            Ok(_) => {
                app_state.write().authenticated = true;
                router.replace(Route::Home {});
            }
            Err(_) => {
                return;
            }
        }
    };

    rsx! {
        div { class: "w-full h-full flex items-center justify-center min-h-screen bg",
            div { class: "w-full max-w-md",
                div { class: "relative flex flex-col items-center justify-center rounded-base border-2 border-black bg-mainAccent p-10 pt-12 font-base shadow-base",
                    h2 { class: "text-2xl font-bold mb-6 text-center text-white", "Login" }
                    form { class: "space-y-6", onsubmit,
                        div {
                            label { class: "block text-sm font-medium text-white", "Username" }
                            input {
                                class: "w-full h-12 rounded-base border-2 border-black p-[10px] font-base ring-offset-white focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-slate-950 focus-visible:ring-offset-2 outline-none transition-all",
                                r#type: "text",
                                name: "username",
                                id: "username",
                                required: true,
                                autocomplete: "on",
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-white", "Password" }
                            input {
                                class: "w-full h-12 rounded-base border-2 border-black p-[10px] font-base ring-offset-white focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-slate-950 focus-visible:ring-offset-2 outline-none transition-all",
                                r#type: "password",
                                name: "password",
                                id: "password",
                                required: true,
                                autocomplete: "on",
                            }
                        }
                        div { class: "w-full items-center",
                            button {
                                class: "flex m-auto text-align-center cursor-pointer items-center rounded-base border-2 border-black bg-main w-18 px-4 py-2 text-sm font-base shadow-base transition-all hover:translate-x-boxShadowX hover:translate-y-boxShadowY hover:shadow-none",
                                r#type: "submit",
                                "Login"
                            }
                        }
                    }
                }
            }
        }
    }
}
