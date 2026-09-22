use dioxus::prelude::*;
use std::cmp;

use crate::{models::FolderContent, SelectionState};

/// Builds the click / shift-click multi-select handler shared by the file and
/// folder grid items. `idx` is the item's position in `sorted_files` (both
/// callers already hold this list via context); shift-click extends the
/// selection to a contiguous range from the last selected item, a plain
/// click toggles just that one item.
pub fn selection_handler(
    mut selection_state: Signal<SelectionState>,
    sorted_files: Memo<Option<Vec<FolderContent>>>,
) -> impl FnMut((usize, bool)) + Copy {
    move |(idx, is_shift): (usize, bool)| {
        let mut state = selection_state.write();
        if let Some(files) = sorted_files.read().as_ref() {
            let item = &files[idx];
            let item_key = item.key();

            if is_shift {
                if let Some(last_idx) = state.last_selected_index {
                    let start = cmp::min(last_idx, idx);
                    let end = cmp::max(last_idx, idx);

                    for item in files.iter().take(end + 1).skip(start) {
                        state.selected_files.insert(item.key());
                    }
                } else {
                    state.selected_files.insert(item_key);
                    state.last_selected_index = Some(idx);
                }
            } else {
                if state.selected_files.contains(&item_key) {
                    state.selected_files.remove(&item_key);
                } else {
                    state.selected_files.insert(item_key);
                }
                state.last_selected_index = Some(idx);
            }
        }
    }
}
