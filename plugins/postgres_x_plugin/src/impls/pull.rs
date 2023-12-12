use crate::{
    database::models::{
        alert_status::AlertStatusModel, alerts::DatabaseAlert, annotations::Annotation,
        labels::Label,
    },
    PostgresXPlugin,
};
use async_trait::async_trait;
use models::StandAloneAlert;
use plugins_definitions::Plugin;
use pull_definitions::{Pull, PullAlertsFilter, PullError};

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
                error: error.into(),
            })?;

        tracing::trace!("Successfully pulled.");

        Ok(database_alerts
            .into_iter()
            .map(|alert| alert.into())
            .collect())
    }
}
