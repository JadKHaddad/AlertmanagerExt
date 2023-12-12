use async_trait::async_trait;
use models::AlertmanagerPush;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Plugin initialization failed: {error}")]
pub struct InitializeError {
    #[from]
    #[source]
    pub error: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(ThisError, Debug)]
#[error("Push failed: {error}")]
pub struct PushError {
    #[from]
    #[source]
    pub error: Box<dyn std::error::Error + Send + Sync>,
}

#[async_trait]
pub trait Push: Send + Sync + 'static {
    /// Initialize on startup
    async fn initialize(&mut self) -> Result<(), InitializeError>;

    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError>;
}
