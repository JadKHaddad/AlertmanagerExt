use aide::{
    axum::{
        routing::{get, get_with, post_with},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    redoc::Redoc,
    transform::TransformOpenApi,
};
use anyhow::{Context, Result as AnyResult};
use axum::{extract::State, Extension};
use models::AlermanagerPush;
use postgres_plugin::PostgresPlugin;
use push_server::{
    api_response::{ApiErrorResponse, ApiErrorResponseType, ApiOkResponse},
    extractors::{ApiJson, ApiPath},
    push_response::{PluginPushResponse, PluginPushStatus, PushResponse, PushStatus},
    traits::{HasOperationDocs, HasResponseDocs, PushAndPlugin},
};

use std::{net::SocketAddr, ops::Deref, sync::Arc};

#[derive(Clone)]
struct ApiV1State {
    inner: Arc<ApiV1StateInner>,
}

struct ApiV1StateInner {
    plugins: Vec<Box<dyn PushAndPlugin>>,
}

impl Deref for ApiV1State {
    type Target = ApiV1StateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
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

async fn health_named(
    State(state): State<ApiV1State>,
    ApiPath(plugin_name): ApiPath<String>,
) -> &'static str {
    todo!()
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "push_server=trace,postgres_plugin=trace,tower_http=info",
        );
    }

    tracing_subscriber::fmt()
        //.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_line_number(false)
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_level(true)
        .with_ansi(true)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let postgres_plugin = PostgresPlugin::new(String::from(
        "postgres://user:password@localhost:5432/database",
    ))
    .await
    .context("Failed to create Postgres plugin.")?;

    let plugins: Vec<Box<dyn PushAndPlugin>> = vec![Box::new(postgres_plugin)];
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
        .summary("AlertmanagerExt")
        .version(env!("CARGO_PKG_VERSION"))
        .default_response_with::<ApiJson<ApiErrorResponse>, _>(ApiErrorResponse::response_docs)
}
