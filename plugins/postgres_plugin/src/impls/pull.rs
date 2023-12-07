use crate::{
    database::{
        self,
        models::{
            alerts::{Alert, AlertAnnotation, AlertLabel, DatabaseAlert},
            annotations::Annotation,
            labels::Label,
        },
    },
    error::InternalPullError,
    PostgresPlugin,
};
use async_trait::async_trait;
use diesel::{BelongingToDsl, GroupedBy, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use models::StandAloneAlert;
use plugins_definitions::Plugin;
use pull_definitions::{Pull, PullAlertsFilter, PullError};

impl PostgresPlugin {
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
