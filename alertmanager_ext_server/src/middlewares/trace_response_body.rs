use crate::error_response::ErrorResponse;
use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;

/// Middlware to trace the response body
pub async fn trace_response_body(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, ErrorResponse> {
    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = body.collect().await?.to_bytes();

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::info!("Response: {}", body);
    }

    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}
