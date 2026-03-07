use dioxus::prelude::*;
fn test() {
    let _ = rsx! {
        div {
            onwheel: move |e| {
                let _ = e.delta();
            }
        }
    };
}
