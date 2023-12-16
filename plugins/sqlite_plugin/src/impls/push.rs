use crate::{error::InternalInitializeError, SqlitePlugin, MIGRATIONS};
use async_trait::async_trait;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::MigrationHarness;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use tokio::task::JoinHandle;

impl SqlitePlugin {
    async fn initialize_with_internal_error(&mut self) -> Result<(), InternalInitializeError> {
        // Always be nice and give memory back to the OS. ;)
        let config = self
            .config
            .take()
            .ok_or_else(|| InternalInitializeError::AlreadyInitialized)?;

        let connection_string = config.connection_string;
        let handle: JoinHandle<Result<(), InternalInitializeError>> =
            tokio::task::spawn_blocking(move || {
                let mut conn = SqliteConnection::establish(&connection_string)?;

                conn.run_pending_migrations(MIGRATIONS)
                    .map_err(InternalInitializeError::Migrations)?;

                Ok(())
            });

        handle.await??;

        Ok(())
    }
}

#[async_trait]
impl Push for SqlitePlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        self.initialize_with_internal_error().await?;

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
