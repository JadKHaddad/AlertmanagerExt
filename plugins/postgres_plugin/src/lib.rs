use crate::database::models::alert_status::AlertStatusModel;
use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use chrono::Duration;
use diesel::result::Error as DieselError;
use diesel::{Connection, PgConnection};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use models::AlermanagerPush;
use plugins_definitions::{HealthError, InitializeError, Plugin};
use push_definitions::{Push, PushError};
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

pub struct PostgresPlugin {
    name: String,
    connection_string: String,
    pool: Pool,
}

impl PostgresPlugin {
    pub async fn new(name: String, connection_string: String) -> AnyResult<Self> {
        let manager = AsyncDieselConnectionManager::new(connection_string.clone());
        let pool = bb8::Pool::builder()
            .max_size(15)
            .connection_timeout(std::time::Duration::from_secs(3))
            .build(manager)
            .await
            .context("Failed to create pool.")?;

        Ok(Self {
            name,
            connection_string,
            pool,
        })
    }
}

#[async_trait]
impl Plugin for PostgresPlugin {
    fn type_(&self) -> &str {
        "PostgresPlugin"
    }

    fn name(&self) -> &str {
        &self.name
    }

    #[tracing::instrument(name = "PostgresPlugin initialize", skip(self), fields(name = %self.name))]
    async fn initialize(&self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");
        let connection_string = self.connection_string.clone();
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

    #[tracing::instrument(name = "PostgresPlugin health", skip(self), fields(self.name = %self.name))]
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
    #[tracing::instrument(name = "PostgresPlugin push_alert", skip_all)]
    async fn push_alert(&self, alertmanager_push: &AlermanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");
        let mut conn = self.pool.get().await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        // TODO: Test the transaction with some invalid dates!
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
                        alert_group_id,
                        name: group_label.0,
                        value: group_label.1,
                    };

                    diesel::insert_into(database::schema::group_label::table)
                        .values(&group_label)
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
                        alert_group_id,
                        name: common_label.0,
                        value: common_label.1,
                    };

                    diesel::insert_into(database::schema::common_label::table)
                        .values(&common_label)
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
                        alert_group_id,
                        name: common_annotation.0,
                        value: common_annotation.1,
                    };

                    diesel::insert_into(database::schema::common_annotation::table)
                        .values(&common_annotation)
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
                            alert_id,
                            name: label.0,
                            value: label.1,
                        };

                        diesel::insert_into(database::schema::alert_label::table)
                            .values(&label)
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
                            alert_id,
                            name: annotation.0,
                            value: annotation.1,
                        };

                        diesel::insert_into(database::schema::alert_annotation::table)
                            .values(&annotation)
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
