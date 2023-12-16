use crate::{error::InternalHealthError, SqlitePlugin};
use async_trait::async_trait;
use plugins_definitions::{HealthError, Plugin, PluginMeta};

impl SqlitePlugin {
    async fn health_with_internal_error(&self) -> Result<(), InternalHealthError> {
        let _conn = self.pool.get().await?;

        Ok(())
    }
}

#[async_trait]
impl Plugin for SqlitePlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "sqlite",
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
