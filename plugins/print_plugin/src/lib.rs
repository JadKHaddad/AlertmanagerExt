use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use push_definitions::{InitializeError, Push, PushError};
use tokio::io::{self, AsyncWriteExt};

/// The `PrintType` enum is used to specify the type of printing that should be done
pub enum PrintType {
    /// Prints as a debug string
    Debug,
    /// Prints as a pretty string
    Pretty,
    /// Prints as a json object
    Json,
    /// Prints as a yaml object
    Yaml,
}

/// Configuration for the Print plugin
pub struct PrintPluginConfig {
    /// The type of printing to do
    pub print_type: PrintType,
}

/// Metadata for the Print plugin
pub struct PrintPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// The Print plugin
pub struct PrintPlugin {
    /// Meta information for the plugin
    meta: PrintPluginMeta,
    /// Configuration for the plugin
    config: PrintPluginConfig,
}

impl PrintPlugin {
    pub fn new(meta: PrintPluginMeta, config: PrintPluginConfig) -> Self {
        Self { meta, config }
    }
}

#[async_trait]
impl Plugin for PrintPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "print",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}

#[async_trait]
impl Push for PrintPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        let mut stdout = io::stdout();

        let mut bytes = match self.config.print_type {
            PrintType::Debug => format!("{:?}", alertmanager_push).into_bytes(),
            PrintType::Pretty => format!("{:#?}", alertmanager_push).into_bytes(),
            PrintType::Json => {
                serde_json::to_vec(alertmanager_push).map_err(|error| PushError {
                    reason: error.to_string(),
                })?
            }
            PrintType::Yaml => serde_yaml::to_string(alertmanager_push)
                .map_err(|error| PushError {
                    reason: error.to_string(),
                })?
                .into_bytes(),
        };

        bytes.push(b'\n');

        stdout.write_all(&bytes).await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
