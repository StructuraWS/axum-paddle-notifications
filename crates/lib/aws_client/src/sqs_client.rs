use anyhow::Result;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use aws_lambda_events::{
  event::sqs::SqsEventObj,
  sqs::{BatchItemFailure, SqsBatchResponse, SqsMessageObj},
};
use futures::Future;
use serde::de::DeserializeOwned;
use tracing::Instrument;

/// This function will handle the message batches from SQS.
/// It calls the provided user function `f` on every message concurrently and reports to SQS
/// which message failed to be processed so that only those are retried.
///
/// Important note: your lambda sqs trigger *needs* to be configured with partial batch response support
/// with the ` ReportBatchItemFailures` flag set to true, otherwise failed message will be dropped,
/// for more details see:
/// https://docs.aws.amazon.com/lambda/latest/dg/with-sqs.html#services-sqs-batchfailurereporting
///
///
/// Note that if you are looking for parallel processing (multithread) instead of concurrent processing,
/// you can do so by spawning a task inside your function `f`.
pub async fn run_sqs_partial_batch_failure<T, D, R>(f: T) -> Result<(), Error>
where
  T: Fn(D) -> R,
  D: DeserializeOwned,
  R: Future<Output = Result<(), Error>>,
{
  run(service_fn(|e| sqs_batch_handler(|d| f(d), e))).await
}

/// Helper function to lift the user provided `f` function from message to batch of messages.
/// See `run_sqs` for the easier function to use.
pub async fn sqs_batch_handler<T, D, F>(
  f: T,
  event: LambdaEvent<SqsEventObj<serde_json::Value>>,
) -> Result<SqsBatchResponse, Error>
where
  T: Fn(D) -> F,
  F: Future<Output = Result<(), Error>>,
  D: DeserializeOwned,
{
  tracing::trace!("Handling batch size {}", event.payload.records.len());
  let create_task = |msg| {
    // We need to keep the message_id to report failures to SQS
    let SqsMessageObj {
      message_id, body, ..
    } = msg;
    let span = tracing::span!(tracing::Level::INFO, "Handling SQS msg", message_id);
    let task = async {
      //TODO catch panics like the `run` function from lambda_runtime
      f(serde_json::from_value(body)?).await
    }
    .instrument(span);
    (message_id.unwrap_or_default(), task)
  };
  let (ids, tasks): (Vec<_>, Vec<_>) = event.payload.records.into_iter().map(create_task).unzip();
  let results = futures::future::join_all(tasks).await; // Run tasks concurrently
  let failure_items = ids
    .into_iter()
    .zip(results)
    .filter_map(
      // Only keep the message_id of failed tasks
      |(id, res)| match res {
        Ok(()) => None,
        Err(err) => {
          tracing::error!("sqs_client Failed to process msg {id}, {err}");
          Some(id)
        }
      },
    )
    .map(|id| BatchItemFailure {
      item_identifier: id,
    })
    .collect();

  Ok(SqsBatchResponse {
    batch_item_failures: failure_items,
  })
}
