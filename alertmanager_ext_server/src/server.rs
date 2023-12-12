use crate::{
    config::Config, error_response::ErrorResponse, openapi::OpenApiDocFinalizer, state::ApiState,
    traits::PushAndPlugin,
};
use anyhow::{Context, Result as AnyResult};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use file_plugin::FilePlugin;
use filter_plugin::FilterPlugin;
use postgres_plugin::PostgresPlugin;
use postgres_sea_plugin::PostgresSeaPlugin;
use postgres_x_plugin::PostgresXPlugin;
use print_plugin::PrintPlugin;
use push_definitions::Push;
use sqlite_plugin::SqlitePlugin;
use std::{collections::HashSet, sync::Arc};
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

async fn create_plugins(config: Config) -> AnyResult<Vec<Arc<dyn PushAndPlugin>>> {
    let mut plugins: Vec<Arc<dyn PushAndPlugin>> = vec![];

    tracing::debug!("Creating plugins.");

    if let Some(plugins_from_file) = config.plugins {
        if let Some(file_plugins) = plugins_from_file.file_plugin {
            for conf_file_plugin in file_plugins {
                let mut file_plugin =
                    FilePlugin::new(conf_file_plugin.meta, conf_file_plugin.config).await?;

                file_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize File plugin")?;

                plugins.push(Arc::new(file_plugin));
            }
        }

        if let Some(filter_plugins) = plugins_from_file.filter_plugin {
            for conf_filter_plugin in filter_plugins {
                let mut filter_plugin =
                    FilterPlugin::new(conf_filter_plugin.meta, conf_filter_plugin.config)
                        .context("Failed to create Filter plugin")?;

                filter_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize Filter plugin")?;

                plugins.push(Arc::new(filter_plugin));
            }
        }

        if let Some(postgres_plugins) = plugins_from_file.postgres_plugin {
            for conf_postgres_plugin in postgres_plugins {
                let mut postgres_plugin =
                    PostgresPlugin::new(conf_postgres_plugin.meta, conf_postgres_plugin.config)
                        .await
                        .context("Failed to create Postgres plugin")?;

                postgres_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize Postgres plugin")?;

                plugins.push(Arc::new(postgres_plugin));
            }
        }

        if let Some(postgres_sea_plugins) = plugins_from_file.postgres_sea_plugin {
            for conf_postgres_sea_plugin in postgres_sea_plugins {
                let mut postgres_sea_plugin = PostgresSeaPlugin::new(
                    conf_postgres_sea_plugin.meta,
                    conf_postgres_sea_plugin.config,
                )
                .await
                .context("Failed to create PostgresSea plugin")?;

                postgres_sea_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize PostgresSea plugin")?;

                plugins.push(Arc::new(postgres_sea_plugin));
            }
        }

        if let Some(postgres_x_plugins) = plugins_from_file.postgres_x_plugin {
            for conf_postgres_x_plugin in postgres_x_plugins {
                let mut postgres_x_plugin = PostgresXPlugin::new(
                    conf_postgres_x_plugin.meta,
                    conf_postgres_x_plugin.config,
                )
                .await
                .context("Failed to create PostgresX plugin")?;

                postgres_x_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize PostgresX plugin")?;

                plugins.push(Arc::new(postgres_x_plugin));
            }
        }

        if let Some(print_plugins) = plugins_from_file.print_plugin {
            for conf_print_plugin in print_plugins {
                let mut print_plugin =
                    PrintPlugin::new(conf_print_plugin.meta, conf_print_plugin.config).await?;

                print_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize Print plugin")?;

                plugins.push(Arc::new(print_plugin));
            }
        }

        if let Some(sqlite_plugins) = plugins_from_file.sqlite_plugin {
            for conf_sqlite_plugin in sqlite_plugins {
                let mut sqlite_plugin =
                    SqlitePlugin::new(conf_sqlite_plugin.meta, conf_sqlite_plugin.config)
                        .context("Failed to create SQLite plugin")?;

                sqlite_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize SQLite plugin")?;

                plugins.push(Arc::new(sqlite_plugin));
            }
        }
    } else {
        tracing::warn!("No plugins configured.");
    }

    let mut plugin_names: HashSet<&str> = HashSet::new();

    for plugin in &plugins {
        let name = plugin.name();
        if plugin_names.contains(name) {
            tracing::warn!(name = name, "Duplicate plugin name.");
        }

        plugin_names.insert(name);

        tracing::debug!(
            name = %name,
            type_ = %plugin.type_(),
            group = %plugin.group(),
            "Plugin ready."
        );
    }

    Ok(plugins)
}

async fn create_router(config: Config) -> AnyResult<Router> {
    let plugins = create_plugins(config).await?;

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

    Ok(app)
}

pub async fn run(config: Config) -> AnyResult<()> {
    let addr = config.addr();

    let app = create_router(config).await?;

    tracing::info!(%addr, "Starting server.");
    axum::Server::try_bind(&addr)
        .context("Failed to bind server")?
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use random_models_generator::generate_random_alertmanager_pushes;
    use tracing_test::traced_test;

    #[ignore]
    #[tokio::test]
    #[traced_test]
    // cargo test --package alertmanager_ext_server --lib --release -- server::tests::push_random_alerts --exact --nocapture --ignored
    async fn push_random_alerts() {
        let config = Config::new_from_yaml_str(include_str!("../../config.yaml"))
            .await
            .expect("Failed to load config.");

        let app = create_router(config)
            .await
            .expect("Failed to create router.");

        let server = TestServer::new(app).expect("Failed to create test server.");

        let pushes = generate_random_alertmanager_pushes(1000);
        for (i, push) in pushes.iter().enumerate() {
            tracing::info!("Pushing alert {}/{}", i + 1, pushes.len());
            server.post("/push").json(&push).await;
        }
    }
}
