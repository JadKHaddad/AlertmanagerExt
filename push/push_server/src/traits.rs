use std::sync::Arc;

use axum::http::StatusCode;
use plugins_definitions::Plugin;
use postgres_plugin::PostgresPlugin;
use push_definitions::Push;

pub trait PushAndPlugin: Push + Plugin {}

impl PushAndPlugin for PostgresPlugin {}

pub trait HasStatusCode {
    fn status_code(&self) -> StatusCode;
}

/// Helper trait
///
/// Allows us to use functions with [`Vec`]<[`Arc`]<dyn [`PushAndPlugin`]>> and [`Vec`]<&[`Arc`]<dyn [`PushAndPlugin`]>> without duplicating code and unnecessary ```iter().collect()``` calls
pub trait HasPushAndPluginArcRef {
    fn arc_ref(&self) -> &Arc<dyn PushAndPlugin>;
}

impl HasPushAndPluginArcRef for Arc<dyn PushAndPlugin> {
    fn arc_ref(&self) -> &Arc<dyn PushAndPlugin> {
        self
    }
}

impl HasPushAndPluginArcRef for &Arc<dyn PushAndPlugin> {
    fn arc_ref(&self) -> &Arc<dyn PushAndPlugin> {
        self
    }
}
