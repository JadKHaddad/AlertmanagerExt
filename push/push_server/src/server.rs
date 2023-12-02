use crate::{
    error_response::ErrorResponse, openapi::OpenApiDocFinalizer, state::ApiState,
    traits::PushAndPlugin,
};
use anyhow::{Context, Result as AnyResult};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use postgres_plugin::{PostgresPlugin, PostgresPluginConfig, PostgresPluginMeta};
use push_definitions::Push;
use sqlite_plugin::{SqlitePlugin, SqlitePluginConfig, SqlitePluginMeta};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

async fn not_found() -> ErrorResponse {
    ErrorResponse::not_found()
}

pub async fn run() -> AnyResult<()> {
    let postgres_plugin_config = PostgresPluginConfig {
        connection_string: String::from("postgres://user:password@localhost:5432/database"),
        max_connections: 15,
        connection_timeout: std::time::Duration::from_secs(5),
    };

    let postgres_plugin_meta = PostgresPluginMeta {
        name: String::from("postgres_plugin_1"),
        group: String::from("default"),
    };

    let mut postgres_plugin = PostgresPlugin::new(postgres_plugin_meta, postgres_plugin_config)
        .await
        .context("Failed to create Postgres plugin.")?;

    postgres_plugin
        .initialize()
        .await
        .context("Failed to initialize Postgres plugin.")?;

    let sqlite_plugin_config = SqlitePluginConfig {
        database_url: String::from("file:push/push_server/db/sqlite.db"),
    };

    let sqlite_plugin_meta = SqlitePluginMeta {
        name: String::from("sqlite_plugin_1"),
        group: String::from("default"),
    };

    let mut sqlite_plugin = SqlitePlugin::new(sqlite_plugin_meta, sqlite_plugin_config)
        .context("Failed to create SQLite plugin.")?;

    sqlite_plugin
        .initialize()
        .await
        .context("Failed to initialize SQLite plugin.")?;

    // Plugins are initialized before they are added to the state.
    // Well because of Arc.
    let plugins: Vec<Arc<dyn PushAndPlugin>> =
        vec![Arc::new(postgres_plugin), Arc::new(sqlite_plugin)];

    let state = ApiState::new(plugins);

    let app = Router::new()
        .fallback(not_found)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", OpenApiDocFinalizer::openapi()),
        )
        .merge(Redoc::with_url("/redoc", OpenApiDocFinalizer::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .route("/metrics", get(crate::routes::metrics::metrics))
        .route("/health", get(crate::routes::health::health))
        .route("/plugin_health", get(crate::routes::health::plugin_health))
        .route("/push", post(crate::routes::push::push))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(crate::middlewares::method_not_allowed))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(false)
                                .level(Level::INFO),
                        )
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                ),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!(%addr, "Starting server.");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Server failed")?;

    Ok(())
}
