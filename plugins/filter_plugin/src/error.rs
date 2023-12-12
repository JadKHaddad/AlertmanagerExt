use push_definitions::PushError;
use regex::Error as RegexError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewFilterPluginError {
    #[error("Invalid regex: {0}")]
    Regex(
        #[source]
        #[from]
        RegexError,
    ),
}

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Loop detected")]
    LoopDetected,
    #[error("Reqwest error: {0}")]
    Reqwest(
        #[source]
        #[from]
        reqwest::Error,
    ),
    #[error("Got error response: status code: {status_code}, body: {body}")]
    ErrorResponse {
        status_code: reqwest::StatusCode,
        body: String,
    },
}

impl From<InternalPushError> for PushError {
    fn from(error: InternalPushError) -> Self {
        Self {
            reason: error.to_string(),
        }
    }
}
