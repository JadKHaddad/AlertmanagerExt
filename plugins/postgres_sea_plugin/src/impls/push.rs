use crate::{
    entities::{
        alerts, alerts_annotations, alerts_labels, annotations, common_annotations, common_labels,
        groups, groups_common_annotations, groups_common_labels, groups_labels, labels,
        sea_orm_active_enums::AlertStatus,
    },
    error::{InternalInitializeError, InternalPushError, LablelInsertionError},
    PostgresSeaPlugin,
};
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use models::{Alert as AlertmanagerPushAlert, AlertmanagerPush};
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};

impl PostgresSeaPlugin {
    async fn initialize_with_internal_error(&mut self) -> Result<(), InternalInitializeError> {
        Migrator::up(&self.db, None).await?;

        Ok(())
    }

    async fn insert_group<'a, C: ConnectionTrait>(
        db: &'a C,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<i32, InternalPushError> {
        let group_id = groups::ActiveModel {
            group_key: Set(alertmanager_push.group_key.clone()),
            receiver: Set(alertmanager_push.receiver.clone()),
            status: Set(AlertStatus::from(&alertmanager_push.status)),
            external_url: Set(alertmanager_push.external_url.clone()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|error| InternalPushError::GroupInsertion {
            group_key: alertmanager_push.group_key.clone(),
            error,
        })?
        .id;

        Ok(group_id)
    }

    async fn assign_group_label<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        label_id: i32,
        label: (&String, &String),
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        tracing::trace!(
            group_id,
            label_id,
            name = %label.0,
            value = %label.1,
            "Assigning group label.");

        groups_labels::ActiveModel {
            group_id: Set(group_id),
            label_id: Set(label_id),
        }
        .insert(db)
        .await
        .map_err(|error| InternalPushError::GroupLabelAssignment {
            group_key: alertmanager_push.group_key.clone(),
            label_name: label.0.clone(),
            label_value: label.1.clone(),
            error,
        })?;

        Ok(())
    }

    async fn get_or_insert_label<'a, C: ConnectionTrait>(
        db: &'a C,
        label: (&String, &String),
    ) -> Result<i32, LablelInsertionError> {
        let label_id_opt = labels::Entity::find()
            .filter(
                labels::Column::Name
                    .eq(label.0)
                    .and(labels::Column::Value.eq(label.1)),
            )
            .one(db)
            .await
            .map_err(LablelInsertionError::Get)?
            .map(|label| label.id);

        let label_id = match label_id_opt {
            Some(label_id) => {
                tracing::trace!(
                    name = %label.0,
                    value = %label.1,
                    "Label already exists."
                );
                label_id
            }
            None => {
                labels::ActiveModel {
                    name: Set(label.0.clone()),
                    value: Set(label.1.clone()),
                    ..Default::default()
                }
                .insert(db)
                .await
                .map_err(LablelInsertionError::Insert)?
                .id
            }
        };

        Ok(label_id)
    }

    async fn insert_group_lables<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for label in alertmanager_push.group_labels.iter() {
            let label_id =
                Self::get_or_insert_label(db, label)
                    .await
                    .map_err(|error| match error {
                        LablelInsertionError::Get(error) => InternalPushError::GroupLabelId {
                            group_key: alertmanager_push.group_key.clone(),
                            label_name: label.0.clone(),
                            label_value: label.1.clone(),
                            error,
                        },
                        LablelInsertionError::Insert(error) => {
                            InternalPushError::GroupLabelInsertion {
                                group_key: alertmanager_push.group_key.clone(),
                                label_name: label.0.clone(),
                                label_value: label.1.clone(),
                                error,
                            }
                        }
                    })?;

            Self::assign_group_label(db, group_id, label_id, label, alertmanager_push).await?;
        }

        Ok(())
    }

    async fn assign_group_common_label<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        common_label_id: i32,
        common_label: (&String, &String),
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        tracing::trace!(
            group_id,
            common_label_id,
            name = %common_label.0,
            value = %common_label.1,
            "Assigning group common label.");

        groups_common_labels::ActiveModel {
            group_id: Set(group_id),
            common_label_id: Set(common_label_id),
        }
        .insert(db)
        .await
        .map_err(|error| InternalPushError::CommonLabelAssignment {
            group_key: alertmanager_push.group_key.clone(),
            label_name: common_label.0.clone(),
            label_value: common_label.1.clone(),
            error,
        })?;

        Ok(())
    }

    async fn insert_common_labels<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for common_label in alertmanager_push.common_labels.iter() {
            let common_label_id_opt = common_labels::Entity::find()
                .filter(
                    common_labels::Column::Name
                        .eq(common_label.0)
                        .and(common_labels::Column::Value.eq(common_label.1)),
                )
                .one(db)
                .await
                .map_err(|error| InternalPushError::CommonLabelId {
                    group_key: alertmanager_push.group_key.clone(),
                    label_name: common_label.0.clone(),
                    label_value: common_label.1.clone(),
                    error,
                })?
                .map(|label| label.id);

            let common_label_id = match common_label_id_opt {
                Some(common_label_id) => {
                    tracing::trace!(
                        name = %common_label.0,
                        value = %common_label.1,
                        "Common Label already exists."
                    );
                    common_label_id
                }
                None => {
                    common_labels::ActiveModel {
                        name: Set(common_label.0.clone()),
                        value: Set(common_label.1.clone()),
                        ..Default::default()
                    }
                    .insert(db)
                    .await
                    .map_err(|error| InternalPushError::CommonLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        label_name: common_label.0.clone(),
                        label_value: common_label.1.clone(),
                        error,
                    })?
                    .id
                }
            };

            Self::assign_group_common_label(
                db,
                group_id,
                common_label_id,
                common_label,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn assign_group_common_annotation<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        common_annotation_id: i32,
        common_annotation: (&String, &String),
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        tracing::trace!(
            group_id,
            common_annotation_id,
            name = %common_annotation.0,
            value = %common_annotation.1,
            "Assigning group common annotation.");

        groups_common_annotations::ActiveModel {
            group_id: Set(group_id),
            common_annotation_id: Set(common_annotation_id),
        }
        .insert(db)
        .await
        .map_err(|error| InternalPushError::CommonAnnotationAssignment {
            group_key: alertmanager_push.group_key.clone(),
            annotation_name: common_annotation.0.clone(),
            annotation_value: common_annotation.1.clone(),
            error,
        })?;

        Ok(())
    }

    async fn insert_common_annotations<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for common_annotation in alertmanager_push.common_annotations.iter() {
            let common_annotation_id_opt = common_annotations::Entity::find()
                .filter(
                    common_annotations::Column::Name
                        .eq(common_annotation.0)
                        .and(common_annotations::Column::Value.eq(common_annotation.1)),
                )
                .one(db)
                .await
                .map_err(|error| InternalPushError::CommonAnnotationId {
                    group_key: alertmanager_push.group_key.clone(),
                    annotation_name: common_annotation.0.clone(),
                    annotation_value: common_annotation.1.clone(),
                    error,
                })?
                .map(|annotation| annotation.id);

            let common_annotation_id = match common_annotation_id_opt {
                Some(common_annotation_id) => {
                    tracing::trace!(
                        name = %common_annotation.0,
                        value = %common_annotation.1,
                        "Common annotation already exists."
                    );
                    common_annotation_id
                }
                None => {
                    common_annotations::ActiveModel {
                        name: Set(common_annotation.0.clone()),
                        value: Set(common_annotation.1.clone()),
                        ..Default::default()
                    }
                    .insert(db)
                    .await
                    .map_err(|error| InternalPushError::CommonAnnotationInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        annotation_name: common_annotation.0.clone(),
                        annotation_value: common_annotation.1.clone(),
                        error,
                    })?
                    .id
                }
            };

            Self::assign_group_common_annotation(
                db,
                group_id,
                common_annotation_id,
                common_annotation,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn insert_alert<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &AlertmanagerPushAlert,
    ) -> Result<i32, InternalPushError> {
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
        .insert(db)
        .await
        .map_err(|error| InternalPushError::AlertInsertion {
            group_key: alertmanager_push.group_key.clone(),
            fingerprint: alert.fingerprint.clone(),
            error,
        })?
        .id;

        Ok(alert_id)
    }

    async fn assign_alert_label<'a, C: ConnectionTrait>(
        db: &'a C,
        alert_id: i32,
        label_id: i32,
        label: (&String, &String),
        alert: &AlertmanagerPushAlert,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        tracing::trace!(
            alert_id,
            label_id,
            name = %label.0,
            value = %label.1,
            "Assigning alert label.");

        alerts_labels::ActiveModel {
            alert_id: Set(alert_id),
            label_id: Set(label_id),
        }
        .insert(db)
        .await
        .map_err(|error| InternalPushError::AlertLabelAssignment {
            group_key: alertmanager_push.group_key.clone(),
            fingerprint: alert.fingerprint.clone(),
            label_name: label.0.clone(),
            label_value: label.1.clone(),
            error,
        })?;

        Ok(())
    }

    async fn insert_alert_labels<'a, C: ConnectionTrait>(
        db: &'a C,
        alert_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &AlertmanagerPushAlert,
    ) -> Result<(), InternalPushError> {
        for label in alert.labels.iter() {
            let label_id_opt = labels::Entity::find()
                .filter(
                    labels::Column::Name
                        .eq(label.0)
                        .and(labels::Column::Value.eq(label.1)),
                )
                .one(db)
                .await
                .map_err(|error| InternalPushError::AlertLabelId {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    label_name: label.0.clone(),
                    label_value: label.1.clone(),
                    error,
                })?
                .map(|label| label.id);

            let label_id = match label_id_opt {
                Some(label_id) => {
                    tracing::trace!(
                        name = %label.0,
                        value = %label.1,
                        "Label already exists."
                    );
                    label_id
                }
                None => {
                    labels::ActiveModel {
                        name: Set(label.0.clone()),
                        value: Set(label.1.clone()),
                        ..Default::default()
                    }
                    .insert(db)
                    .await
                    .map_err(|error| InternalPushError::AlertLabelInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        label_name: label.0.clone(),
                        label_value: label.1.clone(),
                        error,
                    })?
                    .id
                }
            };

            Self::assign_alert_label(db, alert_id, label_id, label, alert, alertmanager_push)
                .await?;
        }
        Ok(())
    }

    async fn assign_alert_annotation<'a, C: ConnectionTrait>(
        db: &'a C,
        alert_id: i32,
        annotation_id: i32,
        annotation: (&String, &String),
        alert: &AlertmanagerPushAlert,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        tracing::trace!(
            alert_id,
            annotation_id,
            name = %annotation.0,
            value = %annotation.1,
            "Assigning alert annotation.");

        alerts_annotations::ActiveModel {
            alert_id: Set(alert_id),
            annotation_id: Set(annotation_id),
        }
        .insert(db)
        .await
        .map_err(|error| InternalPushError::AlertAnnotationAssignment {
            group_key: alertmanager_push.group_key.clone(),
            fingerprint: alert.fingerprint.clone(),
            annotation_name: annotation.0.clone(),
            annotation_value: annotation.1.clone(),
            error,
        })?;

        Ok(())
    }

    async fn insert_alert_annotations<'a, C: ConnectionTrait>(
        db: &'a C,
        alert_id: i32,
        alertmanager_push: &AlertmanagerPush,
        alert: &AlertmanagerPushAlert,
    ) -> Result<(), InternalPushError> {
        for annotation in alert.annotations.iter() {
            let annotation_id_opt = annotations::Entity::find()
                .filter(
                    annotations::Column::Name
                        .eq(annotation.0)
                        .and(annotations::Column::Value.eq(annotation.1)),
                )
                .one(db)
                .await
                .map_err(|error| InternalPushError::AlertAnnotationId {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    annotation_name: annotation.0.clone(),
                    annotation_value: annotation.1.clone(),
                    error,
                })?
                .map(|annotation| annotation.id);

            let annotation_id = match annotation_id_opt {
                Some(annotation_id) => {
                    tracing::trace!(
                        name = %annotation.0,
                        value = %annotation.1,
                        "Annotation already exists."
                    );
                    annotation_id
                }
                None => {
                    annotations::ActiveModel {
                        name: Set(annotation.0.clone()),
                        value: Set(annotation.1.clone()),
                        ..Default::default()
                    }
                    .insert(db)
                    .await
                    .map_err(|error| InternalPushError::AlertAnnotationInsertion {
                        group_key: alertmanager_push.group_key.clone(),
                        fingerprint: alert.fingerprint.clone(),
                        annotation_name: annotation.0.clone(),
                        annotation_value: annotation.1.clone(),
                        error,
                    })?
                    .id
                }
            };

            Self::assign_alert_annotation(
                db,
                alert_id,
                annotation_id,
                annotation,
                alert,
                alertmanager_push,
            )
            .await?;
        }

        Ok(())
    }

    async fn insert_alerts<'a, C: ConnectionTrait>(
        db: &'a C,
        group_id: i32,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        for alert in alertmanager_push.alerts.iter() {
            let alert_id = Self::insert_alert(db, group_id, alertmanager_push, alert).await?;
            Self::insert_alert_labels(db, alert_id, alertmanager_push, alert).await?;
            Self::insert_alert_annotations(db, alert_id, alertmanager_push, alert).await?;
        }

        Ok(())
    }

    async fn push_alert_with_internal_error(
        &self,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        tracing::trace!("Beginning transaction.");

        let txn = self
            .db
            .begin()
            .await
            .map_err(InternalPushError::TransactionBegin)?;

        let group_id = Self::insert_group(&txn, alertmanager_push).await?;
        Self::insert_group_lables(&txn, group_id, alertmanager_push).await?;
        Self::insert_common_labels(&txn, group_id, alertmanager_push).await?;
        Self::insert_common_annotations(&txn, group_id, alertmanager_push).await?;
        Self::insert_alerts(&txn, group_id, alertmanager_push).await?;

        tracing::trace!("Committing transaction.");

        txn.commit()
            .await
            .map_err(InternalPushError::TransactionCommit)?;

        Ok(())
    }
}

#[async_trait]
impl Push for PostgresSeaPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        self.initialize_with_internal_error().await?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        self.push_alert_with_internal_error(alertmanager_push)
            .await?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
