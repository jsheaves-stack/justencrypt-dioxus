use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    #[props(default)]
    pub class: String,

    #[props(default)]
    pub html_for: Option<String>,

    #[props(default)]
    pub tooltip: Option<String>,

    pub children: Element,
}

#[component]
pub fn Label(props: LabelProps) -> Element {
    let base_styles = "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70";

    rsx! {
        label {
            class: "group relative w-fit flex items-center gap-1.5 {base_styles} {props.class}",
            r#for: props.html_for,

            {props.children}

            if let Some(text) = &props.tooltip {
                div { class: "pointer-events-none absolute bottom-full left-1/2 mb-2 hidden -translate-x-1/2 w-max max-w-[250px] text-wrap rounded-md border border-border bg-popover px-3 py-1.5 text-xs text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95 group-hover:block z-50",
                    {text.clone()}

                    div { class: "absolute -bottom-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border-b border-r border-border bg-popover" }
                }
            }
        }
    }
}
