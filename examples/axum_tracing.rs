use std::time::Duration;

use axum::{routing::get, Router};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing_subscriber::fmt::init();

    // console layer for tracing-subscriber
    let console = fmt::Layer::new()
        .with_ansi(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::DEBUG);

    let file_appender = tracing_appender::rolling::daily("logs", "eco.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = fmt::Layer::new()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let addr = "0.0.0.0:8080";
    // build our application with a single route
    let app = Router::new().route("/", get(index_handler));
    info!("Start server on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[instrument]
async fn index_handler() -> &'static str {
    debug!("index_handler called");
    sleep(Duration::from_millis(100)).await;
    let ret = long_task().await;
    info!(http.status = 200, "inder_handler completed");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    sleep(Duration::from_millis(1000)).await;
    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "task is too long");
    "Hello, world!"
}
