use alertmanager_ext_server::{cli::Cli, config::Config};
use anyhow::{Context, Result as AnyResult};
use clap::Parser;

#[tokio::main]
async fn main() -> AnyResult<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "alertmanager_ext_server=trace,alertmanager_ext_server::extractors=trace,alertmanager_ext_server::middlewares::trace_response_body=trace,postgres_plugin=trace,postgres_x_plugin=trace,sqlite_plugin=trace,file_plugin=trace,print_plugin=trace,tower_http=trace",
        );
    }

    let cli = Cli::parse();

    let config = Config::new_from_yaml_file(cli.config_file)
        .await
        .context("Failed to create config")?;

    tracing_subscriber::fmt()
        //.with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
        .with_line_number(false)
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_level(true)
        .with_ansi(true)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .map_err(|error| anyhow::anyhow!(error))
        .context("Failed to initialize tracing subscriber")?;

    alertmanager_ext_server::server::run(config).await
}
