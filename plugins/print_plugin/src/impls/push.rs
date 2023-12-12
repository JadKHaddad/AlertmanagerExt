use crate::{error::InternalPushError, PrintPlugin};
use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use tokio::io::{self, AsyncWriteExt};

impl PrintPlugin {
    async fn push_alert_with_internal_error(
        &self,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let mut stdout = io::stdout();

        let mut bytes = self.formatter.format(alertmanager_push)?.into_bytes();

        bytes.push(b'\n');

        stdout.write_all(&bytes).await?;

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

        self.push_alert_with_internal_error(alertmanager_push)
            .await?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
