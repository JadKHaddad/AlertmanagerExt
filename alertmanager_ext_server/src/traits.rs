use axum::http::StatusCode;
use plugins_definitions::Plugin;
use postgres_plugin::PostgresPlugin;
use push_definitions::Push;
use sqlite_plugin::SqlitePlugin;

pub trait PushAndPlugin: Push + Plugin {}

impl PushAndPlugin for PostgresPlugin {}

impl PushAndPlugin for SqlitePlugin {}

pub trait HasStatusCode {
    fn status_code(&self) -> StatusCode;
}
