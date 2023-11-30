use crate::{error_response::ErrorResponse, state::ApiState};
use axum::extract::State;

/// Prometheus metrics endpoint
#[utoipa::path(get, path = "/metrics", tag = "metrics", responses(
    (status = 200, description = "Metrics."),
    (status = 500, description = "Iternal server error.", body = [ErrorResponse])
))]
pub async fn metrics(State(state): State<ApiState>) -> Result<String, ErrorResponse> {
    let metrics = state.prometheus_client.metrics()?;
    Ok(metrics)
}
