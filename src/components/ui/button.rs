use dioxus::prelude::*;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Ghost,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Icon,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonType {
    #[default]
    Button,
    Submit,
}

impl fmt::Display for ButtonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ButtonType::Button => write!(f, "button"),
            ButtonType::Submit => write!(f, "submit"),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default)]
    pub variant: ButtonVariant,

    #[props(default)]
    pub size: ButtonSize,

    #[props(default)]
    pub r#type: ButtonType,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub disabled: bool,

    #[props(default)]
    pub loading: bool,

    #[props(default)]
    pub autofocus: bool,

    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,

    #[props(default)]
    pub label_for: Option<String>,

    pub children: Element,
}

fn LoaderIcon() -> Element {
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
            class: "mr-2 h-4 w-4 animate-spin",
            path { d: "M21 12a9 9 0 1 1-6.219-8.56" }
        }
    }
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base_styles = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 active:scale-95 duration-200";

    let variant_styles = match props.variant {
        ButtonVariant::Default => "bg-blue-700/80 text-blue-50 hover:bg-blue-700 shadow-sm hover:shadow-[0_0_12px_rgba(29,78,216,0.3)] border border-blue-600/30",
        ButtonVariant::Destructive => "bg-destructive text-destructive-foreground hover:opacity-90 shadow-sm",
        ButtonVariant::Outline => {
            "border border-input bg-background hover:bg-accent hover:text-accent-foreground shadow-sm"
        }
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground",
    };

    let size_styles = match props.size {
        ButtonSize::Default => "h-10 px-4 py-2",
        ButtonSize::Sm => "h-9 rounded-md px-3",
        ButtonSize::Icon => "h-10 w-10",
    };

    let final_class = format!(
        "{} {} {} {}",
        base_styles, variant_styles, size_styles, props.class
    );

    let is_disabled = props.disabled || props.loading;

    if let Some(label_for) = props.label_for {
        rsx! {
            label { class: final_class, r#for: label_for,
                if props.loading {
                    LoaderIcon {}
                }
                {props.children}
            }
        }
    } else {
        rsx! {
            button {
                class: final_class,
                r#type: "{props.r#type}",
                disabled: is_disabled,
                autofocus: props.autofocus,
                onclick: move |evt| {
                    if !is_disabled {
                        props.onclick.call(evt);
                    }
                },

                if props.loading {
                    LoaderIcon {}
                }

                {props.children}
            }
        }
    }
}
