use anyhow::{Context, Result as AnyResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod database;
mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the MysqlOX plugin
pub struct MysqlOXPluginConfig {
    /// Connection string for the MysqlOX database
    pub connection_string: String,
    /// Max number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Metadata for the MysqlOX plugin
pub struct MysqlOXPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// MysqlOX plugin
///
/// Based on [`sqlx`] and [`ormx`].
pub struct MysqlOXPlugin {
    /// Meta information for the plugin
    meta: MysqlOXPluginMeta,
    /// Pool of connections to the database
    pool: sqlx::Pool<sqlx::MySql>,
}

impl MysqlOXPlugin {
    pub async fn new(meta: MysqlOXPluginMeta, config: MysqlOXPluginConfig) -> AnyResult<Self> {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .connect_timeout(config.connection_timeout)
            .connect(&config.connection_string)
            .await
            .context("Failed to create pool")?;

        Ok(Self { meta, pool })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use models::utils::generate_random_alertmanager_pushes;
    use push_definitions::Push;
    use tracing_test::traced_test;

    async fn create_and_init_plugin() -> MysqlOXPlugin {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let mysql_ox_plugin_config = MysqlOXPluginConfig {
            connection_string: database_url,
            max_connections: 15,
            connection_timeout: std::time::Duration::from_secs(5),
        };

        let mysql_ox_plugin_meta = MysqlOXPluginMeta {
            name: String::from("mysql_ox_plugin_1"),
            group: String::from("default"),
        };

        let mut mysql_ox_plugin = MysqlOXPlugin::new(mysql_ox_plugin_meta, mysql_ox_plugin_config)
            .await
            .expect("Failed to create MysqlOX plugin.");

        mysql_ox_plugin
            .initialize()
            .await
            .expect("Failed to initialize MysqlOX plugin.");

        mysql_ox_plugin
    }

    #[ignore]
    #[tokio::test]
    #[traced_test]
    // cargo test --package mysql_ox_plugin --lib -- test::init_plugin --exact --nocapture --ignored
    async fn init_plugin() {
        let _ = create_and_init_plugin().await;
    }

    #[ignore]
    #[tokio::test]
    #[traced_test]
    // cargo test --package mysql_ox_plugin --lib --release -- test::push_random_alerts --exact --nocapture --ignored
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
