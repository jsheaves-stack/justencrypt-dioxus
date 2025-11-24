use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    rsx! {
        div {
            class: "rounded-lg border border-border bg-card text-card-foreground shadow-sm {props.class}",
            onclick: move |evt| props.onclick.call(evt),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardHeaderProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col space-y-1.5 p-6 {props.class}",
            onclick: move |evt| props.onclick.call(evt),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardTitleProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    rsx! {
        h3 {
            class: "text-2xl font-semibold leading-none tracking-tight {props.class}",
            onclick: move |evt| props.onclick.call(evt),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardDescriptionProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    rsx! {
        p {
            class: "text-sm text-muted-foreground {props.class}",
            onclick: move |evt| props.onclick.call(evt),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardContentProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    rsx! {
        div {
            class: "p-6 pt-0 {props.class}",
            onclick: move |evt| props.onclick.call(evt),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardFooterProps {
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    rsx! {
        div {
            class: "flex items-center p-6 pt-0 {props.class}",
            onclick: move |evt| props.onclick.call(evt),
            {props.children}
        }
    }
}
