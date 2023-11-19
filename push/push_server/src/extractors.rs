use aide::operation::OperationIo;
use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::IntoResponse,
};
use axum_jsonschema::JsonSchemaRejection;
use axum_macros::FromRequest;
use serde::Serialize;
use serde_json::json;

use crate::errors::ApiError;

#[derive(FromRequest, OperationIo)]
#[from_request(via(axum_jsonschema::Json), rejection(ApiError))]
#[aide(
    input_with = "axum_jsonschema::Json<T>",
    output_with = "axum_jsonschema::Json<T>",
    json_schema
)]
pub struct AideJson<T>(pub T);

impl<T> IntoResponse for AideJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

impl From<JsonSchemaRejection> for ApiError {
    fn from(rejection: JsonSchemaRejection) -> Self {
        match rejection {
            JsonSchemaRejection::Json(j) => Self::new(&j.to_string()),
            JsonSchemaRejection::Serde(_) => Self::new("invalid request"),
            JsonSchemaRejection::Schema(s) => {
                Self::new("invalid request").with_details(json!({ "schemaValidation": s }))
            }
        }
    }
}

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(OperationIo)]
pub struct AidePath<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for AidePath<T>
where
    T: DeserializeOwned + Debug + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let path = Path::<T>::from_request_parts(parts, _state).await;
        match path {
            Ok(path) => Ok(AidePath(path.0)),
            Err(path_rejection) => {
                let reason = path_rejection.body_text();
                let status_code = path_rejection.status();

                return Err(ApiError::new("invalid request")
                    .with_status(status_code)
                    .with_details(json!({ "path": reason })));
            }
        }
    }
}
