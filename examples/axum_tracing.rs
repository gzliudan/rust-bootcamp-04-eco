use std::time::Duration;

use axum::{extract::Request, routing::get, Router};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, RandomIdGenerator, Tracer},
    Resource,
};
use tokio::{
    join,
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
        .with_filter(LevelFilter::INFO);

    // file appender layer for tracing-subscriber
    let file_appender = tracing_appender::rolling::daily("logs", "eco.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file = fmt::Layer::new()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    // opentelemetry tracing layer for tracing-subscriber
    let tracer = init_tracer()?;
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .with(opentelemetry)
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

#[instrument(fields(http.uri = req.uri().path(), http.method = req.method().as_str()))]
async fn index_handler(req: Request) -> &'static str {
    debug!("index handler started");
    sleep(Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status_code = 200, "index handler completed");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();

    // spawn multiple tasks
    let sl = sleep(Duration::from_millis(500));
    let t1 = task1();
    let t2 = task2();
    let t3 = task3();
    join!(sl, t1, t2, t3);

    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, World!"
}

#[instrument]
async fn task1() {
    sleep(Duration::from_millis(100)).await;
}

#[instrument]
async fn task2() {
    sleep(Duration::from_millis(200)).await;
}

#[instrument]
async fn task3() {
    sleep(Duration::from_millis(300)).await;
}

fn init_tracer() -> anyhow::Result<Tracer> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(32)
                .with_max_attributes_per_span(64)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "axum-tracing",
                )])),
        )
        .install_batch(runtime::Tokio)?;
    Ok(tracer)
}
