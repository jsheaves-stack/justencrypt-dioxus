use std::collections::HashMap;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ReqwestClient;

#[derive(Deserialize, Serialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login(authenticated: Signal<bool>) -> Element {
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let onsubmit = move |event: FormEvent| async move {
        let username = event
            .data
            .values()
            .get("username")
            .and_then(|v| v.get(0))
            .unwrap()
            .clone();

        let password = event
            .data
            .values()
            .get("password")
            .and_then(|v| v.get(0))
            .unwrap()
            .clone();

        let mut login_form = HashMap::new();

        login_form.insert("username", username);
        login_form.insert("password", password);

        let resp = reqwest_client_context()
            .0
            .post("http://localhost:8000/session/create")
            .json(&login_form)
            .send()
            .await;

        match resp {
            // Parse data from here, such as storing a response token
            Ok(_) => authenticated.set(true),
            //Handle any errors from the fetch here
            Err(_err) => {
                tracing::error!("Login failed: {}", _err);
            }
        }
    };

    rsx! {
        div { class: "flex items-center justify-center min-h-screen bg-bg",
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
                                autocomplete: "on"
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
                                autocomplete: "on"
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
