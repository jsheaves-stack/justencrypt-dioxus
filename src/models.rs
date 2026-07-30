use serde::{Deserialize, Serialize};

/// Represents a folder in the remote file system.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Folder {
    /// Name of the folder
    pub file_name: String,
    /// Total aggregate size of all files contained inside the folder (if computed)
    #[serde(default)]
    pub file_size: Option<i64>,
    /// Date when the folder was created
    #[serde(default)]
    pub created_date: Option<String>,
    /// Date when the folder was last modified
    #[serde(default)]
    pub last_modified_date: Option<String>,
    /// Total number of files directly or indirectly in this folder
    #[serde(default)]
    pub file_count: i64,
    /// Total number of sub-folders directly or indirectly in this folder
    #[serde(default)]
    pub folder_count: i64,
}

/// Represents a file in the remote file system.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct File {
    /// Name of the file with extension
    pub file_name: String,
    /// The extension of the file (e.g. "txt", "mp4")
    #[serde(default)]
    pub file_extension: Option<String>,
    /// Size of the file in bytes
    #[serde(default)]
    pub file_size: Option<i64>,
    /// Date when the file was created
    #[serde(default)]
    pub created_date: Option<String>,
    /// Date when the file was last modified
    #[serde(default)]
    pub last_modified_date: Option<String>,
}

/// An abstraction over items returned by the file system API, which can be either a `File` or a `Folder`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum FolderContent {
    File(File),
    Folder(Folder),
}

impl FolderContent {
    /// Check if this item is a File.
    pub fn is_file(&self) -> bool {
        matches!(self, FolderContent::File(_))
    }

    /// Retrieve the name of the file or folder.
    pub fn file_name(&self) -> &str {
        match self {
            FolderContent::File(f) => &f.file_name,
            FolderContent::Folder(f) => &f.file_name,
        }
    }

    /// Retrieve the size of the file or folder.
    pub fn file_size(&self) -> Option<i64> {
        match self {
            FolderContent::File(f) => f.file_size,
            FolderContent::Folder(f) => f.file_size,
        }
    }

    /// Retrieve the extension of the file, if it is a File.
    pub fn file_extension(&self) -> Option<String> {
        match self {
            FolderContent::File(f) => f.file_extension.clone(),
            FolderContent::Folder(_) => None,
        }
    }
}
