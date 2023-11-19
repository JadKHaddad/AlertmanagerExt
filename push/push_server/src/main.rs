use aide::{
    axum::{
        routing::{get, get_with, post_with},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    redoc::Redoc,
    transform::{TransformOpenApi, TransformOperation},
    OperationIo,
};
use anyhow::{Context, Result as AnyResult};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension};
use models::AlermanagerPush;
use plugins_definitions::Plugin;
use push_definitions::Push;
use push_server::{
    api_response::{ApiErrorResponse, ApiErrorResponseType, ApiOkResponse},
    api_traits::{HasOperationDocs, HasResponseDocs, HasStatusCode},
    extractors::{ApiJson, ApiPath},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, ops::Deref, sync::Arc};

trait Serverble: Push + Plugin {}

#[derive(Clone)]
struct ApiV1State {
    inner: Arc<ApiV1StateInner>,
}

struct ApiV1StateInner {
    plugins: Vec<Box<dyn Serverble>>,
}

impl Deref for ApiV1State {
    type Target = ApiV1StateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Push status
pub enum PushStatus {
    /// Push was successful
    Ok,
    /// Some alerts were pushed successfully
    Partial,
    /// Push failed
    Failed,
}

impl HasStatusCode for PushStatus {
    fn status_code(&self) -> StatusCode {
        match self {
            PushStatus::Ok => StatusCode::ACCEPTED,
            PushStatus::Partial => StatusCode::MULTI_STATUS,
            PushStatus::Failed => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "error")]
/// Push status for a plugin
pub enum PluginPushStatus {
    /// Push was successful
    Ok,
    /// Plugin was not found
    NotFound,
    /// Push failed
    Failed {
        /// Error message
        error_message: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo)]
#[serde(rename_all = "camelCase")]
/// Response for a plugin push
pub struct PluginPushResponse {
    /// Status of the push for the plugin
    pub status: PluginPushStatus,
    /// Name of the plugin
    pub plugin_name: String,
}

impl HasOperationDocs for PluginPushResponse {
    fn operation_docs(op: TransformOperation) -> TransformOperation {
        op.description("Push alerts to a plugin")
            .response_with::<201, ApiJson<Self>, _>(|res| {
                res.description("Alerts were pushed successfully").example({
                    PluginPushResponse {
                        status: PluginPushStatus::Ok,
                        plugin_name: "Plugin 1".to_string(),
                    }
                })
            })
            .response_with::<500, ApiJson<Self>, _>(|res| {
                res.description("Failed to push alerts")
                    .example(PluginPushResponse {
                        status: PluginPushStatus::Failed {
                            error_message: "Some error".to_string(),
                        },
                        plugin_name: "Plugin 1".to_string(),
                    })
            })
    }
}

impl IntoResponse for PluginPushResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status.status_code(), ApiJson(self)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo)]
#[serde(rename_all = "camelCase")]
/// Response for a push
pub struct PushResponse {
    /// Status of the push
    pub status: PushStatus,
    /// Responses for each plugin
    pub plugins: Vec<PluginPushResponse>,
}

impl HasOperationDocs for PushResponse {
    fn operation_docs(op: TransformOperation) -> TransformOperation {
        op.description("Push alerts to plugins")
            .response_with::<201, ApiJson<Self>, _>(|res| {
                res.description("All alerts were pushed successfully")
                    .example({
                        PushResponse {
                            status: PushStatus::Ok,
                            plugins: vec![
                                PluginPushResponse {
                                    status: PluginPushStatus::Ok,
                                    plugin_name: "Plugin 1".to_string(),
                                },
                                PluginPushResponse {
                                    status: PluginPushStatus::Ok,
                                    plugin_name: "Plugin 2".to_string(),
                                },
                            ],
                        }
                    })
            })
            .response_with::<207, ApiJson<Self>, _>(|res| {
                res.description("Some alerts were pushed successfully")
                    .example({
                        PushResponse {
                            status: PushStatus::Partial,
                            plugins: vec![
                                PluginPushResponse {
                                    status: PluginPushStatus::Ok,
                                    plugin_name: "Plugin 1".to_string(),
                                },
                                PluginPushResponse {
                                    status: PluginPushStatus::Failed {
                                        error_message: "Some error".to_string(),
                                    },
                                    plugin_name: "Plugin 2".to_string(),
                                },
                            ],
                        }
                    })
            })
            .response_with::<500, ApiJson<Self>, _>(|res| {
                res.description("Failed to push alerts")
                    .example(PushResponse {
                        status: PushStatus::Failed,
                        plugins: vec![
                            PluginPushResponse {
                                status: PluginPushStatus::Failed {
                                    error_message: "Some error".to_string(),
                                },
                                plugin_name: "Plugin 1".to_string(),
                            },
                            PluginPushResponse {
                                status: PluginPushStatus::Failed {
                                    error_message: "Some error".to_string(),
                                },
                                plugin_name: "Plugin 2".to_string(),
                            },
                        ],
                    })
            })
    }
}

impl IntoResponse for PushResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status.status_code(), ApiJson(self)).into_response()
    }
}

async fn push(
    State(state): State<ApiV1State>,
    ApiJson(alertmanager_push): ApiJson<AlermanagerPush>,
) -> PushResponse {
    let mut plugins = vec![];
    // TODO: Eventually this should be parallelized
    for plugin in &state.plugins {
        match plugin.push_alert(&alertmanager_push).await {
            Ok(_) => plugins.push(PluginPushResponse {
                status: PluginPushStatus::Ok,
                plugin_name: plugin.name().to_string(),
            }),
            Err(e) => plugins.push(PluginPushResponse {
                status: PluginPushStatus::Failed {
                    error_message: e.to_string(),
                },
                plugin_name: plugin.name().to_string(),
            }),
        }
    }

    let status = if plugins.iter().any(|p| p.status != PluginPushStatus::Ok) {
        PushStatus::Failed
    } else if plugins.iter().any(|p| p.status == PluginPushStatus::Ok) {
        PushStatus::Partial
    } else {
        PushStatus::Ok
    };

    PushResponse { status, plugins }
}

async fn push_named(
    State(state): State<ApiV1State>,
    ApiPath(plugin_name): ApiPath<String>,
    ApiJson(alertmanager_push): ApiJson<AlermanagerPush>,
) -> PluginPushResponse {
    let plugin = state.plugins.iter().find(|p| p.name() == plugin_name);
    match plugin {
        Some(plugin) => match plugin.push_alert(&alertmanager_push).await {
            Ok(_) => PluginPushResponse {
                status: PluginPushStatus::Ok,
                plugin_name,
            },
            Err(e) => PluginPushResponse {
                status: PluginPushStatus::Failed {
                    error_message: e.to_string(),
                },
                plugin_name,
            },
        },
        None => PluginPushResponse {
            status: PluginPushStatus::NotFound,
            plugin_name,
        },
    }
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    ApiJson(api)
}

async fn not_found() -> ApiErrorResponse {
    ApiErrorResponse {
        error_type: ApiErrorResponseType::NotFound,
    }
}

async fn health() -> ApiOkResponse {
    ApiOkResponse {}
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let plugins: Vec<Box<dyn Serverble>> = vec![];
    for plugin in &plugins {
        plugin
            .initialize()
            .await
            .context(format!("Failed to initialize plugin: {}", plugin.name()))?;
    }
    let api_v1_state = ApiV1State {
        inner: Arc::new(ApiV1StateInner { plugins }),
    };

    let v1 = ApiRouter::new()
        .api_route("/push", post_with(push, PushResponse::operation_docs))
        .api_route(
            "/push_named/:plugin_name",
            post_with(push_named, PluginPushResponse::operation_docs),
        )
        .with_state(api_v1_state);

    let api_v1 = ApiRouter::new().nest("/v1", v1);

    let app = ApiRouter::new()
        .fallback(not_found)
        .api_route(
            "/health",
            get_with(health, |op| {
                op.description("Health check")
                    .response_with::<200, ApiJson<ApiOkResponse>, _>(|res| {
                        res.description("API is healthy").example(ApiOkResponse {})
                    })
            }),
        )
        .route("/redoc", Redoc::new("/api.json").axum_route())
        .route("/api.json", get(serve_api))
        .nest_api_service("/api", api_v1);

    let mut open_api = OpenApi {
        info: Info {
            title: "AlertmanagerExt".to_string(),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(
            app.finish_api_with(&mut open_api, api_docs)
                .layer(Extension(open_api))
                .into_make_service(),
        )
        .await
        .context("Server failed")?;

    Ok(())
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("AlertmanagerExt API")
        .summary("API for AlertmanagerExt")
        .default_response_with::<ApiJson<ApiErrorResponse>, _>(ApiErrorResponse::response_docs)
}
