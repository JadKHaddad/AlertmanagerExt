use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use error::NewSqlitePluginError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod database;
mod error;
mod impls;

pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type Pool = deadpool_diesel::sqlite::Pool;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the SQLite plugin
pub struct SqlitePluginConfig {
    /// Path to the database file
    pub connection_string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub fn new(
        meta: SqlitePluginMeta,
        config: SqlitePluginConfig,
    ) -> Result<Self, NewSqlitePluginError> {
        let manager = deadpool_diesel::sqlite::Manager::new(
            &config.connection_string,
            deadpool_diesel::Runtime::Tokio1,
        );
        let pool = deadpool_diesel::sqlite::Pool::builder(manager).build()?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }
}
