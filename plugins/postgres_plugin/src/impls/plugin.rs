use crate::PostgresPlugin;
use async_trait::async_trait;
use plugins_definitions::{HealthError, Plugin, PluginMeta};

#[async_trait]
impl Plugin for PostgresPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "postgres",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        let _conn = self.pool.get().await.map_err(|error| HealthError {
            error: error.into(),
        })?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}
