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

    let onsubmit = move |event: FormEvent| async move {
        event.prevent_default();

        let login_form: LoginForm = event.parsed_values().unwrap();
        let create_session_result = create_session(reqwest_client_context, login_form).await;

        is_loading.set(false);

        match create_session_result {
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
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-white pb-2",
                                    "Password"
                                }
                                Input {
                                    r#type: "password",
                                    name: "password",
                                    id: "password",
                                    required: true,
                                    autocomplete: "on",
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
            }
        }
    }
}
