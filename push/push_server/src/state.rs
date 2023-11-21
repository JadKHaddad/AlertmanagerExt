use std::{ops::Deref, sync::Arc};

use crate::traits::PushAndPlugin;

#[derive(Clone)]
pub struct ApiV1State {
    inner: Arc<ApiV1StateInner>,
}

impl ApiV1State {
    pub fn new(plugins: Vec<Box<dyn PushAndPlugin>>) -> Self {
        Self {
            inner: Arc::new(ApiV1StateInner { plugins }),
        }
    }
}

pub struct ApiV1StateInner {
    pub plugins: Vec<Box<dyn PushAndPlugin>>,
}

impl Deref for ApiV1State {
    type Target = ApiV1StateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
