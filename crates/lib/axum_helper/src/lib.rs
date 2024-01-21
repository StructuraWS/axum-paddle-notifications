use axum::{body::Bytes, Router};
use http::header::{AUTHORIZATION, COOKIE, PROXY_AUTHORIZATION, SET_COOKIE};
use std::{iter::once, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
  classify::ServerErrorsFailureClass,
  normalize_path::NormalizePathLayer,
  sensitive_headers::{
    SetSensitiveHeadersLayer, SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
  },
  timeout::TimeoutLayer,
  trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
  LatencyUnit, ServiceBuilderExt,
};
use tracing::{Level, Span};

pub fn add_middleware_layers<S, B>(router: Router<S, B>) -> Router<S, B>
where
  B: http_body::Body + Send + 'static,
  S: Clone + Send + Sync + 'static,
{
  // todo log all 404 requests with the requested path and method

  let headers: Arc<[_]> = Arc::new([AUTHORIZATION, PROXY_AUTHORIZATION, COOKIE, SET_COOKIE]);

  // Mark headers as sensitive so it doesn't show in logs
  let router = router.layer(
    ServiceBuilder::new()
      .layer(SetSensitiveRequestHeadersLayer::from_shared(Arc::clone(
        &headers,
      )))
      .layer(SetSensitiveHeadersLayer::new(once(AUTHORIZATION))),
  );
  let router = add_tracing_middleware(router);
  router.layer(
    ServiceBuilder::new()
      .layer(SetSensitiveResponseHeadersLayer::from_shared(headers))
      .layer(NormalizePathLayer::trim_trailing_slash())
      .layer(TimeoutLayer::new(Duration::from_secs(20)))
      //    // Box the response body so it implements `Default` which is required by axum
      .map_response_body(axum::body::boxed)
      .compression(),
  )
}

fn add_tracing_middleware<S, B>(router: Router<S, B>) -> Router<S, B>
where
  B: http_body::Body + Send + 'static,
  S: Clone + Send + Sync + 'static,
{
  // Add high level tracing/logging to all requests
  router.layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(
            TraceLayer::new_for_http()
                .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                    tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
                })
                // .make_span_with(DefaultMakeSpan::new().include_headers(true))
    // .on_request(|request: &Request<_>, _span: &Span| {
                //     // You can use `_span.record("some_other_field", value)` in one of these
                //     // closures to attach a value to the initially empty field in the info_span
                //     // created above.
                //     tracing::debug!("Body: {:?}", request.uri());
                // })
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros))
                // .on_request(|request: &Request<_>, _span: &Span| {
                //     debug!("req: {}", request.uri());
                // })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        tracing::error!("Server error: {:?}", error)
                    },
                ),
        )
}
