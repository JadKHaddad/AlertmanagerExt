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
    fn name(&self) -> &str;

    async fn initialize(&self) -> Result<(), InitializeError>;

    async fn health(&self) -> Result<(), HealthError>;
}
