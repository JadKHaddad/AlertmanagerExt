use async_trait::async_trait;
use models::AlertmanagerPush;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Plugin initialization failed: {reason}")]
pub struct InitializeError {
    pub reason: String,
}

#[derive(ThisError, Debug)]
#[error("Push failed: {reason}")]
pub struct PushError {
    pub reason: String,
}

#[async_trait]
pub trait Push: Send + Sync + 'static {
    /// Initialize on startup
    async fn initialize(&mut self) -> Result<(), InitializeError>;

    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError>;
}
