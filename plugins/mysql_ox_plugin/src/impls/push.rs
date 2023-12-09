use crate::{database::models::groups::InsertGroup, error::InternalPushError, MysqlOXPlugin};
use async_trait::async_trait;
use models::AlertmanagerPush;
use ormx::Insert;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use sqlx::Connection;

#[async_trait]
impl Push for MysqlOXPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        sqlx::migrate!()
            .run(&self.pool)
            .await
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(InternalPushError::Acquire)?;

        tracing::trace!("Beginning transaction.");
        let mut tx = conn
            .begin()
            .await
            .map_err(InternalPushError::TransactionBegin)?;

        let group_id = InsertGroup {
            group_key: alertmanager_push.group_key.clone(),
            receiver: alertmanager_push.receiver.clone(),
            status: alertmanager_push.status.clone().to_string(),
            external_url: alertmanager_push.external_url.clone(),
        }
        .insert(&mut tx)
        .await
        .map_err(|error| InternalPushError::GroupInsertion {
            group_key: alertmanager_push.group_key.clone(),
            error,
        })?
        .id;

        println!("group_id: {}", group_id);

        tracing::trace!("Committing transaction.");
        tx.commit()
            .await
            .map_err(InternalPushError::TransactionCommit)?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
