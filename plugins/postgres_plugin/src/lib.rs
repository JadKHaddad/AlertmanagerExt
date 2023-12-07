use anyhow::{Context, Result as AnyResult};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

mod database;
mod error;
mod impls;

pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

/// Configuration for the Postgres plugin
pub struct PostgresPluginConfig {
    /// Connection string for the Postgres database
    pub connection_string: String,
    /// Max number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
}

/// Metadata for the Postgres plugin
pub struct PostgresPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// Postgres plugin
///
/// Based on [`diesel`], [`diesel_async`] and [`bb8`].
pub struct PostgresPlugin {
    /// Meta information for the plugin
    meta: PostgresPluginMeta,
    /// Configuration for the plugin
    config: Option<Box<PostgresPluginConfig>>,
    /// Connection pool
    pool: Pool,
}

impl PostgresPlugin {
    pub async fn new(meta: PostgresPluginMeta, config: PostgresPluginConfig) -> AnyResult<Self> {
        let manager = AsyncDieselConnectionManager::new(config.connection_string.clone());
        let pool = bb8::Pool::builder()
            .max_size(config.max_connections)
            .connection_timeout(config.connection_timeout)
            .build(manager)
            .await
            .context("Failed to create pool")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use models::utils::generate_random_alertmanager_pushes;
    use pull_definitions::{Pull, PullAlertsFilter};
    use push_definitions::Push;
    use tracing_test::traced_test;

    async fn create_and_init_plugin() -> PostgresPlugin {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let postgres_plugin_config = PostgresPluginConfig {
            connection_string: database_url,
            max_connections: 15,
            connection_timeout: std::time::Duration::from_secs(5),
        };

        let postgres_plugin_meta = PostgresPluginMeta {
            name: String::from("postgres_plugin_1"),
            group: String::from("default"),
        };

        let mut postgres_plugin = PostgresPlugin::new(postgres_plugin_meta, postgres_plugin_config)
            .await
            .expect("Failed to create Postgres plugin.");

        postgres_plugin
            .initialize()
            .await
            .expect("Failed to initialize Postgres plugin.");

        postgres_plugin
    }

    #[ignore]
    #[tokio::test]
    #[traced_test]
    // cargo test --package postgres_plugin --lib --release -- test::push_random_alerts --exact --nocapture
    async fn push_random_alerts() {
        let plugin = create_and_init_plugin().await;
        let pushes = generate_random_alertmanager_pushes(200);
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
    // cargo test --package postgres_plugin --lib --release -- test::pull_alerts --exact --nocapture
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
