use anyhow::Result;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
  let fmt_layer = fmt::layer()
    // disable printing the name of the module in every log line.
    .with_target(false)
    // disabling time because CloudWatch will add the ingestion time.
    .without_time();
  let filter_layer = EnvFilter::try_from_default_env()
    .or_else(|_| EnvFilter::try_new("trace,hyper=debug,tower_http=debug,axum::rejection=trace"))
    .unwrap();

  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();

  myhtmxlib::run_server().await
}
