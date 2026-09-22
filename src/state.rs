use crate::models::ItemKey;
use reqwest::Client;
use std::collections::HashSet;

/// Status of an individual file upload.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Done,
    Error,
}

/// Represents a single file upload task.
#[derive(Clone, PartialEq, Debug)]
pub struct UploadTask {
    /// Name of the file being uploaded
    pub file_name: String,
    /// Current upload status
    pub status: UploadStatus,
    /// Upload progress percentage (0.0 to 100.0)
    pub progress: f64,
}

/// Global state tracking file uploads across the app.
#[derive(Clone, PartialEq, Debug)]
pub struct UploadState {
    /// List of all active and completed upload tasks
    pub tasks: Vec<UploadTask>,
    /// Maximum number of concurrent uploads allowed
    pub max_concurrent: usize,
}

impl Default for UploadState {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            max_concurrent: 10,
        }
    }
}

/// The target of a delete operation: (items to delete, current_path)
pub type DeleteModalTarget = (Vec<ItemKey>, Vec<String>);

/// Represents the various modals that can be active in the application.
#[derive(Clone, PartialEq, Debug)]
pub enum AppModal {
    /// Modal to create a new folder
    CreateFolder,
    /// Modal to create a new empty file
    CreateFile,
    /// Modal to edit a file's contents: (file_name, current_path)
    EditFile(String, Vec<String>),
    /// Modal to rename an item: (item, current_path)
    Rename(ItemKey, Vec<String>),
    /// Modal to delete items
    Delete(DeleteModalTarget),
    /// Image/video gallery: (file_name, current_path)
    Gallery(String, Vec<String>),
    /// Details/Properties modal for an item: (item_details, current_path)
    Details(crate::models::FolderContent, Vec<String>),
    /// Modal to move items: (items_to_move, current_path)
    Move(Vec<ItemKey>, Vec<String>),
}

impl AppModal {
    /// Checks if this modal requires resetting if the directory path changes.
    /// This prevents modals from staying open on stale paths.
    pub fn is_path_stale(&self, active_path: &[String]) -> bool {
        match self {
            AppModal::EditFile(_, p) => p != active_path,
            AppModal::Rename(_, p) => p != active_path,
            AppModal::Delete((_, p)) => p != active_path,
            AppModal::Gallery(_, p) => p != active_path,
            AppModal::Details(_, p) => p != active_path,
            AppModal::Move(_, p) => p != active_path,
            AppModal::CreateFolder | AppModal::CreateFile => true,
        }
    }
}

/// Which modal, if any, is currently open. Kept as its own signal/context
/// (rather than a field of `AppState`) so opening or closing a modal doesn't
/// re-render components that only care about auth, theme, or selection -
/// Dioxus signals track reads at the whole-value granularity, so bundling
/// unrelated state into one struct means a write to any field invalidates
/// every reader of any other field.
#[derive(Default)]
pub struct ModalState {
    /// Currently active modal, if any. Only one modal can be active at a time.
    pub active_modal: Option<AppModal>,
}

impl ModalState {
    /// Checks if any modal is currently active.
    pub fn is_any_modal_open(&self) -> bool {
        self.active_modal.is_some()
    }

    /// Closes the currently active modal.
    pub fn close_all_modals(&mut self) {
        self.active_modal = None;
    }

    /// Closes the modal if the current path has changed.
    pub fn close_modals_if_path_changed(&mut self, active_path: &[String]) {
        if let Some(modal) = &self.active_modal {
            if modal.is_path_stale(active_path) {
                self.active_modal = None;
            }
        }
    }
}

/// Multi-select state for the file/folder grid. Kept separate from
/// `AppState` for the same reactivity reason as `ModalState` above -
/// selecting/deselecting items shouldn't re-render components that only
/// care about auth, theme, or the active modal.
#[derive(Default)]
pub struct SelectionState {
    /// Currently selected files in the explorer
    pub selected_files: HashSet<ItemKey>,
    /// Index of the last selected item (used for shift-click range selection)
    pub last_selected_index: Option<usize>,
}

/// Global application state that doesn't warrant its own signal.
pub struct AppState {
    /// User's chosen theme (e.g. "dark" or "light")
    pub _theme: String,
    /// Authentication status
    pub authenticated: bool,
    /// Triggers a re-fetch of the current folder's contents
    pub needs_refetch: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            _theme: "dark".to_string(),
            authenticated: false,
            needs_refetch: false,
        }
    }
}

/// Severity of a toast notification, used to pick its styling.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ToastKind {
    Error,
    Success,
    Info,
}

/// A single toast notification.
#[derive(Clone, PartialEq, Debug)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
}

/// App-wide toast notifications, for actions with no other place to report
/// success or failure - there's no modal or inline error slot for e.g.
/// logging out or downloading a file from the context menu, so without this
/// those failures were previously silent (logged, but never shown).
#[derive(Default)]
pub struct ToastState {
    pub toasts: Vec<Toast>,
    next_id: u64,
}

impl ToastState {
    /// Queues a new toast and returns its id (for `dismiss`).
    pub fn push(&mut self, message: impl Into<String>, kind: ToastKind) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(Toast {
            id,
            message: message.into(),
            kind,
        });
        id
    }

    /// Removes a toast by id; a no-op if it's already gone (e.g. dismissed
    /// manually before its auto-dismiss timer fired).
    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }
}

/// Shared Reqwest client used for API calls.
#[derive(Clone)]
pub struct ReqwestClient(pub Client);

#[cfg(test)]
mod tests {
    use super::*;

    fn path(segments: &[&str]) -> Vec<String> {
        segments.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn create_folder_and_create_file_are_always_stale() {
        // These have no path of their own, so they're always dropped on
        // navigation rather than compared against it.
        assert!(AppModal::CreateFolder.is_path_stale(&path(&["a"])));
        assert!(AppModal::CreateFile.is_path_stale(&[]));
    }

    #[test]
    fn modal_with_matching_path_is_not_stale() {
        let modal = AppModal::EditFile("f.txt".to_string(), path(&["a", "b"]));
        assert!(!modal.is_path_stale(&path(&["a", "b"])));
    }

    #[test]
    fn modal_with_different_path_is_stale() {
        let modal = AppModal::EditFile("f.txt".to_string(), path(&["a", "b"]));
        assert!(modal.is_path_stale(&path(&["a"])));
    }

    #[test]
    fn close_modals_if_path_changed_closes_stale_modal() {
        let mut state = ModalState {
            active_modal: Some(AppModal::EditFile("f.txt".to_string(), path(&["a"]))),
        };
        state.close_modals_if_path_changed(&path(&["b"]));
        assert_eq!(state.active_modal, None);
    }

    #[test]
    fn close_modals_if_path_changed_keeps_current_modal() {
        let mut state = ModalState {
            active_modal: Some(AppModal::EditFile("f.txt".to_string(), path(&["a"]))),
        };
        state.close_modals_if_path_changed(&path(&["a"]));
        assert!(state.active_modal.is_some());
    }

    #[test]
    fn toast_push_assigns_increasing_ids() {
        let mut toasts = ToastState::default();
        let first = toasts.push("one", ToastKind::Info);
        let second = toasts.push("two", ToastKind::Error);
        assert_ne!(first, second);
        assert_eq!(toasts.toasts.len(), 2);
    }

    #[test]
    fn toast_dismiss_removes_only_the_matching_toast() {
        let mut toasts = ToastState::default();
        let keep = toasts.push("keep", ToastKind::Success);
        let remove = toasts.push("remove", ToastKind::Error);
        toasts.dismiss(remove);
        assert_eq!(toasts.toasts.len(), 1);
        assert_eq!(toasts.toasts[0].id, keep);
    }

    #[test]
    fn toast_dismiss_of_unknown_id_is_a_no_op() {
        let mut toasts = ToastState::default();
        toasts.push("one", ToastKind::Info);
        toasts.dismiss(999);
        assert_eq!(toasts.toasts.len(), 1);
    }
}
