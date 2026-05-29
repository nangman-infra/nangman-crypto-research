use super::super::{
    AppError, AppResult, RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION, RetestCycleSourceState,
    get_object_bytes, list_payload_objects_with_prefix, normalize_prefix,
    read_retest_horizon_status_from_bytes, s3_client, select_latest_payload_keys,
};

pub async fn read_latest_retest_cycle_source_state_from_s3(
    bucket: &str,
    prefix: &str,
) -> AppResult<RetestCycleSourceState> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest cycle source state S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "retest-cycle-source-state/schema=research_retest_cycle_source_state_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("retest-cycle-source-state/") {
        return Err(AppError::config(
            "retest cycle source state S3 prefix must start with retest-cycle-source-state/",
        ));
    }
    let keys = list_payload_objects_with_prefix(&client, bucket, &prefix, "/state.json", 1_000)
        .await
        .map(|objects| select_latest_payload_keys(objects, 1))?;
    let key = keys
        .first()
        .ok_or_else(|| AppError::AwsNotFound(format!("s3://{bucket}/{prefix}")))?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    let state = serde_json::from_slice::<RetestCycleSourceState>(&bytes)?;
    if state.schema_version != RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION {
        return Err(AppError::validation(format!(
            "retest cycle source state schema_version must be {RETEST_CYCLE_SOURCE_STATE_SCHEMA_VERSION}; got {}",
            state.schema_version
        )));
    }
    Ok(state)
}

pub async fn read_latest_retest_horizon_status_from_s3(
    bucket: &str,
    prefix: &str,
) -> AppResult<serde_json::Value> {
    if bucket.trim().is_empty() {
        return Err(AppError::config(
            "retest horizon status S3 bucket must not be empty",
        ));
    }
    let client = s3_client().await?;
    let prefix = normalize_prefix(if prefix.trim().is_empty() {
        "retest-horizon-status/schema=research_horizon_status_checkpoint_v1"
    } else {
        prefix
    });
    if !prefix.starts_with("retest-horizon-status/") {
        return Err(AppError::config(
            "retest horizon status S3 prefix must start with retest-horizon-status/",
        ));
    }
    let keys = list_payload_objects_with_prefix(
        &client,
        bucket,
        &prefix,
        "/retest-horizon-status.json",
        1_000,
    )
    .await
    .map(|objects| select_latest_payload_keys(objects, 1))?;
    let key = keys
        .first()
        .ok_or_else(|| AppError::AwsNotFound(format!("s3://{bucket}/{prefix}")))?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_retest_horizon_status_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}
