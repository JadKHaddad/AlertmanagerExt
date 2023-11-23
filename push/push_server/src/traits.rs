use axum::http::StatusCode;
use plugins_definitions::Plugin;
use postgres_plugin::PostgresPlugin;
use push_definitions::Push;

pub trait PushAndPlugin: Push + Plugin {}

impl PushAndPlugin for PostgresPlugin {}

pub trait HasStatusCode {
    fn status_code(&self) -> StatusCode;
}
