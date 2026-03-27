#![allow(unused)]

use crate::{
    components::ui::button::{Button, ButtonSize, ButtonVariant},
    requests::session::destroy_session,
    ReqwestClient, Route,
};
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    let router = router();
    let mut is_menu_open = use_signal(|| false);
    let mut app_state = use_context::<Signal<crate::AppState>>();
    let reqwest_client_context = use_context::<Signal<ReqwestClient>>();

    let menu_entries = vec![("Home", Route::FileExplorer { path: vec![] })];

    rsx! {
        header { class: "sticky top-0 z-50 w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60",
            div { class: "flex h-16 w-full p-4 items-center",
                div { class: "mr-4 hidden md:flex",
                    span {
                        class: "mr-6 flex items-center space-x-2 font-bold cursor-pointer",
                        onclick: move |_| {
                            router
                                .push(Route::FileExplorer {
                                    path: Vec::<String>::new(),
                                });
                        },
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            class: "h-6 w-6",
                            path { d: "M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3" }
                        }
                        div { class: "relative flex flex-col items-start justify-center",
                            span { class: "hidden font-bold sm:inline-block", "JustEncrypt" }
                            span { class: "absolute -bottom-3.5 left-0 text-[10px] text-muted-foreground font-medium whitespace-nowrap",
                                "v{crate::APP_VERSION}"
                            }
                        }
                    }

                    nav { class: "flex items-center gap-4 text-sm font-medium lg:gap-6",
                        for (text , to) in menu_entries.clone() {
                            Link { key: "{text}", to, "{text}" }
                        }
                    }
                }

                div { class: "flex md:hidden items-center",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        onclick: move |_| is_menu_open.set(!is_menu_open()),
                        if is_menu_open() {
                            // X Icon
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
                                class: "h-5 w-5",
                                path { d: "M18 6 6 18" }
                                path { d: "m6 6 12 12" }
                            }
                        } else {
                            // Hamburger Icon
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
                                class: "h-5 w-5",
                                line {
                                    x1: "4",
                                    x2: "20",
                                    y1: "12",
                                    y2: "12",
                                }
                                line {
                                    x1: "4",
                                    x2: "20",
                                    y1: "6",
                                    y2: "6",
                                }
                                line {
                                    x1: "4",
                                    x2: "20",
                                    y1: "18",
                                    y2: "18",
                                }
                            }
                        }
                    }

                    if !is_menu_open() {
                        div { class: "relative flex flex-col items-start justify-center ml-2",
                            span { class: "font-bold flex items-center", "JustEncrypt" }
                            span { class: "absolute -bottom-3.5 left-0 text-[10px] text-muted-foreground font-medium whitespace-nowrap",
                                "v{crate::APP_VERSION}"
                            }
                        }
                    }
                }

                div { class: "flex flex-1 items-center justify-between space-x-2 md:justify-end",
                    div { class: "w-full flex-1 md:w-auto md:flex-none" }
                    nav { class: "flex items-center space-x-2",
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            onclick: |_| {},
                            // Sun Icon
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
                                class: "h-5 w-5",
                                circle { cx: "12", cy: "12", r: "4" }
                                path { d: "M12 2v2" }
                                path { d: "M12 20v2" }
                                path { d: "m4.93 4.93 1.41 1.41" }
                                path { d: "m17.66 17.66 1.41 1.41" }
                                path { d: "M2 12h2" }
                                path { d: "M20 12h2" }
                                path { d: "m6.34 17.66-1.41 1.41" }
                                path { d: "m19.07 4.93-1.41 1.41" }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Default,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                async move {
                                    match destroy_session(reqwest_client_context).await {
                                        Ok(_) => {
                                            tracing::debug!("Successfully logged out");
                                        }
                                        Err(_) => {
                                            tracing::warn!(
                                                "Failed to cleanly destroy session, logging out locally anyway"
                                            );
                                        }
                                    }
                                    app_state.write().authenticated = false;
                                }
                            },
                            "Log Out"
                        }
                    }
                }
            }

            if is_menu_open() {
                div { class: "md:hidden border-t border-border bg-background p-4",
                    nav { class: "flex flex-col space-y-4",
                        for (text , to) in menu_entries.clone() {
                            MobileLink {
                                key: "{text}",
                                to,
                                onclick: move |_| is_menu_open.set(false),
                                "{text}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Link(to: Route, children: Element) -> Element {
    rsx! {
        dioxus::prelude::Link {
            class: "transition-colors hover:text-foreground/80 text-foreground/60",
            to,
            {children}
        }
    }
}

#[component]
fn MobileLink(to: Route, onclick: EventHandler<MouseEvent>, children: Element) -> Element {
    rsx! {
        dioxus::prelude::Link {
            class: "text-foreground hover:text-foreground/80 font-medium transition-colors",
            to,
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}
