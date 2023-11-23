use crate::traits::{HasResponseDocs, HasStatusCode};
use aide::transform::TransformResponse;
use axum::{http::StatusCode, response::IntoResponse, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Default error response
pub struct ErrorResponse {
    /// Error type
    pub error_type: ErrorResponseType,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "error")]
#[serde(rename_all = "camelCase")]
pub enum ErrorResponseType {
    /// Payload is invalid
    PayloadInvalid(PayloadInvalid),
    /// Path is invalid
    PathInvalid(PathInvalid),
    /// Internal server error
    InternalServerError(InternalServerError),
    /// Not found
    NotFound,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PayloadInvalid {
    #[serde(skip)]
    pub(crate) status_code: StatusCode,
    pub(crate) reason: String,
    pub(crate) expected_schema: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PathInvalid {
    #[serde(skip)]
    pub(crate) status_code: StatusCode,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InternalServerError {
    reason: String,
}

impl<E> From<E> for InternalServerError
where
    E: Into<anyhow::Error> + std::fmt::Display,
{
    fn from(error: E) -> Self {
        tracing::error!(%error, "Internal server error");
        InternalServerError {
            reason: error.to_string(),
        }
    }
}

impl HasStatusCode for ErrorResponseType {
    fn status_code(&self) -> StatusCode {
        match self {
            ErrorResponseType::PayloadInvalid(payload_invalid) => payload_invalid.status_code,
            ErrorResponseType::PathInvalid(path_invalid) => path_invalid.status_code,
            ErrorResponseType::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponseType::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl<E> From<E> for ErrorResponse
where
    E: Into<anyhow::Error> + std::fmt::Display,
{
    fn from(error: E) -> Self {
        ErrorResponse {
            error_type: ErrorResponseType::InternalServerError(InternalServerError::from(error)),
        }
    }
}

impl From<ErrorResponseType> for ErrorResponse {
    fn from(error_type: ErrorResponseType) -> Self {
        Self { error_type }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.error_type.status_code(), Json(self)).into_response()
    }
}

impl HasResponseDocs for ErrorResponse {
    fn response_docs<R>(res: TransformResponse<R>) -> TransformResponse<R>
    where
        R: Serialize,
        ErrorResponse: Into<R>,
    {
        res.description("Error response").example(ErrorResponse {
            error_type: ErrorResponseType::PayloadInvalid(PayloadInvalid {
                status_code: StatusCode::BAD_REQUEST,
                reason: "Invalid payload".to_string(),
                expected_schema: r#"{"type":"object","properties":{"apiOk":{"type":"boolean"}}}"#
                    .to_string(),
            }),
        })
    }
}
