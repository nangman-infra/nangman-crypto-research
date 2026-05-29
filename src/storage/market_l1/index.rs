use super::super::objects::{get_object_bytes, is_missing_market_artifact};
use super::contract::{
    is_success_l1_index_pointer, is_success_l1_manifest, l1_artifact_key_from_manifest,
    l1_index_pointer_key, l1_manifest_key_from_pointer,
};
use crate::error::{AppError, AppResult};
use aws_sdk_s3::Client;

pub(in crate::storage::market_l1) async fn latest_key_from_l1_index(
    client: &Client,
    bucket: &str,
    window_start_ms: i64,
    manifest_key_field: &str,
) -> AppResult<Option<String>> {
    let pointer_key = l1_index_pointer_key(window_start_ms)?;
    let pointer_bytes = match get_object_bytes(client, bucket, &pointer_key).await {
        Ok(bytes) => bytes,
        Err(error) if is_missing_market_artifact(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    let pointer = serde_json::from_slice::<serde_json::Value>(&pointer_bytes).map_err(|error| {
        AppError::validation(format!(
            "invalid Market-L1 index pointer s3://{bucket}/{pointer_key}: {error}"
        ))
    })?;
    if !is_success_l1_index_pointer(&pointer) {
        return Ok(None);
    }
    let manifest_key = l1_manifest_key_from_pointer(&pointer).ok_or_else(|| {
        AppError::validation(format!(
            "Market-L1 index pointer missing canonical manifest key: s3://{bucket}/{pointer_key}"
        ))
    })?;
    let manifest_bytes = match get_object_bytes(client, bucket, &manifest_key).await {
        Ok(bytes) => bytes,
        Err(error) if is_missing_market_artifact(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    let manifest =
        serde_json::from_slice::<serde_json::Value>(&manifest_bytes).map_err(|error| {
            AppError::validation(format!(
                "invalid Market-L1 manifest s3://{bucket}/{manifest_key}: {error}"
            ))
        })?;
    if !is_success_l1_manifest(&manifest) {
        return Ok(None);
    }
    Ok(l1_artifact_key_from_manifest(&manifest, manifest_key_field))
}
