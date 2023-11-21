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
use axum::Extension;
use postgres_plugin::PostgresPlugin;
use push_server::{
    error_response::{ErrorResponse, ErrorResponseType},
    extractors::ApiJson,
    state::ApiV1State,
    traits::{HasOperationDocs, HasResponseDocs, PushAndPlugin},
};

use std::net::SocketAddr;

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    ApiJson(api)
}

async fn not_found() -> ErrorResponse {
    ErrorResponse {
        error_type: ErrorResponseType::NotFound,
    }
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

    let api_v1_state = ApiV1State::new(plugins);

    let app = ApiRouter::new()
        .fallback(not_found)
        .api_route(
            "/health",
            get_with(
                push_server::routes::health::health,
                push_server::routes::health::HealthResponse::operation_docs,
            ),
        )
        .route("/redoc", Redoc::new("/api.json").axum_route())
        .route("/api.json", get(serve_api))
        .nest_api_service(
            "/api",
            ApiRouter::new().nest(
                "/v1",
                ApiRouter::new()
                    .api_route(
                        "/push",
                        post_with(
                            push_server::routes::push::push,
                            push_server::routes::push::PushResponse::operation_docs,
                        ),
                    )
                    .api_route(
                        "/push_named/:plugin_name",
                        post_with(
                            push_server::routes::push::push_named,
                            push_server::routes::push::PluginPushResponse::operation_docs,
                        ),
                    )
                    .with_state(api_v1_state),
            ),
        );

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
        .default_response_with::<ApiJson<ErrorResponse>, _>(ErrorResponse::response_docs)
}
