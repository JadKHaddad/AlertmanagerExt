use formatter::{FormatError, NewFormatterError};
use push_definitions::PushError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewPrintPluginError {
    #[error("Failed to create formatter: {0}")]
    Formatter(
        #[source]
        #[from]
        NewFormatterError,
    ),
}

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Failed to format: {0}")]
    Format(
        #[source]
        #[from]
        FormatError,
    ),
    #[error("Failed to write to stdout: {0}")]
    Write(
        #[source]
        #[from]
        tokio::io::Error,
    ),
}

impl From<InternalPushError> for PushError {
    fn from(error: InternalPushError) -> Self {
        Self {
            error: error.into(),
        }
    }
}
