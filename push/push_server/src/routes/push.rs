use crate::{
    extractors::{ApiJson, ApiPath},
    state::ApiState,
    traits::{HasPushAndPluginArcRef, HasStatusCode, PushAndPlugin},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use models::AlermanagerPush;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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
    ///
    /// If pushed to a group, this means that the group is empty
    /// If pushed to all plugins, this means that there are no plugins
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "content")]
/// Push status for a plugin
pub enum PluginPushStatus {
    /// Push was successful
    Ok,
    /// Plugin was not found
    NotFound,
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
            PluginPushStatus::NotFound => StatusCode::NOT_FOUND,
            PluginPushStatus::Failed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Response for a plugin push
pub struct PluginPushResponse {
    /// Status of the push for the plugin
    pub status: PluginPushStatus,
    /// Name of the plugin
    pub plugin_name: String,
}

impl IntoResponse for PluginPushResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status.status_code(), ApiJson(self)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
///
/// Avoids duplicate code in ```push```, ```push_grouped``` (push_async) and ```push_named```
async fn match_plugin_push(
    plugin: &Arc<dyn PushAndPlugin>,
    alertmanager_push: &AlermanagerPush,
) -> PluginPushResponse {
    match plugin.push_alert(alertmanager_push).await {
        Ok(_) => PluginPushResponse {
            status: PluginPushStatus::Ok,
            plugin_name: plugin.name().to_string(),
        },
        Err(error) => {
            tracing::error!(name=plugin.name(), %error, "Failed to push alerts to plugin.");
            PluginPushResponse {
                status: PluginPushStatus::Failed {
                    message: error.to_string(),
                },
                plugin_name: plugin.name().to_string(),
            }
        }
    }
}

/// Join handle for a plugin push response
struct PluginPushResponseJoinHandle {
    /// Join handle
    join_handle: JoinHandle<PluginPushResponse>,
    /// Name of the plugin, in case the join handle panics or is cancelled, we still want to know which plugin it was
    plugin_name: String,
}

/// Helper function
///
/// Pushes alerts asynchronously
async fn push_async<A: HasPushAndPluginArcRef>(
    affected_plugins: &Vec<A>,
    alertmanager_push: &AlermanagerPush,
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

    for plugin in affected_plugins {
        let plugin_c = Arc::clone(plugin.arc_ref());
        let alertmanager_push_c = alertmanager_push.clone();
        let handle =
            tokio::spawn(async move { match_plugin_push(&plugin_c, &alertmanager_push_c).await });
        plugin_response_handles.push(PluginPushResponseJoinHandle {
            join_handle: handle,
            plugin_name: plugin.arc_ref().name().to_string(),
        });
    }

    for plugin_response_handle in plugin_response_handles {
        let plugin_push_response = match plugin_response_handle.join_handle.await {
            Ok(plugin_push_response) => plugin_push_response,
            Err(error) => {
                if error.is_cancelled() {
                    tracing::error!(name=plugin_response_handle.plugin_name, %error, "Plugin push handler was cancelled.");
                    PluginPushResponse {
                        status: PluginPushStatus::Failed {
                            message: error.to_string(),
                        },
                        plugin_name: plugin_response_handle.plugin_name,
                    }
                } else {
                    tracing::error!(name=plugin_response_handle.plugin_name, %error, "Plugin push handler panicked.");
                    PluginPushResponse {
                        status: PluginPushStatus::Failed {
                            message: error.to_string(),
                        },
                        plugin_name: plugin_response_handle.plugin_name,
                    }
                }
            }
        };
        if let PluginPushStatus::Ok = plugin_push_response.status {
            ok_push_count += 1;
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
#[tracing::instrument(name = "push", skip_all, fields(group_key = alertmanager_push.group_key))]
pub async fn push(
    State(state): State<ApiState>,
    ApiJson(alertmanager_push): ApiJson<AlermanagerPush>,
) -> PushResponse {
    tracing::trace!("Pushing alerts to plugins.");

    let affected_plugins = &state.plugins;

    push_async(affected_plugins, &alertmanager_push).await
}

/// Push alerts to plugins in a group asynchronously
#[tracing::instrument(name = "push_grouped", skip_all, fields(group_key = alertmanager_push.group_key))]
pub async fn push_grouped(
    State(state): State<ApiState>,
    ApiPath(plugin_group): ApiPath<String>,
    ApiJson(alertmanager_push): ApiJson<AlermanagerPush>,
) -> PushResponse {
    tracing::trace!("Pushing alerts to plugins.");

    let affected_plugins: Vec<&Arc<dyn PushAndPlugin>> = state
        .plugins
        .iter()
        .filter(|p| p.group() == plugin_group)
        .collect();

    push_async(&affected_plugins, &alertmanager_push).await
}

/// Push alerts to a specific plugin
#[tracing::instrument(name = "push_named",  skip_all, fields(group_key = alertmanager_push.group_key))]
pub async fn push_named(
    State(state): State<ApiState>,
    ApiPath(plugin_name): ApiPath<String>,
    ApiJson(alertmanager_push): ApiJson<AlermanagerPush>,
) -> PluginPushResponse {
    tracing::trace!(name = plugin_name, "Pushing alerts to plugin.");
    let plugin = state.plugins.iter().find(|p| p.name() == plugin_name);
    match plugin {
        Some(plugin) => match_plugin_push(plugin, &alertmanager_push).await,
        None => PluginPushResponse {
            status: PluginPushStatus::NotFound,
            plugin_name,
        },
    }
}
