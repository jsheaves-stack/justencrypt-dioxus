use dioxus::prelude::*;

/// Consistent inline error text for a modal's body, just above its footer
/// buttons. Centralizes the destructive-text styling that was previously
/// duplicated (with drifting margins/weight) across every modal.
#[derive(Props, Clone, PartialEq)]
pub struct ErrorMessageProps {
    pub message: Option<String>,

    /// Extra utility classes (e.g. spacing) layered onto the base style.
    #[props(default)]
    pub class: String,
}

#[component]
pub fn ErrorMessage(props: ErrorMessageProps) -> Element {
    let Some(message) = props.message else {
        return rsx! {};
    };

    rsx! {
        span { class: "text-sm text-destructive font-medium {props.class}", {message} }
    }
}
