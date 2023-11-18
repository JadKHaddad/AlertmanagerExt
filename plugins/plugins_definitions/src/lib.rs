use async_trait::async_trait;
use std::error::Error as StdError;

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    async fn initialize(&self) -> Result<(), Box<dyn StdError>>;
}
