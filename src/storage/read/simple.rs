use super::super::*;

pub async fn read_candidate_bundles_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_candidate_bundles_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_oss_adapter_runs_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<OssAdapterRun>> {
    let client = s3_client().await?;
    let mut runs = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        runs.extend(read_oss_adapter_runs_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(runs)
}

pub async fn read_shadow_validation_runs_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<ShadowValidationRun>> {
    let client = s3_client().await?;
    let mut runs = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        runs.extend(read_shadow_validation_runs_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(runs)
}

pub async fn read_research_input_manifest_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<ResearchInputManifest> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_research_input_manifest_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_research_run_report_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<ResearchRunReport> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_research_run_report_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_paper_watch_candidates_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<PaperWatchCandidate>> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_paper_watch_candidates_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_market_live_ticks_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<MarketLiveTick>> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_market_live_ticks_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_paper_watch_live_marks_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<PaperWatchLiveMark>> {
    let client = s3_client().await?;
    let mut marks = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        marks.extend(read_paper_watch_live_marks_from_bytes(
            &format!("s3://{bucket}/{key}"),
            bytes.as_ref(),
        )?);
    }
    Ok(marks)
}

pub async fn read_retest_horizon_status_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<serde_json::Value> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_retest_horizon_status_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}

pub async fn read_retest_horizon_plan_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<serde_json::Value> {
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    read_retest_horizon_plan_from_bytes(&format!("s3://{bucket}/{key}"), bytes.as_ref())
}
