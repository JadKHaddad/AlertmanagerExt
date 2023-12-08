use crate::{SqlitePlugin, MIGRATIONS};
use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::MigrationHarness;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use tokio::task::JoinHandle;

#[async_trait]
impl Push for SqlitePlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        // Always be nice and give memory back to the OS. ;)
        let config = self.config.take().ok_or_else(|| InitializeError {
            reason: "Already initialized.".to_string(),
        })?;

        let connection_string = config.connection_string;
        let handle: JoinHandle<AnyResult<()>> = tokio::task::spawn_blocking(move || {
            let mut conn = SqliteConnection::establish(&connection_string)
                .context("Failed to establish connection")?;

            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|error| anyhow::anyhow!(error))
                .context("Failed to run migrations")?;

            Ok(())
        });

        handle
            .await
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?
            .map_err(|error| InitializeError {
                reason: format!("{:#}", error),
            })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        // TODO
        tracing::warn!("Not implemented yet.");

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
