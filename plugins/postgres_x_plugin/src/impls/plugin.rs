use crate::{error::InternalHealthError, PostgresXPlugin};
use async_trait::async_trait;
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use sqlx::Connection;

impl PostgresXPlugin {
    async fn health_with_internal_error(&self) -> Result<(), InternalHealthError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(InternalHealthError::Acquire)?;

        conn.ping().await.map_err(InternalHealthError::Ping)?;

        Ok(())
    }
}

#[async_trait]
impl Plugin for PostgresXPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "postgres_x",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        self.health_with_internal_error().await?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}
