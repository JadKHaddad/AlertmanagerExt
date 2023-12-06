use crate::error_response::ErrorResponse;
use axum::{http::Request, middleware::Next, response::IntoResponse};

/// Middlware to trace the response body
#[tracing::instrument(skip(req, next))]
pub async fn trace_response_body<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = hyper::body::to_bytes(body).await?;

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::trace!(body);
    }

    let res = hyper::Response::from_parts(parts, hyper::Body::from(bytes));

    Ok(res)
}
