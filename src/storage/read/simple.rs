use super::super::{
    AppResult, IntelCandidateEvidenceBundle, MarketLiveTick, OssAdapterRun, PaperWatchCandidate,
    PaperWatchLiveMark, ResearchInputManifest, ResearchRunReport, ShadowValidationRun,
    get_object_bytes, read_candidate_bundles_from_bytes, read_market_live_ticks_from_bytes,
    read_oss_adapter_runs_from_bytes, read_paper_watch_candidates_from_bytes,
    read_paper_watch_live_marks_from_bytes, read_research_input_manifest_from_bytes,
    read_research_run_report_from_bytes, read_retest_horizon_plan_from_bytes,
    read_retest_horizon_status_from_bytes, read_shadow_validation_runs_from_bytes, s3_client,
};

pub async fn read_candidate_bundles_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<IntelCandidateEvidenceBundle>> {
    read_s3_object(bucket, key, read_candidate_bundles_from_bytes).await
}

pub async fn read_oss_adapter_runs_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<OssAdapterRun>> {
    read_s3_objects(bucket, keys, read_oss_adapter_runs_from_bytes).await
}

pub async fn read_shadow_validation_runs_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<ShadowValidationRun>> {
    read_s3_objects(bucket, keys, read_shadow_validation_runs_from_bytes).await
}

pub async fn read_research_input_manifest_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<ResearchInputManifest> {
    read_s3_object(bucket, key, read_research_input_manifest_from_bytes).await
}

pub async fn read_research_run_report_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<ResearchRunReport> {
    read_s3_object(bucket, key, read_research_run_report_from_bytes).await
}

pub async fn read_paper_watch_candidates_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<PaperWatchCandidate>> {
    read_s3_object(bucket, key, read_paper_watch_candidates_from_bytes).await
}

pub async fn read_market_live_ticks_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<Vec<MarketLiveTick>> {
    read_s3_object(bucket, key, read_market_live_ticks_from_bytes).await
}

pub async fn read_paper_watch_live_marks_from_s3(
    bucket: &str,
    keys: &[String],
) -> AppResult<Vec<PaperWatchLiveMark>> {
    read_s3_objects(bucket, keys, read_paper_watch_live_marks_from_bytes).await
}

pub async fn read_retest_horizon_status_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<serde_json::Value> {
    read_s3_object(bucket, key, read_retest_horizon_status_from_bytes).await
}

pub async fn read_retest_horizon_plan_from_s3(
    bucket: &str,
    key: &str,
) -> AppResult<serde_json::Value> {
    read_s3_object(bucket, key, read_retest_horizon_plan_from_bytes).await
}

async fn read_s3_object<T, F>(bucket: &str, key: &str, decode: F) -> AppResult<T>
where
    F: FnOnce(&str, &[u8]) -> AppResult<T>,
{
    let client = s3_client().await?;
    let bytes = get_object_bytes(&client, bucket, key).await?;
    let label = s3_object_label(bucket, key);
    decode(&label, bytes.as_ref())
}

async fn read_s3_objects<T, F>(bucket: &str, keys: &[String], decode: F) -> AppResult<Vec<T>>
where
    F: Fn(&str, &[u8]) -> AppResult<Vec<T>>,
{
    let client = s3_client().await?;
    let mut values = Vec::new();
    for key in keys {
        let bytes = get_object_bytes(&client, bucket, key).await?;
        let label = s3_object_label(bucket, key);
        values.extend(decode(&label, bytes.as_ref())?);
    }
    Ok(values)
}

fn s3_object_label(bucket: &str, key: &str) -> String {
    format!("s3://{bucket}/{key}")
}
