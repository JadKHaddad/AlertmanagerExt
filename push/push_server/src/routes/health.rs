use crate::traits::HasStatusCode;
use crate::{extractors::ApiPath, state::ApiState};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {}

impl HasStatusCode for HealthResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
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
    State(state): State<ApiState>,
    ApiPath(plugin_name): ApiPath<String>,
) -> &'static str {
    todo!()
}
