use async_trait::async_trait;
use models::AlermanagerPush;
use std::error::Error as StdError;

#[async_trait]
pub trait Push: Send + Sync + 'static {
    async fn push_alert(
        &self,
        alertmanager_push: &AlermanagerPush,
    ) -> Result<(), Box<dyn StdError>>;
}
