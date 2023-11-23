use std::{ops::Deref, sync::Arc};

use crate::traits::PushAndPlugin;

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiV1StateInner>,
}

impl ApiState {
    pub fn new(plugins: Vec<Arc<dyn PushAndPlugin>>) -> Self {
        Self {
            inner: Arc::new(ApiV1StateInner { plugins }),
        }
    }
}

pub struct ApiV1StateInner {
    pub plugins: Vec<Arc<dyn PushAndPlugin>>,
}

impl Deref for ApiState {
    type Target = ApiV1StateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
