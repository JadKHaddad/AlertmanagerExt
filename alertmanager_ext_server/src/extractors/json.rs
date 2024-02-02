use crate::error_response::{ErrorResponse, ErrorResponseType, PayloadInvalid};
use async_trait::async_trait;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::{extract::FromRequest, Json as AxumJson};
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// A Wrapper around [`axum::extract::Json`] that rejects with an [`ErrorResponse`]
pub struct ApiJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned + JsonSchema + Debug,

    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    #[tracing::instrument(name = "json_extractor", skip_all)]
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let json = AxumJson::<T>::from_request(req, state).await;

        match json {
            Ok(json) => {
                tracing::trace!(json=?json.0, "Extracted");
                Ok(ApiJson(json.0))
            }
            Err(json_rejection) => {
                tracing::error!(rejection=?json_rejection, "Rejection");

                let reason = json_rejection.body_text();
                let status_code = json_rejection.status();

                let expected_payload_schema = serde_json::to_string(&schema_for!(T))?;

                let payload_invalid = PayloadInvalid {
                    status_code,
                    expected_payload_schema,
                    reason,
                };

                let error = ErrorResponseType::PayloadInvalid(payload_invalid);
                Err(error.into())
            }
        }
    }
}

impl<T> IntoResponse for ApiJson<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        AxumJson(self.0).into_response()
    }
}

impl<T> From<T> for ApiJson<T>
where
    T: Serialize,
{
    fn from(t: T) -> Self {
        ApiJson(t)
    }
}
