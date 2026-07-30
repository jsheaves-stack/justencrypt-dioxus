use reqwest::Client;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Done,
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UploadTask {
    pub file_name: String,
    pub status: UploadStatus,
    pub progress: f64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UploadState {
    pub tasks: Vec<UploadTask>,
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

pub type DeleteModalTarget = (Vec<(String, bool)>, Vec<String>);

pub struct AppState {
    pub _theme: String,
    pub authenticated: bool,
    pub create_folder_modal: bool,
    pub create_file_modal: bool,
    pub edit_file_modal: Option<(String, Vec<String>)>,
    pub rename_modal: Option<(String, Vec<String>, bool)>,
    pub delete_modal: Option<DeleteModalTarget>,
    pub photo_viewer_modal: Option<(String, Vec<String>)>,
    pub details_modal: Option<(crate::models::FolderContent, Vec<String>)>,
    pub move_modal: Option<(Vec<String>, Vec<String>)>,
    pub needs_refetch: bool,
    pub selected_files: std::collections::HashSet<(String, bool)>,
    pub last_selected_index: Option<usize>,
}

impl AppState {
    pub fn is_any_modal_open(&self) -> bool {
        self.create_folder_modal
            || self.create_file_modal
            || self.edit_file_modal.is_some()
            || self.rename_modal.is_some()
            || self.delete_modal.is_some()
            || self.photo_viewer_modal.is_some()
            || self.details_modal.is_some()
            || self.move_modal.is_some()
    }

    pub fn close_all_modals(&mut self) {
        self.create_folder_modal = false;
        self.create_file_modal = false;
        self.edit_file_modal = None;
        self.rename_modal = None;
        self.delete_modal = None;
        self.photo_viewer_modal = None;
        self.details_modal = None;
        self.move_modal = None;
    }
}

#[derive(Clone)]
pub struct ReqwestClient(pub Client);
