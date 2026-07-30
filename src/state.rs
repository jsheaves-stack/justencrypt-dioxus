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

/// The target of a delete operation: (list of (filename, is_file), current_path)
pub type DeleteModalTarget = (Vec<(String, bool)>, Vec<String>);

/// Represents the various modals that can be active in the application.
#[derive(Clone, PartialEq, Debug)]
pub enum AppModal {
    /// Modal to create a new folder
    CreateFolder,
    /// Modal to create a new empty file
    CreateFile,
    /// Modal to edit a file's contents: (file_name, current_path)
    EditFile(String, Vec<String>),
    /// Modal to rename an item: (old_name, current_path, is_file)
    Rename(String, Vec<String>, bool),
    /// Modal to delete items
    Delete(DeleteModalTarget),
    /// Photo/Video viewer modal: (file_name, current_path)
    PhotoViewer(String, Vec<String>),
    /// Details/Properties modal for an item: (item_details, current_path)
    Details(crate::models::FolderContent, Vec<String>),
    /// Modal to move items: (items_to_move, current_path)
    Move(Vec<(String, bool)>, Vec<String>),
}

impl AppModal {
    /// Checks if this modal requires resetting if the directory path changes.
    /// This prevents modals from staying open on stale paths.
    pub fn is_path_stale(&self, active_path: &[String]) -> bool {
        match self {
            AppModal::EditFile(_, p) => p != active_path,
            AppModal::Rename(_, p, _) => p != active_path,
            AppModal::Delete((_, p)) => p != active_path,
            AppModal::PhotoViewer(_, p) => p != active_path,
            AppModal::Details(_, p) => p != active_path,
            AppModal::Move(_, p) => p != active_path,
            AppModal::CreateFolder | AppModal::CreateFile => true,
        }
    }
}

/// Global application state.
pub struct AppState {
    /// User's chosen theme (e.g. "dark" or "light")
    pub _theme: String,
    /// Authentication status
    pub authenticated: bool,
    /// Currently active modal, if any. Only one modal can be active at a time.
    pub active_modal: Option<AppModal>,
    /// Triggers a re-fetch of the current folder's contents
    pub needs_refetch: bool,
    /// Currently selected files in the explorer: HashSet of (file_name, is_file)
    pub selected_files: HashSet<(String, bool)>,
    /// Index of the last selected item (used for shift-click range selection)
    pub last_selected_index: Option<usize>,
}

impl AppState {
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

/// Shared Reqwest client used for API calls.
#[derive(Clone)]
pub struct ReqwestClient(pub Client);
