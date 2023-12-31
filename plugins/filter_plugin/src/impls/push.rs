use crate::{error::InternalPushError, FilterPlugin};
use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};

impl FilterPlugin {
    async fn push_alert_with_internal_error(
        &self,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        if self.is_signature_present(alertmanager_push) {
            tracing::warn!("Signature present. Loop detected.");
            return Err(InternalPushError::LoopDetected)?;
        }

        let mut alertmanager_push = self.filter(alertmanager_push);

        self.add_signature(&mut alertmanager_push);

        let response = reqwest::Client::new()
            .post(self.config.webhook_url.clone())
            .json(&alertmanager_push)
            .send()
            .await?;

        let status_code = response.status();
        if !status_code.is_success() {
            let body = response.text().await?;

            return Err(InternalPushError::ErrorResponse { status_code, body });
        };

        Ok(())
    }
}

#[async_trait]
impl Push for FilterPlugin {
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
