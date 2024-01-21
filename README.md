# psawsrs

## Running tests locally with localstack

Make sure to have a localstack profile in ~/.aws/credentials with the following contents:

```toml
[localstack]
region = eu-west-1
aws_access_key_id = AKIDLOCALSTACK
aws_secret_access_key = localstacksecret
```

Run tests

```shell
export AWS_DEFAULT_REGION=eu-west-1
export AWS_PROFILE=localstack
docker compose up -d

# create the required dynamodb tables
DYNAMO_DB_ENDPOINT=http://localhost:8000 cargo test -p sales --lib test_insert_persion

# run all tests
DYNAMO_DB_ENDPOINT=http://localhost:8000 cargo test --workspace --all -- --nocapture
```

## Monitoring

SQS
https://eu-west-1.console.aws.amazon.com/sqs/v2/home?region=eu-west-1#/queues

Lambda
https://eu-west-1.console.aws.amazon.com/lambda/home?region=eu-west-1#/functions?sb=lastModified&so=DESCENDING

Cloudformation stacks
https://eu-west-1.console.aws.amazon.com/cloudformation/home?region=eu-west-1

## SAM deployment

Deploy SAM stack for the first time

```shell
make moreh-build
make account-build
sam deploy --guided --capabilities CAPABILITY_IAM CAPABILITY_AUTO_EXPAND --resolve-s3
```

## SAM sync local build with dev environment

```shell
# with potential drift
sam sync --stack-name "psawsrs-dev" --no-watch --build-in-source --build-in-source

# prevent drift
sam sync --stack-name "psawsrs-dev" --no-skip-deploy-sync --no-watch --build-in-source

# sync only the moreh-notifications lambda
sam sync --stack-name "psawsrs-dev" --no-watch --resource-id MoREHSQSConsumers/MoREHPaddleNotificationsFunction --build-in-source
```

## Generate test events

Generate test event json data for sqs

```shell
sam local generate-event sqs receive-message --body '{"hello": "world"}'
```

```json
{
  "Records": [
    {
      "messageId": "19dd0b57-b21e-4ac1-bd88-01bbb068cb78",
      "receiptHandle": "MessageReceiptHandle",
      "body": "{"hello": "world"}",
      "attributes": {
        "ApproximateReceiveCount": "1",
        "SentTimestamp": "1523232000000",
        "SenderId": "123456789012",
        "ApproximateFirstReceiveTimestamp": "1523232000001"
      },
      "messageAttributes": {},
      "md5OfBody": "49dfdd54b01cbcd2d2ab5e9e5ee6b9b9",
      "eventSource": "aws:sqs",
      "eventSourceARN": "arn:aws:sqs:us-east-1:123456789012:MyQueue",
      "awsRegion": "us-east-1"
    }
  ]
}
```

Generate test event for http-api-proxy

```shell
sam local generate-event apigateway http-api-proxy > test_events/apigateway_event.json
```
