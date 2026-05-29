use super::super::*;

pub async fn read_replay_runs_from_s3(bucket: &str, keys: &[String]) -> AppResult<Vec<ReplayRun>> {
    let client = s3_client().await?;
    let mut runs = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        runs.extend(read_replay_runs_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(runs)
}

pub async fn read_replay_run_index_records_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    let client = s3_client().await?;
    let mut records = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        records.extend(read_replay_run_index_records_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(records)
}

pub async fn discover_replay_run_index_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "historical replay-run-index",
    )
    .await
}

pub async fn discover_shadow_validation_run_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "shadow validation run",
    )
    .await
}

pub async fn discover_paper_watch_candidate_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "paper-watch candidate",
    )
    .await
}

pub async fn discover_paper_watch_live_mark_keys_from_s3(
    bucket: &str,
    prefix: &str,
    read_limit: usize,
    scan_limit: usize,
) -> AppResult<Vec<String>> {
    discover_latest_part_jsonl_keys_from_s3(
        bucket,
        prefix,
        read_limit,
        scan_limit,
        "paper-watch live mark",
    )
    .await
}
