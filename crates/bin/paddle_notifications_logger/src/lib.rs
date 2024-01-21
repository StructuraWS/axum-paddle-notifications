use hyper::body::Bytes;
use lambda_http::run;
use std::net::SocketAddr;
use std::{env::var, error::Error};

use anyhow::{anyhow, Result};
use axum::{
  routing::{get, post},
  Router,
};
use tracing::info;
pub async fn run_server() -> Result<()> {
  info!("Starting");

  if var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
    return run_aws().await;
  }
  run_no_aws().await
}

async fn run_aws() -> Result<()> {
  info!("Initialized. Starting lambda_http");
  // TODO fix this:
  // let app = axum_helper::add_middleware_layers(get_routes());

  let app = axum_helper::add_middleware_layers(
    Router::new()
      .route("/", get(hello))
      .route("/Prod/hello", get(hello))
      .route("/Stage/hello", get(hello))
      .route("/PaddleHooks", post(process_paddle_events)),
  );

  run(app).await.map_err(|err| anyhow!("{err:?}"))
}

async fn run_no_aws() -> Result<()> {
  info!("Running without AWS Lambda context");

  let app = axum_helper::add_middleware_layers(get_routes());

  // Run app on local server
  let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .map_err(|err| anyhow!("{err:?}"))
}

async fn hello() -> &'static str {
  "Hello!"
}

async fn process_paddle_events(body: Bytes) -> &'static str {
  let body_text = std::str::from_utf8(&body).unwrap_or("Cannot convert to string");
  info!("on_paddle_notification: {}", body_text);
  "Thanks"
}

fn get_routes<S, B>() -> Router<S, B>
where
  B: http_body::Body + Send + Sync + 'static,
  S: Clone + Send + Sync + 'static,
  <B as http_body::Body>::Data: std::marker::Send, // <B as HttpBody>::Error: Sync,
  <B as http_body::Body>::Error: Send + Sync + Error,
{
  Router::new()
    .route("/", get(hello))
    .route("/Prod/hello", get(hello))
    .route("/Stage/hello", get(hello))
    .route("/PaddleHooks", post(process_paddle_events))

  // prod is the stage, this has to be the same as in the application gateway!
  // better yet: have a rewrite_request_url middleware that removes all stage names from the path
  // let middleware = middleware::from_fn(rewrite_request_uri);

  // .route("/prod/process-json", post(process_json::process_json))
  // )
}
