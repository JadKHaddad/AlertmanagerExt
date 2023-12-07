use crate::database::models::alert_status::AlertStatusModel;
use crate::database::models::alerts::{Alert, AlertAnnotation, AlertLabel};
use crate::database::models::annotations::Annotation;
use crate::database::models::labels::Label;
use crate::error::{InternalPushError, LablelInsertionError};
use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use database::models::alerts::{
    DatabaseAlert, InsertableAlert, InsertableAlertAnnotation, InsertableAlertLabel,
};
use database::models::annotations::{InsertableAnnotation, InsertableCommonAnnotation};
use database::models::groups::{
    InsertableGroup, InsertableGroupCommonAnnotation, InsertableGroupCommonLabel,
    InsertableGroupLabel,
};
use database::models::labels::{InsertableCommonLabel, InsertableLabel};
use diesel::{
    BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, GroupedBy,
    OptionalExtension, PgConnection, QueryDsl, SelectableHelper,
};

use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncConnection, AsyncPgConnection,
    RunQueryDsl,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use error::InternalPullError;
use models::{Alert as AlertmanagerPushAlert, AlertmanagerPush, StandAloneAlert};
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use pull_definitions::{Pull, PullAlertsFilter, PullError};
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
            .context("Failed to create pool")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }

    async fn insert_group(
        conn: &mut AsyncPgConnection,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<i32, InternalPushError> {
        let group = InsertableGroup {
            receiver: &alertmanager_push.receiver,
            status: &AlertStatusModel::from(&alertmanager_push.status),
            external_url: &alertmanager_push.external_url,
            group_key: &alertmanager_push.group_key,
        };

        let group_id = diesel::insert_into(database::schema::groups::table)
            .values(&group)
            .returning(database::schema::groups::id)
            .get_result::<i32>(conn)
            .await
            .map_err(|error| InternalPushError::GroupInsertion {
                group_key: alertmanager_push.group_key.clone(),
                error,
            })?;

        Ok(group_id)
    }

    async fn assign_group_label(
        conn: &mut AsyncPgConnection,
        group_id: i32,
        label_id: i32,
        label: &InsertableLabel<'_>,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let group_label = InsertableGroupLabel { group_id, label_id };

        diesel::insert_into(database::schema::groups_labels::table)
            .values(&group_label)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::GroupLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                label_name: label.name.to_owned(),
                label_value: label.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    /// Helper function
    ///
    /// Only labels are shared between [`crate::database::models::groups::Group`] and [`crate::database::models::alerts::Alert`].
    async fn insert_label(
        conn: &mut AsyncPgConnection,
        label: &InsertableLabel<'_>,
    ) -> Result<i32, LablelInsertionError> {
        let label_id_opt = database::schema::labels::table
            .filter(
                database::schema::labels::name
                    .eq(&label.name)
                    .and(database::schema::labels::value.eq(&label.value)),
            )
            .select(database::schema::labels::id)
            .get_result::<i32>(conn)
            .await
            .optional()
            .map_err(LablelInsertionError::Get)?;

        let label_id = match label_id_opt {
            Some(label_id) => {
                tracing::trace!(
                    name = %label.name,
                    value = %label.value,
                    "Label already exists."
                );
                label_id
            }
            None => diesel::insert_into(database::schema::labels::table)
                .values(label)
                .returning(database::schema::labels::id)
                .get_result::<i32>(conn)
                .await
                .map_err(LablelInsertionError::Insert)?,
        };

        Ok(label_id)
    }

    async fn insert_group_lables(
        conn: &mut AsyncPgConnection,
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for label in alertmanager_push.group_labels.iter() {
            let label = InsertableLabel {
                name: label.0,
                value: label.1,
            };

            let label_id = Self::insert_label(conn, &label)
                .await
                .map_err(|error| match error {
                    LablelInsertionError::Get(error) => InternalPushError::GroupLabelId {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: label.name.to_owned(),
                        label_value: label.value.to_owned(),
                        error,
                    },
                    LablelInsertionError::Insert(error) => InternalPushError::GroupLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: label.name.to_owned(),
                        label_value: label.value.to_owned(),
                        error,
                    },
                })?;

            Self::assign_group_label(conn, group_id, label_id, &label, alertmanager_push).await?;
        }

        Ok(())
    }

    async fn assign_group_common_label(
        conn: &mut AsyncPgConnection,
        group_id: i32,
        common_label_id: i32,
        common_label: &InsertableCommonLabel<'_>,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let group_common_label = InsertableGroupCommonLabel {
            group_id,
            common_label_id,
        };

        diesel::insert_into(database::schema::groups_common_labels::table)
            .values(&group_common_label)
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
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for common_label in alertmanager_push.common_labels.iter() {
            let common_label = InsertableCommonLabel {
                name: common_label.0,
                value: common_label.1,
            };

            let common_label_id_opt = database::schema::common_labels::table
                .filter(
                    database::schema::common_labels::name
                        .eq(&common_label.name)
                        .and(database::schema::common_labels::value.eq(&common_label.value)),
                )
                .select(database::schema::common_labels::id)
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
                None => diesel::insert_into(database::schema::common_labels::table)
                    .values(&common_label)
                    .returning(database::schema::common_labels::id)
                    .get_result::<i32>(conn)
                    .await
                    .map_err(|error| InternalPushError::CommonLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: common_label.name.to_owned(),
                        label_value: common_label.value.to_owned(),
                        error,
                    })?,
            };

            Self::assign_group_common_label(
                conn,
                group_id,
                common_label_id,
                &common_label,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn assign_group_common_annotation(
        conn: &mut AsyncPgConnection,
        group_id: i32,
        common_annotation_id: i32,
        common_annotation: &InsertableCommonAnnotation<'_>,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let group_common_annotation = InsertableGroupCommonAnnotation {
            group_id,
            common_annotation_id,
        };

        diesel::insert_into(database::schema::groups_common_annotations::table)
            .values(&group_common_annotation)
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
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for common_annotation in alertmanager_push.common_annotations.iter() {
            let common_annotation = InsertableCommonAnnotation {
                name: common_annotation.0,
                value: common_annotation.1,
            };

            let common_annotation_id_opt = database::schema::common_annotations::table
                .filter(
                    database::schema::common_annotations::name
                        .eq(&common_annotation.name)
                        .and(
                            database::schema::common_annotations::value
                                .eq(&common_annotation.value),
                        ),
                )
                .select(database::schema::common_annotations::id)
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
                None => diesel::insert_into(database::schema::common_annotations::table)
                    .values(&common_annotation)
                    .returning(database::schema::common_annotations::id)
                    .get_result::<i32>(conn)
                    .await
                    .map_err(|error| InternalPushError::CommonAnnotationInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        annotation_name: common_annotation.name.to_owned(),
                        annotation_value: common_annotation.value.to_owned(),
                        error,
                    })?,
            };

            Self::assign_group_common_annotation(
                conn,
                group_id,
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
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &AlertmanagerPushAlert,
    ) -> Result<i32, InternalPushError> {
        let insertable_alert = InsertableAlert {
            group_id,
            group_key: &alertmanager_push.group_key,
            status: &AlertStatusModel::from(&alert.status),
            starts_at: alert.starts_at,
            ends_at: alert.ends_at,
            generator_url: &alert.generator_url,
            fingerprint: &alert.fingerprint,
        };

        let alert_id = diesel::insert_into(database::schema::alerts::table)
            .values(&insertable_alert)
            .returning(database::schema::alerts::id)
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
        label_id: i32,
        label: &InsertableLabel<'_>,
        alert: &AlertmanagerPushAlert,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let alert_label = InsertableAlertLabel { alert_id, label_id };

        diesel::insert_into(database::schema::alerts_labels::table)
            .values(&alert_label)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::AlertLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                label_name: label.name.to_owned(),
                label_value: label.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    async fn insert_alert_labels(
        conn: &mut AsyncPgConnection,
        alert_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &AlertmanagerPushAlert,
    ) -> Result<(), InternalPushError> {
        for label in alert.labels.iter() {
            let label = InsertableLabel {
                name: label.0,
                value: label.1,
            };

            let label_id = Self::insert_label(conn, &label)
                .await
                .map_err(|error| match error {
                    LablelInsertionError::Get(error) => InternalPushError::AlertLabelId {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        label_name: label.name.to_owned(),
                        label_value: label.value.to_owned(),
                        error,
                    },
                    LablelInsertionError::Insert(error) => InternalPushError::AlertLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        label_name: label.name.to_owned(),
                        label_value: label.value.to_owned(),
                        error,
                    },
                })?;

            Self::assign_alert_label(conn, alert_id, label_id, &label, alert, alertmanager_push)
                .await?;
        }

        Ok(())
    }

    async fn assign_alert_annotation(
        conn: &mut AsyncPgConnection,
        alert_id: i32,
        annotation_id: i32,
        annotation: &InsertableAnnotation<'_>,
        alert: &AlertmanagerPushAlert,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let alert_annotation = InsertableAlertAnnotation {
            alert_id,
            annotation_id,
        };

        diesel::insert_into(database::schema::alerts_annotations::table)
            .values(&alert_annotation)
            .execute(conn)
            .await
            .map_err(|error| InternalPushError::AlertAnnotationAssignment {
                group_key: alertmanager_push.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                annotation_name: annotation.name.to_owned(),
                annotation_value: annotation.value.to_owned(),
                error,
            })?;

        Ok(())
    }

    async fn insert_alert_annotations(
        conn: &mut AsyncPgConnection,
        alert_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &AlertmanagerPushAlert,
    ) -> Result<(), InternalPushError> {
        for annotation in alert.annotations.iter() {
            let annotation = InsertableAnnotation {
                name: annotation.0,
                value: annotation.1,
            };

            let alert_annotation_id_opt = database::schema::annotations::table
                .filter(
                    database::schema::annotations::name
                        .eq(&annotation.name)
                        .and(database::schema::annotations::value.eq(&annotation.value)),
                )
                .select(database::schema::annotations::id)
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
                        "Annotation already exists."
                    );
                    alert_annotation_id
                }
                None => diesel::insert_into(database::schema::annotations::table)
                    .values(&annotation)
                    .returning(database::schema::annotations::id)
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
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for alert in alertmanager_push.alerts.iter() {
            let alert_id = Self::insert_alert(conn, group_id, alertmanager_push, alert).await?;
            Self::insert_alert_labels(conn, alert_id, alertmanager_push, alert).await?;
            Self::insert_alert_annotations(conn, alert_id, alertmanager_push, alert).await?;
        }

        Ok(())
    }

    async fn pull_alerts_with_internal_pull_error(
        conn: &mut AsyncPgConnection,
        filter: &PullAlertsFilter,
    ) -> Result<Vec<StandAloneAlert>, InternalPullError> {
        let alerts: Vec<Alert> = database::schema::alerts::table
            .select(Alert::as_select())
            .load(conn)
            .await
            .map_err(InternalPullError::Alerts)?;

        let labels: Vec<(AlertLabel, Label)> = AlertLabel::belonging_to(&alerts)
            .inner_join(database::schema::labels::table)
            .select((AlertLabel::as_select(), Label::as_select()))
            .load(conn)
            .await
            .map_err(InternalPullError::Labels)?;

        let annotations: Vec<(AlertAnnotation, Annotation)> =
            AlertAnnotation::belonging_to(&alerts)
                .inner_join(database::schema::annotations::table)
                .select((AlertAnnotation::as_select(), Annotation::as_select()))
                .load(conn)
                .await
                .map_err(InternalPullError::Annotations)?;

        let labels_per_alert: Vec<(&Alert, Vec<Label>)> = labels
            .grouped_by(&alerts)
            .into_iter()
            .zip(&alerts)
            .map(|(labels, alert)| (alert, labels.into_iter().map(|(_, label)| label).collect()))
            .collect();

        let annotations_per_alert: Vec<(&Alert, Vec<Annotation>)> = annotations
            .grouped_by(&alerts)
            .into_iter()
            .zip(&alerts)
            .map(|(annotations, alert)| {
                (
                    alert,
                    annotations
                        .into_iter()
                        .map(|(_, annotation)| annotation)
                        .collect(),
                )
            })
            .collect();

        let database_alerts: Vec<DatabaseAlert> = labels_per_alert
            .into_iter()
            .zip(annotations_per_alert)
            .map(|((alert, labels), (_, annotations))| DatabaseAlert {
                alert: alert.clone(),
                labels,
                annotations,
            })
            .collect();

        Ok(database_alerts
            .into_iter()
            .map(|alert| alert.into())
            .collect())
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
                .context("Failed to establish connection")?;

            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|error| anyhow::anyhow!(error))
                .context("Failed to run migrations")?;

            Ok(())
        });

        handle
            .await
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?
            .map_err(|error| InitializeError {
                reason: format!("{:#}", error),
            })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        let mut conn = self.pool.get().await.map_err(InternalPushError::Acquire)?;

        conn.transaction::<(), InternalPushError, _>(|conn| {
            async move {
                tracing::trace!("Starting transaction.");

                let group_id = Self::insert_group(conn, alertmanager_push).await?;
                Self::insert_group_lables(conn, group_id, alertmanager_push).await?;
                Self::insert_common_labels(conn, group_id, alertmanager_push).await?;
                Self::insert_common_annotations(conn, group_id, alertmanager_push).await?;
                Self::insert_alerts(conn, group_id, alertmanager_push).await?;

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

#[async_trait]
impl Pull for PostgresPlugin {
    #[tracing::instrument(name = "pull_alerts", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn pull_alerts(
        &self,
        filter: &PullAlertsFilter,
    ) -> Result<Vec<StandAloneAlert>, PullError> {
        tracing::trace!("Pulling.");

        let mut conn = self.pool.get().await.map_err(InternalPullError::Acquire)?;

        let alerts = Self::pull_alerts_with_internal_pull_error(&mut conn, filter).await?;

        tracing::trace!("Successfully pulled.");
        Ok(alerts)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use models::utils::generate_random_alertmanager_pushes;
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
