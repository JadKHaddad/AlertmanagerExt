use axum::http::StatusCode;
use file_plugin::FilePlugin;
use plugins_definitions::Plugin;
use postgres_plugin::PostgresPlugin;
use postgres_x_plugin::PostgresXPlugin;
use print_plugin::PrintPlugin;
use push_definitions::Push;
use sqlite_plugin::SqlitePlugin;

pub trait PushAndPlugin: Push + Plugin {}

impl PushAndPlugin for PostgresPlugin {}

impl PushAndPlugin for PostgresXPlugin {}

impl PushAndPlugin for SqlitePlugin {}

impl PushAndPlugin for FilePlugin {}

impl PushAndPlugin for PrintPlugin {}

pub trait HasStatusCode {
    fn status_code(&self) -> StatusCode;
}
