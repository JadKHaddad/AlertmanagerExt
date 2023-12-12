use crate::error::InternalPushError;
use crate::{
    database::models::{
        alert::{InsertableAlert, InsertableAlertAnnotation, InsertableAlertLabel},
        group::{
            InsertableAlertGroup, InsertableCommonAnnotation, InsertableCommonLabel,
            InsertableGroupLabel,
        },
    },
    MongoPlugin,
};
use async_trait::async_trait;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};

impl MongoPlugin {
    async fn push_alert_with_internal_error(
        &self,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let mut session = self
            .client
            .start_session(None)
            .await
            .map_err(InternalPushError::StartSession)?;

        session
            .start_transaction(None)
            .await
            .map_err(InternalPushError::TransactionBegin)?;

        let alert_group = InsertableAlertGroup {
            group_key: alertmanager_push.group_key.clone(),
            truncated_alerts: alertmanager_push.truncated_alerts,
            status: alertmanager_push.status.clone(),
            receiver: alertmanager_push.receiver.clone(),
            external_url: alertmanager_push.external_url.clone(),
        };

        tracing::trace!("Inserting alert group.");
        let alert_group_id = self
            .alert_group_collection()
            .insert_one_with_session(alert_group, None, &mut session)
            .await
            .map_err(|error| InternalPushError::GroupInsertion {
                group_key: alertmanager_push.group_key.clone(),
                error,
            })?
            .inserted_id
            .as_object_id()
            .ok_or(InternalPushError::GroupId {
                group_key: alertmanager_push.group_key.clone(),
            })?;

        let group_labels = alertmanager_push
            .group_labels
            .iter()
            .map(|(name, value)| InsertableGroupLabel {
                alert_group_id,
                name: name.clone(),
                value: value.clone(),
            })
            .collect::<Vec<_>>();

        tracing::trace!("Inserting group labels.");
        self.group_label_collection()
            .insert_many_with_session(group_labels, None, &mut session)
            .await
            .map_err(|error| InternalPushError::GroupLabelsInsertion {
                group_key: alertmanager_push.group_key.clone(),
                error,
            })?;

        let common_labels = alertmanager_push
            .common_labels
            .iter()
            .map(|(name, value)| InsertableCommonLabel {
                alert_group_id,
                name: name.clone(),
                value: value.clone(),
            })
            .collect::<Vec<_>>();

        tracing::trace!("Inserting common labels.");
        self.common_label_collection()
            .insert_many_with_session(common_labels, None, &mut session)
            .await
            .map_err(|error| InternalPushError::CommonLabelsInsertion {
                group_key: alertmanager_push.group_key.clone(),
                error,
            })?;

        let common_annotations = alertmanager_push
            .common_annotations
            .iter()
            .map(|(name, value)| InsertableCommonAnnotation {
                alert_group_id,
                name: name.clone(),
                value: value.clone(),
            })
            .collect::<Vec<_>>();

        tracing::trace!("Inserting common annotations.");
        self.common_annotation_collection()
            .insert_many_with_session(common_annotations, None, &mut session)
            .await
            .map_err(|error| InternalPushError::CommonAnnotationsInsertion {
                group_key: alertmanager_push.group_key.clone(),
                error,
            })?;

        tracing::trace!("Inserting alerts.");
        for alert in alertmanager_push.alerts.iter() {
            let insertable_alert = InsertableAlert {
                status: alert.status.clone(),
                starts_at: alert.starts_at,
                ends_at: alert.ends_at,
                generator_url: alert.generator_url.clone(),
                fingerprint: alert.fingerprint.clone(),
            };

            let alert_id = self
                .alert_collection()
                .insert_one_with_session(insertable_alert, None, &mut session)
                .await
                .map_err(|error| InternalPushError::AlertInsertion {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    error,
                })?
                .inserted_id
                .as_object_id()
                .ok_or(InternalPushError::AlertId {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                })?;

            let labels = alert
                .labels
                .iter()
                .map(|(name, value)| InsertableAlertLabel {
                    alert_id,
                    name: name.clone(),
                    value: value.clone(),
                })
                .collect::<Vec<_>>();

            tracing::trace!("Inserting alert labels.");
            self.alert_label_collection()
                .insert_many_with_session(labels, None, &mut session)
                .await
                .map_err(|error| InternalPushError::AlertLabelsInsertion {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    error,
                })?;

            let annotations = alert
                .annotations
                .iter()
                .map(|(name, value)| InsertableAlertAnnotation {
                    alert_id,
                    name: name.clone(),
                    value: value.clone(),
                })
                .collect::<Vec<_>>();

            tracing::trace!("Inserting alert annotations.");
            self.alert_annotation_collection()
                .insert_many_with_session(annotations, None, &mut session)
                .await
                .map_err(|error| InternalPushError::AlertAnnotationsInsertion {
                    group_key: alertmanager_push.group_key.clone(),
                    fingerprint: alert.fingerprint.clone(),
                    error,
                })?;
        }

        tracing::trace!("Committing transaction.");
        session
            .commit_transaction()
            .await
            .map_err(InternalPushError::CommitTransaction)?;

        Ok(())
    }
}

#[async_trait]
impl Push for MongoPlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        // TODO
        tracing::warn!("Not implemented yet.");
        let _ = self.config.take();

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
