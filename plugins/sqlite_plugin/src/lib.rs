use anyhow::{Context, Result as AnyResult};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

mod database;
mod impls;

pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type Pool = deadpool_diesel::sqlite::Pool;

/// Configuration for the SQLite plugin
pub struct SqlitePluginConfig {
    /// Path to the database file
    pub database_url: String,
}

pub struct SqlitePluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// SQLite plugin
///
/// Based on [`diesel`] and [`deadpool_diesel`]
pub struct SqlitePlugin {
    /// Meta information for the plugin
    pub meta: SqlitePluginMeta,
    /// Configuration for the plugin
    pub config: Option<Box<SqlitePluginConfig>>,
    /// Connection pool
    pool: Pool,
}

impl SqlitePlugin {
    pub fn new(meta: SqlitePluginMeta, config: SqlitePluginConfig) -> AnyResult<Self> {
        let manager = deadpool_diesel::sqlite::Manager::new(
            &config.database_url,
            deadpool_diesel::Runtime::Tokio1,
        );
        let pool = deadpool_diesel::sqlite::Pool::builder(manager)
            .build()
            .context("Failed to create pool")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }
}
