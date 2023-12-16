use deadpool_diesel::{sqlite::BuildError, PoolError};
use diesel::ConnectionError;
use plugins_definitions::HealthError;
use push_definitions::InitializeError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewSqlitePluginError {
    #[error("Failed to create pool: {0}")]
    Pool(
        #[source]
        #[from]
        BuildError,
    ),
}

#[derive(ThisError, Debug)]
pub enum InternalInitializeError {
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("Failed to establish connection: {0}")]
    Connection(
        #[from]
        #[source]
        ConnectionError,
    ),
    #[error("Failed to run migrations: {0}")]
    Migrations(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Failed to join task: {0}")]
    Join(
        #[from]
        #[source]
        tokio::task::JoinError,
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
pub enum InternalHealthError {
    #[error("Failed to get connection: {0}")]
    Connection(
        #[from]
        #[source]
        PoolError,
    ),
}

impl From<InternalHealthError> for HealthError {
    fn from(error: InternalHealthError) -> Self {
        Self {
            error: error.into(),
        }
    }
}
