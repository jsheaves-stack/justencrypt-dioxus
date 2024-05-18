use dioxus::prelude::*;

use crate::Login;

#[component]
pub fn Home() -> Element {
    rsx! {
        Login {}
    }
}
