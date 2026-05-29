use super::validation::{validate_content_type, validate_s3_location};
use crate::error::{AppError, AppResult};
use crate::storage::client::aws_error_detail;
use aws_sdk_s3::Client;
use aws_sdk_s3::error::ProvideErrorMetadata;

pub(in crate::storage) async fn put_object_json<T>(
    client: &Client,
    bucket: &str,
    key: &str,
    value: &T,
) -> AppResult<()>
where
    T: serde::Serialize,
{
    let body = serde_json::to_vec_pretty(value)?;
    put_object_bytes(client, bucket, key, body, "application/json").await
}

pub(in crate::storage) async fn put_jsonl_object<T>(
    client: &Client,
    bucket: &str,
    key: &str,
    values: &[T],
) -> AppResult<()>
where
    T: serde::Serialize,
{
    let mut body = Vec::new();
    for value in values {
        serde_json::to_writer(&mut body, value)?;
        body.push(b'\n');
    }
    put_object_bytes(client, bucket, key, body, "application/x-ndjson").await
}

async fn put_object_bytes(
    client: &Client,
    bucket: &str,
    key: &str,
    body: Vec<u8>,
    content_type: &str,
) -> AppResult<()> {
    validate_put_object_input(bucket, key, content_type)?;
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .body(body.into())
        .send()
        .await
        .map_err(|error| {
            AppError::aws(format!(
                "s3 put_object s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            ))
        })?;
    Ok(())
}

pub(in crate::storage) enum PutIfAbsentResult {
    Created,
    AlreadyExists,
}

pub(in crate::storage) async fn put_object_bytes_if_absent(
    client: &Client,
    bucket: &str,
    key: &str,
    body: Vec<u8>,
    content_type: &str,
) -> AppResult<PutIfAbsentResult> {
    validate_put_object_input(bucket, key, content_type)?;
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type(content_type)
        .if_none_match("*")
        .body(body.into())
        .send()
        .await
        .map(|_| PutIfAbsentResult::Created)
        .or_else(|error| {
            if let Some(service_error) = error.as_service_error()
                && matches!(
                    service_error.code(),
                    Some("PreconditionFailed" | "ConditionalRequestConflict")
                )
            {
                return Ok(PutIfAbsentResult::AlreadyExists);
            }
            Err(AppError::Aws(format!(
                "s3 put_object if_absent s3://{bucket}/{key}: {}",
                aws_error_detail(&error)
            )))
        })
}

fn validate_put_object_input(bucket: &str, key: &str, content_type: &str) -> AppResult<()> {
    validate_s3_location(bucket, key, "S3")?;
    validate_content_type(content_type)
}
