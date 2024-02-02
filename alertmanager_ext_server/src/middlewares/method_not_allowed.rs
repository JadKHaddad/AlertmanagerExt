use crate::error_response::{ErrorResponse, ErrorResponseType};
use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};

/// Middleware to map axum's `MethodNotAllowed` rejection to our `ErrorResponse`
pub async fn method_not_allowed(req: Request, next: Next) -> impl IntoResponse {
    let resp = next.run(req).await;
    let status = resp.status();
    match status {
        StatusCode::METHOD_NOT_ALLOWED => Err(ErrorResponse {
            error_type: ErrorResponseType::MethodNotAllowed,
        }),
        _ => Ok(resp),
    }
}
