use crate::{
    error::{InternalInitializeError, InternalPushError},
    FilePlugin,
};
use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use std::path::PathBuf;

impl FilePlugin {
    fn decide_file_path(&self, alertmanager_push: &AlertmanagerPush) -> PathBuf {
        let dir_path = &self.config.dir_path;
        let mut file_path = dir_path.clone();
        file_path.push(format!(
            "{}.{}",
            alertmanager_push.group_key, self.config.extension
        ));

        file_path
    }

    fn initialize_with_internal_error(&mut self) -> Result<(), InternalInitializeError> {
        self.dir_exists()?;

        Ok(())
    }

    async fn push_alert_with_internal_error(
        &self,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let file_path = self.decide_file_path(alertmanager_push);

        let contents = self.formatter.format(alertmanager_push)?;

        tokio::fs::write(file_path, contents).await?;

        Ok(())
    }
}

#[async_trait]
impl Push for FilePlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        self.initialize_with_internal_error()?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        self.push_alert_with_internal_error(alertmanager_push)
            .await?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
