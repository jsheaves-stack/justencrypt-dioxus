use dioxus::prelude::*;

use crate::{
    components::ui::{
        card::Card,
        context_menu::{ContextMenuState, ContextMenuTarget},
        long_press::use_long_press,
        selection::selection_handler,
    },
    models::FolderContent,
    SelectionState,
};

/// Shared chrome for the file/folder grid cards: the long-press-to-select
/// gesture, the click-routing (shift/ctrl-click extends the selection, a
/// plain click with an active selection toggles this item, a plain click
/// with none active does whatever `on_open` says), the context-menu
/// wiring, the selection ring, and the checkbox overlay. Callers supply
/// only the type-specific visual body (a `CardContent` + `CardFooter`) as
/// `children`.
#[derive(Props, Clone, PartialEq)]
pub struct SelectableGridItemProps {
    /// The item this card represents, used for its selection key and to
    /// populate the right-click context menu.
    pub content: FolderContent,
    pub current_path: Vec<String>,
    /// This item's position in the sorted listing (see `selection_handler`).
    pub index: usize,
    /// Called on a plain click when nothing is currently selected - i.e.
    /// the "open" action: navigate into a folder, or open a media file's
    /// gallery viewer.
    pub on_open: EventHandler<()>,
    pub children: Element,
}

#[component]
pub fn SelectableGridItem(props: SelectableGridItemProps) -> Element {
    let mut context_menu = use_context::<Signal<ContextMenuState>>();
    let selection_state = use_context::<Signal<SelectionState>>();
    let sorted_files = use_context::<Memo<Option<Vec<FolderContent>>>>();

    let mut on_select = selection_handler(selection_state, sorted_files);
    let mut long_press = use_long_press();

    let key = props.content.key();
    let index = props.index;
    let on_open = props.on_open;

    let content_for_menu = props.content.clone();
    let current_path_for_menu = props.current_path.clone();

    rsx! {
        div {
            class: "flex justify-center h-40 sm:h-48 md:h-56 w-full cursor-pointer relative group",
            // Grid items are never unmounted as the list grows (see
            // `display_count` in file_explorer.rs), so a large, scrolled-far
            // folder can leave hundreds of these - each with hover
            // transitions, blurred/glowing badges, and an <img> - sitting in
            // the DOM at once. `content-visibility: auto` tells the renderer
            // to skip layout/paint/hit-testing entirely for instances far
            // from the viewport, which is what actually bounds the paint
            // cost rather than just how many are technically mounted.
            // `contain-intrinsic-size` gives it a placeholder size to use
            // before it's ever been rendered (the card is h-40/48/56 - this
            // just needs to be a reasonable estimate, not exact: the `auto`
            // prefix has the browser remember the real size after the first
            // time this item is actually visible). A no-op, not a functional
            // change, wherever `content-visibility` isn't supported.
            style: "content-visibility: auto; contain-intrinsic-size: auto 220px;",
            onmousedown: move |_| long_press.start(move || on_select((index, false))),
            onmouseup: move |_| long_press.cancel(),
            onmouseleave: move |_| long_press.cancel(),
            oncontextmenu: move |e| {
                e.prevent_default();
                e.stop_propagation();
                let coords = e.client_coordinates();
                tracing::debug!("Item right-clicked at {}, {}", coords.x, coords.y);
                let mut state = context_menu.write();
                state.target = ContextMenuTarget::Item(
                    content_for_menu.clone(),
                    current_path_for_menu.clone(),
                );
                state.position = (coords.x, coords.y);
            },
            Card {
                class: format!(
                    "w-full h-full grid grid-rows-[1fr,auto] active:scale-95 transition duration-300 hover:shadow-lg group select-none {}",
                    if selection_state.read().selected_files.contains(&key) {
                        "ring-2 ring-primary bg-primary/5"
                    } else {
                        ""
                    },
                ),
                onclick: move |e: Event<MouseData>| {
                    if long_press.consume() {
                        e.stop_propagation();
                        return;
                    }

                    let has_selection = !selection_state.read().selected_files.is_empty();

                    if e.modifiers().contains(Modifiers::SHIFT)
                        || e.modifiers().contains(Modifiers::CONTROL)
                    {
                        e.stop_propagation();
                        on_select((index, e.modifiers().contains(Modifiers::SHIFT)));
                    } else if has_selection {
                        e.stop_propagation();
                        on_select((index, false));
                    } else {
                        e.stop_propagation();
                        on_open.call(());
                    }
                },
                {props.children}
            }
            div {
                class: format!(
                    "absolute top-2 right-2 z-10 transition-opacity duration-200 {}",
                    if selection_state.read().selected_files.contains(&key) {
                        "opacity-100"
                    } else {
                        "opacity-0"
                    },
                ),
                input {
                    r#type: "checkbox",
                    class: "w-5 h-5 rounded border-gray-300 text-primary focus:ring-primary cursor-pointer",
                    checked: selection_state.read().selected_files.contains(&key),
                    onclick: move |e| {
                        e.stop_propagation();
                        on_select((index, false));
                    },
                }
            }
        }
    }
}
