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
use file_plugin::FilePluginMeta;
use postgres_plugin::{PostgresPlugin, PostgresPluginConfig, PostgresPluginMeta};
use postgres_x_plugin::{PostgresXPlugin, PostgresXPluginConfig, PostgresXPluginMeta};
use push_definitions::Push;
use sqlite_plugin::{SqlitePlugin, SqlitePluginConfig, SqlitePluginMeta};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
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
        .context("Failed to create Postgres plugin")?;

    postgres_plugin
        .initialize()
        .await
        .context("Failed to initialize Postgres plugin")?;

    let postgres_x_plugin_config = PostgresXPluginConfig {
        connection_string: String::from("postgres://user:password@localhost:5433/database"),
        max_connections: 15,
        connection_timeout: std::time::Duration::from_secs(5),
    };

    let postgres_x_plugin_meta = PostgresXPluginMeta {
        name: String::from("postgres_x_plugin_1"),
        group: String::from("default"),
    };

    let mut postgres_x_plugin =
        PostgresXPlugin::new(postgres_x_plugin_meta, postgres_x_plugin_config)
            .await
            .context("Failed to create Postgres plugin")?;

    postgres_x_plugin
        .initialize()
        .await
        .context("Failed to initialize Postgres plugin")?;

    let sqlite_plugin_config = SqlitePluginConfig {
        database_url: String::from("file:alertmanager_ext_server/db/sqlite.db"),
    };

    let sqlite_plugin_meta = SqlitePluginMeta {
        name: String::from("sqlite_plugin_1"),
        group: String::from("default"),
    };

    let mut sqlite_plugin = SqlitePlugin::new(sqlite_plugin_meta, sqlite_plugin_config)
        .context("Failed to create SQLite plugin")?;

    sqlite_plugin
        .initialize()
        .await
        .context("Failed to initialize SQLite plugin")?;

    let file_plugin_meta = FilePluginMeta {
        name: String::from("file_plugin_1"),
        group: String::from("default"),
    };

    let file_plugin_config = file_plugin::FilePluginConfig {
        dir_path: std::path::PathBuf::from("alertmanager_ext_server/pushes"),
        file_type: file_plugin::FileType::Json,
    };

    let mut file_plugin = file_plugin::FilePlugin::new(file_plugin_meta, file_plugin_config);

    file_plugin
        .initialize()
        .await
        .context("Failed to initialize File plugin")?;

    let print_plugin_meta = print_plugin::PrintPluginMeta {
        name: String::from("print_plugin_1"),
        group: String::from("default"),
    };

    let print_plugin_config = print_plugin::PrintPluginConfig {
        print_type: print_plugin::PrintType::Debug,
    };

    let mut print_plugin = print_plugin::PrintPlugin::new(print_plugin_meta, print_plugin_config);

    print_plugin
        .initialize()
        .await
        .context("Failed to initialize Print plugin")?;

    // Plugins are initialized before they are added to the state.
    // Well because of Arc.
    let plugins: Vec<Arc<dyn PushAndPlugin>> = vec![
        Arc::new(postgres_plugin),
        Arc::new(postgres_x_plugin),
        Arc::new(sqlite_plugin),
        Arc::new(file_plugin),
        Arc::new(print_plugin),
    ];

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
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(
                    crate::middlewares::method_not_allowed::method_not_allowed,
                ))
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
                )
                .layer(middleware::from_fn(
                    crate::middlewares::trace_response_body::trace_response_body,
                )),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 5050));

    tracing::info!(%addr, "Starting server.");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server failed")?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler.");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler.")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutting down.");
}
