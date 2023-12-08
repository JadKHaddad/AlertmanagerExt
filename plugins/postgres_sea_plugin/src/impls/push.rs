use crate::{
    entity::{
        alerts, alerts_annotations, alerts_labels, annotations, common_annotations, common_labels,
        groups, groups_common_annotations, groups_common_labels, groups_labels, labels, prelude::*,
        sea_orm_active_enums::AlertStatus,
    },
    error::InternalPushError,
    PostgresSeaPlugin,
};
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};

#[async_trait]
impl Push for PostgresSeaPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        Migrator::up(&self.db, None)
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

        tracing::trace!("Beginning transaction.");
        let txn = self
            .db
            .begin()
            .await
            .map_err(InternalPushError::TransactionBegin)?;

        let group_id = groups::ActiveModel {
            group_key: Set(alertmanager_push.group_key.clone()),
            receiver: Set(alertmanager_push.receiver.clone()),
            status: Set(AlertStatus::from(&alertmanager_push.status)),
            external_url: Set(alertmanager_push.external_url.clone()),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(|error| InternalPushError::GroupInsertion {
            group_key: alertmanager_push.group_key.clone(),
            error,
        })?
        .id;

        for (label_name, label_value) in alertmanager_push.group_labels.iter() {
            let label_id_opt = Labels::find()
                .filter(
                    labels::Column::Name
                        .eq(label_name)
                        .and(labels::Column::Value.eq(label_value)),
                )
                .one(&txn)
                .await
                .map_err(|error| InternalPushError::GroupLabelId {
                    group_key: alertmanager_push.group_key.clone(),
                    label_name: label_name.clone(),
                    label_value: label_value.clone(),
                    error,
                })?
                .map(|label| label.id);

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
                    labels::ActiveModel {
                        name: Set(label_name.clone()),
                        value: Set(label_value.clone()),
                        ..Default::default()
                    }
                    .insert(&txn)
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

            groups_labels::ActiveModel {
                group_id: Set(group_id),
                label_id: Set(label_id),
            }
            .insert(&txn)
            .await
            .map_err(|error| InternalPushError::GroupLabelAssignment {
                group_key: alertmanager_push.group_key.clone(),
                label_name: label_name.clone(),
                label_value: label_value.clone(),
                error,
            })?;
        }

        for (common_label_name, common_label_value) in alertmanager_push.common_labels.iter() {
            let common_label_id_opt = CommonLabels::find()
                .filter(
                    common_labels::Column::Name
                        .eq(common_label_name)
                        .and(common_labels::Column::Value.eq(common_label_value)),
                )
                .one(&txn)
                .await
                .map_err(|error| InternalPushError::CommonLabelId {
                    group_key: alertmanager_push.group_key.clone(),
                    label_name: common_label_name.clone(),
                    label_value: common_label_value.clone(),
                    error,
                })?
                .map(|label| label.id);

            let common_label_id = match common_label_id_opt {
                Some(common_label_id) => {
                    tracing::trace!(
                        name = %common_label_name,
                        value = %common_label_value,
                        "Common Label already exists."
                    );
                    common_label_id
                }
                None => {
                    common_labels::ActiveModel {
                        name: Set(common_label_name.clone()),
                        value: Set(common_label_value.clone()),
                        ..Default::default()
                    }
                    .insert(&txn)
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

            groups_common_labels::ActiveModel {
                group_id: Set(group_id),
                common_label_id: Set(common_label_id),
            }
            .insert(&txn)
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
            let common_annotation_id_opt = CommonAnnotations::find()
                .filter(
                    common_annotations::Column::Name
                        .eq(common_annotation_name)
                        .and(common_annotations::Column::Value.eq(common_annotation_value)),
                )
                .one(&txn)
                .await
                .map_err(|error| InternalPushError::CommonAnnotationId {
                    group_key: alertmanager_push.group_key.clone(),
                    annotation_name: common_annotation_name.clone(),
                    annotation_value: common_annotation_value.clone(),
                    error,
                })?
                .map(|annotation| annotation.id);

            let common_annotation_id = match common_annotation_id_opt {
                Some(common_annotation_id) => {
                    tracing::trace!(
                        name = %common_annotation_name,
                        value = %common_annotation_value,
                        "Common annotation already exists."
                    );
                    common_annotation_id
                }
                None => {
                    common_annotations::ActiveModel {
                        name: Set(common_annotation_name.clone()),
                        value: Set(common_annotation_value.clone()),
                        ..Default::default()
                    }
                    .insert(&txn)
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

            groups_common_annotations::ActiveModel {
                group_id: Set(group_id),
                common_annotation_id: Set(common_annotation_id),
            }
            .insert(&txn)
            .await
            .map_err(|error| InternalPushError::CommonAnnotationAssignment {
                group_key: alertmanager_push.group_key.clone(),
                annotation_name: common_annotation_name.clone(),
                annotation_value: common_annotation_value.clone(),
                error,
            })?;
        }

        for alert in alertmanager_push.alerts.iter() {
            let alert_id = alerts::ActiveModel {
                group_id: Set(group_id),
                group_key: Set(alertmanager_push.group_key.clone()),
                status: Set(AlertStatus::from(&alert.status)),
                starts_at: Set(alert.starts_at),
                ends_at: Set(alert.ends_at),
                generator_url: Set(alert.generator_url.clone()),
                fingerprint: Set(alert.fingerprint.clone()),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(|error| InternalPushError::AlertInsertion {
                group_key: alertmanager_push.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                error,
            })?
            .id;

            for (label_name, label_value) in alert.labels.iter() {
                let label_id_opt = Labels::find()
                    .filter(
                        labels::Column::Name
                            .eq(label_name)
                            .and(labels::Column::Value.eq(label_value)),
                    )
                    .one(&txn)
                    .await
                    .map_err(|error| InternalPushError::AlertLabelId {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        label_name: label_name.clone(),
                        label_value: label_value.clone(),
                        error,
                    })?
                    .map(|label| label.id);

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
                        labels::ActiveModel {
                            name: Set(label_name.clone()),
                            value: Set(label_value.clone()),
                            ..Default::default()
                        }
                        .insert(&txn)
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

                alerts_labels::ActiveModel {
                    alert_id: Set(alert_id),
                    label_id: Set(label_id),
                }
                .insert(&txn)
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
                let annotation_id_opt = Annotations::find()
                    .filter(
                        annotations::Column::Name
                            .eq(annotation_name)
                            .and(annotations::Column::Value.eq(annotation_value)),
                    )
                    .one(&txn)
                    .await
                    .map_err(|error| InternalPushError::AlertAnnotationId {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        annotation_name: annotation_name.clone(),
                        annotation_value: annotation_value.clone(),
                        error,
                    })?
                    .map(|annotation| annotation.id);

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
                        annotations::ActiveModel {
                            name: Set(annotation_name.clone()),
                            value: Set(annotation_value.clone()),
                            ..Default::default()
                        }
                        .insert(&txn)
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

                alerts_annotations::ActiveModel {
                    alert_id: Set(alert_id),
                    annotation_id: Set(annotation_id),
                }
                .insert(&txn)
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
        txn.commit()
            .await
            .map_err(InternalPushError::TransactionCommit)?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
