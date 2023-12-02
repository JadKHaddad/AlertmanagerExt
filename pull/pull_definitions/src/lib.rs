use async_trait::async_trait;
use models::StandAloneAlert;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Pull failed: {reason}")]
pub struct PullError {
    pub reason: String,
}

pub struct PullAlertsFilter {}

#[async_trait]
pub trait Pull: Send + Sync + 'static {
    async fn pull_alerts(
        &self,
        filter: &PullAlertsFilter,
    ) -> Result<Vec<StandAloneAlert>, PullError>;
}
