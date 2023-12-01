use anyhow::{Context, Result as AnyResult};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use postgres_plugin::{PostgresPlugin, PostgresPluginConfig, PostgresPluginMeta};
use push_definitions::Push;
use push_server::{
    error_response::ErrorResponse, openapi::OpenApiDocFinalizer, state::ApiState,
    traits::PushAndPlugin,
};
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

#[tokio::main]
async fn main() -> AnyResult<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "push_server=trace,push_server::extractors=off,postgres_plugin=trace,tower_http=trace",
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

    // Plugins are initialized before they are added to the state.
    // Well because of Arc.
    let plugins: Vec<Arc<dyn PushAndPlugin>> = vec![Arc::new(postgres_plugin)];

    let state = ApiState::new(plugins);

    let app = Router::new()
        .fallback(not_found)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", OpenApiDocFinalizer::openapi()),
        )
        .merge(Redoc::with_url("/redoc", OpenApiDocFinalizer::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .route("/metrics", get(push_server::routes::metrics::metrics))
        .route("/health", get(push_server::routes::health::health))
        .route("/health_all", get(push_server::routes::health::health_all))
        .route(
            "/health_named/:plugin_name",
            get(push_server::routes::health::health_named),
        )
        .route("/push", post(push_server::routes::push::push))
        .route(
            "/push_grouped/:plugin_group",
            post(push_server::routes::push::push_grouped),
        )
        .route(
            "/push_grouped_exclusive/:plugin_group",
            post(push_server::routes::push::push_grouped_exclusive),
        )
        .route(
            "/push_named_exclusive/:plugin_name",
            post(push_server::routes::push::push_named_exclusive),
        )
        .route(
            "/push_named/:plugin_name",
            post(push_server::routes::push::push_named),
        )
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(
                    push_server::middlewares::method_not_allowed,
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
