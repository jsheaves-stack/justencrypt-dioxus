use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    #[props(default)]
    pub class: String,

    #[props(default = "text".to_string())]
    pub r#type: String,

    #[props(default)]
    pub value: Option<String>,

    #[props(default)]
    pub name: String,

    #[props(default)]
    pub id: String,

    #[props(default)]
    pub required: bool,

    #[props(default)]
    pub autocomplete: String,

    #[props(default)]
    pub placeholder: String,

    #[props(default)]
    pub disabled: bool,

    #[props(default)]
    pub oninput: EventHandler<FormEvent>,

    #[props(default)]
    pub onchange: EventHandler<FormEvent>,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    rsx! {
        input {
            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {props.class}",
            r#type: "{props.r#type}",
            value: props.value,
            name: "{props.name}",
            id: "{props.id}",
            required: "{props.required}",
            autocomplete: "{props.autocomplete}",
            placeholder: "{props.placeholder}",
            disabled: props.disabled,
            oninput: move |evt| props.oninput.call(evt),
            onchange: move |evt| props.onchange.call(evt),
        }
    }
}
