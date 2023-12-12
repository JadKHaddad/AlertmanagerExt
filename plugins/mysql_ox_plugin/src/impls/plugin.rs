use crate::MysqlOXPlugin;
use async_trait::async_trait;
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use sqlx::Connection;

#[async_trait]
impl Plugin for MysqlOXPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "mysql_ox",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        let mut conn = self.pool.acquire().await.map_err(|error| HealthError {
            error: error.into(),
        })?;

        conn.ping().await.map_err(|error| HealthError {
            error: error.into(),
        })?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}
