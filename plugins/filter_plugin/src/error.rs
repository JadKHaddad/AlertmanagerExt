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
