use crate::error_response::{ErrorResponse, ErrorResponseType, PathInvalid, PayloadInvalid};
use async_trait::async_trait;
use axum::response::IntoResponse;
use axum::{body::HttpBody, extract::FromRequest, http::Request, BoxError, Json as AxumJson};
use axum::{
    extract::{FromRequestParts, Path as AxumPath},
    http::request::Parts,
};
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

pub struct ApiPath<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ApiPath<T>
where
    T: DeserializeOwned + Debug + Send,
    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    #[tracing::instrument(name = "path_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let path = AxumPath::<T>::from_request_parts(parts, _state).await;
        match path {
            Ok(path) => {
                tracing::trace!(path=?path.0, "Path extracted");
                Ok(ApiPath(path.0))
            }
            Err(path_rejection) => {
                tracing::trace!("Path rejection");

                let reason = path_rejection.body_text();
                let status_code = path_rejection.status();

                let path_invalid = PathInvalid {
                    status_code,
                    reason,
                };

                let error = ErrorResponseType::PathInvalid(path_invalid);
                Err(error.into())
            }
        }
    }
}

pub struct ApiJson<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ApiJson<T>
where
    T: DeserializeOwned + JsonSchema + Debug,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    #[tracing::instrument(name = "json_extractor", skip_all)]
    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let json = AxumJson::<T>::from_request(req, state).await;

        match json {
            Ok(json) => {
                tracing::trace!(json=?json.0, "Json extracted");
                Ok(ApiJson(json.0))
            }
            Err(json_rejection) => {
                tracing::trace!("Json rejection");

                let reason = json_rejection.body_text();
                let status_code = json_rejection.status();

                let expected_schema = serde_json::to_string(&schema_for!(T))?;

                let payload_invalid = PayloadInvalid {
                    status_code,
                    expected_schema,
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
