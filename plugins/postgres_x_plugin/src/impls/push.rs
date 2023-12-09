use crate::{
    database::models::alert_status::AlertStatusModel, error::InternalPushError, PostgresXPlugin,
};
use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use sqlx::{Connection, Executor};

impl PostgresXPlugin {
    fn filter_already_exists_error(
        result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>,
    ) -> Result<(), sqlx::Error> {
        match result {
            Ok(_) => Ok(()),
            Err(error) => match error {
                sqlx::Error::Database(ref database_error) => {
                    if [
                        Some(std::borrow::Cow::Borrowed("42710")),
                        Some(std::borrow::Cow::Borrowed("42P07")),
                    ]
                    .contains(&database_error.code())
                    {
                        return Ok(());
                    }

                    Err(error)
                }
                error => Err(error),
            },
        }
    }
}

#[async_trait]
impl Push for PostgresXPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        sqlx::migrate!()
            .run(&self.pool)
            .await
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(InternalPushError::Acquire)?;

        tracing::trace!("Beginning transaction.");
        let mut tx = conn
            .begin()
            .await
            .map_err(InternalPushError::TransactionBegin)?;

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

        for (label_name, label_value) in alertmanager_push.group_labels.iter() {
            let label_id_opt = sqlx::query!(
                r#"
                SELECT id FROM labels WHERE name = $1 AND value = $2
                "#,
                label_name,
                label_value
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| InternalPushError::GroupLabelId {
                group_key: alertmanager_push.group_key.clone(),
                label_name: label_name.clone(),
                label_value: label_value.clone(),
                error,
            })?
            .map(|row| row.id);

            let label_id = match label_id_opt {
                Some(label_id) => {
                    tracing::trace!(
                        name = %label_name,
                        value = %label_value,
                        "Label already exists."
                    );
                    label_id
                }
                None => {
                    sqlx::query!(
                        r#"
                    INSERT INTO labels (name, value) VALUES ($1, $2) RETURNING id
                    "#,
                        label_name,
                        label_value
                    )
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|error| InternalPushError::GroupLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: label_name.clone(),
                        label_value: label_value.clone(),
                        error,
                    })?
                    .id
                }
            };

            sqlx::query!(
                r#"
                INSERT INTO groups_labels (group_id, label_id) VALUES ($1, $2)
                "#,
                group_id,
                label_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|error| InternalPushError::GroupLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                label_name: label_name.clone(),
                label_value: label_value.clone(),
                error,
            })?;
        }

        for (common_label_name, common_label_value) in alertmanager_push.common_labels.iter() {
            let common_label_id_opt = sqlx::query!(
                r#"
                SELECT id FROM common_labels WHERE name = $1 AND value = $2
                "#,
                common_label_name,
                common_label_value
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| InternalPushError::CommonLabelId {
                group_key: alertmanager_push.group_key.clone(),
                label_name: common_label_name.clone(),
                label_value: common_label_value.clone(),
                error,
            })?
            .map(|row| row.id);

            let common_label_id = match common_label_id_opt {
                Some(label_id) => {
                    tracing::trace!(
                        name = %common_label_name,
                        value = %common_label_value,
                        "Common label already exists."
                    );
                    label_id
                }
                None => {
                    sqlx::query!(
                        r#"
                    INSERT INTO common_labels (name, value) VALUES ($1, $2) RETURNING id
                    "#,
                        common_label_name,
                        common_label_value
                    )
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|error| InternalPushError::CommonLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: common_label_name.clone(),
                        label_value: common_label_value.clone(),
                        error,
                    })?
                    .id
                }
            };

            sqlx::query!(
                r#"
                INSERT INTO groups_common_labels (group_id, common_label_id) VALUES ($1, $2)
                "#,
                group_id,
                common_label_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|error| InternalPushError::CommonLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                label_name: common_label_name.clone(),
                label_value: common_label_value.clone(),
                error,
            })?;
        }

        for (common_annotation_name, common_annotation_value) in
            alertmanager_push.common_annotations.iter()
        {
            let common_annotation_id_opt = sqlx::query!(
                r#"
                SELECT id FROM common_annotations WHERE name = $1 AND value = $2
                "#,
                common_annotation_name,
                common_annotation_value
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| InternalPushError::CommonAnnotationId {
                group_key: alertmanager_push.group_key.clone(),
                annotation_name: common_annotation_name.clone(),
                annotation_value: common_annotation_value.clone(),
                error,
            })?
            .map(|row| row.id);

            let common_annotation_id = match common_annotation_id_opt {
                Some(annotation_id) => {
                    tracing::trace!(
                        name = %common_annotation_name,
                        value = %common_annotation_value,
                        "Common annotation already exists."
                    );
                    annotation_id
                }
                None => {
                    sqlx::query!(
                        r#"
                    INSERT INTO common_annotations (name, value) VALUES ($1, $2) RETURNING id
                    "#,
                        common_annotation_name,
                        common_annotation_value
                    )
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|error| InternalPushError::CommonAnnotationInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        annotation_name: common_annotation_name.clone(),
                        annotation_value: common_annotation_value.clone(),
                        error,
                    })?
                    .id
                }
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
                )
                .fetch_optional(&mut *tx)
                .await
                .map_err(|error| InternalPushError::AlertLabelId {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    label_name: label_name.clone(),
                    label_value: label_value.clone(),
                    error,
                })?
                .map(|row| row.id);

                let label_id = match label_id_opt {
                    Some(label_id) => {
                        tracing::trace!(
                            name = %label_name,
                            value = %label_value,
                            "Label already exists."
                        );
                        label_id
                    }
                    None => {
                        sqlx::query!(
                            r#"
                        INSERT INTO labels (name, value) VALUES ($1, $2) RETURNING id
                        "#,
                            label_name,
                            label_value
                        )
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|error| InternalPushError::AlertLabelInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            fingerprint: alert.fingerprint.clone(),
                            label_name: label_name.clone(),
                            label_value: label_value.clone(),
                            error,
                        })?
                        .id
                    }
                };

                sqlx::query!(
                    r#"
                    INSERT INTO alerts_labels (alert_id, label_id) VALUES ($1, $2)
                    "#,
                    alert_id,
                    label_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|error| InternalPushError::AlertLabelAssignment {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    label_name: label_name.clone(),
                    label_value: label_value.clone(),
                    error,
                })?;
            }

            for (annotation_name, annotation_value) in alert.annotations.iter() {
                let annotation_id_opt = sqlx::query!(
                    r#"
                    SELECT id FROM annotations WHERE name = $1 AND value = $2
                    "#,
                    annotation_name,
                    annotation_value
                )
                .fetch_optional(&mut *tx)
                .await
                .map_err(|error| InternalPushError::AlertAnnotationId {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    annotation_name: annotation_name.clone(),
                    annotation_value: annotation_value.clone(),
                    error,
                })?
                .map(|row| row.id);

                let annotation_id = match annotation_id_opt {
                    Some(annotation_id) => {
                        tracing::trace!(
                            name = %annotation_name,
                            value = %annotation_value,
                            "Annotation already exists."
                        );
                        annotation_id
                    }
                    None => {
                        sqlx::query!(
                            r#"
                        INSERT INTO annotations (name, value) VALUES ($1, $2) RETURNING id
                        "#,
                            annotation_name,
                            annotation_value
                        )
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|error| InternalPushError::AlertAnnotationInsertion {
                            group_key: alertmanager_push.group_key.clone(),
                            fingerprint: alert.fingerprint.clone(),
                            annotation_name: annotation_name.clone(),
                            annotation_value: annotation_value.clone(),
                            error,
                        })?
                        .id
                    }
                };

                sqlx::query!(
                    r#"
                    INSERT INTO alerts_annotations (alert_id, annotation_id) VALUES ($1, $2)
                    "#,
                    alert_id,
                    annotation_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|error| InternalPushError::AlertAnnotationAssignment {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    annotation_name: annotation_name.clone(),
                    annotation_value: annotation_value.clone(),
                    error,
                })?;
            }
        }

        tracing::trace!("Committing transaction.");
        tx.commit()
            .await
            .map_err(InternalPushError::TransactionCommit)?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
