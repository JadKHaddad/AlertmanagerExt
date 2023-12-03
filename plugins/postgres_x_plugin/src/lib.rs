use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use database::models::{
    alert_status::AlertStatusModel, alerts::DatabaseAlert, annotations::Annotation, labels::Label,
};
use models::StandAloneAlert;
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use pull_definitions::{Pull, PullAlertsFilter, PullError};

mod database;

/// Configuration for the PostgresX plugin
pub struct PostgresXPluginConfig {
    /// Connection string for the PostgresX database
    pub connection_string: String,
    /// Max number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
}

/// Metadata for the PostgresX plugin
pub struct PostgresXPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// PostgresX plugin
pub struct PostgresXPlugin {
    /// Meta information for the plugin
    meta: PostgresXPluginMeta,
    /// Configuration for the plugin
    config: Option<Box<PostgresXPluginConfig>>,
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
            .context("Failed to create pool.")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }
}

#[async_trait]
impl Plugin for PostgresXPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "postgres_x",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        let _ = self.pool.acquire().await.map_err(|error| HealthError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}

#[async_trait]
impl Pull for PostgresXPlugin {
    #[tracing::instrument(name = "pull_alerts", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn pull_alerts(
        &self,
        filter: &PullAlertsFilter,
    ) -> Result<Vec<StandAloneAlert>, PullError> {
        tracing::trace!("Pulling.");

        let database_alerts = sqlx::query_file_as!(DatabaseAlert, "queries/pull_alerts.sql",)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| PullError {
                reason: error.to_string(),
            })?;

        tracing::trace!("Successfully pulled.");

        Ok(database_alerts
            .into_iter()
            .map(|alert| alert.into())
            .collect())
    }
}