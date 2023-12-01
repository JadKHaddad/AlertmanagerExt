use async_trait::async_trait;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Plugin health check failed: {reason}")]
/// Error returned by the health check
pub struct HealthError {
    pub reason: String,
}

#[derive(Debug)]
/// Meta information about the plugin
///
/// See [`OwnedPluginMeta`].
pub struct PluginMeta<'a> {
    /// Name of the plugin
    ///
    /// Used to identify the plugin among others of the same type.
    pub name: &'a str,
    /// Type of plugin
    ///
    /// Used to identify the plugin.
    pub type_: &'static str,
    /// Group of the plugin
    ///
    /// Multiple plugins can be grouped together.
    pub group: &'a str,
}

#[derive(Debug, Clone)]
/// Owned version of [`PluginMeta`]
pub struct OwnedPluginMeta {
    pub name: String,
    pub type_: &'static str,
    pub group: String,
}

impl From<PluginMeta<'_>> for OwnedPluginMeta {
    fn from(meta: PluginMeta) -> Self {
        Self {
            name: meta.name.to_owned(),
            type_: meta.type_,
            group: meta.group.to_owned(),
        }
    }
}

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Meta information about the plugin
    fn meta(&self) -> PluginMeta;

    /// Owned meta information about the plugin
    fn meta_owned(&self) -> OwnedPluginMeta {
        self.meta().into()
    }

    /// Name of the plugin
    fn name(&self) -> &str {
        self.meta().name
    }

    /// Type of plugin
    fn type_(&self) -> &str {
        self.meta().type_
    }

    /// Group of the plugin
    fn group(&self) -> &str {
        self.meta().group
    }

    /// Health check
    async fn health(&self) -> Result<(), HealthError>;
}
