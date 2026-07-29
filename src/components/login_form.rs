use crate::{
    components::ui::{
        button::{Button, ButtonType},
        card::{Card, CardContent, CardTitle},
        input::Input,
    },
    requests::session::create_session,
    AppState, ReqwestClient, Route,
};
use dioxus::{prelude::*, router::RouterContext};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[component]
pub fn Login() -> Element {
    let reqwest_client_context: Signal<ReqwestClient> = use_context::<Signal<ReqwestClient>>();
    let mut app_state: Signal<AppState> = use_context::<Signal<AppState>>();
    let router: RouterContext = router();
    let mut is_loading = use_signal(|| false);
    let mut show_password = use_signal(|| false);
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);

    let onsubmit = move |event: FormEvent| async move {
        event.prevent_default();
        is_loading.set(true);

        match event.parsed_values::<LoginForm>() {
            Ok(login_form) => {
                let create_session_result =
                    create_session(reqwest_client_context, login_form).await;
                if create_session_result.is_ok() {
                    app_state.write().authenticated = true;
                    router.replace(Route::Home {});
                } else {
                    tracing::error!("Failed to create session");
                }
            }
            Err(e) => {
                tracing::error!("Failed to parse login form values: {:?}", e);
            }
        }

        is_loading.set(false);
    };

    rsx! {
        div { class: "w-full h-full flex items-center justify-center min-h-screen bg-background",
            div { class: "w-full max-w-md",
                // div { class: "relative flex flex-col items-center justify-center rounded-base border-black p-10 pt-12 font-base shadow-base",
                Card {
                    CardTitle { class: "pt-4",
                        span { class: "block w-full text-center", "Login" }
                    }
                    CardContent { class: "pb-0",
                        form { class: "space-y-6 p-4", onsubmit,
                            div {
                                label { class: "block text-sm font-medium text-white pb-2",
                                    "Username"
                                }
                                Input {
                                    r#type: "text",
                                    name: "username",
                                    id: "username",
                                    required: true,
                                    autocomplete: "on",
                                    value: Some(username.read().clone()),
                                    oninput: move |e: FormEvent| username.set(e.value()),
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-white pb-2",
                                    "Password"
                                }
                                div { class: "relative",
                                    Input {
                                        r#type: if *show_password.read() { "text" } else { "password" },
                                        name: "password",
                                        id: "password",
                                        required: true,
                                        autocomplete: "on",
                                        value: Some(password.read().clone()),
                                        oninput: move |e: FormEvent| password.set(e.value()),
                                    }
                                    button {
                                        r#type: "button",
                                        class: "absolute right-1 top-1/2 -translate-y-1/2 p-2 text-muted-foreground hover:text-foreground transition-colors",
                                        onclick: move |_| {
                                            let current = *show_password.read();
                                            show_password.set(!current);
                                        },
                                        if *show_password.read() {
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "16",
                                                height: "16",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "2",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                path { d: "M9.88 9.88a3 3 0 1 0 4.24 4.24" }
                                                path { d: "M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68" }
                                                path { d: "M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61" }
                                                line {
                                                    x1: "2",
                                                    y1: "2",
                                                    x2: "22",
                                                    y2: "22",
                                                }
                                            }
                                        } else {
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                width: "16",
                                                height: "16",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "2",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                path { d: "M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" }
                                                circle {
                                                    cx: "12",
                                                    cy: "12",
                                                    r: "3",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "w-min m-auto",
                                Button {
                                    r#type: ButtonType::Submit,
                                    loading: *is_loading.read(),
                                    "Login"
                                }
                            }
                        }
                    }
                }
                div { class: "mt-4 w-full text-center",
                    span { class: "text-xs text-muted-foreground font-medium",
                        "JustEncrypt v{crate::APP_VERSION}"
                    }
                }
            }
        }
    }
}
