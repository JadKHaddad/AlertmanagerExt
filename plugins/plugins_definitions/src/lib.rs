use async_trait::async_trait;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Plugin initialization failed: {reason}")]
pub struct InitializeError {
    pub reason: String,
}

#[derive(ThisError, Debug)]
#[error("Plugin health check failed: {reason}")]
pub struct HealthError {
    pub reason: String,
}

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// For prometheus labels
    fn type_(&self) -> &str;

    /// For prometheus labels
    fn name(&self) -> &str;

    /// Initialize the plugin on startup
    async fn initialize(&self) -> Result<(), InitializeError>;

    /// Health check
    async fn health(&self) -> Result<(), HealthError>;
}
