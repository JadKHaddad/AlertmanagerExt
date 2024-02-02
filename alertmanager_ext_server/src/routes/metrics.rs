use crate::{error_response::ErrorResponse, state::ApiState};
use axum::extract::State;

/// Prometheus metrics endpoint
#[utoipa::path(get, path = "/metrics", tag = "metrics", responses(
    (status = 200, description = "Prometheus metrics.", body = String, example = 
        json!("
# HELP push_success_total Total number of successful pushes.
# TYPE push_success_total counter
# HELP push_failed_total Total number of failed pushes.
# TYPE push_failed_total counter
# EOF
        ")),
    (status = 500, description = "Iternal server error.")
))]
pub async fn metrics(State(state): State<ApiState>) -> Result<String, ErrorResponse> {
    let metrics = state.prometheus_client.metrics()?;
    Ok(metrics)
}
