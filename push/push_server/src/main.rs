use aide::{
    axum::{
        routing::{get, post_with},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    redoc::Redoc,
    transform::TransformOperation,
    OperationIo,
};
use anyhow::{Context, Result as AnyResult};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use models::AlermanagerPush;
use plugins_definitions::Plugin;
use push_definitions::Push;
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
pub enum PushStatus {
    Ok,
    Partial,
    Failed,
}

impl PushStatus {
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
pub enum PluginPushStatus {
    Ok,
    NotFound,
    Failed { error_message: String },
}

impl PluginPushStatus {
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
pub struct PluginPushResponse {
    pub status: PluginPushStatus,
    pub plugin_name: String,
}

impl PluginPushResponse {
    fn create_docs(op: TransformOperation) -> TransformOperation {
        op.description("Push alerts to a plugin")
            .response_with::<201, Json<Self>, _>(|res| {
                res.description("Alerts were pushed successfully").example({
                    PluginPushResponse {
                        status: PluginPushStatus::Ok,
                        plugin_name: "Plugin 1".to_string(),
                    }
                })
            })
            .response_with::<500, Json<Self>, _>(|res| {
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
        (self.status.status_code(), Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo)]
#[serde(rename_all = "camelCase")]
pub struct PushResponse {
    pub status: PushStatus,
    pub plugins: Vec<PluginPushResponse>,
}

impl PushResponse {
    fn create_docs(op: TransformOperation) -> TransformOperation {
        op.description("Push alerts to plugins")
            .response_with::<201, Json<Self>, _>(|res| {
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
            .response_with::<207, Json<Self>, _>(|res| {
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
            .response_with::<500, Json<Self>, _>(|res| {
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
        (self.status.status_code(), Json(self)).into_response()
    }
}

async fn push(
    State(state): State<ApiV1State>,
    Json(alertmanager_push): Json<AlermanagerPush>,
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
    Path(plugin_name): Path<String>,
    Json(alertmanager_push): Json<AlermanagerPush>,
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
    Json(api)
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let plugins: Vec<Box<dyn Serverble>> = vec![];
    for plugin in &plugins {
        plugin.initialize().await.unwrap();
    }
    let api_v1_state = ApiV1State {
        inner: Arc::new(ApiV1StateInner { plugins }),
    };

    let v1 = ApiRouter::new()
        .api_route("/push", post_with(push, PushResponse::create_docs))
        .api_route(
            "/push_named/:plugin_name",
            post_with(push_named, PluginPushResponse::create_docs),
        )
        .with_state(api_v1_state);

    let api_v1 = ApiRouter::new().nest("/v1", v1);

    let app = ApiRouter::new()
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
            app.finish_api(&mut open_api)
                .layer(Extension(open_api))
                .into_make_service(),
        )
        .await
        .context("Server failed")?;

    Ok(())
}