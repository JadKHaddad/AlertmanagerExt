use std::path::PathBuf;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum DirError {
    #[error("Directory does not exist. path: {path}")]
    DoesNotExist { path: PathBuf },
    #[error("Path is not a directory. path: {path}")]
    NotADirectory { path: PathBuf },
}

#[derive(ThisError, Debug)]
#[error("Failed to serialize: {reason}")]
pub struct SerializeError {
    pub reason: String,
}
