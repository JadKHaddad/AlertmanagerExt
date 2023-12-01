use crate::{
    extractors::{ApiJson, ApiQuery},
    prometheus_client::PushLabel,
    state::ApiState,
    traits::{HasStatusCode, PushAndPlugin}, routes::utils,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use models::AlertmanagerPush;
use schemars::JsonSchema;
use serde::Serialize;
use std::sync::Arc;
use tokio::task::JoinHandle;
use utoipa::ToSchema;

use super::models::{PluginFilterQuery, PluginResponseMeta};

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Push status
pub enum PushStatus {
    /// Push was successful
    Ok,
    /// Some alerts were pushed successfully
    Partial,
    /// Push failed
    Failed,
    /// No plugins were found
    NoPlugins,
}

impl HasStatusCode for PushStatus {
    fn status_code(&self) -> StatusCode {
        match self {
            PushStatus::Ok => StatusCode::ACCEPTED,
            PushStatus::Partial => StatusCode::MULTI_STATUS,
            PushStatus::Failed => StatusCode::INTERNAL_SERVER_ERROR,
            PushStatus::NoPlugins => StatusCode::NOT_FOUND,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "content")]
/// Push status for a plugin
pub enum PluginPushStatus {
    /// Push was successful
    Ok,
    /// Push failed
    Failed {
        /// Error message
        message: String,
    },
}

impl HasStatusCode for PluginPushStatus {
    fn status_code(&self) -> StatusCode {
        match self {
            PluginPushStatus::Ok => StatusCode::ACCEPTED,
            PluginPushStatus::Failed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Response for a plugin push
pub struct PluginPushResponse {
    /// Status of the push for the plugin
    pub status: PluginPushStatus,
    /// Meta information about the plugin
    pub plugin_meta: PluginResponseMeta,
}

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Response for a push
pub struct PushResponse {
    /// Status of the push
    pub status: PushStatus,
    /// Responses for each plugin
    pub plugin_push_responses: Vec<PluginPushResponse>,
}

impl IntoResponse for PushResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status.status_code(), ApiJson(self)).into_response()
    }
}

/// Helper function
async fn match_plugin_push(
    plugin: &Arc<dyn PushAndPlugin>,
    alertmanager_push: &AlertmanagerPush,
) -> PluginPushResponse {
    match plugin.push_alert(alertmanager_push).await {
        Ok(_) => PluginPushResponse {
            status: PluginPushStatus::Ok,
            plugin_meta: plugin.meta().into(),
        },
        Err(error) => {
            tracing::error!(name=plugin.name(), %error, "Failed to push alerts to plugin.");
            PluginPushResponse {
                status: PluginPushStatus::Failed {
                    message: error.to_string(),
                },
                plugin_meta: plugin.meta().into(),
            }
        }
    }
}

/// Helper struct
///
/// Join handle for a plugin push response.
struct PluginPushResponseJoinHandle {
    /// Join handle
    join_handle: JoinHandle<PluginPushResponse>,
    /// In case the join handle panics or is cancelled, we still want to know which plugin it was
    plugin: Arc<dyn PushAndPlugin>,
}

/// Helper function
///
/// Pushes alerts asynchronously.
async fn push_async(
    state: &ApiState,
    affected_plugins: Vec<&Arc<dyn PushAndPlugin>>,
    alertmanager_push: &AlertmanagerPush,
) -> PushResponse {
    if affected_plugins.is_empty() {
        return PushResponse {
            status: PushStatus::NoPlugins,
            plugin_push_responses: vec![],
        };
    }

    let mut plugin_push_responses = vec![];
    let mut plugin_response_handles = vec![];
    let mut ok_push_count: usize = 0;

    for plugin in affected_plugins.iter() {
        let plugin_c = Arc::clone(plugin);
        let plugin_cc = Arc::clone(plugin);
        let alertmanager_push_c = alertmanager_push.clone();
        let handle =
            tokio::spawn(async move { match_plugin_push(&plugin_c, &alertmanager_push_c).await });
        plugin_response_handles.push(PluginPushResponseJoinHandle {
            join_handle: handle,
            plugin: plugin_cc,
        });
    }

    for plugin_response_handle in plugin_response_handles {
        let plugin_push_response = match plugin_response_handle.join_handle.await {
            Ok(plugin_push_response) => plugin_push_response,
            Err(error) => {
                if error.is_cancelled() {
                    tracing::error!(name=plugin_response_handle.plugin.name(), %error, "Plugin push handler was cancelled.");
                } else {
                    tracing::error!(name=plugin_response_handle.plugin.name(), %error, "Plugin push handler panicked.");
                }
                PluginPushResponse {
                    status: PluginPushStatus::Failed {
                        message: error.to_string(),
                    },
                    plugin_meta: plugin_response_handle.plugin.meta().clone().into(),
                }
            }
        };

        let push_label = PushLabel::from(plugin_response_handle.plugin.meta());

        match plugin_push_response.status {
            PluginPushStatus::Ok => {
                ok_push_count += 1;
                state.prometheus_client.add_success_push(&push_label);
            }
            _ => {
                state.prometheus_client.add_failed_push(&push_label);
            }
        }
        plugin_push_responses.push(plugin_push_response);
    }

    let status = match ok_push_count {
        0 => PushStatus::Failed,
        n if n == affected_plugins.len() => PushStatus::Ok,
        _ => PushStatus::Partial,
    };

    PushResponse {
        status,
        plugin_push_responses,
    }
}

/// Push alerts to all plugins asynchronously
#[utoipa::path(
    post, 
    path = "/push", 
    tag = "push",
    params(
        PluginFilterQuery
    ),
    request_body = AlermanagerPush, 
    responses(
        (status = 200, description = "Push was successful.", body = PushResponse),
        (status = 207, description = "Some pushes were successful.", body = PushResponse),
        (status = 500, description = "Push failed.", body = PushResponse),
        (status = 404, description = "No plugins were found.", body = PushResponse)
    )
)]
#[tracing::instrument(name = "push", skip_all, fields(group_key = alertmanager_push.group_key))]
pub async fn push(
    State(state): State<ApiState>,
    ApiQuery(filter_query): ApiQuery<PluginFilterQuery>,
    ApiJson(alertmanager_push): ApiJson<AlertmanagerPush>,
) -> PushResponse {
    tracing::trace!("Pushing alerts to plugins.");

    let affected_plugins = utils::filter_plugins(&state.plugins, &filter_query);

    push_async(&state, affected_plugins, &alertmanager_push).await
}
