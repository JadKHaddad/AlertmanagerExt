use crate::error_response::{ErrorResponse, ErrorResponseType, QueryInvalid};
use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Query as AxumQuery},
    http::request::Parts,
};
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

/// A Wrapper around [`axum::extract::Query`] that rejects with an [`ErrorResponse`]
pub struct ApiQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    #[tracing::instrument(name = "query_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = AxumQuery::<T>::from_request_parts(parts, _state).await;
        match query {
            Ok(query) => {
                tracing::trace!(query=?query.0, "Extracted");
                Ok(ApiQuery(query.0))
            }
            Err(query_rejection) => {
                tracing::error!(rejection=?query_rejection, "Rejection");

                let reason = query_rejection.body_text();
                let status_code = query_rejection.status();

                let expected_query_schema = serde_json::to_string(&schema_for!(T))?;

                let query_invalid = QueryInvalid {
                    status_code,
                    expected_query_schema,
                    reason,
                };

                let error = ErrorResponseType::QueryInvalid(query_invalid);
                Err(error.into())
            }
        }
    }
}
