mod contract;
mod index;
#[cfg(test)]
mod tests;

use super::client::s3_client;
use super::listing::latest_key_with_prefix;
use crate::error::{AppError, AppResult};

use self::index::latest_key_from_l1_index;

pub(super) async fn discover_latest_market_l1_keys_from_s3(
    bucket: &str,
    window_starts_ms: &[i64],
    family_prefix: &str,
    file_suffix: &str,
    manifest_key_field: &str,
) -> AppResult<Vec<String>> {
    if window_starts_ms.is_empty() {
        return Ok(Vec::new());
    }
    if bucket.trim().is_empty() {
        return Err(AppError::config("market L1 S3 bucket must not be empty"));
    }
    let client = s3_client().await?;
    let mut keys = Vec::new();
    for window_start_ms in window_starts_ms {
        let prefix = format!("{family_prefix}/run_id=l1_{window_start_ms}_");
        if let Some(key) = latest_key_with_prefix(&client, bucket, &prefix, file_suffix).await? {
            keys.push(key);
        }
        if let Some(key) =
            latest_key_from_l1_index(&client, bucket, *window_start_ms, manifest_key_field).await?
        {
            keys.push(key);
        }
    }
    keys.sort();
    keys.dedup();
    Ok(keys)
}
