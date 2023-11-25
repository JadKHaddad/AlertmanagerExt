use async_trait::async_trait;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Plugin health check failed: {reason}")]
pub struct HealthError {
    pub reason: String,
}

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Type of plugin
    ///
    /// Used to identify the plugin
    fn type_(&self) -> &str;

    /// Name of the plugin
    ///
    /// Used to identify the plugin among others of the same type
    fn name(&self) -> &str;

    /// Group of the plugin
    ///
    /// Multiple plugins can be grouped together
    fn group(&self) -> &str;

    /// Health check
    async fn health(&self) -> Result<(), HealthError>;
}
