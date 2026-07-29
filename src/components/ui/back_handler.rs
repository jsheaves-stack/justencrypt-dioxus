#[cfg(target_arch = "wasm32")]
use crate::AppState;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn BackHandler() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    use_hook(move || {
        let mut eval = document::eval(
            r#"
            window.modalOpenCount = 0;

            window.pushModalState = function() {
                window.modalOpenCount++;
                if (window.modalOpenCount === 1) {
                    history.pushState({modal: true}, '', location.href);
                }
            };

            window.popModalState = function() {
                if (window.modalOpenCount > 0) {
                    window.modalOpenCount--;
                    if (window.modalOpenCount === 0) {
                        if (history.state && history.state.modal) {
                            history.back();
                        }
                    }
                }
            };

            window.addEventListener('popstate', function(e) {
                if (window.modalOpenCount > 0) {
                    window.modalOpenCount = 0;
                    dioxus.send("close_modals");
                }
            });
        "#,
        );

        spawn(async move {
            loop {
                if let Ok(_) = eval.recv::<String>().await {
                    app_state.write().close_all_modals();
                }
            }
        });
    });

    let is_any_open = app_state.read().is_any_modal_open();
    let mut was_open = use_signal(|| false);

    use_effect(use_reactive!(|is_any_open| {
        let currently_open = is_any_open;
        let previously_open = *was_open.read();

        if currently_open && !previously_open {
            let _ = document::eval("window.pushModalState()");
            was_open.set(true);
        } else if !currently_open && previously_open {
            let _ = document::eval("window.popModalState()");
            was_open.set(false);
        }
    }));

    rsx! {}
}

#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn BackHandler() -> Element {
    rsx! {}
}
