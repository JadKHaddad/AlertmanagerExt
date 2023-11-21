use crate::extractors::ApiJson;
use crate::traits::{HasOperationDocs, HasStatusCode};
use crate::{extractors::ApiPath, state::ApiV1State};
use aide::transform::TransformOperation;
use aide::OperationIo;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, OperationIo)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {}

impl HasStatusCode for HealthResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl HasOperationDocs for HealthResponse {
    fn operation_docs(op: TransformOperation) -> TransformOperation {
        op.summary("Health check")
            .description("Health check")
            .response_with::<200, ApiJson<Self>, _>(|res| {
                res.description("Healthy").example(HealthResponse {})
            })
    }
}

impl IntoResponse for HealthResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}

pub async fn health() -> HealthResponse {
    HealthResponse {}
}

pub async fn health_named(
    State(state): State<ApiV1State>,
    ApiPath(plugin_name): ApiPath<String>,
) -> &'static str {
    todo!()
}
