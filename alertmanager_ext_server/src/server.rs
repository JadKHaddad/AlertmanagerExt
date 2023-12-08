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
use file_plugin::FilePlugin;
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

pub async fn run(config: crate::config::Config) -> AnyResult<()> {
    let addr = config.addr();

    let mut plugins: Vec<Arc<dyn PushAndPlugin>> = vec![];

    tracing::debug!("Creating plugins.");
    if let Some(plugins_from_file) = config.plugins {
        if let Some(file_plugins) = plugins_from_file.file_plugin {
            for conf_file_plugin in file_plugins {
                let mut file_plugin =
                    FilePlugin::new(conf_file_plugin.meta, conf_file_plugin.config);

                file_plugin
                    .initialize()
                    .await
                    .context("Failed to initialize File plugin")?;

                plugins.push(Arc::new(file_plugin));
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
                    PrintPlugin::new(conf_print_plugin.meta, conf_print_plugin.config);

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

    {
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
    }

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
