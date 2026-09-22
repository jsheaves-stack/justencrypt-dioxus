use dioxus::prelude::*;

use crate::{requests::RequestError, AppState, ModalState, SelectionState};

/// Finishes a modal's single save/rename/create request: on success, closes
/// the modal and flags a refetch; on failure, surfaces `action`'s error
/// message. Shared by every modal that fires exactly one request per submit,
/// so the close-vs-error branching only lives in one place.
pub fn finish_modal_action(
    mut app_state: Signal<AppState>,
    mut modal_state: Signal<ModalState>,
    mut is_saving: Signal<bool>,
    mut error_msg: Signal<Option<String>>,
    result: Result<(), RequestError>,
    action: &str,
) {
    is_saving.set(false);
    match result {
        Ok(_) => {
            modal_state.write().close_all_modals();
            app_state.write().needs_refetch = true;
        }
        Err(e) => {
            error_msg.set(Some(e.user_message(action)));
        }
    }
}

/// Finishes a bulk delete/move: on success, closes the modal, clears the
/// selection, and flags a refetch; on failure, shows either that one item's
/// own error message or a generic bulk message, depending on how many items
/// were targeted. Shared by the delete and move modals, which otherwise only
/// differ in the request they fire per item.
#[allow(clippy::too_many_arguments)]
pub fn finish_bulk_modal_action(
    mut app_state: Signal<AppState>,
    mut modal_state: Signal<ModalState>,
    mut selection_state: Signal<SelectionState>,
    mut is_saving: Signal<bool>,
    mut error_msg: Signal<Option<String>>,
    first_error: Option<RequestError>,
    items_count: usize,
    action: &str,
    bulk_error_message: &str,
) {
    is_saving.set(false);
    match first_error {
        Some(e) if items_count == 1 => {
            error_msg.set(Some(e.user_message(action)));
        }
        Some(_) => {
            error_msg.set(Some(bulk_error_message.to_string()));
        }
        None => {
            selection_state.write().selected_files.clear();
            modal_state.write().close_all_modals();
            app_state.write().needs_refetch = true;
        }
    }
}
