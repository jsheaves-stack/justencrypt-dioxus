use dioxus::prelude::*;

/// Shared chrome for centered dialog-style modals: the fixed backdrop,
/// Escape-to-close, click-outside-to-close, the header with a close button,
/// and a footer action row. Individual modals only need to supply a title,
/// body content, and footer buttons.
///
/// `closable` gates both Escape and click-outside/× closing - pass `false`
/// (e.g. `!is_saving()`) while an operation is in flight so users can't
/// dismiss the modal out from under a pending request.
#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    pub title: Element,

    /// Width/height utility classes for the dialog panel (e.g. `"max-w-[400px]"`).
    #[props(default)]
    pub class: String,

    #[props(default = true)]
    pub closable: bool,

    pub on_close: EventHandler<()>,

    #[props(default = rsx! {})]
    pub footer: Element,

    pub children: Element,
}

#[component]
pub fn Modal(props: ModalProps) -> Element {
    let closable = props.closable;
    let on_close = props.on_close;

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm focus:outline-none p-4 animate-in fade-in duration-200",
            tabindex: "-1",
            onkeydown: move |e| {
                if closable && e.key().to_string() == "Escape" {
                    on_close.call(());
                }
            },
            onclick: move |_| {
                if closable {
                    on_close.call(());
                }
            },
            div {
                class: "relative flex w-full flex-col rounded-lg border border-border bg-background p-6 text-foreground shadow-lg animate-in zoom-in-95 duration-200 {props.class}",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex justify-between items-center mb-4 pb-3 border-b border-border flex-shrink-0",
                    h2 { class: "text-xl font-semibold", {props.title} }
                    button {
                        class: "text-muted-foreground hover:text-foreground p-2 disabled:opacity-50 disabled:pointer-events-none",
                        disabled: !closable,
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                {props.children}

                div { class: "flex justify-end mt-4 pt-4 gap-2 border-t border-border flex-shrink-0",
                    {props.footer}
                }
            }
        }
    }
}
