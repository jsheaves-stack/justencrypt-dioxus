use crate::{requests::folder::get_folder, Route};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let router = router();
    let mut app_state = use_context::<Signal<crate::AppState>>();
    let authenticated = app_state.read().authenticated;

    let current_route = use_route::<Route>();

    let session_check = use_resource(move || async move {
        if app_state.read().authenticated {
            return Ok(());
        }

        let mut root_path = Vec::<String>::new();
        root_path.push(String::from(""));

        match get_folder(root_path).await {
            Ok(_) => {
                if !app_state.read().authenticated {
                    app_state.write().authenticated = true;
                }

                Ok(())
            }
            Err(e) => {
                app_state.write().authenticated = false;
                tracing::error!("Session check failed: {:?}", e);
                Err(())
            }
        }
    });

    if authenticated {
        if let Route::Home {} = current_route {
            let new_path: Vec<String> = vec![];

            router.replace(Route::FileExplorer { path: new_path });

            rsx! {
                div { "Redirecting..." }
            }
        } else {
            rsx! {
                Outlet::<Route> {}
            }
        }
    } else {
        if !session_check.finished() {
            rsx! {
                div { "Loading..." }
            }
        } else {
            router.replace(Route::Login {});
            rsx! {
                div { "Redirecting..." }
            }
        }
    }
}
