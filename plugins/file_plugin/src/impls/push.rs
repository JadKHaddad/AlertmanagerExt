use crate::FilePlugin;
use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};

#[async_trait]
impl Push for FilePlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        self.dir_exists().map_err(|error| InitializeError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        let file_path = self.decide_file_path(alertmanager_push);

        let contents = self
            .to_string(alertmanager_push)
            .map_err(|error| PushError {
                reason: error.to_string(),
            })?;

        tokio::fs::write(file_path, contents)
            .await
            .map_err(|error| PushError {
                reason: error.to_string(),
            })?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
