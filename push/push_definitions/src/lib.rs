use async_trait::async_trait;
use openapi::models::PostableAlert;
use std::error::Error as StdError;

#[async_trait]
pub trait Push: Send + Sync + 'static {
    async fn push(&self, postable_alerts: &[PostableAlert]) -> Result<(), Box<dyn StdError>>;
}
