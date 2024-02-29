use crate::traits::HasStatusCode;
use axum::{http::StatusCode, response::IntoResponse, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Default error response
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ErrorResponse {
    /// Error type
    pub error_type: ErrorResponseType,
}

impl ErrorResponse {
    pub fn not_found() -> Self {
        Self {
            error_type: ErrorResponseType::NotFound,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(tag = "type", content = "error")]
#[non_exhaustive]
pub enum ErrorResponseType {
    /// Payload is invalid
    PayloadInvalid(PayloadInvalid),
    /// Query is invalid
    QueryInvalid(QueryInvalid),
    /// Filter is invalid
    PluginFilterInvalid(PluginFilterInvalid),
    /// Path is invalid
    PathInvalid(PathInvalid),
    /// Internal server error
    InternalServerError(InternalServerError),
    /// Not found
    NotFound,
    /// Method not allowed
    MethodNotAllowed,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct QueryInvalid {
    #[serde(skip)]
    pub(crate) status_code: StatusCode,
    pub(crate) reason: String,
    pub(crate) expected_query_schema: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct PluginFilterInvalid {
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct PayloadInvalid {
    #[serde(skip)]
    pub(crate) status_code: StatusCode,
    pub(crate) reason: String,
    pub(crate) expected_payload_schema: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct PathInvalid {
    #[serde(skip)]
    pub(crate) status_code: StatusCode,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[non_exhaustive]
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
            ErrorResponseType::QueryInvalid(query_invalid) => query_invalid.status_code,
            ErrorResponseType::PluginFilterInvalid(..) => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorResponseType::PathInvalid(path_invalid) => path_invalid.status_code,
            ErrorResponseType::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorResponseType::NotFound => StatusCode::NOT_FOUND,
            ErrorResponseType::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
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
