use super::super::*;

pub(in crate::cli) async fn load_historical_replay_runs(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
    max_historical_replay_run_ref_count: usize,
) -> AppResult<Vec<ReplayRun>> {
    let mut replay_runs = Vec::new();
    for path in &args.historical_replay_run_files {
        append_unique_replay_runs(&mut replay_runs, read_replay_runs(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.historical_replay_run_refs {
            append_unique_replay_runs(
                &mut replay_runs,
                read_replay_runs_from_ref(artifact_ref).await?,
            );
        }
    }
    if !args.historical_replay_run_s3_keys.is_empty() {
        let bucket = args
            .historical_replay_run_s3_bucket
            .as_deref()
            .ok_or_else(|| {
                AppError::config("RESEARCH_HISTORICAL_REPLAY_RUN_S3_BUCKET is required")
            })?;
        append_unique_replay_runs(
            &mut replay_runs,
            read_replay_runs_from_s3(bucket, &args.historical_replay_run_s3_keys).await?,
        );
    }
    let index_records = load_historical_replay_run_index_records(
        args,
        manifest,
        max_historical_replay_run_ref_count,
    )
    .await?;
    append_unique_replay_runs(
        &mut replay_runs,
        load_replay_runs_from_index_records(&index_records).await?,
    );
    Ok(replay_runs)
}

pub(in crate::cli) async fn load_historical_replay_run_index_records(
    args: &Args,
    manifest: Option<&ResearchInputManifest>,
    max_historical_replay_run_ref_count: usize,
) -> AppResult<Vec<ReplayRunIndexRecord>> {
    let mut records = Vec::new();
    for path in &args.historical_replay_run_index_files {
        records.extend(read_replay_run_index_records(path)?);
    }
    if let Some(manifest) = manifest {
        for artifact_ref in &manifest.historical_replay_run_index_refs {
            records.extend(read_replay_run_index_records_from_ref(artifact_ref).await?);
        }
    }
    if !args.historical_replay_run_index_s3_keys.is_empty() {
        let bucket = args
            .historical_replay_run_index_s3_bucket
            .as_deref()
            .ok_or_else(|| {
                AppError::config("RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET is required")
            })?;
        records.extend(
            read_replay_run_index_records_from_s3(
                bucket,
                &args.historical_replay_run_index_s3_keys,
            )
            .await?,
        );
    }
    if let Some(prefix) = env_string("RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX") {
        let bucket = args
            .historical_replay_run_index_s3_bucket
            .as_deref()
            .or(args.output_s3_bucket.as_deref())
            .ok_or_else(|| {
                AppError::config(
                    "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_BUCKET or RESEARCH_OUTPUT_S3_BUCKET is required when RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_PREFIX is set",
                )
            })?;
        let read_limit = env_usize(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_READ_LIMIT",
            DEFAULT_HISTORICAL_REPLAY_RUN_INDEX_READ_LIMIT,
        )?;
        let scan_limit = env_usize(
            "RESEARCH_HISTORICAL_REPLAY_RUN_INDEX_S3_SCAN_LIMIT",
            DEFAULT_HISTORICAL_REPLAY_RUN_INDEX_SCAN_LIMIT,
        )?;
        let discovered_keys =
            discover_replay_run_index_keys_from_s3(bucket, &prefix, read_limit, scan_limit).await?;
        enforce_budget(
            "historical_replay_run_index_s3_prefix_key_count",
            discovered_keys.len(),
            max_historical_replay_run_ref_count,
        )?;
        records.extend(read_replay_run_index_records_from_s3(bucket, &discovered_keys).await?);
    }
    Ok(records)
}
