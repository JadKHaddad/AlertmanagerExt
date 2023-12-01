use crate::database::models::alert_status::AlertStatusModel;
use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use diesel::OptionalExtension;
use diesel::{
    query_dsl::methods::{FilterDsl, SelectDsl},
    result::Error as DieselError,
    BoolExpressionMethods, Connection, ExpressionMethods, PgConnection,
};
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
    #[error("Error while inserting group label: group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting common label: group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting common annotation: group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationInsertion {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
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
    #[error("Error while inserting alert label: group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelInsertion {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error while inserting alert annotation: group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationInsertion {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
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

                tracing::trace!("Inserting group labels.");
                for group_label in alertmanager_push.group_labels.iter() {
                    let group_label = database::models::group::InsertableGroupLabel {
                        name: group_label.0,
                        value: group_label.1,
                    };

                    let group_label_id_opt = database::schema::group_label::table
                        .filter(
                            database::schema::group_label::name
                                .eq(&group_label.name)
                                .and(database::schema::group_label::value.eq(&group_label.value)),
                        )
                        .select(database::schema::group_label::id)
                        .get_result::<i32>(&mut conn)
                        .await
                        .optional()
                        .map_err(|error| InternalPushError::GroupLabelInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            label_name: group_label.name.to_owned(),
                            label_value: group_label.value.to_owned(),
                            error,
                        })?;

                    let group_label_id = match group_label_id_opt {
                        Some(group_label_id) => {
                            tracing::trace!(
                                name = %group_label.name,
                                value = %group_label.value,
                                "Group label already exists."
                            );
                            group_label_id
                        }
                        None => diesel::insert_into(database::schema::group_label::table)
                            .values(&group_label)
                            .returning(database::schema::group_label::id)
                            .get_result::<i32>(&mut conn)
                            .await
                            .map_err(|error| InternalPushError::GroupLabelInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                label_name: group_label.name.to_owned(),
                                label_value: group_label.value.to_owned(),
                                error,
                            })?,
                    };

                    let assign_group_label_to_group =
                        database::models::group::AssignGroupLabelToGroup {
                            alert_group_id,
                            group_label_id,
                        };

                    diesel::insert_into(database::schema::alert_group_group_labels::table)
                        .values(&assign_group_label_to_group)
                        .execute(&mut conn)
                        .await
                        .map_err(|error| InternalPushError::GroupLabelInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            label_name: group_label.name.to_owned(),
                            label_value: group_label.value.to_owned(),
                            error,
                        })?;
                }

                tracing::trace!("Inserting common labels.");
                for common_label in alertmanager_push.common_labels.iter() {
                    let common_label = database::models::group::InsertableCommonLabel {
                        name: common_label.0,
                        value: common_label.1,
                    };

                    let common_label_id_opt = database::schema::common_label::table
                        .filter(
                            database::schema::common_label::name
                                .eq(&common_label.name)
                                .and(database::schema::common_label::value.eq(&common_label.value)),
                        )
                        .select(database::schema::common_label::id)
                        .get_result::<i32>(&mut conn)
                        .await
                        .optional()
                        .map_err(|error| InternalPushError::CommonLabelInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            label_name: common_label.name.to_owned(),
                            label_value: common_label.value.to_owned(),
                            error,
                        })?;

                    let common_label_id = match common_label_id_opt {
                        Some(common_label_id) => {
                            tracing::trace!(
                                name = %common_label.name,
                                value = %common_label.value,
                                "Common label already exists."
                            );
                            common_label_id
                        }
                        None => diesel::insert_into(database::schema::common_label::table)
                            .values(&common_label)
                            .returning(database::schema::common_label::id)
                            .get_result::<i32>(&mut conn)
                            .await
                            .map_err(|error| InternalPushError::CommonLabelInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                label_name: common_label.name.to_owned(),
                                label_value: common_label.value.to_owned(),
                                error,
                            })?,
                    };

                    let assign_common_label_to_group =
                        database::models::group::AssignCommonLabelToGroup {
                            alert_group_id,
                            common_label_id,
                        };

                    diesel::insert_into(database::schema::alert_group_common_labels::table)
                        .values(&assign_common_label_to_group)
                        .execute(&mut conn)
                        .await
                        .map_err(|error| InternalPushError::CommonLabelInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            label_name: common_label.name.to_owned(),
                            label_value: common_label.value.to_owned(),
                            error,
                        })?;
                }

                tracing::trace!("Inserting common annotations.");
                for common_annotation in alertmanager_push.common_annotations.iter() {
                    let common_annotation = database::models::group::InsertableCommonAnnotation {
                        name: common_annotation.0,
                        value: common_annotation.1,
                    };

                    let common_annotation_id_opt = database::schema::common_annotation::table
                        .filter(
                            database::schema::common_annotation::name
                                .eq(&common_annotation.name)
                                .and(
                                    database::schema::common_annotation::value
                                        .eq(&common_annotation.value),
                                ),
                        )
                        .select(database::schema::common_annotation::id)
                        .get_result::<i32>(&mut conn)
                        .await
                        .optional()
                        .map_err(|error| InternalPushError::CommonAnnotationInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            annotation_name: common_annotation.name.to_owned(),
                            annotation_value: common_annotation.value.to_owned(),
                            error,
                        })?;

                    let common_annotation_id = match common_annotation_id_opt {
                        Some(common_annotation_id) => {
                            tracing::trace!(
                                name = %common_annotation.name,
                                value = %common_annotation.value,
                                "Common annotation already exists."
                            );
                            common_annotation_id
                        }
                        None => diesel::insert_into(database::schema::common_annotation::table)
                            .values(&common_annotation)
                            .returning(database::schema::common_annotation::id)
                            .get_result::<i32>(&mut conn)
                            .await
                            .map_err(|error| InternalPushError::CommonAnnotationInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                annotation_name: common_annotation.name.to_owned(),
                                annotation_value: common_annotation.value.to_owned(),
                                error,
                            })?,
                    };

                    let assign_common_annotation_to_group =
                        database::models::group::AssignCommonAnnotationToGroup {
                            alert_group_id,
                            common_annotation_id,
                        };

                    diesel::insert_into(database::schema::alert_group_common_annotations::table)
                        .values(&assign_common_annotation_to_group)
                        .execute(&mut conn)
                        .await
                        .map_err(|error| InternalPushError::CommonAnnotationInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            annotation_name: common_annotation.name.to_owned(),
                            annotation_value: common_annotation.value.to_owned(),
                            error,
                        })?;
                }

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
                        group_key: &alertmanager_push.group_key,
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

                    for label in alert.labels.iter() {
                        let label = database::models::alert::InsertableAlertLabel {
                            name: label.0,
                            value: label.1,
                        };

                        let alert_label_id_opt = database::schema::alert_label::table
                            .filter(
                                database::schema::alert_label::name
                                    .eq(&label.name)
                                    .and(database::schema::alert_label::value.eq(&label.value)),
                            )
                            .select(database::schema::alert_label::id)
                            .get_result::<i32>(&mut conn)
                            .await
                            .optional()
                            .map_err(|error| InternalPushError::AlertLabelInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                fingerprint: alert.fingerprint.clone(),
                                label_name: label.name.to_owned(),
                                label_value: label.value.to_owned(),
                                error,
                            })?;

                        let alert_label_id = match alert_label_id_opt {
                            Some(alert_label_id) => {
                                tracing::trace!(
                                    name = %label.name,
                                    value = %label.value,
                                    "Alert label already exists."
                                );
                                alert_label_id
                            }
                            None => diesel::insert_into(database::schema::alert_label::table)
                                .values(&label)
                                .returning(database::schema::alert_label::id)
                                .get_result::<i32>(&mut conn)
                                .await
                                .map_err(|error| InternalPushError::AlertLabelInsertion {
                                    group_key: alertmanager_push.group_key.clone(),
                                    fingerprint: alert.fingerprint.clone(),
                                    label_name: label.name.to_owned(),
                                    label_value: label.value.to_owned(),
                                    error,
                                })?,
                        };

                        let assign_alert_label_to_alert =
                            database::models::alert::AssignAlertLabelToAlert {
                                alert_id,
                                alert_label_id,
                            };

                        diesel::insert_into(database::schema::alert_alert_labels::table)
                            .values(&assign_alert_label_to_alert)
                            .execute(&mut conn)
                            .await
                            .map_err(|error| InternalPushError::AlertLabelInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                fingerprint: alert.fingerprint.clone(),
                                label_name: label.name.to_owned(),
                                label_value: label.value.to_owned(),
                                error,
                            })?;
                    }

                    for annotation in alert.annotations.iter() {
                        let annotation = database::models::alert::InsertableAlertAnnotation {
                            name: annotation.0,
                            value: annotation.1,
                        };

                        let alert_annotation_id_opt = database::schema::alert_annotation::table
                            .filter(
                                database::schema::alert_annotation::name
                                    .eq(&annotation.name)
                                    .and(
                                        database::schema::alert_annotation::value
                                            .eq(&annotation.value),
                                    ),
                            )
                            .select(database::schema::alert_annotation::id)
                            .get_result::<i32>(&mut conn)
                            .await
                            .optional()
                            .map_err(|error| InternalPushError::AlertAnnotationInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                fingerprint: alert.fingerprint.clone(),
                                annotation_name: annotation.name.to_owned(),
                                annotation_value: annotation.value.to_owned(),
                                error,
                            })?;

                        let alert_annotation_id = match alert_annotation_id_opt {
                            Some(alert_annotation_id) => {
                                tracing::trace!(
                                    name = %annotation.name,
                                    value = %annotation.value,
                                    "Alert annotation already exists."
                                );
                                alert_annotation_id
                            }
                            None => diesel::insert_into(database::schema::alert_annotation::table)
                                .values(&annotation)
                                .returning(database::schema::alert_annotation::id)
                                .get_result::<i32>(&mut conn)
                                .await
                                .map_err(|error| InternalPushError::AlertAnnotationInsertion {
                                    group_key: alertmanager_push.group_key.clone(),
                                    fingerprint: alert.fingerprint.clone(),
                                    annotation_name: annotation.name.to_owned(),
                                    annotation_value: annotation.value.to_owned(),
                                    error,
                                })?,
                        };

                        let assign_alert_annotation_to_alert =
                            database::models::alert::AssignAlertAnnotationToAlert {
                                alert_id,
                                alert_annotation_id,
                            };

                        diesel::insert_into(database::schema::alert_alert_annotations::table)
                            .values(&assign_alert_annotation_to_alert)
                            .execute(&mut conn)
                            .await
                            .map_err(|error| InternalPushError::AlertAnnotationInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                fingerprint: alert.fingerprint.clone(),
                                annotation_name: annotation.name.to_owned(),
                                annotation_value: annotation.value.to_owned(),
                                error,
                            })?;
                    }
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
