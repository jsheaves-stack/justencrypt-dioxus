use dioxus::prelude::*;

use crate::{FileExplorer, Login};

#[component]
pub fn Home() -> Element {
    let authenticated = use_signal(|| false);
    
    rsx! {
        div { class: "w-screen h-screen bg-slate-900",
            if !authenticated() {
                Login { authenticated }
            } else {
                FileExplorer {}
            }
        }
    }
}
