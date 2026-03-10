use dioxus::prelude::*;

#[component]
pub fn Test() -> Element {
    rsx! {
        div {
            ondocumentkeydown: move |e| {
                println!("{:?}", e.key());
            }
        }
    }
}
