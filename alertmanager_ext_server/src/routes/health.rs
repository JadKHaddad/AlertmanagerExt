use crate::{
    extractors::query::ApiPluginFilterQuery,
    routes::models::{PluginFilterQuery, PluginResponseMeta},
};
use crate::{
    state::ApiState,
    traits::{HasStatusCode, PushAndPlugin},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use schemars::JsonSchema;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, JsonSchema, ToSchema)]
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
#[utoipa::path(
    get,
    path = "/health", 
    tag = "health", 
    responses(
        (status = 200, description = "Server is healthy.", body = ServerHealthResponse)
    )
)]
pub async fn health() -> ServerHealthResponse {
    ServerHealthResponse {}
}

#[derive(Clone, Debug, Serialize, JsonSchema, ToSchema)]
/// Health status for all plugins
pub enum HealthStatus {
    /// All plugins are healthy
    Healthy,
    /// Some plugins are unhealthy
    Partial,
    /// All plugins are unhealthy
    Unhealthy,
    /// No plugins were found
    NoPlugins,
}

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
#[serde(tag = "type", content = "content")]
/// Plugin health status
pub enum PluginHealthStatus {
    /// Plugin is healthy
    Healthy,
    /// Plugin is unhealthy
    Unhealthy {
        /// Reason why plugin is unhealthy
        message: String,
    },
}

#[derive(Clone, Debug, Serialize, JsonSchema, ToSchema)]
pub struct PlugingHealthResponse {
    /// Health status for the plugin
    pub status: PluginHealthStatus,
    /// Meta information about the plugin
    pub plugin_meta: PluginResponseMeta,
}

impl HasStatusCode for PlugingHealthResponse {
    fn status_code(&self) -> StatusCode {
        match self.status {
            PluginHealthStatus::Healthy => StatusCode::OK,
            PluginHealthStatus::Unhealthy { .. } => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

#[derive(Clone, Debug, Serialize, JsonSchema, ToSchema)]
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
            HealthStatus::NoPlugins => StatusCode::NOT_FOUND,
        }
    }
}

impl IntoResponse for PluginsHealthResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Json(self)).into_response()
    }
}

/// Helper function
async fn match_plugin_health(plugin: &Arc<dyn PushAndPlugin>) -> PlugingHealthResponse {
    match plugin.health().await {
        Ok(_) => PlugingHealthResponse {
            status: PluginHealthStatus::Healthy,
            plugin_meta: plugin.meta().into(),
        },
        Err(error) => {
            tracing::error!(name=plugin.name(), %error, "Plugin is unhealthy.");
            PlugingHealthResponse {
                status: PluginHealthStatus::Unhealthy {
                    message: error.to_string(),
                },
                plugin_meta: plugin.meta().into(),
            }
        }
    }
}

/// Health check for plugins
#[utoipa::path(
    get,
    path = "/plugin_health", 
    tag = "health", 
    params(
        PluginFilterQuery
    ),
    responses(
        (status = 200, description = "All affected plugins are healthy.", body = PluginsHealthResponse, example = json!({
            "status": "Healthy",
            "plugin_health_responses": [
                {
                    "status": "Healthy",
                    "plugin_meta": {
                        "plugin_name": "example",
                        "plugin_type": "push",
                        "plugin_group": "example"
                    }
                }
            ]
        })),
        (status = 207, description = "Some affected plugins are unhealthy.", body = PluginsHealthResponse),
        (status = 503, description = "All affected plugins are unhealthy.", body = PluginsHealthResponse),
        (status = 404, description = "No plugins were found.", body = PluginsHealthResponse)
    )
)]
#[tracing::instrument(name = "plugin_health", skip_all)]
pub async fn plugin_health(
    State(state): State<ApiState>,
    ApiPluginFilterQuery(exp): ApiPluginFilterQuery,
) -> PluginsHealthResponse {
    tracing::trace!("Health check for plugins");

    let mut plugin_health_responses = vec![];
    let mut healthy_plugins_count: usize = 0;

    let affected_plugins: Vec<&Arc<dyn PushAndPlugin>> = if let Some(ref exp) = exp {
        state
            .plugins
            .iter()
            .filter(|plugin| exp.is_match(&plugin.meta()))
            .collect()
    } else {
        state.plugins.iter().collect()
    };

    if affected_plugins.is_empty() {
        return PluginsHealthResponse {
            status: HealthStatus::NoPlugins,
            plugin_health_responses: vec![],
        };
    }

    for plugin in affected_plugins.iter() {
        let res = match_plugin_health(plugin).await;
        if let PluginHealthStatus::Healthy = res.status {
            healthy_plugins_count += 1;
        }
        plugin_health_responses.push(res);
    }

    let status = match healthy_plugins_count {
        0 => HealthStatus::Unhealthy,
        n if n == affected_plugins.len() => HealthStatus::Healthy,
        _ => HealthStatus::Partial,
    };

    PluginsHealthResponse {
        status,
        plugin_health_responses,
    }
}
