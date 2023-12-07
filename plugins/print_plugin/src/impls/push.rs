use crate::{PrintPlugin, PrintType};
use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use tokio::io::{self, AsyncWriteExt};

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
