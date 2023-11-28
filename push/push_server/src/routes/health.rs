use crate::traits::{HasStatusCode, PushAndPlugin};
use crate::{extractors::ApiPath, state::ApiState};
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerHealthResponse {}

impl HasStatusCode for ServerHealthResponse {
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl IntoResponse for ServerHealthResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}

/// Health check for the server
#[utoipa::path(get, path = "/health", tag = "health", responses(
    (status = 200, description = "Server is healthy.", body = [ServerHealthResponse])
))]
pub async fn health() -> ServerHealthResponse {
    ServerHealthResponse {}
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Health status for all plugins
pub enum HealthStatus {
    /// All plugins are healthy
    Healthy,
    /// Some plugins are unhealthy
    Partial,
    /// All plugins are unhealthy
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "content")]
/// Plugin health status
pub enum PluginHealthStatus {
    /// Plugin is healthy
    Healthy,
    /// Plugin was not found
    NotFound,
    /// Plugin is unhealthy
    Unhealthy {
        /// Reason why plugin is unhealthy
        message: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlugingHealthResponse {
    pub status: PluginHealthStatus,
    pub plugin_name: String,
}

impl HasStatusCode for PlugingHealthResponse {
    fn status_code(&self) -> StatusCode {
        match self.status {
            PluginHealthStatus::Healthy => StatusCode::OK,
            PluginHealthStatus::NotFound => StatusCode::NOT_FOUND,
            PluginHealthStatus::Unhealthy { .. } => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl IntoResponse for PlugingHealthResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PluginsHealthResponse {
    /// Health status for all plugins
    pub status: HealthStatus,
    /// Health check for individual plugins
    pub plugin_health_responses: Vec<PlugingHealthResponse>,
}

impl HasStatusCode for PluginsHealthResponse {
    fn status_code(&self) -> StatusCode {
        match self.status {
            HealthStatus::Healthy => StatusCode::OK,
            HealthStatus::Partial => StatusCode::MULTI_STATUS,
            HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl IntoResponse for PluginsHealthResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}

/// Avoids duplicate code in health_all and health_named
async fn match_plugin_health(plugin: &Arc<dyn PushAndPlugin>) -> PlugingHealthResponse {
    match plugin.health().await {
        Ok(_) => PlugingHealthResponse {
            status: PluginHealthStatus::Healthy,
            plugin_name: plugin.name().to_string(),
        },
        Err(error) => {
            tracing::error!(name=plugin.name(), %error, "Plugin is unhealthy.");
            PlugingHealthResponse {
                status: PluginHealthStatus::Unhealthy {
                    message: error.to_string(),
                },
                plugin_name: plugin.name().to_string(),
            }
        }
    }
}

/// Health check for all plugins
#[utoipa::path(get, path = "/health_all", tag = "health", responses(
    (status = 200, description = "All plugins are healthy.", body = [PluginsHealthResponse]),
    (status = 207, description = "Some plugins are unhealthy.", body = [PluginsHealthResponse]),
    (status = 503, description = "All plugins are unhealthy.", body = [PluginsHealthResponse]),
))]
#[tracing::instrument(name = "health_all", skip_all)]
pub async fn health_all(State(state): State<ApiState>) -> PluginsHealthResponse {
    tracing::trace!("Health check for all plugins");

    let mut plugin_health_responses = vec![];
    let mut healthy_plugins_count: usize = 0;

    for plugin in state.plugins.iter() {
        let res = match_plugin_health(plugin).await;
        if let PluginHealthStatus::Healthy = res.status {
            healthy_plugins_count += 1;
        }
        plugin_health_responses.push(res);
    }

    let status = match healthy_plugins_count {
        0 => HealthStatus::Unhealthy,
        n if n == state.plugins.len() => HealthStatus::Healthy,
        _ => HealthStatus::Partial,
    };

    PluginsHealthResponse {
        status,
        plugin_health_responses,
    }
}

/// Health check for a specific plugin
#[utoipa::path(get, path = "/health_named/{plugin_name}", tag = "health",
    params(
        ("plugin_name" = String, Path, description = "Name of the plugin to check.")
    ),
    responses(
        (status = 200, description = "Plugin is healthy.", body = [PlugingHealthResponse]),
        (status = 404, description = "Plugin was not found.", body = [PlugingHealthResponse]),
        (status = 503, description = "Plugin is unhealthy.", body = [PlugingHealthResponse])
))]
#[tracing::instrument(name = "health_named", skip_all)]
pub async fn health_named(
    State(state): State<ApiState>,
    ApiPath(plugin_name): ApiPath<String>,
) -> PlugingHealthResponse {
    tracing::trace!(plugin_name = plugin_name, "Health check for plugin.");
    let plugin = state.plugins.iter().find(|p| p.name() == plugin_name);
    match plugin {
        Some(plugin) => match_plugin_health(plugin).await,
        None => PlugingHealthResponse {
            status: PluginHealthStatus::NotFound,
            plugin_name,
        },
    }
}
