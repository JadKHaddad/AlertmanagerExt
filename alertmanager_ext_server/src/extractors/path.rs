use crate::error_response::{ErrorResponse, ErrorResponseType, PathInvalid};
use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Path as AxumPath},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

/// A Wrapper around [`axum::extract::Path`] that rejects with an [`ErrorResponse`]
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
                tracing::trace!(path=?path.0, "Extracted");
                Ok(ApiPath(path.0))
            }
            Err(path_rejection) => {
                tracing::error!(rejection=?path_rejection, "Rejection");

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
