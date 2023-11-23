use crate::traits::PushAndPlugin;
use std::{ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    pub fn new(plugins: Vec<Arc<dyn PushAndPlugin>>) -> Self {
        Self {
            inner: Arc::new(ApiStateInner { plugins }),
        }
    }
}

pub struct ApiStateInner {
    pub plugins: Vec<Arc<dyn PushAndPlugin>>,
}

impl Deref for ApiState {
    type Target = ApiStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
