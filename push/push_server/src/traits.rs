use aide::transform::{TransformOperation, TransformResponse};
use axum::http::StatusCode;
use plugins_definitions::Plugin;
use postgres_plugin::PostgresPlugin;
use push_definitions::Push;
use serde::Serialize;

pub trait PushAndPlugin: Push + Plugin {}

impl PushAndPlugin for PostgresPlugin {}

pub trait HasStatusCode {
    fn status_code(&self) -> StatusCode;
}

pub trait HasOperationDocs {
    fn operation_docs(op: TransformOperation) -> TransformOperation;
}

pub trait HasResponseDocs {
    fn response_docs<R>(res: TransformResponse<R>) -> TransformResponse<R>
    where
        R: Serialize,
        Self: Into<R>;
}
