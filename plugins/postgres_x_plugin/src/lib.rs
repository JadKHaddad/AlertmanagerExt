use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use database::models::{
    alert_status::AlertStatusModel, alerts::DatabaseAlert, annotations::Annotation, labels::Label,
};
use models::{AlertmanagerPush, StandAloneAlert};
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use pull_definitions::{Pull, PullAlertsFilter, PullError};
use push_definitions::{InitializeError, Push, PushError};
use sqlx::Connection;

mod database;
mod error;

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

        let mut conn = self.pool.acquire().await.map_err(|error| HealthError {
            reason: error.to_string(),
        })?;

        conn.ping().await.map_err(|error| HealthError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}

#[async_trait]
impl Push for PostgresXPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        // TODO
        tracing::warn!("Not implemented yet.");
        let _ = self.config.take();

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        // Failed to acquire connection
        let mut conn = self.pool.acquire().await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        // Failed to begin transaction
        let mut tx = conn.begin().await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        let status = AlertStatusModel::from(&alertmanager_push.status);
        let group_id = sqlx::query!(r#"INSERT INTO groups (group_key, receiver, status , external_url) VALUES ($1, $2, $3, $4) RETURNING id"#, alertmanager_push.group_key, alertmanager_push.receiver, status as AlertStatusModel, alertmanager_push.external_url)
                    .fetch_one(&mut *tx)
                    .await.map_err(|error| PushError {
                        reason: error.to_string(),
                    })
                    ?.id;

        dbg!(group_id);

        // Failed to commit transaction
        tx.commit().await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        // this will stay here for a while
        // let s: Result<i32, sqlx::Error> = conn.transaction(|txn| {
        //     Box::pin(async move {
        //         let status = AlertStatusModel::from(&alertmanager_push.status);
        //         let group_id = sqlx::query!(r#"INSERT INTO groups (group_key, receiver, status , external_url) VALUES ($1, $2, $3, $4) RETURNING id"#, alertmanager_push.group_key, alertmanager_push.receiver, status as AlertStatusModel, alertmanager_push.external_url)
        //             .fetch_one(&mut **txn)
        //             .await
        //             ?
        //             .id;

        //         Ok(group_id)
        //     })
        // })
        // .await;

        tracing::trace!("Successfully pushed.");
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

#[cfg(test)]
mod test {
    use super::*;
    use tracing_test::traced_test;

    async fn create_plugin() -> PostgresXPlugin {
        let postgres_x_plugin_config = PostgresXPluginConfig {
            connection_string: String::from("postgres://user:password@localhost:5432/database"),
            max_connections: 15,
            connection_timeout: std::time::Duration::from_secs(5),
        };

        let postgres_x_plugin_meta = PostgresXPluginMeta {
            name: String::from("postgres_x_plugin_1"),
            group: String::from("default"),
        };

        PostgresXPlugin::new(postgres_x_plugin_meta, postgres_x_plugin_config)
            .await
            .expect("Failed to create Postgres plugin.")
    }

    #[tokio::test]
    #[traced_test]
    async fn pull_alerts() {
        let plugin = create_plugin().await;
        let filter = PullAlertsFilter {};
        let alerts = plugin
            .pull_alerts(&filter)
            .await
            .expect("Failed to get all alerts.");

        let alerts = &alerts[0..15];

        for alert in alerts.iter() {
            println!("{:#?}", alert);
        }
    }
}
