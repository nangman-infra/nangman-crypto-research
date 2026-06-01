mod latest;
mod limits;
mod page;
mod payload;
mod selection;
mod validation;

use super::client::s3_client;
use crate::error::AppResult;
pub(super) use latest::latest_key_with_prefix;
use limits::PayloadListOptions;
use payload::list_payload_objects_with_options;
pub(super) use payload::list_payload_objects_with_prefix;
pub(super) use selection::select_latest_payload_keys;
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

#[cfg(test)]
mod tests;
