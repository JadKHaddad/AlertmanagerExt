use async_trait::async_trait;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Plugin health check failed: {reason}")]
/// Error returned by the health check
pub struct HealthError {
    pub reason: String,
}

#[derive(Debug, Clone)]
/// Meta information about the plugin
pub struct PluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Type of the plugin
    pub type_: &'static str,
    /// Group of the plugin
    pub group: String,
}

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Meta information about the plugin
    fn meta(&self) -> PluginMeta;

    /// Health check
    async fn health(&self) -> Result<(), HealthError>;
}
