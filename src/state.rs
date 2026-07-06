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

pub struct AppState {
    pub _theme: String,
    pub authenticated: bool,
    pub create_folder_modal: bool,
    pub create_file_modal: bool,
    pub edit_file_modal: Option<(String, Vec<String>)>,
    pub rename_modal: Option<(String, Vec<String>, bool)>,
    pub delete_modal: Option<(String, Vec<String>, bool)>,
    pub photo_viewer_modal: Option<(String, Vec<String>)>,
    pub details_modal: Option<(crate::components::file_explorer::FolderItem, Vec<String>)>,
    pub needs_refetch: bool,
}

#[derive(Clone)]
pub struct ReqwestClient(pub Client);
