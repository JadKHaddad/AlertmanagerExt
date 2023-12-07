use crate::{
    entity::{groups, groups_labels, labels, prelude::*, sea_orm_active_enums::AlertStatus},
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

        tracing::trace!("Committing transaction.");
        txn.commit()
            .await
            .map_err(InternalPushError::TransactionCommit)?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
