use super::build::build_shadow_accumulation_manifest_dispatch;
use super::types::ShadowAccumulationDispatch;
use super::*;

pub(in crate::cli) async fn try_build_shadow_accumulation_manifest_from_latest_state(
    args: &Args,
    shadow_runs: &[ShadowValidationRun],
    latest_l1_as_of_ms: Option<i64>,
    output_partition_at_ms: i64,
) -> AppResult<Option<ShadowAccumulationDispatch>> {
    let deficit_lifecycle_keys =
        shadow_sample_deficit_lifecycle_keys(shadow_runs, latest_l1_as_of_ms);
    let Some(bucket) = args.output_s3_bucket.as_deref() else {
        return Ok(None);
    };
    let state = match read_latest_retest_cycle_source_state_from_s3(bucket, "").await {
        Ok(state) => state,
        Err(AppError::AwsNotFound(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let status = match read_latest_retest_horizon_status_from_s3(bucket, "").await {
        Ok(status) => status,
        Err(AppError::AwsNotFound(_)) => return Ok(None),
        Err(error) => return Err(error),
    };
    let source_manifest = read_research_input_manifest_from_s3(
        &state.source_manifest_s3_bucket,
        &state.source_manifest_s3_key,
    )
    .await?;
    validate_input_manifest(Some(&source_manifest))?;

    let Some(dispatch_build) = build_shadow_accumulation_manifest_dispatch(
        args,
        &state,
        &status,
        &source_manifest,
        latest_l1_as_of_ms,
        output_partition_at_ms,
        deficit_lifecycle_keys,
    )?
    else {
        return Ok(None);
    };

    let write_result = write_research_input_manifest_to_exact_s3_key_if_absent(
        bucket,
        &dispatch_build.key,
        &dispatch_build.manifest,
    )
    .await?;
    let created = write_result.is_some();
    let manifest_uri =
        write_result.unwrap_or_else(|| format!("s3://{bucket}/{}", dispatch_build.key));

    Ok(Some(ShadowAccumulationDispatch {
        manifest_uri,
        created,
        focused_horizon_count: dispatch_build.focused_horizon_count,
        focused_candidate_bundle_refs: dispatch_build.focused_candidate_bundle_refs,
        deficit_lifecycle_keys: dispatch_build.deficit_lifecycle_keys,
    }))
}
