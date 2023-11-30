use crate::{
    extractors::{ApiJson, ApiPath},
    prometheus_client::PushLabel,
    state::ApiState,
    traits::{HasPushAndPluginArcRef, HasStatusCode, PushAndPlugin},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use models::AlermanagerPush;
use plugins_definitions::PluginMeta;
use schemars::JsonSchema;
use serde::Serialize;
use std::sync::Arc;
use tokio::task::JoinHandle;
use utoipa::ToSchema;

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

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, ToSchema)]
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

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Response for a plugin push
pub struct PluginPushResponse {
    /// Status of the push for the plugin
    pub status: PluginPushStatus,
    /// Name of the plugin
    pub plugin_meta: PluginPushResponseMeta,
}

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Meta information about a plugin
pub struct PluginPushResponseMeta {
    /// Name of the plugin
    pub plugin_name: String,
    /// Type of the plugin, if found
    pub plugin_type: Option<&'static str>,
    /// Group of the plugin, if found
    pub plugin_group: Option<String>,
}

impl PluginPushResponseMeta {
    /// Helper function
    ///
    /// Creates a ```PluginPushResponseMeta``` for a plugin that was not found
    fn not_found(plugin_name: String) -> Self {
        Self {
            plugin_name,
            plugin_type: None,
            plugin_group: None,
        }
    }
}

impl From<PluginMeta> for PluginPushResponseMeta {
    fn from(plugin_meta: PluginMeta) -> Self {
        Self {
            plugin_name: plugin_meta.name,
            plugin_type: Some(plugin_meta.type_),
            plugin_group: Some(plugin_meta.group),
        }
    }
}

impl IntoResponse for PluginPushResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status.status_code(), ApiJson(self)).into_response()
    }
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
///
/// Avoids duplicate code in ```push```, ```push_grouped``` (push_async) and ```push_named```
async fn match_plugin_push(
    plugin: &Arc<dyn PushAndPlugin>,
    alertmanager_push: &AlermanagerPush,
) -> PluginPushResponse {
    match plugin.push_alert(alertmanager_push).await {
        Ok(_) => PluginPushResponse {
            status: PluginPushStatus::Ok,
            plugin_meta: plugin.meta().into(),
        },
        Err(error) => {
            tracing::error!(name=plugin.meta().name, %error, "Failed to push alerts to plugin.");
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
/// Join handle for a plugin push response
struct PluginPushResponseJoinHandle {
    /// Join handle
    join_handle: JoinHandle<PluginPushResponse>,
    /// in case the join handle panics or is cancelled, we still want to know which plugin it was
    plugin_meta: PluginMeta,
}

/// Helper function
///
/// Pushes alerts asynchronously
async fn push_async<A: HasPushAndPluginArcRef>(
    state: &ApiState,
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
            plugin_meta: plugin.arc_ref().meta(),
        });
    }

    for plugin_response_handle in plugin_response_handles {
        let plugin_push_response = match plugin_response_handle.join_handle.await {
            Ok(plugin_push_response) => plugin_push_response,
            Err(error) => {
                if error.is_cancelled() {
                    tracing::error!(name=&plugin_response_handle.plugin_meta.name, %error, "Plugin push handler was cancelled.");
                } else {
                    tracing::error!(name=plugin_response_handle.plugin_meta.name, %error, "Plugin push handler panicked.");
                }
                PluginPushResponse {
                    status: PluginPushStatus::Failed {
                        message: error.to_string(),
                    },
                    plugin_meta: plugin_response_handle.plugin_meta.clone().into(),
                }
            }
        };

        let push_label = PushLabel::from(plugin_response_handle.plugin_meta);

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
#[utoipa::path(post, path = "/push", tag = "push", request_body = AlermanagerPush, responses(
    (status = 200, description = "Push was successful.", body = [PushResponse]),
    (status = 207, description = "Some pushes were successful.", body = [PushResponse]),
    (status = 500, description = "Push failed.", body = [PushResponse]),
    (status = 404, description = "No plugins were found.", body = [PushResponse])
))]
#[tracing::instrument(name = "push", skip_all, fields(group_key = alertmanager_push.group_key))]
pub async fn push(
    State(state): State<ApiState>,
    ApiJson(alertmanager_push): ApiJson<AlermanagerPush>,
) -> PushResponse {
    tracing::trace!("Pushing alerts to plugins.");

    let affected_plugins = &state.plugins;

    push_async(&state, affected_plugins, &alertmanager_push).await
}

/// Push alerts to plugins in a group asynchronously
#[utoipa::path(post, path = "/push_grouped/{plugin_group}", tag = "push",
    params(
        ("plugin_group" = String, Path, description = "Name of the plugin group to push to.")
    ),
    request_body = AlermanagerPush, responses(
    (status = 200, description = "Push was successful.", body = [PushResponse]),
    (status = 207, description = "Some pushes were successful.", body = [PushResponse]),
    (status = 500, description = "Push failed.", body = [PushResponse]),
    (status = 404, description = "No plugins were found.", body = [PushResponse])
))]
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
        .filter(|p| p.meta().group == plugin_group)
        .collect();

    push_async(&state, &affected_plugins, &alertmanager_push).await
}

/// Push alerts to a specific plugin
#[utoipa::path(post, path = "/push_named/{plugin_name}", tag = "push",
    params(
        ("plugin_name" = String, Path, description = "Name of the plugin to push to.")
    ),
    request_body = AlermanagerPush, responses(
    (status = 200, description = "Push was successful.", body = [PluginPushResponse]),
    (status = 404, description = "Plugin was not found.", body = [PluginPushResponse]),
    (status = 500, description = "Push failed.", body = [PluginPushResponse])
))]
#[tracing::instrument(name = "push_named",  skip_all, fields(group_key = alertmanager_push.group_key))]
pub async fn push_named(
    State(state): State<ApiState>,
    ApiPath(plugin_name): ApiPath<String>,
    ApiJson(alertmanager_push): ApiJson<AlermanagerPush>,
) -> PluginPushResponse {
    tracing::trace!(name = plugin_name, "Pushing alerts to plugin.");
    let plugin = state.plugins.iter().find(|p| p.meta().name == plugin_name);
    match plugin {
        Some(plugin) => {
            let push_response = match_plugin_push(plugin, &alertmanager_push).await;
            let push_label = PushLabel::from(plugin.meta());
            match push_response.status {
                PluginPushStatus::Ok => {
                    state.prometheus_client.add_success_push(&push_label);
                }
                _ => {
                    state.prometheus_client.add_failed_push(&push_label);
                }
            }
            push_response
        }
        None => PluginPushResponse {
            status: PluginPushStatus::NotFound,
            plugin_meta: PluginPushResponseMeta::not_found(plugin_name),
        },
    }
}
