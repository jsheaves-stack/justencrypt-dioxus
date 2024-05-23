use dioxus::prelude::*;

use crate::{FileExplorer, Login};

#[component]
pub fn Home() -> Element {
    let authenticated = use_signal(|| false);
    
    rsx! {
        div { class: "h-screen bg",
            if !authenticated() {
                Login { authenticated }
            } else {
                FileExplorer {}
            }
        }
    }
}
