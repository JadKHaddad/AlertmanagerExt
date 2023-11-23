use crate::database::models::alert_status::AlertStatusModel;
use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use diesel::{Connection, PgConnection};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use models::AlermanagerPush;
use plugins_definitions::{HealthError, InitializeError, Plugin};
use push_definitions::{Push, PushError};
use scoped_futures::ScopedFutureExt;
use tokio::task::JoinHandle;

mod database;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub struct PushErrorWrapper(pub PushError);

impl From<PushErrorWrapper> for PushError {
    fn from(wrapper: PushErrorWrapper) -> Self {
        wrapper.0
    }
}

impl From<PushError> for PushErrorWrapper {
    fn from(error: PushError) -> Self {
        PushErrorWrapper(error)
    }
}

impl From<diesel::result::Error> for PushErrorWrapper {
    fn from(error: diesel::result::Error) -> Self {
        PushErrorWrapper(PushError {
            reason: error.to_string(),
        })
    }
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
        conn.transaction::<(), PushErrorWrapper, _>(|mut conn| {
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
                    .map_err(|error| PushError {
                        reason: format!("Failed to insert alert group: {}", error),
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
                        .map_err(|error| PushError {
                            reason: format!("Failed to insert group label: {}", error),
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
                        .map_err(|error| PushError {
                            reason: format!("Failed to insert common label: {}", error),
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
                        .map_err(|error| PushError {
                            reason: format!("Failed to insert common annotation: {}", error),
                        })?;
                }

                tracing::trace!("Inserting alerts.");
                for alert in alertmanager_push.alerts.iter() {
                    let fingerprint_in_error_message =
                        format!("fingerprint: {}", alert.fingerprint);

                    // TODO: Error message (got: act. starts_at)
                    let starts_at = chrono::DateTime::parse_from_rfc3339(&alert.starts_at)
                        .map_err(|error| PushError {
                            reason: format!("Failed to parse starts_at, {fingerprint_in_error_message}, error: {error}")
                        })?
                        .naive_utc();
                    
                    // TODO: Error message (got: act. ends_at)
                    let ends_at = chrono::DateTime::parse_from_rfc3339(&alert.ends_at)
                        .map_err(|error| PushError {
                            reason: format!("Failed to parse ends_at: {fingerprint_in_error_message}, error: {error}")
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
                        .map_err(|error| PushError {
                            reason: format!("Failed to insert alert, {fingerprint_in_error_message}, error: {error}")
                        })?;
                    
                    // TODO: Annotations and labels in error messages
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
                            .map_err(|error| PushError {
                                reason: format!("Failed to insert alert label, {fingerprint_in_error_message}, error: {error}")
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
                            .map_err(|error| PushError {
                                reason: format!("Failed to insert alert annotation, {fingerprint_in_error_message}, error: {error}")
                            })?;
                    }
                }
                tracing::trace!("Committing transaction.");
                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
