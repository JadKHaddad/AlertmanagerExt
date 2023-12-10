use anyhow::{Context, Result as AnyResult};
use schemars::JsonSchema;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};

#[allow(clippy::enum_variant_names)]
mod entity;
mod entity_ext;
mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the PostgresSea plugin
pub struct PostgresSeaPluginConfig {
    /// Connection string for the Postgres database
    pub connection_string: String,
    /// Max number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Connection to the database
    db: DatabaseConnection,
}

impl PostgresSeaPlugin {
    pub async fn new(
        meta: PostgresSeaPluginMeta,
        config: PostgresSeaPluginConfig,
    ) -> AnyResult<Self> {
        let connect_options = ConnectOptions::new(&config.connection_string)
            .max_connections(config.max_connections)
            .connect_timeout(config.connection_timeout)
            .to_owned();

        let db = Database::connect(connect_options)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { meta, db })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use push_definitions::Push;
    use random_models_generator::generate_random_alertmanager_pushes;
    use tracing_test::traced_test;

    async fn create_and_init_plugin() -> PostgresSeaPlugin {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let postgres_sea_plugin_config = PostgresSeaPluginConfig {
            connection_string: database_url,
            max_connections: 5,
            connection_timeout: std::time::Duration::from_secs(5),
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
    // cargo test --package postgres_sea_plugin --lib --release -- test::push_random_alerts --exact --nocapture --ignored
    async fn push_random_alerts() {
        let plugin = create_and_init_plugin().await;
        let pushes = generate_random_alertmanager_pushes(100);
        for (i, push) in pushes.iter().enumerate() {
            tracing::info!("Pushing alert {}/{}", i + 1, pushes.len());
            if let Err(error) = plugin.push_alert(push).await {
                eprintln!("Failed to push alert: {:?}", error)
            }
        }
    }
}
