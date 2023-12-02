use crate::database::models::alert_status::AlertStatusModel;
use crate::error::InternalPushError;
use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use diesel::OptionalExtension;
use diesel::{
    query_dsl::methods::{FilterDsl, SelectDsl},
    BoolExpressionMethods, Connection, ExpressionMethods, PgConnection,
};
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncConnection, AsyncPgConnection,
    RunQueryDsl,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use models::{Alert, AlertmanagerPush};
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use push_definitions::{InitializeError, Push, PushError};
use scoped_futures::ScopedFutureExt;
use tokio::task::JoinHandle;

mod database;
mod error;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

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
            .context("Failed to create pool.")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }

    async fn insert_alert_group(
        conn: &mut AsyncPgConnection,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<i32, InternalPushError> {
        let alert_group = database::models::group::InsertableAlertGroup {
            receiver: &alertmanager_push.receiver,
            status: &AlertStatusModel::from(&alertmanager_push.status),
            external_url: &alertmanager_push.external_url,
            group_key: &alertmanager_push.group_key,
        };

        let alert_group_id = diesel::insert_into(database::schema::alert_group::table)
            .values(&alert_group)
            .returning(database::schema::alert_group::id)
            .get_result::<i32>(conn)
            .await
            .map_err(|error| InternalPushError::GroupInsertion {
                group_key: alertmanager_push.group_key.clone(),
                error,
            })?;

        Ok(alert_group_id)
    }

    async fn assign_group_label(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        group_label_id: i32,
        group_label: &database::models::group::InsertableGroupLabel<'_>,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let assign_group_label_to_group = database::models::group::AssignGroupLabel {
            alert_group_id,
            group_label_id,
        };

        diesel::insert_into(database::schema::assign_group_label::table)
            .values(&assign_group_label_to_group)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::GroupLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                label_name: group_label.name.to_owned(),
                label_value: group_label.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    async fn insert_group_lables(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
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
                .get_result::<i32>(conn)
                .await
                .optional()
                .map_err(|error| InternalPushError::GroupLabelId {
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
                    .get_result::<i32>(conn)
                    .await
                    .map_err(|error| InternalPushError::GroupLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: group_label.name.to_owned(),
                        label_value: group_label.value.to_owned(),
                        error,
                    })?,
            };

            Self::assign_group_label(
                conn,
                alert_group_id,
                group_label_id,
                &group_label,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn assign_common_label(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        common_label_id: i32,
        common_label: &database::models::group::InsertableCommonLabel<'_>,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let assign_common_label_to_group = database::models::group::AssignCommonLabel {
            alert_group_id,
            common_label_id,
        };

        diesel::insert_into(database::schema::assign_common_label::table)
            .values(&assign_common_label_to_group)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::CommonLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                label_name: common_label.name.to_owned(),
                label_value: common_label.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    async fn insert_common_labels(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
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
                .get_result::<i32>(conn)
                .await
                .optional()
                .map_err(|error| InternalPushError::CommonLabelId {
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
                    .get_result::<i32>(conn)
                    .await
                    .map_err(|error| InternalPushError::CommonLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: common_label.name.to_owned(),
                        label_value: common_label.value.to_owned(),
                        error,
                    })?,
            };

            Self::assign_common_label(
                conn,
                alert_group_id,
                common_label_id,
                &common_label,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn assign_common_annotation(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        common_annotation_id: i32,
        common_annotation: &database::models::group::InsertableCommonAnnotation<'_>,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let assign_common_annotation_to_group = database::models::group::AssignCommonAnnotation {
            alert_group_id,
            common_annotation_id,
        };

        diesel::insert_into(database::schema::assign_common_annotation::table)
            .values(&assign_common_annotation_to_group)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::CommonAnnotationAssignment {
                group_key: alertmanager_push.group_key.clone(),
                annotation_name: common_annotation.name.to_owned(),
                annotation_value: common_annotation.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    async fn insert_common_annotations(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
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
                            database::schema::common_annotation::value.eq(&common_annotation.value),
                        ),
                )
                .select(database::schema::common_annotation::id)
                .get_result::<i32>(conn)
                .await
                .optional()
                .map_err(|error| InternalPushError::CommonAnnotationId {
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
                    .get_result::<i32>(conn)
                    .await
                    .map_err(|error| InternalPushError::CommonAnnotationInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        annotation_name: common_annotation.name.to_owned(),
                        annotation_value: common_annotation.value.to_owned(),
                        error,
                    })?,
            };

            Self::assign_common_annotation(
                conn,
                alert_group_id,
                common_annotation_id,
                &common_annotation,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn insert_alert(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &Alert,
    ) -> Result<i32, InternalPushError> {
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
            .get_result::<i32>(conn)
            .await
            .map_err(|error| InternalPushError::AlertInsertion {
                group_key: alertmanager_push.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                error,
            })?;

        Ok(alert_id)
    }

    async fn assign_alert_label(
        conn: &mut AsyncPgConnection,
        alert_id: i32,
        alert_label_id: i32,
        alert_label: &database::models::alert::InsertableAlertLabel<'_>,
        alert: &Alert,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let assign_alert_label_to_alert = database::models::alert::AssignAlertLabel {
            alert_id,
            alert_label_id,
        };

        diesel::insert_into(database::schema::assign_alert_label::table)
            .values(&assign_alert_label_to_alert)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::AlertLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                label_name: alert_label.name.to_owned(),
                label_value: alert_label.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    async fn insert_alert_labels(
        conn: &mut AsyncPgConnection,
        alert_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &Alert,
    ) -> Result<(), InternalPushError> {
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
                .get_result::<i32>(conn)
                .await
                .optional()
                .map_err(|error| InternalPushError::AlertLabelId {
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
                    .get_result::<i32>(conn)
                    .await
                    .map_err(|error| InternalPushError::AlertLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        label_name: label.name.to_owned(),
                        label_value: label.value.to_owned(),
                        error,
                    })?,
            };

            Self::assign_alert_label(
                conn,
                alert_id,
                alert_label_id,
                &label,
                alert,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn assign_alert_annotation(
        conn: &mut AsyncPgConnection,
        alert_id: i32,
        alert_annotation_id: i32,
        alert_annotation: &database::models::alert::InsertableAlertAnnotation<'_>,
        alert: &Alert,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let assign_alert_annotation_to_alert = database::models::alert::AssignAlertAnnotation {
            alert_id,
            alert_annotation_id,
        };

        diesel::insert_into(database::schema::assign_alert_annotation::table)
            .values(&assign_alert_annotation_to_alert)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::AlertAnnotationAssignment {
                group_key: alertmanager_push.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                annotation_name: alert_annotation.name.to_owned(),
                annotation_value: alert_annotation.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    async fn insert_alert_annotations(
        conn: &mut AsyncPgConnection,
        alert_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &Alert,
    ) -> Result<(), InternalPushError> {
        for annotation in alert.annotations.iter() {
            let annotation = database::models::alert::InsertableAlertAnnotation {
                name: annotation.0,
                value: annotation.1,
            };

            let alert_annotation_id_opt = database::schema::alert_annotation::table
                .filter(
                    database::schema::alert_annotation::name
                        .eq(&annotation.name)
                        .and(database::schema::alert_annotation::value.eq(&annotation.value)),
                )
                .select(database::schema::alert_annotation::id)
                .get_result::<i32>(conn)
                .await
                .optional()
                .map_err(|error| InternalPushError::AlertAnnotationId {
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
                    .get_result::<i32>(conn)
                    .await
                    .map_err(|error| InternalPushError::AlertAnnotationInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        annotation_name: annotation.name.to_owned(),
                        annotation_value: annotation.value.to_owned(),
                        error,
                    })?,
            };

            Self::assign_alert_annotation(
                conn,
                alert_id,
                alert_annotation_id,
                &annotation,
                alert,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn insert_alerts(
        conn: &mut AsyncPgConnection,
        alert_group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for alert in alertmanager_push.alerts.iter() {
            let alert_id =
                Self::insert_alert(conn, alert_group_id, alertmanager_push, alert).await?;
            Self::insert_alert_labels(conn, alert_id, alertmanager_push, alert).await?;
            Self::insert_alert_annotations(conn, alert_id, alertmanager_push, alert).await?;
        }

        Ok(())
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
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");
        let mut conn = self.pool.get().await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        conn.transaction::<(), InternalPushError, _>(|conn| {
            async move {
                tracing::trace!("Starting transaction.");

                let alert_group_id = Self::insert_alert_group(conn, alertmanager_push).await?;
                Self::insert_group_lables(conn, alert_group_id, alertmanager_push).await?;
                Self::insert_common_labels(conn, alert_group_id, alertmanager_push).await?;
                Self::insert_common_annotations(conn, alert_group_id, alertmanager_push).await?;
                Self::insert_alerts(conn, alert_group_id, alertmanager_push).await?;

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
