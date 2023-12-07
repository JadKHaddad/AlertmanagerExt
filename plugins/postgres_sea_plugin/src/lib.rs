use anyhow::Result as AnyResult;
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use models::AlertmanagerPush;
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use push_definitions::{InitializeError, Push, PushError};
use sea_orm::{ConnectOptions, Database};

#[allow(clippy::enum_variant_names)]
mod entity;

/// Configuration for the PostgresSea plugin
pub struct PostgresSeaPluginConfig {
    /// Connection string for the Postgres database
    pub connection_string: String,
}

/// Metadata for the PostgresSea plugin
pub struct PostgresSeaPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// PostgresSea plugin
///
/// Based on [`sea_orm`].
pub struct PostgresSeaPlugin {
    /// Meta information for the plugin
    meta: PostgresSeaPluginMeta,
    /// Configuration for the plugin
    config: Option<Box<PostgresSeaPluginConfig>>,
}

impl PostgresSeaPlugin {
    pub async fn new(
        meta: PostgresSeaPluginMeta,
        config: PostgresSeaPluginConfig,
    ) -> AnyResult<Self> {
        Ok(Self {
            meta,
            config: Some(Box::new(config)),
        })
    }
}

#[async_trait]
impl Plugin for PostgresSeaPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "postgres_sea",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        // TODO
        tracing::warn!("Not implemented.");

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}

#[async_trait]
impl Push for PostgresSeaPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        let config = self.config.take().ok_or_else(|| InitializeError {
            reason: "Already initialized.".to_string(),
        })?;

        let connection_string = config.connection_string;

        let connect_options = ConnectOptions::new(connection_string).to_owned();

        let db = Database::connect(connect_options)
            .await
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?;

        Migrator::up(&db, None)
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

        // TODO
        tracing::warn!("Not implemented.");

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use tracing_test::traced_test;

    async fn create_and_init_plugin() -> PostgresSeaPlugin {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let postgres_sea_plugin_config = PostgresSeaPluginConfig {
            connection_string: database_url,
        };

        let postgres_sea_plugin_meta = PostgresSeaPluginMeta {
            name: String::from("postgres_sea_plugin_1"),
            group: String::from("default"),
        };

        let mut postgres_sea_plugin =
            PostgresSeaPlugin::new(postgres_sea_plugin_meta, postgres_sea_plugin_config)
                .await
                .expect("Failed to create PostgresSea plugin.");

        postgres_sea_plugin
            .initialize()
            .await
            .expect("Failed to initialize PostgresSea plugin.");

        postgres_sea_plugin
    }

    #[ignore]
    #[tokio::test]
    #[traced_test]
    async fn initialize() {
        let _ = create_and_init_plugin().await;
    }
}
