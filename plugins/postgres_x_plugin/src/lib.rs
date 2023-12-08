use anyhow::{Context, Result as AnyResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod database;
mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the PostgresX plugin
pub struct PostgresXPluginConfig {
    /// Connection string for the PostgresX database
    pub connection_string: String,
    /// Max number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Metadata for the PostgresX plugin
pub struct PostgresXPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// PostgresX plugin
///
/// Based on [`sqlx`].
pub struct PostgresXPlugin {
    /// Meta information for the plugin
    meta: PostgresXPluginMeta,
    /// Pool of connections to the database
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl PostgresXPlugin {
    pub async fn new(meta: PostgresXPluginMeta, config: PostgresXPluginConfig) -> AnyResult<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(config.connection_timeout)
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
    use pull_definitions::{Pull, PullAlertsFilter};
    use push_definitions::Push;
    use tracing_test::traced_test;

    async fn create_and_init_plugin() -> PostgresXPlugin {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let postgres_x_plugin_config = PostgresXPluginConfig {
            connection_string: database_url,
            max_connections: 15,
            connection_timeout: std::time::Duration::from_secs(5),
        };

        let postgres_x_plugin_meta = PostgresXPluginMeta {
            name: String::from("postgres_x_plugin_1"),
            group: String::from("default"),
        };

        let mut postgres_x_plugin =
            PostgresXPlugin::new(postgres_x_plugin_meta, postgres_x_plugin_config)
                .await
                .expect("Failed to create PostgresX plugin.");

        postgres_x_plugin
            .initialize()
            .await
            .expect("Failed to initialize PostgresX plugin.");

        postgres_x_plugin
    }

    #[ignore]
    #[tokio::test]
    #[traced_test]
    // cargo test --package postgres_x_plugin --lib --release -- test::push_random_alerts --exact --nocapture
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

    #[ignore]
    #[tokio::test]
    #[traced_test]
    // cargo test --package postgres_x_plugin --lib --release -- test::pull_alerts --exact --nocapture
    async fn pull_alerts() {
        let plugin = create_and_init_plugin().await;
        let filter = PullAlertsFilter {};
        let alerts = plugin
            .pull_alerts(&filter)
            .await
            .expect("Failed to get all alerts.");

        for alert in alerts.iter().take(10) {
            println!("{:#?}", alert);
        }

        println!("Total pulled: {}", alerts.len());
    }
}
