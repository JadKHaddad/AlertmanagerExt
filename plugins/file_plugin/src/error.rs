use plugins_definitions::HealthError;
use push_definitions::{InitializeError, PushError};
use std::path::PathBuf;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewFilePluginError {
    #[error("Failed to create jinja renderer: {0}")]
    JinjaRenderer(
        #[source]
        #[from]
        jinja_renderer::NewJinjaRendererError,
    ),
}

#[derive(ThisError, Debug)]
pub enum DirError {
    #[error("Directory does not exist: path: {path}")]
    DoesNotExist { path: PathBuf },
    #[error("Path is not a directory: path: {path}")]
    NotADirectory { path: PathBuf },
}

#[derive(ThisError, Debug)]
pub enum InternalHealthError {
    #[error("Directory error: {0}")]
    Dir(
        #[source]
        #[from]
        DirError,
    ),
}

impl From<InternalHealthError> for HealthError {
    fn from(error: InternalHealthError) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum InternalInitializeError {
    #[error("Directory error: {0}")]
    Dir(
        #[source]
        #[from]
        DirError,
    ),
}

impl From<InternalInitializeError> for InitializeError {
    fn from(error: InternalInitializeError) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum FormatError {
    #[error("Failed to convert to json: {0}")]
    Json(
        #[source]
        #[from]
        serde_json::Error,
    ),
    #[error("Failed to convert to yaml: {0}")]
    Yaml(
        #[source]
        #[from]
        serde_yaml::Error,
    ),
    #[error("Failed to render template: {0}")]
    JinjaRender(
        #[source]
        #[from]
        jinja_renderer::RenderError,
    ),
    #[error("Jinja renderer not initialized")]
    JinjaUninitialized,
}

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Failed to write to file: {0}")]
    Write(
        #[source]
        #[from]
        tokio::io::Error,
    ),
    #[error("Failed to format: {0}")]
    Format(
        #[source]
        #[from]
        FormatError,
    ),
}

impl From<InternalPushError> for PushError {
    fn from(error: InternalPushError) -> Self {
        Self {
            error: error.into(),
        }
    }
}
