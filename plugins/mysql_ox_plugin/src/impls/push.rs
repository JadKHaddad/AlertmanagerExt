use crate::{
    database::models::{
        annotations::{InsertCommonAnnotation},
        groups::{
            InsertGroup, InsertGroupCommonAnnotation, InsertGroupCommonLabel, InsertGroupLabel,
        },
        labels::{InsertCommonLabel, InsertLabel},
    },
    error::InternalPushError,
    MysqlOXPlugin,
};
use async_trait::async_trait;
use models::AlertmanagerPush;
use ormx::Insert;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use sqlx::Connection;

#[async_trait]
impl Push for MysqlOXPlugin {
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

        let group_id = InsertGroup {
            group_key: alertmanager_push.group_key.clone(),
            receiver: alertmanager_push.receiver.clone(),
            status: alertmanager_push.status.clone().to_string(),
            external_url: alertmanager_push.external_url.clone(),
        }
        .insert(&mut tx)
        .await
        .map_err(|error| InternalPushError::GroupInsertion {
            group_key: alertmanager_push.group_key.clone(),
            error,
        })?
        .id;

        for (label_name, label_value) in alertmanager_push.group_labels.iter() {
            let label_id_opt = sqlx::query!(
                r#"
                SELECT id FROM labels WHERE name = ? AND value = ?
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
                    InsertLabel {
                        name: label_name.clone(),
                        value: label_value.clone(),
                    }
                    .insert(&mut tx)
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

            InsertGroupLabel { group_id, label_id }
                .insert(&mut tx)
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
                SELECT id FROM common_labels WHERE name = ? AND value = ?
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
                    InsertCommonLabel {
                        name: common_label_name.clone(),
                        value: common_label_value.clone(),
                    }
                    .insert(&mut tx)
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

            InsertGroupCommonLabel {
                group_id,
                common_label_id,
            }
            .insert(&mut tx)
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
                SELECT id FROM common_annotations WHERE name = ? AND value = ?
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
                    InsertCommonAnnotation {
                        name: common_annotation_name.clone(),
                        value: common_annotation_value.clone(),
                    }
                    .insert(&mut tx)
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

            InsertGroupCommonAnnotation {
                group_id,
                common_annotation_id,
            }
            .insert(&mut tx)
            .await
            .map_err(|error| InternalPushError::CommonAnnotationAssignment {
                group_key: alertmanager_push.group_key.clone(),
                annotation_name: common_annotation_name.clone(),
                annotation_value: common_annotation_value.clone(),
                error,
            })?;
        }

        // TODO insert alerts

        tracing::trace!("Committing transaction.");
        tx.commit()
            .await
            .map_err(InternalPushError::TransactionCommit)?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
