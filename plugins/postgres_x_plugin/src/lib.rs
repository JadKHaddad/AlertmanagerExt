use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use database::models::{
    alert_status::AlertStatusModel, alerts::DatabaseAlert, annotations::Annotation, labels::Label,
};
use error::InternalInitializeError;
use models::{AlertmanagerPush, StandAloneAlert};
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use pull_definitions::{Pull, PullAlertsFilter, PullError};
use push_definitions::{InitializeError, Push, PushError};
use sqlx::Connection;

use crate::error::InternalPushError;

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
/// 
/// Based on [`sqlx`]
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
            .context("Failed to create pool")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            pool,
        })
    }

    fn filter_already_exists_error(resource: &str, result: Result<sqlx::postgres::PgQueryResult,  sqlx::Error>) -> Result<(), sqlx::Error> {
        match result {
            Ok(_) => Ok(()),
            Err(error) => match error {
                sqlx::Error::Database(ref database_error) => {
                    if [Some(std::borrow::Cow::Borrowed("42710")) , Some(std::borrow::Cow::Borrowed("42P07"))].contains(&database_error.code()) {
                        tracing::trace!(resource = %resource, "Already exists.");
                        return Ok(());
                    }
                    
                    Err(error)

                }
                error => {
                    Err(error)
                }
            }
        }
    }

    async fn initialize_with_internal_initialize_error(&self) -> Result<(), InternalInitializeError> {
        let result = sqlx::query(include_str!("../queries/initialize/01_alert_status.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("alert_status",result).map_err(InternalInitializeError::AlertStatus)?;

        let result = sqlx::query(include_str!("../queries/initialize/02_groups.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("groups", result).map_err(InternalInitializeError::Groups)?;

        let result = sqlx::query(include_str!("../queries/initialize/03_alerts.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("alerts", result).map_err(InternalInitializeError::Alerts)?;

        let result = sqlx::query(include_str!("../queries/initialize/04_labels.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("labels", result).map_err(InternalInitializeError::Labels)?;

        let result = sqlx::query(include_str!("../queries/initialize/05_annotations.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("annotations", result).map_err(InternalInitializeError::Annotations)?;

        let result = sqlx::query(include_str!("../queries/initialize/06_common_labels.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("common_labels", result).map_err(InternalInitializeError::CommonLabels)?;

        let result = sqlx::query(include_str!("../queries/initialize/07_common_annotations.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("common_annotations", result).map_err(InternalInitializeError::CommonAnnotations)?;

        let result = sqlx::query(include_str!("../queries/initialize/08_groups_labels.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("groups_labels", result).map_err(InternalInitializeError::GroupsLabels)?;

        let result = sqlx::query(include_str!("../queries/initialize/09_groups_common_labels.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("groups_common_labels", result).map_err(InternalInitializeError::GroupsCommonLabels)?;

        let result = sqlx::query(include_str!("../queries/initialize/10_groups_common_annotations.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("groups_common_annotations", result).map_err(InternalInitializeError::GroupsCommonAnnotations)?;

        let result = sqlx::query(include_str!("../queries/initialize/11_alerts_labels.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("alerts_labels", result).map_err(InternalInitializeError::AlertsLabels)?;

        let result = sqlx::query(include_str!("../queries/initialize/12_alerts_annotations.sql")).execute(&self.pool).await;
        Self::filter_already_exists_error("alerts_annotations", result).map_err(InternalInitializeError::AlertsAnnotations)?;

        Ok(())
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
        let _ = self.config.take();

        self.initialize_with_internal_initialize_error().await?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        let mut conn = self.pool.acquire().await.map_err(InternalPushError::Acquire)?;

        let mut tx = conn.begin().await.map_err(InternalPushError::TransactionStart)?;

        let status = AlertStatusModel::from(&alertmanager_push.status);
        let group_id = sqlx::query!(
            r#"
            INSERT INTO groups (group_key, receiver, status, external_url) VALUES ($1, $2, $3, $4) RETURNING id
            "#, 
            alertmanager_push.group_key, 
            alertmanager_push.receiver, 
            status as AlertStatusModel, 
            alertmanager_push.external_url
        )
        .fetch_one(&mut *tx)
        .await.map_err(|error| InternalPushError::GroupInsertion{
            group_key: alertmanager_push.group_key.clone(),
            error
        })?
        .id;

        for (label_name, label_value)  in alertmanager_push.group_labels.iter() {
            let label_id_opt = sqlx::query!(
                r#"
                SELECT id FROM labels WHERE name = $1 AND value = $2
                "#,
                label_name,
                label_value
            ).fetch_optional(&mut *tx).await.map_err(|error| InternalPushError::GroupLabelId{
                group_key: alertmanager_push.group_key.clone(),
                label_name: label_name.clone(),
                label_value: label_value.clone(),
                error
            })?.map(|row| row.id);

            let label_id = match label_id_opt {
                Some(label_id) => {
                    tracing::trace!(
                        name = %label_name,
                        value = %label_value,
                        "Label already exists."
                    );
                    label_id
                }
                None => sqlx::query!(
                    r#"
                    INSERT INTO labels (name, value) VALUES ($1, $2) RETURNING id
                    "#,
                    label_name,
                    label_value
                ).fetch_one(&mut *tx).await.map_err(|error| InternalPushError::GroupLabelInsertion{
                    group_key: alertmanager_push.group_key.clone(),
                    label_name: label_name.clone(),
                    label_value: label_value.clone(),
                    error
                })?.id
            };

            sqlx::query!(
                r#"
                INSERT INTO groups_labels (group_id, label_id) VALUES ($1, $2)
                "#,
                group_id,
                label_id
            ).execute(&mut *tx).await.map_err(|error| InternalPushError::GroupLabelAssignment{
                group_key: alertmanager_push.group_key.clone(),
                label_name: label_name.clone(),
                label_value: label_value.clone(),
                error
            })?;
        }

        for (common_label_name, common_label_value)  in alertmanager_push.common_labels.iter() {
            let common_label_id_opt = sqlx::query!(
                r#"
                SELECT id FROM common_labels WHERE name = $1 AND value = $2
                "#,
                common_label_name,
                common_label_value
            ).fetch_optional(&mut *tx).await.map_err(|error| InternalPushError::CommonLabelId{
                group_key: alertmanager_push.group_key.clone(),
                label_name: common_label_name.clone(),
                label_value: common_label_value.clone(),
                error
            })?.map(|row| row.id);

            let common_label_id = match common_label_id_opt {
                Some(label_id) => {
                    tracing::trace!(
                        name = %common_label_name,
                        value = %common_label_value,
                        "Common label already exists."
                    );
                    label_id
                }
                None => sqlx::query!(
                    r#"
                    INSERT INTO common_labels (name, value) VALUES ($1, $2) RETURNING id
                    "#,
                    common_label_name,
                    common_label_value
                ).fetch_one(&mut *tx).await.map_err(|error| InternalPushError::CommonLabelInsertion{
                    group_key: alertmanager_push.group_key.clone(),
                    label_name: common_label_name.clone(),
                    label_value: common_label_value.clone(),
                    error
                })?.id
            };

            sqlx::query!(
                r#"
                INSERT INTO groups_common_labels (group_id, common_label_id) VALUES ($1, $2)
                "#,
                group_id,
                common_label_id
            ).execute(&mut *tx).await.map_err(|error| InternalPushError::CommonLabelAssignment{
                group_key: alertmanager_push.group_key.clone(),
                label_name: common_label_name.clone(),
                label_value: common_label_value.clone(),
                error
            })?;
        }

        for (common_annotation_name, common_annotation_value)  in alertmanager_push.common_annotations.iter() {
            let common_annotation_id_opt = sqlx::query!(
                r#"
                SELECT id FROM common_annotations WHERE name = $1 AND value = $2
                "#,
                common_annotation_name,
                common_annotation_value
            ).fetch_optional(&mut *tx).await.map_err(|error| InternalPushError::CommonAnnotationId{
                group_key: alertmanager_push.group_key.clone(),
                annotation_name: common_annotation_name.clone(),
                annotation_value: common_annotation_value.clone(),
                error
            })?.map(|row| row.id);

            let common_annotation_id = match common_annotation_id_opt {
                Some(annotation_id) => {
                    tracing::trace!(
                        name = %common_annotation_name,
                        value = %common_annotation_value,
                        "Common annotation already exists."
                    );
                    annotation_id
                }
                None => sqlx::query!(
                    r#"
                    INSERT INTO common_annotations (name, value) VALUES ($1, $2) RETURNING id
                    "#,
                    common_annotation_name,
                    common_annotation_value
                ).fetch_one(&mut *tx).await.map_err(|error| InternalPushError::CommonAnnotationInsertion{
                    group_key: alertmanager_push.group_key.clone(),
                    annotation_name: common_annotation_name.clone(),
                    annotation_value: common_annotation_value.clone(),
                    error
                })?.id
            };

            sqlx::query!(
                r#"
                INSERT INTO groups_common_annotations (group_id, common_annotation_id) VALUES ($1, $2)
                "#,
                group_id,
                common_annotation_id
            ).execute(&mut *tx).await.map_err(|error| InternalPushError::CommonAnnotationAssignment{
                group_key: alertmanager_push.group_key.clone(),
                annotation_name: common_annotation_name.clone(),
                annotation_value: common_annotation_value.clone(),
                error
            })?;
        }

        for alert in alertmanager_push.alerts.iter() {
            let status = AlertStatusModel::from(&alert.status);
            let alert_id = sqlx::query!(
                r#"
                INSERT INTO alerts (group_id, group_key, status, starts_at, ends_at, generator_url, fingerprint) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id
                "#,
                group_id,
                alertmanager_push.group_key,
                status as AlertStatusModel,
                alert.starts_at,
                alert.ends_at,
                alert.generator_url,
                alert.fingerprint
            )
            .fetch_one(&mut *tx)
            .await.map_err(|error| InternalPushError::AlertInsertion{
                group_key: alertmanager_push.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                error
            })?
            .id;

            for (label_name, label_value) in alert.labels.iter() {
                let label_id_opt = sqlx::query!(
                    r#"
                    SELECT id FROM labels WHERE name = $1 AND value = $2
                    "#,
                    label_name,
                    label_value
                ).fetch_optional(&mut *tx).await.map_err(|error| InternalPushError::AlertLabelId{
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    label_name: label_name.clone(),
                    label_value: label_value.clone(),
                    error
                })?.map(|row| row.id);

                let label_id = match label_id_opt {
                    Some(label_id) => {
                        tracing::trace!(
                            name = %label_name,
                            value = %label_value,
                            "Label already exists."
                        );
                        label_id
                    }
                    None => sqlx::query!(
                        r#"
                        INSERT INTO labels (name, value) VALUES ($1, $2) RETURNING id
                        "#,
                        label_name,
                        label_value
                    ).fetch_one(&mut *tx).await.map_err(|error| InternalPushError::AlertLabelInsertion{
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        label_name: label_name.clone(),
                        label_value: label_value.clone(),
                        error
                    })?.id
                };

                sqlx::query!(
                    r#"
                    INSERT INTO alerts_labels (alert_id, label_id) VALUES ($1, $2)
                    "#,
                    alert_id,
                    label_id
                ).execute(&mut *tx).await.map_err(|error| InternalPushError::AlertLabelAssignment{
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    label_name: label_name.clone(),
                    label_value: label_value.clone(),
                    error
                })?;
            }

            for (annotation_name, annotation_value) in alert.annotations.iter() {
                let annotation_id_opt = sqlx::query!(
                    r#"
                    SELECT id FROM annotations WHERE name = $1 AND value = $2
                    "#,
                    annotation_name,
                    annotation_value
                ).fetch_optional(&mut *tx).await.map_err(|error| InternalPushError::AlertAnnotationId{
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    annotation_name: annotation_name.clone(),
                    annotation_value: annotation_value.clone(),
                    error
                })?.map(|row| row.id);

                let annotation_id = match annotation_id_opt {
                    Some(annotation_id) => {
                        tracing::trace!(
                            name = %annotation_name,
                            value = %annotation_value,
                            "Annotation already exists."
                        );
                        annotation_id
                    }
                    None => sqlx::query!(
                        r#"
                        INSERT INTO annotations (name, value) VALUES ($1, $2) RETURNING id
                        "#,
                        annotation_name,
                        annotation_value
                    ).fetch_one(&mut *tx).await.map_err(|error| InternalPushError::AlertAnnotationInsertion{
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        annotation_name: annotation_name.clone(),
                        annotation_value: annotation_value.clone(),
                        error
                    })?.id
                };

                sqlx::query!(
                    r#"
                    INSERT INTO alerts_annotations (alert_id, annotation_id) VALUES ($1, $2)
                    "#,
                    alert_id,
                    annotation_id
                ).execute(&mut *tx).await.map_err(|error| InternalPushError::AlertAnnotationAssignment{
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    annotation_name: annotation_name.clone(),
                    annotation_value: annotation_value.clone(),
                    error
                })?;
            }
        }

        tx.commit().await.map_err(InternalPushError::TransactionCommit)?;

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
    use models::utils::generate_random_alertmanager_pushes;
    use tracing_test::traced_test;

    async fn create_plugin() -> PostgresXPlugin {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let postgres_x_plugin_config = PostgresXPluginConfig {
            connection_string: database_url,
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
    async fn initialize() {
        let mut plugin = create_plugin().await;
        plugin.initialize().await.expect("Failed to initialize plugin.");
    }

    #[tokio::test]
    #[traced_test]
    // cargo test --package postgres_x_plugin --lib --release -- test::push_random_alerts --exact --nocapture
    async fn push_random_alerts() {
        let plugin = create_plugin().await;
        let pushes = generate_random_alertmanager_pushes(100);
        for (i, push) in pushes.iter().enumerate() {
            tracing::info!("Pushing alert {}/{}", i + 1, pushes.len());
            if let Err(error) = plugin.push_alert(push).await {
                eprintln!("Failed to push alert: {:?}", error)
            }
        }
    }

    #[tokio::test]
    #[traced_test]
    // cargo test --package postgres_x_plugin --lib --release -- test::pull_alerts --exact --nocapture
    async fn pull_alerts() {
        let plugin = create_plugin().await;
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
