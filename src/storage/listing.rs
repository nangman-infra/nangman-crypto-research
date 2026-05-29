use super::client::{aws_error_detail, s3_client};
use crate::error::{AppError, AppResult};
use aws_sdk_s3::Client;

mod limits;
mod selection;
mod validation;

use limits::{PayloadListOptions, ensure_scan_limit};
pub(super) use selection::{ListedPayloadObject, select_latest_payload_keys};
use validation::validate_discovery_request;

pub(super) async fn discover_latest_part_jsonl_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
    artifact_label: &str,
) -> AppResult<Vec<String>> {
    validate_discovery_request(bucket, prefix, read_limit, scan_limit, artifact_label)?;
    let client = s3_client().await?;
    let options = PayloadListOptions {
        file_suffix: "/part-000001.jsonl",
        scan_limit,
        artifact_label,
    };
    let objects = list_payload_objects_with_options(&client, bucket, prefix, options).await?;
    Ok(select_latest_payload_keys(objects, read_limit))
}

pub(super) async fn latest_key_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &str,
    file_suffix: &str,
) -> AppResult<Option<String>> {
    let mut latest: Option<String> = None;
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = client.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(token) = continuation_token.as_deref() {
            request = request.continuation_token(token);
        }
        let output = request.send().await.map_err(|error| {
            AppError::Aws(format!(
                "s3 list_objects_v2 s3://{bucket}/{prefix}: {}",
                aws_error_detail(&error)
            ))
        })?;

        for object in output.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            if !key.ends_with(file_suffix) {
                continue;
            }
            if latest
                .as_deref()
                .is_none_or(|current_latest| key > current_latest)
            {
                latest = Some(key.to_owned());
            }
        }

        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(latest)
}

pub(super) async fn list_payload_objects_with_prefix(
    client: &Client,
    bucket: &str,
    prefix: &str,
    file_suffix: &str,
    scan_limit: usize,
) -> AppResult<Vec<ListedPayloadObject>> {
    let options = PayloadListOptions {
        file_suffix,
        scan_limit,
        artifact_label: "S3 payload",
    };
    list_payload_objects_with_options(client, bucket, prefix, options).await
}

async fn list_payload_objects_with_options(
    client: &Client,
    bucket: &str,
    prefix: &str,
    options: PayloadListOptions<'_>,
) -> AppResult<Vec<ListedPayloadObject>> {
    let mut objects = Vec::new();
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = client.list_objects_v2().bucket(bucket).prefix(prefix);
        if let Some(token) = continuation_token.as_deref() {
            request = request.continuation_token(token);
        }
        let output = request.send().await.map_err(|error| {
            AppError::Aws(format!(
                "s3 list_objects_v2 s3://{bucket}/{prefix}: {}",
                aws_error_detail(&error)
            ))
        })?;

        for object in output.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            if !key.ends_with(options.file_suffix) {
                continue;
            }
            objects.push(ListedPayloadObject {
                key: key.to_owned(),
                last_modified_ms: object
                    .last_modified()
                    .and_then(|last_modified| last_modified.to_millis().ok())
                    .unwrap_or(0),
            });
            ensure_scan_limit(objects.len(), bucket, prefix, options)?;
        }

        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(objects)
}

#[cfg(test)]
mod tests;
