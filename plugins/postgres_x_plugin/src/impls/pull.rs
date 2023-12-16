use crate::{
    database::models::{
        alert_status::AlertStatusModel, alerts::DatabaseAlert, annotations::Annotation,
        labels::Label,
    },
    error::InternalPullError,
    PostgresXPlugin,
};
use async_trait::async_trait;
use models::StandAloneAlert;
use plugins_definitions::Plugin;
use pull_definitions::{Pull, PullAlertsFilter, PullError};

impl PostgresXPlugin {
    async fn pull_alerts_with_internal_error(
        &self,
        filter: &PullAlertsFilter,
    ) -> Result<Vec<StandAloneAlert>, InternalPullError> {
        let database_alerts = sqlx::query_file_as!(DatabaseAlert, "queries/pull_alerts.sql",)
            .fetch_all(&self.pool)
            .await?;

        Ok(database_alerts
            .into_iter()
            .map(|alert| alert.into())
            .collect())
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

        let alerts = self.pull_alerts_with_internal_error(filter).await?;

        tracing::trace!("Successfully pulled.");
        Ok(alerts)
    }
}
