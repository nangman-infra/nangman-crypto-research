use crate::error::{AppError, AppResult};
use crate::model::{ResearchInputManifest, RetestCycleSourceState};
use crate::storage::client::s3_client;
use crate::storage::objects::{
    PutIfAbsentResult, put_object_bytes_if_absent, put_object_json,
    validate_research_input_manifest_s3_key, validate_s3_location,
};

mod keys;

use keys::{
    research_input_manifest_key, retest_cycle_source_state_key, retest_horizon_plan_key,
    retest_horizon_status_key,
};

pub async fn write_research_input_manifest_to_s3(
    bucket: &str,
    prefix: &str,
    manifest: &ResearchInputManifest,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "research input manifest output S3 bucket must not be empty",
        ));
    }
    let key = research_input_manifest_key(prefix, manifest, output_partition_at_ms)?;
    validate_s3_location(bucket, &key, "research input manifest output")?;
    let client = s3_client().await?;
    put_object_json(&client, bucket, &key, manifest).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_research_input_manifest_to_exact_s3_key_if_absent(
    bucket: &str,
    key: &str,
    manifest: &ResearchInputManifest,
) -> AppResult<Option<String>> {
    validate_s3_location(bucket, key, "research input manifest output")?;
    validate_research_input_manifest_s3_key(key)?;
    let client = s3_client().await?;
    let body = serde_json::to_vec_pretty(manifest)?;
    match put_object_bytes_if_absent(&client, bucket, key, body, "application/json").await? {
        PutIfAbsentResult::Created => Ok(Some(format!("s3://{bucket}/{key}"))),
        PutIfAbsentResult::AlreadyExists => Ok(None),
    }
}

pub async fn write_retest_cycle_source_state_to_s3(
    bucket: &str,
    prefix: &str,
    state: &RetestCycleSourceState,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest cycle source state output S3 bucket must not be empty",
        ));
    }
    let key = retest_cycle_source_state_key(prefix, state, output_partition_at_ms)?;
    validate_s3_location(bucket, &key, "retest cycle source state output")?;
    let client = s3_client().await?;
    put_object_json(&client, bucket, &key, state).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_retest_horizon_plan_to_s3(
    bucket: &str,
    prefix: &str,
    plan: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest horizon plan output S3 bucket must not be empty",
        ));
    }
    let key = retest_horizon_plan_key(prefix, plan, output_partition_at_ms)?;
    validate_s3_location(bucket, &key, "retest horizon plan output")?;
    let client = s3_client().await?;
    put_object_json(&client, bucket, &key, plan).await?;
    Ok(format!("s3://{bucket}/{key}"))
}

pub async fn write_retest_horizon_status_to_s3(
    bucket: &str,
    prefix: &str,
    status: &serde_json::Value,
    output_partition_at_ms: i64,
) -> AppResult<String> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest horizon status output S3 bucket must not be empty",
        ));
    }
    let key = retest_horizon_status_key(prefix, status, output_partition_at_ms)?;
    validate_s3_location(bucket, &key, "retest horizon status output")?;
    let client = s3_client().await?;
    put_object_json(&client, bucket, &key, status).await?;
    Ok(format!("s3://{bucket}/{key}"))
}
