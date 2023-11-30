use crate::database::models::alert_status::AlertStatusModel;
use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use diesel::{result::Error as DieselError, Connection, PgConnection};
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncConnection, AsyncPgConnection,
    RunQueryDsl,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use models::AlermanagerPush;
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use push_definitions::{InitializeError, Push, PushError};
use scoped_futures::ScopedFutureExt;
use thiserror::Error as ThisError;
use tokio::task::JoinHandle;

mod database;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(ThisError, Debug)]
enum InternalPushError {
    #[error("Transaction error: {0}")]
    Transaction(
        #[source]
        #[from]
        DieselError,
    ),
    #[error("Error while inserting alert group: group_key: {group_key}, error: {error}")]
    GroupInsertion {
        group_key: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting group labels: group_key: {group_key}, error: {error}")]
    GroupLabelsInsertion {
        group_key: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting common labels: group_key: {group_key}, error: {error}")]
    CommonLabelsInsertion {
        group_key: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting common annotations: group_key: {group_key}, error: {error}")]
    CommonAnnotationsInsertion {
        group_key: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while parsing starts_at: group_key: {group_key}, fingerprint: {fingerprint}, got_starts_at: {got_starts_at}, error: {error}")]
    StartsAtParsing {
        group_key: String,
        fingerprint: String,
        got_starts_at: String,
        #[source]
        error: chrono::ParseError,
    },
    #[error("Error while parsing ends_at: group_key: {group_key}, fingerprint: {fingerprint}, got_ends_at: {got_ends_at}, error: {error}")]
    EndsAtParsing {
        group_key: String,
        fingerprint: String,
        got_ends_at: String,
        #[source]
        error: chrono::ParseError,
    },
    #[error("Error while inserting alert: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
    AlertInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting alert labels: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
    AlertLabelsInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting alert annotations: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
    AlertAnnotationsInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: DieselError,
    },
}

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
    meta: PostgresPluginMeta,
    config: Option<Box<PostgresPluginConfig>>,
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
            .context("Failed to create pool.")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }
}

#[async_trait]
impl Plugin for PostgresPlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "postgres",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");
        let _conn = self.pool.get().await.map_err(|error| HealthError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}

#[async_trait]
impl Push for PostgresPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        // Always be nice and give memory back to the OS. ;)
        let config = self.config.take().ok_or_else(|| InitializeError {
            reason: "Already initialized.".to_string(),
        })?;

        let connection_string = config.connection_string;
        let handle: JoinHandle<AnyResult<()>> = tokio::task::spawn_blocking(move || {
            let mut conn = PgConnection::establish(&connection_string)
                .context("Failed to establish connection.")?;

            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|error| anyhow::anyhow!(error))
                .context("Failed to run migrations.")?;

            Ok(())
        });

        handle
            .await
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlermanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");
        let mut conn = self.pool.get().await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Starting transaction.");
        conn.transaction::<(), InternalPushError, _>(|mut conn| {
            async move {
                let alert_group = database::models::group::InsertableAlertGroup {
                    receiver: &alertmanager_push.receiver,
                    status: &AlertStatusModel::from(&alertmanager_push.status),
                    external_url: &alertmanager_push.external_url,
                    group_key: &alertmanager_push.group_key,
                };

                tracing::trace!("Inserting alert group.");
                let alert_group_id = diesel::insert_into(database::schema::alert_group::table)
                    .values(&alert_group)
                    .returning(database::schema::alert_group::id)
                    .get_result::<i32>(&mut conn)
                    .await
                    .map_err(|error| InternalPushError::GroupInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        error,
                    })?;

                let group_labels = alertmanager_push
                    .group_labels
                    .iter()
                    .map(|label| database::models::group::InsertableGroupLabel {
                        alert_group_id,
                        name: label.0,
                        value: label.1,
                    })
                    .collect::<Vec<_>>();

                tracing::trace!("Inserting group labels.");
                diesel::insert_into(database::schema::group_label::table)
                    .values(&group_labels)
                    .execute(&mut conn)
                    .await
                    .map_err(|error| InternalPushError::GroupLabelsInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        error,
                    })?;

                let common_labels = alertmanager_push
                    .common_labels
                    .iter()
                    .map(|label| database::models::group::InsertableCommonLabel {
                        alert_group_id,
                        name: label.0,
                        value: label.1,
                    })
                    .collect::<Vec<_>>();

                tracing::trace!("Inserting common labels.");
                diesel::insert_into(database::schema::common_label::table)
                    .values(&common_labels)
                    .execute(&mut conn)
                    .await
                    .map_err(|error| InternalPushError::CommonLabelsInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        error,
                    })?;

                let common_annotations = alertmanager_push
                    .common_annotations
                    .iter()
                    .map(
                        |annotation| database::models::group::InsertableCommonAnnotation {
                            alert_group_id,
                            name: annotation.0,
                            value: annotation.1,
                        },
                    )
                    .collect::<Vec<_>>();

                tracing::trace!("Inserting common annotations.");
                diesel::insert_into(database::schema::common_annotation::table)
                    .values(&common_annotations)
                    .execute(&mut conn)
                    .await
                    .map_err(|error| InternalPushError::CommonAnnotationsInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        error,
                    })?;

                tracing::trace!("Inserting alerts.");
                for alert in alertmanager_push.alerts.iter() {
                    let starts_at = chrono::DateTime::parse_from_rfc3339(&alert.starts_at)
                        .map_err(|error| InternalPushError::StartsAtParsing {
                            group_key: alertmanager_push.group_key.clone(),
                            fingerprint: alert.fingerprint.clone(),
                            got_starts_at: alert.starts_at.clone(),
                            error,
                        })?
                        .naive_utc();

                    let ends_at = chrono::DateTime::parse_from_rfc3339(&alert.ends_at)
                        .map_err(|error| InternalPushError::EndsAtParsing {
                            group_key: alertmanager_push.group_key.clone(),
                            fingerprint: alert.fingerprint.clone(),
                            got_ends_at: alert.ends_at.clone(),
                            error,
                        })?
                        .naive_utc();

                    let ends_at = if ends_at > starts_at {
                        Some(ends_at)
                    } else {
                        None
                    };

                    let insertable_alert = database::models::alert::InsertableAlert {
                        alert_group_id,
                        status: &AlertStatusModel::from(&alert.status),
                        starts_at,
                        ends_at,
                        generator_url: &alert.generator_url,
                        fingerprint: &alert.fingerprint,
                    };

                    let alert_id = diesel::insert_into(database::schema::alert::table)
                        .values(&insertable_alert)
                        .returning(database::schema::alert::id)
                        .get_result::<i32>(&mut conn)
                        .await
                        .map_err(|error| InternalPushError::AlertInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            fingerprint: alert.fingerprint.clone(),
                            error,
                        })?;

                    let labls = alert
                        .labels
                        .iter()
                        .map(|label| database::models::alert::InsertableAlertLabel {
                            alert_id,
                            name: label.0,
                            value: label.1,
                        })
                        .collect::<Vec<_>>();

                    tracing::trace!("Inserting alert labels.");
                    diesel::insert_into(database::schema::alert_label::table)
                        .values(&labls)
                        .execute(&mut conn)
                        .await
                        .map_err(|error| InternalPushError::AlertLabelsInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            fingerprint: alert.fingerprint.clone(),
                            error,
                        })?;

                    let annotations = alert
                        .annotations
                        .iter()
                        .map(
                            |annotation| database::models::alert::InsertableAlertAnnotation {
                                alert_id,
                                name: annotation.0,
                                value: annotation.1,
                            },
                        )
                        .collect::<Vec<_>>();

                    tracing::trace!("Inserting alert annotations.");
                    diesel::insert_into(database::schema::alert_annotation::table)
                        .values(&annotations)
                        .execute(&mut conn)
                        .await
                        .map_err(|error| InternalPushError::AlertAnnotationsInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            fingerprint: alert.fingerprint.clone(),
                            error,
                        })?;
                }

                tracing::trace!("Committing transaction.");

                Ok(())
            }
            .scope_boxed()
        })
        .await
        .map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
