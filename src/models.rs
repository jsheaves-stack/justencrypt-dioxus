use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Folder {
    pub file_name: String,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub created_date: Option<String>,
    #[serde(default)]
    pub last_modified_date: Option<String>,
    #[serde(default)]
    pub file_count: i64,
    #[serde(default)]
    pub folder_count: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct File {
    pub file_name: String,
    #[serde(default)]
    pub file_extension: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub created_date: Option<String>,
    #[serde(default)]
    pub last_modified_date: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum FolderContent {
    File(File),
    Folder(Folder),
}

impl FolderContent {
    pub fn is_file(&self) -> bool {
        matches!(self, FolderContent::File(_))
    }

    pub fn file_name(&self) -> &str {
        match self {
            FolderContent::File(f) => &f.file_name,
            FolderContent::Folder(f) => &f.file_name,
        }
    }

    pub fn file_size(&self) -> Option<i64> {
        match self {
            FolderContent::File(f) => f.file_size,
            FolderContent::Folder(f) => f.file_size,
        }
    }

    pub fn file_extension(&self) -> Option<String> {
        match self {
            FolderContent::File(f) => f.file_extension.clone(),
            FolderContent::Folder(_) => None,
        }
    }
}
