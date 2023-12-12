use crate::MongoPlugin;
use async_trait::async_trait;
use mongodb::bson::doc;
use plugins_definitions::{HealthError, Plugin, PluginMeta};

#[async_trait]
impl Plugin for MongoPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "mongo",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        self.database()
            .run_command(doc! { "ping": 1 }, None)
            .await
            .map_err(|error| HealthError {
                error: error.into(),
            })?;

        tracing::trace!("Successfully checked health.");

        Ok(())
    }
}
