use crate::{
    database::{
        self,
        models::{
            alert_status::AlertStatusModel,
            alerts::{InsertableAlert, InsertableAlertAnnotation, InsertableAlertLabel},
            annotations::{InsertableAnnotation, InsertableCommonAnnotation},
            groups::{
                InsertableGroup, InsertableGroupCommonAnnotation, InsertableGroupCommonLabel,
                InsertableGroupLabel,
            },
            labels::{InsertableCommonLabel, InsertableLabel},
        },
    },
    error::{InternalInitializeError, InternalPushError, LablelInsertionError},
    PostgresPlugin, MIGRATIONS,
};
use async_trait::async_trait;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use models::{Alert as AlertmanagerPushAlert, AlertmanagerPush};
use plugins_definitions::Plugin;
use push_definitions::{InitializeError, Push, PushError};
use scoped_futures::ScopedFutureExt;
use tokio::task::JoinHandle;

impl PostgresPlugin {
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
        tracing::trace!(
            group_id,
            label_id,
            name = %label.name,
            value = %label.value,
            "Assigning group label.");

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
    async fn get_or_insert_label(
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

            let label_id = Self::get_or_insert_label(conn, &label)
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
        tracing::trace!(
            group_id,
            common_label_id,
            name = %common_label.name,
            value = %common_label.value,
            "Assigning group common label.");

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
        tracing::trace!(
            group_id,
            common_annotation_id,
            name = %common_annotation.name,
            value = %common_annotation.value,
            "Assigning group common annotation.");

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
        tracing::trace!(
            alert_id,
            label_id,
            name = %label.name,
            value = %label.value,
            "Assigning alert label.");

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

            let label_id = Self::get_or_insert_label(conn, &label)
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
        tracing::trace!(
            alert_id,
            annotation_id,
            name = %annotation.name,
            value = %annotation.value,
            "Assigning alert annotation.");

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

    async fn push_alert_with_internal_error(
        &self,
        alertmanager_push: &AlertmanagerPush,
    ) -> Result<(), InternalPushError> {
        let mut conn = self.pool.get().await.map_err(InternalPushError::Acquire)?;

        conn.transaction::<(), InternalPushError, _>(|conn| {
            async move {
                tracing::trace!("Beginning transaction.");

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

        Ok(())
    }

    async fn initialize_with_internal_error(&mut self) -> Result<(), InternalInitializeError> {
        // Always be nice and give memory back to the OS. ;)
        let config = self
            .config
            .take()
            .ok_or_else(|| InternalInitializeError::AlreadyInitialized)?;

        let connection_string = config.connection_string;
        let handle: JoinHandle<Result<(), InternalInitializeError>> =
            tokio::task::spawn_blocking(move || {
                let mut conn = PgConnection::establish(&connection_string)?;

                conn.run_pending_migrations(MIGRATIONS)
                    .map_err(InternalInitializeError::Migrations)?;

                Ok(())
            });

        handle.await??;

        Ok(())
    }
}

#[async_trait]
impl Push for PostgresPlugin {
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
