use async_trait::async_trait;
use models::AlermanagerPush;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
#[error("Push failed: {reason}")]
pub struct PushError {
    pub reason: String,
}

#[async_trait]
pub trait Push: Send + Sync + 'static {
    async fn push_alert(&self, alertmanager_push: &AlermanagerPush) -> Result<(), PushError>;
}
