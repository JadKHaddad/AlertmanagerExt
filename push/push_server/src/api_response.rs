use aide::{transform::TransformResponse, OperationIo};
use axum::{http::StatusCode, response::IntoResponse, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::traits::{HasResponseDocs, HasStatusCode};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, OperationIo)]
#[serde(rename_all = "camelCase")]
pub struct ApiOkResponse {}

impl HasStatusCode for ApiOkResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl IntoResponse for ApiOkResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Default error response
pub struct ApiErrorResponse {
    /// Error type
    pub error_type: ApiErrorResponseType,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "error")]
#[serde(rename_all = "camelCase")]
pub enum ApiErrorResponseType {
    /// Payload is invalid
    PayloadInvalid { payload_invalid: PayloadInvalid },
    /// Path is invalid
    PathInvalid { path_invalid: PathInvalid },
    /// Internal server error
    InternalServerError {
        internal_server_error: InternalServerError,
    },
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
    pub(crate) reason: String,
}

impl<E> From<E> for InternalServerError
where
    E: Into<anyhow::Error> + std::fmt::Display,
{
    fn from(error: E) -> Self {
        InternalServerError {
            reason: error.to_string(),
        }
    }
}

impl HasStatusCode for ApiErrorResponseType {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiErrorResponseType::PayloadInvalid { payload_invalid } => payload_invalid.status_code,
            ApiErrorResponseType::PathInvalid { path_invalid } => path_invalid.status_code,
            ApiErrorResponseType::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorResponseType::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl<E> From<E> for ApiErrorResponseType
where
    E: Into<anyhow::Error> + std::fmt::Display,
{
    fn from(error: E) -> Self {
        ApiErrorResponseType::InternalServerError {
            internal_server_error: InternalServerError::from(error),
        }
    }
}

impl IntoResponse for ApiErrorResponseType {
    fn into_response(self) -> axum::response::Response {
        (
            self.status_code(),
            Json(ApiErrorResponse { error_type: self }),
        )
            .into_response()
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.error_type.status_code(), Json(self)).into_response()
    }
}

impl HasResponseDocs for ApiErrorResponse {
    fn response_docs<R>(res: TransformResponse<R>) -> TransformResponse<R>
    where
        R: Serialize,
        ApiErrorResponse: Into<R>,
    {
        res.description("Error response")
            .example(ApiErrorResponse {
                error_type: ApiErrorResponseType::PayloadInvalid {
                    payload_invalid: PayloadInvalid {
                        status_code: StatusCode::BAD_REQUEST,
                        reason: "Invalid payload".to_string(),
                        expected_schema:
                            r#"{"type":"object","properties":{"apiOk":{"type":"boolean"}}}"#
                                .to_string(),
                    },
                },
            })
    }
}
