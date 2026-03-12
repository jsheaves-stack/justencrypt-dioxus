use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SpinnerProps {
    #[props(default = "h-4 w-4".to_string())]
    pub class: String,
}

#[component]
pub fn Spinner(props: SpinnerProps) -> Element {
    let final_class = format!("animate-spin {}", props.class);

    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "{final_class}",
            path { d: "M21 12a9 9 0 1 1-6.219-8.56" }
        }
    }
}
