use crate::{prometheus_client::PromtheusClient, traits::PushAndPlugin};
use std::{ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    pub fn new(plugins: Vec<Arc<dyn PushAndPlugin>>) -> Self {
        Self {
            inner: Arc::new(ApiStateInner {
                plugins,
                prometheus_client: PromtheusClient::default(),
            }),
        }
    }
}

pub struct ApiStateInner {
    pub plugins: Vec<Arc<dyn PushAndPlugin>>,
    pub prometheus_client: PromtheusClient,
}

impl Deref for ApiState {
    type Target = ApiStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
