use crate::{error::InternalHealthError, FilePlugin};
use async_trait::async_trait;
use plugins_definitions::{HealthError, Plugin, PluginMeta};

impl FilePlugin {
    fn health_with_internal_error(&self) -> Result<(), InternalHealthError> {
        self.dir_exists()?;

        Ok(())
    }
}

#[async_trait]
impl Plugin for FilePlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "file",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        self.health_with_internal_error()?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}
