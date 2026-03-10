use dioxus::prelude::*;

#[component]
pub fn Test() -> Element {
    rsx! {
        div {
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onkeydown: move |e| {
                println!("{:?}", e.key());
            }
        }
    }
}
