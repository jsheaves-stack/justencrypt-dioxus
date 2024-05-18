use std::collections::HashMap;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{Authenticated, ReqwestClient};

#[derive(Deserialize, Serialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();
    let mut authenticated_context = use_context::<Signal<Authenticated>>();

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
            Ok(_) => authenticated_context.write().0 = true,
            //Handle any errors from the fetch here
            Err(_err) => {
                tracing::error!("Login failed: {}", _err);
            }
        }
    };

    rsx! {
        div { class: "flex items-center justify-center min-h-screen bg-slate-900",
            div { class: "w-full max-w-md",
                div { class: "bg-slate-800 p-8 rounded-lg shadow-md",
                    h2 { class: "text-2xl font-bold mb-6 text-center text-slate-300", "Login" }
                    form { class: "space-y-6", onsubmit,
                        div {
                            label { class: "block text-sm font-medium text-slate-300", "Username" }
                            input {
                                class: "w-full px-3 py-2 border dark:border-gray-300 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "text",
                                name: "username",
                                id: "username",
                                required: true,
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-slate-300", "Password" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                                r#type: "password",
                                name: "password",
                                id: "password",
                                required: true,
                            }
                        }
                        div {
                            button {
                                class: "w-full px-4 py-3  bg-blue-500 text-white rounded-lg shadow-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-600",
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
