use anyhow::Result as AnyResult;

#[tokio::main]
async fn main() -> AnyResult<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "alertmanager_ext_server=trace,push_server::extractors=trace,postgres_plugin=trace,sqlite_plugin=trace,tower_http=trace",
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

    alertmanager_ext_server::server::run().await
}
